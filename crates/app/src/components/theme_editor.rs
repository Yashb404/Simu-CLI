use leptos::prelude::*;

#[component]
pub fn ThemeEditor() -> impl IntoView {
    view! {
        <section class="theme-editor">
            <h3>"Theme"</h3>
            <p>"Theme controls (colors, font, prompt) will be available here."</p>
        </section>
    }
}
