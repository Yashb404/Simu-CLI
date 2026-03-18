use leptos::prelude::*;
use shared::models::demo::{DemoSettings, Theme};

#[component]
pub fn DemoSettingsForm(
    title: ReadSignal<String>,
    set_title: WriteSignal<String>,
    slug: ReadSignal<String>,
    set_slug: WriteSignal<String>,
    settings: ReadSignal<Option<DemoSettings>>,
    set_settings: WriteSignal<Option<DemoSettings>>,
    theme: ReadSignal<Option<Theme>>,
    set_theme: WriteSignal<Option<Theme>>,
) -> impl IntoView {
    let prompt_string = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.prompt_string)
            .unwrap_or_else(|| "$".to_string())
    });

    let not_found_message = Signal::derive(move || {
        settings
            .get()
            .map(|cfg| cfg.not_found_message)
            .unwrap_or_else(|| "command not found".to_string())
    });

    view! {
        <section class="panel form-grid">
            <label>
                "Title"
                <input
                    prop:value=move || title.get()
                    on:input=move |ev| set_title.set(event_target_value(&ev))
                />
            </label>
            <label>
                "Slug"
                <input
                    prop:value=move || slug.get()
                    on:input=move |ev| set_slug.set(event_target_value(&ev))
                />
            </label>
            <label>
                "Prompt String"
                <input
                    prop:value=prompt_string
                    on:input=move |ev| {
                        let next = event_target_value(&ev);
                        set_theme.update(|value| {
                            if let Some(theme) = value.as_mut() {
                                theme.prompt_string = next.clone();
                            }
                        });
                    }
                />
            </label>
            <label>
                "Not Found Message"
                <input
                    prop:value=not_found_message
                    on:input=move |ev| {
                        let next = event_target_value(&ev);
                        set_settings.update(|value| {
                            if let Some(settings) = value.as_mut() {
                                settings.not_found_message = next.clone();
                            }
                        });
                    }
                />
            </label>
        </section>
    }
}
