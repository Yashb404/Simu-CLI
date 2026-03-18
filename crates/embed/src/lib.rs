pub mod animation;
pub mod api;
pub mod components;
pub mod input_handler;
pub mod matching;
pub mod messaging;

use leptos::prelude::*;

use shared::dto::PublicDemoResponse;

#[cfg(target_arch = "wasm32")]
fn query_param_value(key: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    let query = search.trim_start_matches('?');
    for pair in query.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if k == key {
            return Some(v.to_string());
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[component]
pub fn EmbedApp() -> impl IntoView {
    let (demo, _set_demo) = signal(Option::<Result<PublicDemoResponse, String>>::None);

    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        let set_demo = _set_demo;
        let Some(demo_id) = query_param_value("demo_id") else {
            set_demo.set(Some(Err("Missing demo_id query parameter".to_string())));
            return;
        };

        let api_base = query_param_value("api_base")
            .or_else(|| web_sys::window().and_then(|window| window.location().origin().ok()))
            .unwrap_or_default();
        let endpoint = format!("{api_base}/api/demos/{demo_id}/public");

        leptos::task::spawn_local({
            async move {
                match api::fetch_public_demo(&endpoint).await {
                    Ok(public_demo) => {
                        set_demo.set(Some(Ok(public_demo)));
                    }
                    Err(err) => {
                        set_demo.set(Some(Err(err.to_string())));
                    }
                }
            }
        });
    });

    view! {
        <components::terminal::TerminalUI demo=demo />
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
