#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;

use cuelume::{ALL_SOUNDS, PlayOptions, Player, Sound};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{Document, HtmlInputElement};

thread_local! {
    static PLAYER: RefCell<Option<Player>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
/// Connects the page controls after the WebAssembly module loads.
///
/// # Errors
///
/// Returns an error when the document or a required sound button is missing.
pub fn start() -> Result<(), JsValue> {
    let document = document()?;

    for sound in ALL_SOUNDS {
        let button = document
            .get_element_by_id(&format!("sound-{sound}"))
            .ok_or_else(|| JsValue::from_str("a sound button is missing"))?;
        let on_click = Closure::<dyn FnMut()>::new(move || {
            let message = match play(sound) {
                Ok(()) => format!("Played {sound}"),
                Err(error) => format!("Could not play {sound}: {error}"),
            };
            set_status(&message);
        });
        button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
        on_click.forget();
    }

    Ok(())
}

fn play(sound: Sound) -> Result<(), String> {
    let document = document().map_err(|error| js_error(&error))?;
    let options = PlayOptions {
        volume: input_value(&document, "volume")?,
        panning: input_value(&document, "panning")?,
    };

    PLAYER.with(|player| {
        let mut player = player.borrow_mut();
        if player.is_none() {
            *player = Some(Player::new().map_err(|error| error.to_string())?);
        }
        player
            .as_mut()
            .expect("the player was initialized")
            .play_with(sound, options)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
}

fn input_value(document: &Document, id: &str) -> Result<f32, String> {
    document
        .get_element_by_id(id)
        .ok_or_else(|| format!("input `{id}` is missing"))?
        .dyn_into::<HtmlInputElement>()
        .map_err(|_| format!("element `{id}` is not an input"))?
        .value()
        .parse::<f32>()
        .map_err(|error| format!("input `{id}` is invalid: {error}"))
}

fn set_status(message: &str) {
    if let Ok(document) = document()
        && let Some(status) = document.get_element_by_id("status")
    {
        status.set_text_content(Some(message));
    }
}

fn document() -> Result<Document, JsValue> {
    web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("the browser document is unavailable"))
}

fn js_error(error: &JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "unknown JavaScript error".to_owned())
}
