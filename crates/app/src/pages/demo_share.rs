use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ShareDemoPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_else(|| "unknown".to_string());

    view! {
        <section class="page demo-share-page">
            <h2>"Shared Demo"</h2>
            <p>{move || format!("Public slug: {}", slug())}</p>
            <div class="demo-frame-placeholder">"Terminal render will mount here."</div>
        </section>
    }
}
