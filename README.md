# Cuelume for Rust

Curated interaction sounds for Rust applications. The sounds are synthesized
from code, cached in memory, and played through Kira. The crate includes no
WAV, MP3, or other audio assets.

The palette contains 17 sounds:

`chime`, `sparkle`, `droplet`, `bloom`, `whisper`, `tick`, `press`, `release`,
`toggle`, `success`, `error`, `page`, `loading`, `ready`, `pulse`, `scan`, and
`arrival`.

## Platforms

- macOS through CoreAudio
- Linux through ALSA, with CPAL platform options
- Windows through WASAPI
- WebAssembly through the Web Audio API

## Playback

Add Cuelume to a Rust application:

```sh
cargo add cuelume
```

```rust
use cuelume::{PlayOptions, Player, Sound};

let mut player = Player::new()?;
player.play(Sound::Success)?;
player.play_with(
    Sound::Tick,
    PlayOptions {
        volume: 0.7,
        panning: -0.25,
    },
)?;
# Ok::<(), cuelume::Error>(())
```

`Player` generates and caches the full palette once. Calls to `play` can
overlap. The returned Kira handle can stop or modify an active sound.

## WebAssembly

The same published `cuelume` crate works in browser applications. Add it to
the Rust crate that compiles to `wasm32-unknown-unknown`:

```sh
cargo add cuelume
```

No Cuelume feature is required. Your application still needs its normal WASM
toolchain, such as Trunk, wasm-pack, or wasm-bindgen.

Create and retain `Player` from a click, key press, or another browser user
gesture. A browser can reject Web Audio startup when `Player::new` runs during
initial page loading. The player must remain alive while its sounds play.

```rust,ignore
use std::cell::RefCell;

use cuelume::{Player, Sound};

thread_local! {
    static PLAYER: RefCell<Option<Player>> = const { RefCell::new(None) };
}

// Run this from a click, key press, or another browser user gesture.
PLAYER.with(|player| -> Result<(), cuelume::Error> {
    let mut player = player.borrow_mut();
    let player = player.get_or_insert(Player::new()?);
    player.play(Sound::Success)?;
    Ok(())
})?;
```

The repository also contains a complete browser example with buttons for all
17 sounds, plus volume and panning controls. Start it from the repository
root:

```sh
rustup target add wasm32-unknown-unknown
cargo run --features wasm-example --example wasm
```

Then open `http://127.0.0.1:8080`. You can pass a different address after
`--`:

```sh
cargo run --features wasm-example --example wasm -- 127.0.0.1:3000
```

Click any sound button to start Web Audio and play it. The example creates the
player during that first click so it complies with browser autoplay rules.
The `wasm-example` feature only builds this repository's native Axum launcher.
Applications that depend on Cuelume must not enable it.

## Render Without Playback

The renderer is independent from the audio device and is useful for tests,
previews, and exports.

```rust
let sound = cuelume::render(cuelume::Sound::Chime, 48_000);
assert!(!sound.frames().is_empty());
```

The core library only provides PCM rendering and playback. File encoding is
kept in the `play` example under the opt-in `tools` feature. WAV uses the Rust
`hound` crate. MP3 and audio-only MP4 use `ffmpeg` with LAME and AAC encoders.

Start the persistent Ratatui sound browser:

```sh
cargo run --features tools --example play
```

Use the keyboard to test sounds without restarting the audio engine:

```text
Up/Down or j/k    Select a sound
Enter or Space    Play the selected sound
/                 Search sounds by name
f                 Cycle WAV, MP3, and MP4/AAC
w                 Export in the selected format
-/+               Change volume
h/l               Change panning
0                 Reset volume and panning
q, Esc, or Ctrl-C Quit
```

While searching, type to filter, use Up/Down to select, press Enter to play,
and press Esc to clear the search.

The browser also shows the selected recipe's duration, frame count, and sample
rate. Export writes `<sound>.<format>` in the current directory. WAV export is
native Rust. MP3 and audio-only MP4 export require `ffmpeg` in `PATH`.

Play one sound directly:

```sh
cargo run --features tools --example play -- success
```

Pass optional volume (`0.0` to `1.0`) and panning (`-1.0` to `1.0`):

```sh
cargo run --features tools --example play -- tick 0.7 -0.25
```

List all available sounds:

```sh
cargo run --features tools --example play -- --list
```

## Attribution

The sound recipes are derived from the MIT-licensed
[Cuelume](https://github.com/Danilaa1/cuelume) web package. The Rust renderer
and Kira integration are implemented for this crate.
