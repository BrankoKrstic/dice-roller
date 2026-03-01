pub mod app;
pub mod client;
pub mod dsl;
pub mod shared;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::client::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
