use std::{fmt, str::FromStr};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Sound {
    #[default]
    Chime,
    Sparkle,
    Droplet,
    Bloom,
    Whisper,
    Tick,
    Press,
    Release,
    Toggle,
    Success,
    Error,
    Page,
    Loading,
    Ready,
    Pulse,
    Scan,
    Arrival,
}

pub const ALL_SOUNDS: [Sound; 17] = [
    Sound::Chime,
    Sound::Sparkle,
    Sound::Droplet,
    Sound::Bloom,
    Sound::Whisper,
    Sound::Tick,
    Sound::Press,
    Sound::Release,
    Sound::Toggle,
    Sound::Success,
    Sound::Error,
    Sound::Page,
    Sound::Loading,
    Sound::Ready,
    Sound::Pulse,
    Sound::Scan,
    Sound::Arrival,
];

impl Sound {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chime => "chime",
            Self::Sparkle => "sparkle",
            Self::Droplet => "droplet",
            Self::Bloom => "bloom",
            Self::Whisper => "whisper",
            Self::Tick => "tick",
            Self::Press => "press",
            Self::Release => "release",
            Self::Toggle => "toggle",
            Self::Success => "success",
            Self::Error => "error",
            Self::Page => "page",
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Pulse => "pulse",
            Self::Scan => "scan",
            Self::Arrival => "arrival",
        }
    }

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    pub(crate) const fn recipe(self) -> &'static Recipe {
        &RECIPES[self.index()]
    }
}

impl fmt::Display for Sound {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Sound {
    type Err = ParseSoundError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ALL_SOUNDS
            .into_iter()
            .find(|sound| sound.as_str() == value)
            .ok_or_else(|| ParseSoundError(value.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseSoundError(String);

impl fmt::Display for ParseSoundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown Cuelume sound: {}", self.0)
    }
}

impl std::error::Error for ParseSoundError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Recipe {
    pub master_gain: f32,
    pub layers: &'static [Layer],
    pub shimmer: Option<Shimmer>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Layer {
    Tone(Tone),
    Noise(Noise),
}

impl Layer {
    pub const fn offset(self) -> f32 {
        match self {
            Self::Tone(layer) => layer.offset,
            Self::Noise(layer) => layer.offset,
        }
    }

    pub const fn attack(self) -> f32 {
        match self {
            Self::Tone(layer) => layer.attack,
            Self::Noise(layer) => layer.attack,
        }
    }

