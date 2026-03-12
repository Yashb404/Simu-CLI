pub mod animation;
pub mod api;
pub mod components;
pub mod input_handler;
pub mod matching;
pub mod messaging;

use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[component]
pub fn EmbedApp() -> impl IntoView {
    view! {
        <components::terminal::TerminalUI />
    }
}

#[cfg(target_arch = "wasm32")]
pub fn mount() {
    leptos::mount::mount_to_body(EmbedApp);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct CliSimulator {
    container_selector: String,
    demo_id: String,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl CliSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new(container_selector: String, demo_id: String) -> Self {
        Self {
            container_selector,
            demo_id,
        }
    }

    pub fn start(&self) {
        let _ = (&self.container_selector, &self.demo_id);
        mount();
    }
}
