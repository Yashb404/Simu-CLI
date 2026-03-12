use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <section class="page settings-page">
            <h2>"Demo Settings"</h2>
            <p>"Configure autoplay, loop behavior, and engine mode."</p>
            <label>
                "Engine mode"
                <select>
                    <option value="sequential">"Sequential"</option>
                    <option value="free_play">"Free play"</option>
                </select>
            </label>
        </section>
    }
}
