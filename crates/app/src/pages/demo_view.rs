use leptos::prelude::*;

use crate::components::embed_code_generator::EmbedCodeGenerator;

#[component]
pub fn DemoViewPage() -> impl IntoView {
    view! {
        <section class="page demo-view-page">
            <h2>"Public Demo View"</h2>
            <p>"Full-page shareable terminal experience."</p>
            <div class="demo-frame-placeholder">"Embedded runtime mounts here."</div>
            <EmbedCodeGenerator
                demo_url="https://example.com/d/demo-slug".to_string()
                script_url="https://cdn.example.com/cli-simulator.js".to_string()
                demo_id="demo-slug".to_string()
            />
        </section>
    }
}
