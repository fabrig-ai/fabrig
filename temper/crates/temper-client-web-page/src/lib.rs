//! The shipped page's entry (temper/docs/design/client/web/README.md, 2.1).
#![forbid(unsafe_code)]

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    temper_client_shell_web::start();
}
