use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use shared::services::embed_generator::{generate_iframe_snippet, generate_script_snippet};

use crate::api;

#[component]
pub fn DemoViewPage() -> impl IntoView {
    let query = use_query_map();
    let initial_demo_id = Signal::derive(move || {
        query
            .read()
            .get("id")
            .unwrap_or_else(|| "demo-id".to_string())
    });
    let (demo_id, set_demo_id) = signal(initial_demo_id.get());
    let (api_base, set_api_base) = signal(api::api_base());
    let page_origin = api::browser_origin();
    let iframe_origin = page_origin.clone();
    let script_origin = page_origin.clone();
    let embed_src = Signal::derive(move || {
        format!(
            "/embed/index.html?demo_id={}&api_base={}",
            demo_id.get(),
            api_base.get()
        )
    });

    let iframe_snippet = Signal::derive(move || {
        generate_iframe_snippet(
            &format!("{}/demo/view?id={}", iframe_origin, demo_id.get()),
            "100%",
            "480px",
        )
    });

    let script_snippet = Signal::derive(move || {
        generate_script_snippet(&format!("{}/embed/index.html", script_origin), &demo_id.get())
    });

    view! {
        <section class="page demo-view-page">
            <h2>"MVP Embed Test Page"</h2>
            <p>"Use this page to validate embed runtime behavior quickly tonight."</p>
            <div class="panel form-grid">
                <label>
                    "Demo ID"
                    <input
                        prop:value=move || demo_id.get()
                        on:input=move |ev| set_demo_id.set(event_target_value(&ev))
                    />
                </label>
                <label>
                    "API Base"
                    <input
                        prop:value=move || api_base.get()
                        on:input=move |ev| set_api_base.set(event_target_value(&ev))
                    />
                </label>
                <p>
                    "Open with query param: "
                    <code>"/demo/view?id=YOUR_DEMO_ID"</code>
                </p>
                <p>
                    "Quick smoke test commands (fallback demo): "
                    <code>"help"</code>
                    ", "
                    <code>"run demo"</code>
                </p>
            </div>
            <iframe class="demo-frame-placeholder" src=move || embed_src.get() />
            <section class="embed-code-generator">
                <h3>"Embed Code"</h3>
                <label>
                    "Iframe snippet"
                    <textarea readonly rows="4">{move || iframe_snippet.get()}</textarea>
                </label>
                <label>
                    "Script snippet"
                    <textarea readonly rows="4">{move || script_snippet.get()}</textarea>
                </label>
            </section>
        </section>
    }
}
