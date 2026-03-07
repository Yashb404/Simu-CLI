use leptos::prelude::*;

#[component]
pub fn DemosPage() -> impl IntoView {
    view! {
        <section class="page demos-page">
            <h2>"Demos"</h2>
            <p>"Create and manage terminal demos in this project."</p>
            <button type="button">"New Demo"</button>
        </section>
    }
}
