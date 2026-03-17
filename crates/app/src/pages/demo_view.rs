use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use shared::services::embed_generator::{generate_iframe_snippet, generate_script_snippet};

#[component]
pub fn DemoViewPage() -> impl IntoView {
    let query = use_query_map();
    let demo_id = Signal::derive(move || {
        query
            .read()
            .get("id")
            .unwrap_or_else(|| "demo-id".to_string())
    });

    let iframe_src = Signal::derive(move || {
        format!(
            "/embed/index.html?demo_id={}&api_base=http://localhost:3001",
            demo_id.get()
        )
    });

    let iframe_snippet = Signal::derive(move || {
        generate_iframe_snippet(&format!("http://localhost:8080/demo/view?id={}", demo_id.get()), "100%", "480px")
    });

    let script_snippet = Signal::derive(move || {
        generate_script_snippet("http://localhost:8080/embed/index.html", &demo_id.get())
    });

    view! {
        <section class="page demo-view-page">
            <h2>"Public Demo View"</h2>
            <p>"Full-page shareable terminal experience using the embed runtime."</p>
            <div class="panel form-grid">
                <label>
                    "Demo ID"
                    <input prop:value=demo_id />
                </label>
                <p>
                    "Open with query param: "
                    <code>"/demo/view?id=YOUR_DEMO_ID"</code>
                </p>
            </div>
            <iframe class="demo-frame-placeholder" src=move || iframe_src.get() />
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
