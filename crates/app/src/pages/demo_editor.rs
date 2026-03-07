use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::{live_preview::LivePreviewPanel, step_card::StepCard};

#[component]
pub fn DemoEditorPage() -> impl IntoView {
    let params = use_params_map();
    let demo_id = move || params.read().get("id").unwrap_or_else(|| "unknown".to_string());

    view! {
        <section class="page demo-editor-page">
            <h2>"Demo Editor"</h2>
            <p>{move || format!("Editing demo: {}", demo_id())}</p>
            <div class="editor-grid">
                <section class="step-column">
                    <h3>"Steps"</h3>
                    <StepCard title="Command step" />
                    <StepCard title="Output step" />
                </section>
                <LivePreviewPanel />
            </div>
        </section>
    }
}
