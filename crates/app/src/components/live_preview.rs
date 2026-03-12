use leptos::prelude::*;

#[component]
pub fn LivePreviewPanel() -> impl IntoView {
    view! {
        <section class="live-preview-panel">
            <h3>"Live Preview"</h3>
            <div class="preview-canvas">"Embed runtime preview goes here."</div>
        </section>
    }
}
