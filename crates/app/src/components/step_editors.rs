use leptos::prelude::*;

#[component]
pub fn CommandStepEditor() -> impl IntoView {
    view! {
        <div class="step-editor command-step-editor">
            <h5>"Command Step"</h5>
            <p>"Edit command input and match options."</p>
        </div>
    }
}

#[component]
pub fn OutputStepEditor() -> impl IntoView {
    view! {
        <div class="step-editor output-step-editor">
            <h5>"Output Step"</h5>
            <p>"Edit output lines and formatting."</p>
        </div>
    }
}
