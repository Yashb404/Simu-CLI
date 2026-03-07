use leptos::prelude::*;

#[component]
pub fn StepCard(title: &'static str) -> impl IntoView {
    view! {
        <article class="step-card">
            <h4>{title}</h4>
            <p>"Step configuration UI will be attached here."</p>
        </article>
    }
}
