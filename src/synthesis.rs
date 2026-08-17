use std::{f32::consts::TAU, sync::Arc, time::Duration};

use kira::Frame;

use crate::recipes::{Filter, Layer, Noise, Shimmer, Sound, Tone, Waveform};

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn frame_duration(frame_count: usize, sample_rate: u32) -> Duration {
    Duration::from_secs_f64(frame_count as f64 / f64::from(sample_rate))
}

pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
const SOURCE_STOP_PADDING: f32 = 0.05;
const INAUDIBLE_GAIN: f32 = 0.001;
const OUTPUT_GAIN: f32 = 4.0;

#[derive(Debug, Clone)]
pub struct RenderedSound {
    sample_rate: u32,
    frames: Arc<[Frame]>,
}

impl RenderedSound {
    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[must_use]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    #[must_use]
    pub fn duration(&self) -> Duration {
        frame_duration(self.frames.len(), self.sample_rate)
    }

    pub(crate) fn shared_frames(&self) -> Arc<[Frame]> {
        self.frames.clone()
    }
}

#[must_use]
/// Renders one built-in sound at the given sample rate.
///
/// # Panics
///
/// Panics when `sample_rate` is zero.
pub fn render(sound: Sound, sample_rate: u32) -> RenderedSound {
    assert!(sample_rate > 0, "sample rate must be greater than zero");
    let recipe = sound.recipe();
    let source_end = recipe
        .layers
        .iter()
        .map(|layer| layer.offset() + layer.attack() + layer.decay() + SOURCE_STOP_PADDING)
        .fold(0.0_f32, f32::max);
    let duration = source_end + recipe.shimmer.map_or(0.0, shimmer_tail);
    let frame_count = seconds_to_frames(duration, sample_rate).max(1);
    let mut samples = vec![0.0; frame_count];

    for (layer_index, layer) in recipe.layers.iter().copied().enumerate() {
        match layer {
            Layer::Tone(tone) => render_tone(&mut samples, sample_rate, tone),
            Layer::Noise(noise) => {
                render_noise(
                    &mut samples,
                    sample_rate,
                    noise,
                    noise_seed(sound, layer_index),
                );
            }
        }
    }

    for sample in &mut samples {
        *sample *= recipe.master_gain;
    }
    if let Some(shimmer) = recipe.shimmer {
        apply_shimmer(&mut samples, sample_rate, source_end, shimmer);
    }
    let frames = samples
        .into_iter()
        .map(|sample| Frame::from_mono(limit(sample * OUTPUT_GAIN)))
        .collect::<Vec<_>>()
        .into();

    RenderedSound {
        sample_rate,
        frames,
    }
}

#[allow(clippy::cast_precision_loss)]
fn render_tone(output: &mut [f32], sample_rate: u32, tone: Tone) {
    let start = seconds_to_frames(tone.offset, sample_rate);
    let audible_frames = seconds_to_frames(tone.attack + tone.decay, sample_rate);
    let detune = 2.0_f32.powf(tone.detune / 1200.0);
    let mut phase = 0.0_f32;

    for frame in 0..audible_frames {
        let target = start + frame;
        if target >= output.len() {
            break;
        }
        let time = frame as f32 / sample_rate as f32;
        let frequency = glide_frequency(tone, time) * detune;
        phase = (phase + frequency / sample_rate as f32).fract();
        let oscillator = match tone.waveform {
            Waveform::Sine => (phase * TAU).sin(),
            Waveform::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
        };
        output[target] += oscillator * envelope(time, tone.attack, tone.decay, tone.peak);
    }
}

#[allow(clippy::cast_precision_loss)]
fn render_noise(output: &mut [f32], sample_rate: u32, noise: Noise, seed: u64) {
    let start = seconds_to_frames(noise.offset, sample_rate);
    let audible_frames = seconds_to_frames(noise.attack + noise.decay, sample_rate);
    let mut generator = NoiseGenerator::new(seed);
    let mut filter = Biquad::new(noise.filter, noise.frequency, noise.q, sample_rate);

    for frame in 0..audible_frames {
        let target = start + frame;
        if target >= output.len() {
            break;
        }
        let time = frame as f32 / sample_rate as f32;
        let filtered = filter.process(generator.next());
        output[target] += filtered * envelope(time, noise.attack, noise.decay, noise.peak);
    }
}

