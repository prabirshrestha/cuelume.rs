mod support;

use std::{env, path::PathBuf, process::ExitCode, str::FromStr, thread, time::Duration};

use cuelume::{ALL_SOUNDS, PlayOptions, Player, Sound};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
};
use support::{AudioFormat, export};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1);
    let Some(sound_name) = arguments.next() else {
        return interactive();
    };

    if sound_name == "--list" || sound_name == "-l" {
        for sound in ALL_SOUNDS {
            println!("{sound}");
        }
        return Ok(());
    }

    let sound = Sound::from_str(&sound_name)?;
    let volume = arguments
        .next()
        .map(|value| value.parse::<f32>())
        .transpose()?
        .unwrap_or(1.0);
    let panning = arguments
        .next()
        .map(|value| value.parse::<f32>())
        .transpose()?
        .unwrap_or(0.0);
    play_once(sound, volume, panning)
}

fn interactive() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let result = App::new()?.run(&mut terminal);
    ratatui::restore();
    result
}

struct App {
    player: Player,
    selected: usize,
    search: String,
    searching: bool,
    volume: f32,
    panning: f32,
    export_format: AudioFormat,
    status: String,
}

impl App {
    fn new() -> Result<Self, cuelume::Error> {
        Ok(Self {
            player: Player::new()?,
            selected: 0,
            search: String::new(),
            searching: false,
            volume: 1.0,
            panning: 0.0,
            export_format: AudioFormat::Wav,
            status: "Press Enter or Space to play".to_owned(),
        })
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }
            if self.searching {
                self.handle_search_key(key.code)?;
                continue;
            }
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc if self.search.is_empty() => return Ok(()),
                KeyCode::Esc => {
                    self.search.clear();
                    self.selected = 0;
                }
                KeyCode::Char('/') => self.searching = true,
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_previous();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next();
                }
                KeyCode::Home => self.selected = 0,
                KeyCode::End => self.selected = self.filtered_sounds().len().saturating_sub(1),
                KeyCode::Enter | KeyCode::Char(' ') => self.play()?,
                KeyCode::Char('w') => self.export()?,
                KeyCode::Char('f') => {
                    self.export_format = self.export_format.next();
                    self.status = format!("Export format: {}", self.export_format);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.panning = (self.panning - 0.1).max(-1.0);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.panning = (self.panning + 0.1).min(1.0);
                }
                KeyCode::Char('-') => self.volume = (self.volume - 0.05).max(0.0),
                KeyCode::Char('+' | '=') => {
                    self.volume = (self.volume + 0.05).min(1.0);
                }
                KeyCode::Char('0') => {
                    self.volume = 1.0;
                    self.panning = 0.0;
                }
                _ => {}
            }
        }
    }

    fn handle_search_key(&mut self, key: KeyCode) -> Result<(), cuelume::Error> {
        match key {
            KeyCode::Esc => {
                self.search.clear();
                self.searching = false;
                self.selected = 0;
            }
            KeyCode::Enter => {
                self.play()?;
                self.searching = false;
            }
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Backspace => {
                self.search.pop();
                self.selected = 0;
            }
            KeyCode::Char(character) => {
                self.search.push(character);
                self.selected = 0;
            }
            _ => {}
        }
        Ok(())
    }

    fn select_previous(&mut self) {
        let sound_count = self.filtered_sounds().len();
        if sound_count == 0 {
            self.selected = 0;
            return;
        }
        self.selected = self.selected.checked_sub(1).unwrap_or(sound_count - 1);
    }

    fn select_next(&mut self) {
        let sound_count = self.filtered_sounds().len();
        if sound_count == 0 {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected + 1) % sound_count;
    }

    fn filtered_sounds(&self) -> Vec<Sound> {
        filter_sounds(&self.search)
    }

    fn play(&mut self) -> Result<(), cuelume::Error> {
        let Some(sound) = self.selected_sound() else {
            self.status = format!("No sounds match `{}`", self.search);
            return Ok(());
        };
        self.player.play_with(
            sound,
            PlayOptions {
                volume: self.volume,
                panning: self.panning,
            },
        )?;
        self.status = format!("Played {sound}");
        Ok(())
    }

    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(sound) = self.selected_sound() else {
            self.status = format!("No sounds match `{}`", self.search);
            return Ok(());
        };
        let path = PathBuf::from(format!("{sound}.{}", self.export_format.extension()));
        export(self.player.rendered(sound), &path, self.export_format)?;
        self.status = format!(
            "Exported {sound} as {} to {}",
            self.export_format,
            path.display()
        );
        Ok(())
    }

    fn selected_sound(&self) -> Option<Sound> {
        self.filtered_sounds().get(self.selected).copied()
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        let outer = Block::default()
            .title(" Cuelume Sound Browser ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let area = outer.inner(frame.area());
        frame.render_widget(outer, frame.area());
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(area);

        self.draw_search(frame, rows[0]);
        self.draw_sounds(frame, rows[1]);
        self.draw_controls(frame, rows[2], rows[3]);
        self.draw_footer(frame, rows[4], rows[5], rows[6]);
    }

    fn draw_search(&self, frame: &mut Frame<'_>, area: Rect) {
        let search_style = if self.searching {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let search_text = if self.search.is_empty() && !self.searching {
            "Press / to search".to_owned()
        } else {
            format!("{}{}", self.search, if self.searching { "_" } else { "" })
        };
        frame.render_widget(
            Paragraph::new(search_text)
                .style(search_style)
                .block(Block::default().title(" Search ").borders(Borders::ALL)),
            area,
        );
    }

    fn draw_sounds(&self, frame: &mut Frame<'_>, area: Rect) {
        let sounds = self.filtered_sounds();
        let items = sounds
            .iter()
            .map(|sound| ListItem::new(sound.as_str()))
            .collect::<Vec<_>>();
        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" Sounds ({}) ", sounds.len()))
                    .borders(Borders::ALL),
            )
            .highlight_symbol(" > ")
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        let selection = (!sounds.is_empty()).then_some(self.selected);
        let mut state = ListState::default().with_selected(selection);
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn draw_controls(&self, frame: &mut Frame<'_>, volume_area: Rect, panning_area: Rect) {
        let volume = Gauge::default()
            .block(
                Block::default()
                    .title(" Volume  [-/+] ")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(f64::from(self.volume))
            .label(format!("{:.0}%", self.volume * 100.0));
        frame.render_widget(volume, volume_area);

        let pan_ratio = f64::from((self.panning + 1.0) * 0.5);
        let pan_label = if self.panning < -0.05 {
            format!("Left {:.0}%", -self.panning * 100.0)
        } else if self.panning > 0.05 {
            format!("Right {:.0}%", self.panning * 100.0)
        } else {
            "Center".to_owned()
        };
        let panning = Gauge::default()
            .block(
                Block::default()
                    .title(" Panning  [h/l] ")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio(pan_ratio)
            .label(pan_label);
        frame.render_widget(panning, panning_area);
    }

    fn draw_footer(
        &self,
        frame: &mut Frame<'_>,
        details_area: Rect,
        status_area: Rect,
        help_area: Rect,
    ) {
        let details = self.selected_sound().map_or_else(
            || "No matching recipe".to_owned(),
            |sound| {
                let rendered = self.player.rendered(sound);
                format!(
                    "Recipe: {sound}  |  {:.0} ms  |  {} frames  |  {} Hz",
                    rendered.duration().as_secs_f64() * 1000.0,
                    rendered.frames().len(),
                    rendered.sample_rate()
                )
            },
        );
        frame.render_widget(
            Paragraph::new(details).style(Style::default().fg(Color::Magenta)),
            details_area,
        );

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(&self.status, Style::default().fg(Color::Yellow)),
                Span::raw(format!(
                    "  |  Enter/Space: play  |  f: format ({})  |  w: export",
                    self.export_format
                )),
            ])),
            status_area,
        );
        frame.render_widget(
            Paragraph::new(
                "/: search  |  Up/Down or j/k: select  |  -/+: volume  |  h/l: pan  |  q/Esc/Ctrl-C: quit",
            )
            .style(Style::default().fg(Color::DarkGray)),
            help_area,
        );
    }
}

fn filter_sounds(query: &str) -> Vec<Sound> {
    let query = query.to_ascii_lowercase();
    ALL_SOUNDS
        .into_iter()
        .filter(|sound| sound.as_str().contains(&query))
        .collect()
}

fn play_once(sound: Sound, volume: f32, panning: f32) -> Result<(), Box<dyn std::error::Error>> {
    let mut player = Player::new()?;
    let duration = player.rendered(sound).duration();
    println!("Playing {sound} at volume {volume} and panning {panning}");
    let _handle = player.play_with(sound, PlayOptions { volume, panning })?;
    thread::sleep((duration + Duration::from_millis(500)).max(Duration::from_secs(1)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_filters_sound_names() {
        assert_eq!(filter_sounds("tick"), vec![Sound::Tick]);
        assert_eq!(
            filter_sounds("re"),
            vec![Sound::Press, Sound::Release, Sound::Ready]
        );
        assert!(filter_sounds("missing").is_empty());
    }

    #[test]
    fn empty_search_returns_the_full_palette() {
        assert_eq!(filter_sounds(""), ALL_SOUNDS);
    }
}