    pub const fn decay(self) -> f32 {
        match self {
            Self::Tone(layer) => layer.decay,
            Self::Noise(layer) => layer.decay,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Tone {
    pub waveform: Waveform,
    pub frequency: f32,
    pub detune: f32,
    pub glide_to: Option<f32>,
    pub glide_time: Option<f32>,
    pub offset: f32,
    pub attack: f32,
    pub decay: f32,
    pub peak: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Noise {
    pub filter: Filter,
    pub frequency: f32,
    pub q: f32,
    pub offset: f32,
    pub attack: f32,
    pub decay: f32,
    pub peak: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Shimmer {
    pub delay: f32,
    pub feedback: f32,
    pub wet: f32,
    pub lowpass: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Waveform {
    Sine,
    Triangle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Filter {
    Lowpass,
    Bandpass,
}

const fn tone(
    waveform: Waveform,
    frequency: f32,
    offset: f32,
    attack: f32,
    decay: f32,
    peak: f32,
) -> Layer {
    Layer::Tone(Tone {
        waveform,
        frequency,
        detune: 0.0,
        glide_to: None,
        glide_time: None,
        offset,
        attack,
        decay,
        peak,
    })
}

const fn detuned_tone(
    waveform: Waveform,
    frequency: f32,
    detune: f32,
    offset: f32,
    attack: f32,
    decay: f32,
    peak: f32,
) -> Layer {
    Layer::Tone(Tone {
        waveform,
        frequency,
        detune,
        glide_to: None,
        glide_time: None,
        offset,
        attack,
        decay,
        peak,
    })
}

#[allow(clippy::too_many_arguments)]
const fn glide(
    waveform: Waveform,
    frequency: f32,
    glide_to: f32,
    glide_time: f32,
    offset: f32,
    attack: f32,
    decay: f32,
    peak: f32,
) -> Layer {
    Layer::Tone(Tone {
        waveform,
        frequency,
        detune: 0.0,
        glide_to: Some(glide_to),
        glide_time: Some(glide_time),
        offset,
        attack,
        decay,
        peak,
    })
}

const fn noise(
    filter: Filter,
    frequency: f32,
    q: f32,
    offset: f32,
    attack: f32,
    decay: f32,
    peak: f32,
) -> Layer {
    Layer::Noise(Noise {
        filter,
        frequency,
        q,
        offset,
        attack,
        decay,
        peak,
    })
}

const fn shimmer(delay: f32, feedback: f32, wet: f32, lowpass: f32) -> Shimmer {
    Shimmer {
        delay,
        feedback,
        wet,
        lowpass,
    }
}

const CHIME: &[Layer] = &[
    tone(Waveform::Sine, 1046.5, 0.0, 0.006, 0.22, 0.09),
    tone(Waveform::Sine, 1568.0, 0.09, 0.006, 0.26, 0.08),
];
const SPARKLE: &[Layer] = &[
    tone(Waveform::Sine, 1760.0, 0.0, 0.003, 0.09, 0.045),
    tone(Waveform::Sine, 2217.0, 0.045, 0.003, 0.09, 0.04),
    tone(Waveform::Sine, 2637.0, 0.09, 0.003, 0.1, 0.038),
    tone(Waveform::Sine, 3520.0, 0.135, 0.003, 0.12, 0.032),
];
const DROPLET: &[Layer] = &[glide(
    Waveform::Sine,
    1200.0,
    550.0,
    0.14,
    0.0,
    0.004,
    0.2,
    0.075,
)];
const BLOOM: &[Layer] = &[
    tone(Waveform::Sine, 528.0, 0.0, 0.06, 0.32, 0.06),
    detuned_tone(Waveform::Sine, 528.0, 12.0, 0.0, 0.06, 0.34, 0.05),
];
const WHISPER: &[Layer] = &[
    noise(Filter::Lowpass, 1600.0, 0.7, 0.0, 0.025, 0.13, 0.04),
    glide(Waveform::Sine, 880.0, 660.0, 0.14, 0.01, 0.012, 0.14, 0.025),
];
const TICK: &[Layer] = &[
    noise(Filter::Bandpass, 5400.0, 1.8, 0.0, 0.001, 0.018, 0.14),
    tone(Waveform::Sine, 2600.0, 0.0, 0.001, 0.012, 0.018),
];
const PRESS: &[Layer] = &[noise(Filter::Bandpass, 1700.0, 1.4, 0.0, 0.001, 0.02, 0.13)];
const RELEASE: &[Layer] = &[
    noise(Filter::Bandpass, 4600.0, 1.8, 0.0, 0.001, 0.016, 0.12),
    tone(Waveform::Sine, 3200.0, 0.006, 0.001, 0.05, 0.02),
];
const TOGGLE: &[Layer] = &[
    noise(Filter::Bandpass, 2200.0, 1.6, 0.0, 0.001, 0.016, 0.12),
    noise(Filter::Bandpass, 3800.0, 1.6, 0.024, 0.001, 0.02, 0.1),
];
const SUCCESS: &[Layer] = &[
    tone(Waveform::Sine, 880.0, 0.0, 0.004, 0.09, 0.06),
    tone(Waveform::Sine, 1108.73, 0.06, 0.004, 0.1, 0.06),
    tone(Waveform::Sine, 1318.51, 0.12, 0.004, 0.18, 0.07),
];
const ERROR: &[Layer] = &[
    noise(Filter::Bandpass, 850.0, 1.1, 0.0, 0.001, 0.035, 0.13),
    tone(Waveform::Triangle, 440.0, 0.025, 0.004, 0.09, 0.045),
    tone(Waveform::Triangle, 349.23, 0.1, 0.004, 0.14, 0.04),
];
const PAGE: &[Layer] = &[
    noise(Filter::Lowpass, 1800.0, 0.7, 0.0, 0.006, 0.08, 0.11),
    noise(Filter::Bandpass, 4200.0, 1.2, 0.04, 0.004, 0.065, 0.08),
    tone(Waveform::Sine, 2400.0, 0.075, 0.002, 0.045, 0.02),
];
const LOADING: &[Layer] = &[
    noise(Filter::Lowpass, 1400.0, 0.6, 0.0, 0.035, 0.14, 0.035),
    glide(Waveform::Sine, 420.0, 630.0, 0.18, 0.0, 0.025, 0.18, 0.05),
];
const READY: &[Layer] = &[
    noise(Filter::Bandpass, 3600.0, 1.8, 0.0, 0.001, 0.02, 0.11),
    glide(
        Waveform::Triangle,
        330.0,
        660.0,
        0.12,
        0.012,
        0.004,
        0.16,
        0.055,
    ),
    tone(Waveform::Sine, 990.0, 0.13, 0.004, 0.22, 0.06),
];
const PULSE: &[Layer] = &[
    noise(Filter::Bandpass, 2600.0, 2.4, 0.0, 0.001, 0.022, 0.08),
    glide(
        Waveform::Triangle,
        620.0,
        1240.0,
        0.07,
        0.0,
        0.002,
        0.085,
        0.055,
    ),
];
const SCAN: &[Layer] = &[
    tone(Waveform::Sine, 740.0, 0.0, 0.002, 0.055, 0.05),
    tone(Waveform::Sine, 1110.0, 0.045, 0.002, 0.055, 0.045),
    tone(Waveform::Sine, 1665.0, 0.09, 0.002, 0.07, 0.04),
];
const ARRIVAL: &[Layer] = &[
    noise(Filter::Lowpass, 900.0, 0.8, 0.0, 0.05, 0.24, 0.035),
    glide(Waveform::Sine, 220.0, 440.0, 0.32, 0.0, 0.04, 0.34, 0.055),
    tone(Waveform::Sine, 659.25, 0.12, 0.045, 0.32, 0.04),
    tone(Waveform::Sine, 987.77, 0.19, 0.045, 0.34, 0.032),
];

const RECIPES: [Recipe; 17] = [
    Recipe {
        master_gain: 0.5,
        layers: CHIME,
        shimmer: Some(shimmer(0.12, 0.25, 0.18, 4000.0)),
    },
    Recipe {
        master_gain: 0.5,
        layers: SPARKLE,
        shimmer: Some(shimmer(0.07, 0.35, 0.22, 6000.0)),
    },
    Recipe {
        master_gain: 0.55,
        layers: DROPLET,
        shimmer: Some(shimmer(0.09, 0.2, 0.15, 3000.0)),
    },
    Recipe {
        master_gain: 0.5,
        layers: BLOOM,
        shimmer: Some(shimmer(0.15, 0.2, 0.12, 2500.0)),
    },
    Recipe {
        master_gain: 0.48,
        layers: WHISPER,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.4,
        layers: TICK,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.4,
        layers: PRESS,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.4,
        layers: RELEASE,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.4,
        layers: TOGGLE,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.5,
        layers: SUCCESS,
        shimmer: Some(shimmer(0.1, 0.22, 0.16, 4500.0)),
    },
    Recipe {
        master_gain: 0.42,
        layers: ERROR,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.38,
        layers: PAGE,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.42,
        layers: LOADING,
        shimmer: Some(shimmer(0.11, 0.18, 0.12, 2800.0)),
    },
    Recipe {
        master_gain: 0.48,
        layers: READY,
        shimmer: Some(shimmer(0.1, 0.16, 0.1, 4200.0)),
    },
    Recipe {
        master_gain: 0.42,
        layers: PULSE,
        shimmer: None,
    },
    Recipe {
        master_gain: 0.4,
        layers: SCAN,
        shimmer: Some(shimmer(0.065, 0.16, 0.1, 4200.0)),
    },
    Recipe {
        master_gain: 0.44,
        layers: ARRIVAL,
        shimmer: Some(shimmer(0.16, 0.28, 0.18, 3200.0)),
    },
];

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;

    const UPSTREAM_REVISION: &str = "5c6c20f50cf78c68f31da12fdebdbc1e55cc7071";

    #[test]
    fn palette_matches_upstream_recipe_manifest() {
        assert_eq!(UPSTREAM_REVISION.len(), 40);
        assert_eq!(ALL_SOUNDS.len(), RECIPES.len());
        assert_eq!(
            RECIPES
                .iter()
                .map(|recipe| recipe.layers.len())
                .sum::<usize>(),
            41
        );
        assert_eq!(
            RECIPES
                .iter()
                .filter(|recipe| recipe.shimmer.is_some())
                .count(),
            9
        );
        assert_eq!(manifest(), EXPECTED_MANIFEST);
    }

    fn manifest() -> String {
        let mut output = String::new();
        for sound in ALL_SOUNDS {
            let recipe = sound.recipe();
            write!(output, "{}|{}", sound.as_str(), recipe.master_gain).unwrap();
            for layer in recipe.layers {
                match layer {
                    Layer::Tone(tone) => write!(
                        output,
                        "|t,{},{},{},{},{},{},{},{},{}",
                        waveform_name(tone.waveform),
                        tone.frequency,
                        tone.detune,
                        optional_number(tone.glide_to),
                        optional_number(tone.glide_time),
                        tone.offset,
                        tone.attack,
                        tone.decay,
                        tone.peak
                    )
                    .unwrap(),
                    Layer::Noise(noise) => write!(
                        output,
                        "|n,{},{},{},{},{},{},{}",
                        filter_name(noise.filter),
                        noise.frequency,
                        noise.q,
                        noise.offset,
                        noise.attack,
                        noise.decay,
                        noise.peak
                    )
                    .unwrap(),
                }
            }
            match recipe.shimmer {
                Some(shimmer) => write!(
                    output,
                    "|s,{},{},{},{}",
                    shimmer.delay, shimmer.feedback, shimmer.wet, shimmer.lowpass
                )
                .unwrap(),
                None => output.push_str("|s,-"),
            }
            output.push('\n');
        }
        output
    }

    fn waveform_name(waveform: Waveform) -> &'static str {
        match waveform {
            Waveform::Sine => "sine",
            Waveform::Triangle => "triangle",
        }
    }

    fn filter_name(filter: Filter) -> &'static str {
        match filter {
            Filter::Lowpass => "lowpass",
            Filter::Bandpass => "bandpass",
        }
    }

    fn optional_number(value: Option<f32>) -> String {
        value.map_or_else(|| "-".to_owned(), |value| value.to_string())
    }

    const EXPECTED_MANIFEST: &str = concat!(
        "chime|0.5|t,sine,1046.5,0,-,-,0,0.006,0.22,0.09|t,sine,1568,0,-,-,0.09,0.006,0.26,0.08|s,0.12,0.25,0.18,4000\n",
        "sparkle|0.5|t,sine,1760,0,-,-,0,0.003,0.09,0.045|t,sine,2217,0,-,-,0.045,0.003,0.09,0.04|t,sine,2637,0,-,-,0.09,0.003,0.1,0.038|t,sine,3520,0,-,-,0.135,0.003,0.12,0.032|s,0.07,0.35,0.22,6000\n",
        "droplet|0.55|t,sine,1200,0,550,0.14,0,0.004,0.2,0.075|s,0.09,0.2,0.15,3000\n",
        "bloom|0.5|t,sine,528,0,-,-,0,0.06,0.32,0.06|t,sine,528,12,-,-,0,0.06,0.34,0.05|s,0.15,0.2,0.12,2500\n",
        "whisper|0.48|n,lowpass,1600,0.7,0,0.025,0.13,0.04|t,sine,880,0,660,0.14,0.01,0.012,0.14,0.025|s,-\n",
        "tick|0.4|n,bandpass,5400,1.8,0,0.001,0.018,0.14|t,sine,2600,0,-,-,0,0.001,0.012,0.018|s,-\n",
        "press|0.4|n,bandpass,1700,1.4,0,0.001,0.02,0.13|s,-\n",
        "release|0.4|n,bandpass,4600,1.8,0,0.001,0.016,0.12|t,sine,3200,0,-,-,0.006,0.001,0.05,0.02|s,-\n",
        "toggle|0.4|n,bandpass,2200,1.6,0,0.001,0.016,0.12|n,bandpass,3800,1.6,0.024,0.001,0.02,0.1|s,-\n",
        "success|0.5|t,sine,880,0,-,-,0,0.004,0.09,0.06|t,sine,1108.73,0,-,-,0.06,0.004,0.1,0.06|t,sine,1318.51,0,-,-,0.12,0.004,0.18,0.07|s,0.1,0.22,0.16,4500\n",
        "error|0.42|n,bandpass,850,1.1,0,0.001,0.035,0.13|t,triangle,440,0,-,-,0.025,0.004,0.09,0.045|t,triangle,349.23,0,-,-,0.1,0.004,0.14,0.04|s,-\n",
        "page|0.38|n,lowpass,1800,0.7,0,0.006,0.08,0.11|n,bandpass,4200,1.2,0.04,0.004,0.065,0.08|t,sine,2400,0,-,-,0.075,0.002,0.045,0.02|s,-\n",
        "loading|0.42|n,lowpass,1400,0.6,0,0.035,0.14,0.035|t,sine,420,0,630,0.18,0,0.025,0.18,0.05|s,0.11,0.18,0.12,2800\n",
        "ready|0.48|n,bandpass,3600,1.8,0,0.001,0.02,0.11|t,triangle,330,0,660,0.12,0.012,0.004,0.16,0.055|t,sine,990,0,-,-,0.13,0.004,0.22,0.06|s,0.1,0.16,0.1,4200\n",
        "pulse|0.42|n,bandpass,2600,2.4,0,0.001,0.022,0.08|t,triangle,620,0,1240,0.07,0,0.002,0.085,0.055|s,-\n",
        "scan|0.4|t,sine,740,0,-,-,0,0.002,0.055,0.05|t,sine,1110,0,-,-,0.045,0.002,0.055,0.045|t,sine,1665,0,-,-,0.09,0.002,0.07,0.04|s,0.065,0.16,0.1,4200\n",
        "arrival|0.44|n,lowpass,900,0.8,0,0.05,0.24,0.035|t,sine,220,0,440,0.32,0,0.04,0.34,0.055|t,sine,659.25,0,-,-,0.12,0.045,0.32,0.04|t,sine,987.77,0,-,-,0.19,0.045,0.34,0.032|s,0.16,0.28,0.18,3200\n"
    );
}
