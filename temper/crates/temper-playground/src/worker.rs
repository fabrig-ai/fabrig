//! The worker module's entry (temper/docs/design/playground.md, 3; skein/docs/design/playground.md, 7).
use crate::{
    machine::Machine,
    programs::{self, Process},
    story,
};
use skein_lib::Time;
use skein_playground::slices::Slices;
use skein_ui_web::bridge;

/// Run the worker's one world through skein's messages, slices and timer.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    skein_playground::worker::run(make);
}

fn make(start: &bridge::Start, browser: Time) -> Slices<Process<()>, Machine> {
    let slices = story::start_slices(start, browser)
        .unwrap_or_else(|| panic!("unknown playground story: {}", String::from_utf8_lossy(&start.story)));
    programs::install(story::parameters(slices.world()));
    slices
}
