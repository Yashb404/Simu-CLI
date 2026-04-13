use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_params_map;
use shared::dto::PublicDemoResponse;

use crate::api;
use embed::{EmbedConfig, components::terminal::TerminalUI};

#[component]
pub fn EmbedPlayerPage() -> impl IntoView {
    let params = use_params_map();
    let demo_ref = move || {
        params
            .read()
            .get("id")
            .unwrap_or_else(|| "unknown".to_string())
    };

    let (demo, set_demo) = signal(Option::<PublicDemoResponse>::None);
    let (status, set_status) = signal("Loading demo...".to_string());

    Effect::new(move |_| {
        let reference = demo_ref();
        if reference == "unknown" {
            set_status.set("Missing demo id".to_string());
            return;
        }

        spawn_local_scoped({
            let set_demo = set_demo;
            let set_status = set_status;
            async move {
                match api::get_public_demo(&reference).await {
                    Ok(public_demo) => {
                        set_status.set(String::new());
                        set_demo.set(Some(public_demo));
                    }
                    Err(err) => {
                        set_status.set(format!("Unable to load demo: {err}"));
                    }
                }
            }
        });
    });

    view! {
        <main style="width:100vw;height:100vh;overflow:hidden;background:#050505;color:#00ff41;">
            <Show
                when=move || demo.get().is_some()
                fallback=move || {
                    view! {
                        <div style="width:100%;height:100%;display:grid;place-items:center;font-family:'IBM Plex Mono',monospace;">
                            <div style="max-width:480px;padding:24px;border:1px solid rgba(0,255,65,0.25);background:#0e0e0e;">
                                <p style="margin-bottom:8px;text-transform:uppercase;letter-spacing:0.16em;">"SimuCLI Embed"</p>
                                <p style="color:#b6ffbf;">{move || status.get()}</p>
                            </div>
                        </div>
                    }
                }
            >
                {move || {
                    let demo_data = demo.get().unwrap();
                    view! {
                        <TerminalUI
                            demo=demo_data.clone()
                            config=EmbedConfig {
                                demo_id: demo_data.id.to_string(),
                                api_base: api::api_base(),
                            }
                        />
                    }
                }}
            </Show>
        </main>
    }
}