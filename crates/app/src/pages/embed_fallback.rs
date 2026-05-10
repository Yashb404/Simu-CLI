use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api;

#[component]
pub fn EmbedFallbackPage() -> impl IntoView {
    let params = use_params_map();
    let demo_id = move || {
        params
            .read()
            .get("id")
            .unwrap_or_else(|| "unknown".to_string())
    };

    let embed_src = Signal::derive(move || {
        let base = api::api_base();
        let id = demo_id();
        format!("{base}/embed-runtime/index.html?demo_id={id}&api_base={base}")
    });

    view! {
        <main style="width:100vw;height:100vh;overflow:hidden;background:#050505;">
            <iframe
                src=move || embed_src.get()
                title="SimuCLI Embed"
                referrerpolicy="strict-origin-when-cross-origin"
                style="display:block;width:100%;height:100%;border:0;background:#050505;"
            />
        </main>
    }
}
