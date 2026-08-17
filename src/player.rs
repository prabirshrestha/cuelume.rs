use std::{array, fmt};

use kira::{
    AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Panning,
    backend::cpal,
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
};

use crate::{ALL_SOUNDS, DEFAULT_SAMPLE_RATE, RenderedSound, Sound, render};

#[derive(Debug, Clone, Copy)]
pub struct PlayerConfig {
    pub sample_rate: u32,
    pub volume: f32,
    pub enabled: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            volume: 1.0,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayOptions {
    pub volume: f32,
    pub panning: f32,
}

impl Default for PlayOptions {
    fn default() -> Self {
        Self {
            volume: 1.0,
            panning: 0.0,
        }
    }
}

pub struct Player {
    manager: AudioManager<DefaultBackend>,
    sounds: [RenderedSound; 17],
    volume: f32,
    enabled: bool,
}

impl Player {
    /// Creates a player with the default configuration and cached palette.
    ///
    /// # Errors
    ///
    /// Returns an error when the platform audio backend cannot start.
    pub fn new() -> Result<Self, Error> {
        Self::with_config(PlayerConfig::default())
    }

    /// Creates a player with the requested render and playback configuration.
    ///
    /// # Errors
    ///
    /// Returns an error for a zero sample rate or when the audio backend cannot start.
    pub fn with_config(config: PlayerConfig) -> Result<Self, Error> {
        if config.sample_rate == 0 {
            return Err(Error::InvalidSampleRate);
        }
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(Error::Backend)?;
        let sounds = array::from_fn(|index| render(ALL_SOUNDS[index], config.sample_rate));
        Ok(Self {
            manager,
            sounds,
            volume: normalize_volume(config.volume),
            enabled: config.enabled,
        })
    }

    /// Plays a sound with the current global volume.
    ///
    /// # Errors
    ///
    /// Returns an error when Kira cannot allocate or initialize the sound.
    pub fn play(&mut self, sound: Sound) -> Result<Option<StaticSoundHandle>, Error> {
        self.play_with(sound, PlayOptions::default())
    }

    /// Plays a sound with per-play volume and panning.
    ///
    /// # Errors
    ///
    /// Returns an error when Kira cannot allocate or initialize the sound.
    pub fn play_with(
        &mut self,
        sound: Sound,
        options: PlayOptions,
    ) -> Result<Option<StaticSoundHandle>, Error> {
        let volume = self.volume * normalize_volume(options.volume);
        if !self.enabled || volume == 0.0 {
            return Ok(None);
        }
        let rendered = &self.sounds[sound.index()];
        let data = StaticSoundData {
            sample_rate: rendered.sample_rate(),
            frames: rendered.shared_frames(),
            settings: StaticSoundSettings::default()
                .volume(Decibels(amplitude_to_decibels(volume)))
                .panning(Panning(normalize_panning(options.panning))),
            slice: None,
        };
        self.manager
            .play(data)
            .map(Some)
            .map_err(|error| match error {
                kira::PlaySoundError::SoundLimitReached => Error::SoundLimitReached,
                kira::PlaySoundError::IntoSoundError(()) => Error::SoundInitialization,
            })
    }

    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[must_use]
    pub const fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = normalize_volume(volume);
    }

    #[must_use]
    pub fn rendered(&self, sound: Sound) -> &RenderedSound {
        &self.sounds[sound.index()]
    }
}

fn normalize_volume(volume: f32) -> f32 {
    if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn normalize_panning(panning: f32) -> f32 {
    if panning.is_finite() {
        panning.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

fn amplitude_to_decibels(amplitude: f32) -> f32 {
    if amplitude <= 0.001 {
        Decibels::SILENCE.0
    } else {
        20.0 * amplitude.log10()
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidSampleRate,
    Backend(cpal::Error),
    SoundLimitReached,
    SoundInitialization,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSampleRate => formatter.write_str("sample rate must be greater than zero"),
            Self::Backend(error) => write!(formatter, "could not start the audio backend: {error}"),
            Self::SoundLimitReached => formatter.write_str("the active sound limit was reached"),
            Self::SoundInitialization => formatter.write_str("the sound could not be initialized"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Backend(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn playback_values_are_finite_and_bounded() {
        assert_eq!(normalize_volume(-1.0), 0.0);
        assert_eq!(normalize_volume(2.0), 1.0);
        assert_eq!(normalize_volume(f32::NAN), 1.0);
        assert_eq!(normalize_panning(-2.0), -1.0);
        assert_eq!(normalize_panning(2.0), 1.0);
        assert_eq!(normalize_panning(f32::NAN), 0.0);
        assert_eq!(normalize_panning(f32::INFINITY), 0.0);
    }
}
