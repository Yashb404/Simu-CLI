use leptos::prelude::*;
use shared::services::embed_generator::{generate_iframe_snippet, generate_script_snippet};

#[component]
pub fn EmbedCodeGenerator(demo_url: String, script_url: String, demo_id: String) -> impl IntoView {
    let iframe_code = generate_iframe_snippet(&demo_url, "100%", "480px");
    let script_code = generate_script_snippet(&script_url, &demo_id);

    view! {
        <section class="embed-code-generator">
            <h3>"Embed Code"</h3>
            <label>
                "Iframe snippet"
                <textarea readonly rows="4">{iframe_code}</textarea>
            </label>
            <label>
                "Script snippet"
                <textarea readonly rows="4">{script_code}</textarea>
            </label>
        </section>
    }
}
