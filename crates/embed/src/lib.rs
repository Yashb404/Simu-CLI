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
    fn decode_component(value: &str) -> String {
        js_sys::decode_uri_component(value)
            .ok()
            .and_then(|s| s.as_string())
            .unwrap_or_else(|| value.to_string())
    }

    let search = web_sys::window()?.location().search().ok()?;
    let query = search.trim_start_matches('?');
    for pair in query.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if decode_component(k) == key {
            return Some(decode_component(v));
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[component]
pub fn EmbedApp() -> impl IntoView {
    let (demo, _set_demo) = signal(Option::<PublicDemoResponse>::None);
    let (status, _set_status) = signal(String::new());

    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        let set_demo = _set_demo;
        let set_status = _set_status;
        let Some(demo_id) = query_param_value("demo_id") else {
            set_status.set("Missing demo_id query parameter".to_string());
            return;
        };

        let api_base = query_param_value("api_base")
            .or_else(|| web_sys::window().and_then(|window| window.location().origin().ok()))
            .unwrap_or_default();
        let endpoint = format!("{api_base}/api/public/demos/{demo_id}");

        leptos::task::spawn_local({
            let set_demo = set_demo;
            let set_status = set_status;
            async move {
                match api::fetch_public_demo(&endpoint).await {
                    Ok(public_demo) => {
                        set_demo.set(Some(public_demo));
                        set_status.set(String::new());
                    }
                    Err(err) => {
                        set_status.set(err.to_string());
                    }
                }
            }
        });
    });

    view! {
        <main class="embed-wrapper" style="width:100%;height:100%;background:#050505;color:#00ff41;font-family:'IBM Plex Mono',monospace;">
            <Show
                when=move || demo.get().is_some()
                fallback=move || {
                    view! {
                        <div class="terminal-chrome" style="height:100%;border:none;">
                            <div class="terminal-titlebar">
                                <div class="terminal-dots">
                                    <span class="terminal-dot red"></span>
                                    <span class="terminal-dot yellow"></span>
                                    <span class="terminal-dot green"></span>
                                </div>
                                <span class="terminal-titlebar-text">"CLI Demo Studio"</span>
                            </div>
                            <div class="terminal-body" style="padding:16px;">
                                <p class="terminal-line">"$ init runtime..."</p>
                                <p class="terminal-line">"$ loading demo..."</p>
                                <Show when=move || !status.get().is_empty()>
                                    <p class="terminal-line" style="color:#ff3333;">{move || format!("! error: {}", status.get())}</p>
                                </Show>
                                <p class="terminal-line" style="opacity:0.5;">"_"</p>
                            </div>
                        </div>
                    }
                }
            >
                {move || {
                    let demo_data = demo.get().unwrap();
                    view! { <components::terminal::TerminalUI demo=demo_data /> }
                }}
            </Show>
        </main>
    }
}

#[cfg(target_arch = "wasm32")]
pub fn mount() {
    leptos::mount::mount_to_body(EmbedApp);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    mount();
}
