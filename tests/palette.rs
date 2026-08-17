use std::{collections::HashSet, str::FromStr};

use cuelume::{ALL_SOUNDS, DEFAULT_SAMPLE_RATE, Sound, render};

#[test]
fn every_sound_round_trips_through_its_name() {
    assert_eq!(ALL_SOUNDS.len(), 17);
    for sound in ALL_SOUNDS {
        assert_eq!(Sound::from_str(sound.as_str()), Ok(sound));
    }
}

#[test]
fn every_sound_renders_finite_non_silent_audio() {
    for sound in ALL_SOUNDS {
        let rendered = render(sound, DEFAULT_SAMPLE_RATE);
        assert!(!rendered.frames().is_empty(), "{sound}");
        assert!(rendered.duration().as_millis() >= 60, "{sound}");
        assert!(
            rendered
                .frames()
                .iter()
                .all(|frame| frame.left.is_finite() && frame.right.is_finite()),
            "{sound}"
        );
        assert!(
            rendered
                .frames()
                .iter()
                .any(|frame| frame.left.abs() > 0.000_01),
            "{sound}"
        );
    }
}

#[test]
fn rendering_is_deterministic_and_each_recipe_is_distinct() {
    let mut fingerprints = HashSet::new();
    for sound in ALL_SOUNDS {
        let first = render(sound, DEFAULT_SAMPLE_RATE);
        let second = render(sound, DEFAULT_SAMPLE_RATE);
        assert_eq!(first.frames(), second.frames(), "{sound}");
        let fingerprint = first
            .frames()
            .iter()
            .step_by(97)
            .fold(0_u64, |hash, frame| {
                hash.rotate_left(5) ^ u64::from(frame.left.to_bits())
            });
        assert!(fingerprints.insert(fingerprint), "{sound}");
    }
}
