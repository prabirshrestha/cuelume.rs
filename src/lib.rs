//! Curated interaction sounds synthesized from code.
//!
//! `cuelume` contains no audio assets. It renders its built-in sound recipes to
//! PCM and uses Kira for low-latency, overlapping playback on macOS, Linux,
//! Windows, and WebAssembly.
//!
//! ```no_run
//! use cuelume::{Player, Sound};
//!
//! let mut player = Player::new()?;
//! player.play(Sound::Success)?;
//! # Ok::<(), cuelume::Error>(())
//! ```

mod player;
mod recipes;
mod synthesis;

pub use player::{Error, PlayOptions, Player, PlayerConfig};
pub use recipes::{ALL_SOUNDS, ParseSoundError, Sound};
pub use synthesis::{DEFAULT_SAMPLE_RATE, RenderedSound, render};
