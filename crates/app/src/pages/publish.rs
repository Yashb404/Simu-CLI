use leptos::prelude::*;

use crate::components::embed_code_generator::EmbedCodeGenerator;

#[component]
pub fn PublishPage() -> impl IntoView {
    view! {
        <section class="page publish-page">
            <h2>"Publish"</h2>
            <p>"Publish demo and copy the embed snippet."</p>
            <button type="button">"Publish Demo"</button>
            <EmbedCodeGenerator
                demo_url="https://example.com/d/demo-slug".to_string()
                script_url="https://cdn.example.com/cli-simulator.js".to_string()
                demo_id="demo-slug".to_string()
            />
        </section>
    }
}
