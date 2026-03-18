pub mod app;
pub mod api;
pub mod components;
pub mod pages;

pub use app::App;

#[cfg(target_arch = "wasm32")]
pub fn mount() {
    leptos::mount::mount_to_body(App);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    mount();
}
