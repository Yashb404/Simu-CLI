use leptos::prelude::*;

#[component]
pub fn ProjectsPage() -> impl IntoView {
    view! {
        <section class="page projects-page">
            <h2>"Projects"</h2>
            <p>"Manage projects and create demos."</p>
            <button type="button">"New Project"</button>
        </section>
    }
}