fn glide_frequency(tone: Tone, time: f32) -> f32 {
    let Some(target) = tone.glide_to else {
        return tone.frequency;
    };
    let glide_time = tone.glide_time.unwrap_or(tone.attack + tone.decay);
    let progress = (time / glide_time).clamp(0.0, 1.0);
    tone.frequency * (target / tone.frequency).powf(progress)
}

fn envelope(time: f32, attack: f32, decay: f32, peak: f32) -> f32 {
    const FLOOR: f32 = 0.0001;
    if time < attack {
        if attack == 0.0 {
            return peak;
        }
        return FLOOR * (peak / FLOOR).powf(time / attack);
    }
    if time < attack + decay {
        if decay == 0.0 {
            return 0.0;
        }
        return peak * (FLOOR / peak).powf((time - attack) / decay);
    }
    0.0
}

fn apply_shimmer(output: &mut [f32], sample_rate: u32, source_end: f32, shimmer: Shimmer) {
    let delay_frames = seconds_to_frames(shimmer.delay, sample_rate).max(1);
    let source_frames = seconds_to_frames(source_end, sample_rate).min(output.len());
    let dry = output[..source_frames].to_vec();
    let mut delay = vec![0.0_f32; delay_frames];
    let mut position = 0;
    let mut filter = Biquad::new(Filter::Lowpass, shimmer.lowpass, 0.707, sample_rate);

    for (frame, sample) in output.iter_mut().enumerate() {
        let delayed = delay[position];
        let filtered = filter.process(delayed);
        let input = dry.get(frame).copied().unwrap_or(0.0);
        delay[position] = input + filtered * shimmer.feedback;
        *sample += filtered * shimmer.wet;
        position = (position + 1) % delay_frames;
    }
}

fn shimmer_tail(shimmer: Shimmer) -> f32 {
    if shimmer.feedback <= 0.0 {
        return 0.0;
    }
    if shimmer.feedback >= 1.0 {
        return shimmer.delay;
    }
    shimmer.delay * (1.0 + (INAUDIBLE_GAIN.ln() / shimmer.feedback.ln()).ceil())
}

fn limit(sample: f32) -> f32 {
    const THRESHOLD: f32 = 0.398_107_17;
    const RATIO: f32 = 12.0;
    let magnitude = sample.abs();
    if magnitude <= THRESHOLD {
        return sample;
    }
    sample.signum() * (THRESHOLD + (magnitude - THRESHOLD) / RATIO).min(1.0)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn seconds_to_frames(seconds: f32, sample_rate: u32) -> usize {
    (seconds * sample_rate as f32).ceil() as usize
}

fn noise_seed(sound: Sound, layer_index: usize) -> u64 {
    0x9e37_79b9_7f4a_7c15 ^ ((sound.index() as u64 + 1) << 32) ^ layer_index as u64
}

struct NoiseGenerator(u64);

impl NoiseGenerator {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    #[allow(clippy::cast_precision_loss)]
    fn next(&mut self) -> f32 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        let unit = (value >> 40) as f32 / ((1_u32 << 24) - 1) as f32;
        unit * 2.0 - 1.0
    }
}

struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    #[allow(clippy::cast_precision_loss)]
    fn new(filter: Filter, frequency: f32, q: f32, sample_rate: u32) -> Self {
        let nyquist = sample_rate as f32 * 0.5;
        let omega = TAU * frequency.clamp(1.0, nyquist * 0.99) / sample_rate as f32;
        let cosine = omega.cos();
        let sine = omega.sin();
        let alpha = sine / (2.0 * q.max(0.001));
        let a0 = 1.0 + alpha;
        let (b0, b1, b2) = match filter {
            Filter::Lowpass => ((1.0 - cosine) * 0.5, 1.0 - cosine, (1.0 - cosine) * 0.5),
            Filter::Bandpass => (alpha, 0.0, -alpha),
        };
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: -2.0 * cosine / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}
