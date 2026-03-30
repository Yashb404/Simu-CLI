use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_params_map;
use shared::{
    dto::UpdateDemoRequest,
    models::demo::{DemoSettings, EngineMode, Step, StepType, Theme, WindowStyle},
};

use crate::api;
use crate::components::demo_settings_form::DemoSettingsForm;
use crate::components::live_preview::LivePreviewPanel;
use crate::components::step_editors::{add_default_step, StepListEditor};
use crate::components::cast_import::CastImportButton;

#[component]
pub fn DemoEditorPage() -> impl IntoView {
    let params = use_params_map();
    let demo_id = move || {
        params
            .read()
            .get("id")
            .unwrap_or_else(|| "unknown".to_string())
    };

    let (title, set_title) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (steps, set_steps) = signal(Vec::<Step>::new());
    let (steps_version, set_steps_version) = signal(0u32);
    let (settings, set_settings) = signal(Some(DemoSettings {
        engine_mode: EngineMode::Sequential,
        autoplay: false,
        loop_demo: false,
        loop_delay_ms: 800,
        show_restart_button: true,
        show_hints: false,
        not_found_message: "command not found".to_string(),
    }));
    let (theme, set_theme) = signal(Some(Theme {
        window_style: WindowStyle::MacOs,
        window_title: "Terminal".to_string(),
        preset: Some("default".to_string()),
        bg_color: "#090909".to_string(),
        fg_color: "#f5f5f5".to_string(),
        cursor_color: "#ffffff".to_string(),
        font_family: "IBM Plex Mono".to_string(),
        font_size: 14,
        line_height: 1.4,
        prompt_string: "$".to_string(),
    }));
    let (status, set_status) = signal(String::new());
    let (published_slug, set_published_slug) = signal(String::new());

    Effect::new(move |_| {
        let id = demo_id();
        let _version = steps_version.get();
        if id == "unknown" {
            return;
        }

        spawn_local_scoped({
            let set_title = set_title;
            let set_slug = set_slug;
            let set_steps = set_steps;
            let set_status = set_status;
            let set_settings = set_settings;
            let set_theme = set_theme;
            async move {
                match api::get_demo_detail(&id).await {
                    Ok(demo) => {
                        let settings_value = demo.settings.clone();
                        let theme_value = demo.theme.clone();
                        set_title.set(demo.title);
                        set_slug.set(demo.slug.unwrap_or_default());
                        set_steps.set(demo.steps);
                        set_settings.set(Some(settings_value));
                        set_theme.set(Some(theme_value));
                    }
                    Err(err) => set_status.set(format!("Failed to load demo: {err}")),
                }
            }
        });
    });

    let save_demo = move |_| {
        let id = demo_id();
        let next_title = title.get();
        let next_slug = slug.get();
        let next_steps = steps.get();
        let next_settings = settings.get();
        let next_theme = theme.get();

        if id == "unknown" {
            set_status.set("Invalid demo id".to_string());
            return;
        }

        set_status.set("Saving...".to_string());

        spawn_local_scoped({
            let set_status = set_status;
            let set_steps = set_steps;
            async move {
                match api::update_demo_payload(
                    &id,
                    &UpdateDemoRequest {
                        title: Some(next_title.trim().to_string()),
                        project_id: None,
                        slug: if next_slug.trim().is_empty() {
                            None
                        } else {
                            Some(next_slug.trim().to_string())
                        },
                        theme: next_theme,
                        settings: next_settings,
                        steps: Some(next_steps),
                    },
                )
                .await
                {
                    Ok(demo) => {
                        set_steps.set(demo.steps);
                        set_status.set("Saved".to_string());
                    }
                    Err(err) => set_status.set(format!("Save failed: {err}")),
                }
            }
        });
    };

    let publish_demo = move |_| {
        let id = demo_id();
        let next_title = title.get();
        let next_slug = slug.get();
        let next_steps = steps.get();
        let next_settings = settings.get();
        let next_theme = theme.get();

        if id == "unknown" {
            set_status.set("Invalid demo id".to_string());
            return;
        }
        if next_title.trim().is_empty() {
            set_status.set("Title is required".to_string());
            return;
        }

        spawn_local_scoped({
            let set_status = set_status;
            let set_steps = set_steps;
            let set_published_slug = set_published_slug;
            async move {
                let update_result = api::update_demo_payload(
                    &id,
                    &UpdateDemoRequest {
                        title: Some(next_title.trim().to_string()),
                        project_id: None,
                        slug: if next_slug.trim().is_empty() {
                            None
                        } else {
                            Some(next_slug.trim().to_string())
                        },
                        theme: next_theme,
                        settings: next_settings,
                        steps: Some(next_steps),
                    },
                )
                .await;

                match update_result {
                    Ok(updated_demo) => {
                        set_steps.set(updated_demo.steps);
                        match api::publish_demo(&id).await {
                            Ok(published) => {
                                set_published_slug.set(published.slug);
                                set_status.set("Published and embed code ready".to_string());
                            }
                            Err(err) => set_status.set(format!("Publish failed: {err}")),
                        }
                    }
                    Err(err) => set_status.set(format!("Save failed: {err}")),
                }
            }
        });
    };

    let add_command_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Command);
        });
    };

    let add_output_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Output);
        });
    };

    let add_comment_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Comment);
        });
    };

    let add_prompt_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Prompt);
        });
    };

    let add_spinner_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Spinner);
        });
    };

    let add_cta_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Cta);
        });
    };

    let add_pause_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Pause);
        });
    };

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
        <section class="page demo-editor-page">
            <h2>"Demo Editor"</h2>
            <p>{move || format!("Editing demo: {}", demo_id())}</p>
            <p class="status">{move || status.get()}</p>

            <DemoSettingsForm
                title=title
                set_title=set_title
                slug=slug
                set_slug=set_slug
                settings=settings
                set_settings=set_settings
                theme=theme
                set_theme=set_theme
            />

            <div class="action-bar">
                <button type="button" class="btn-primary" on:click=save_demo>
                    "Save Draft"
                </button>
                <button type="button" class="btn-outline" on:click=publish_demo>
                    "Publish & Get Code"
                </button>
            </div>

            <div class="editor-grid">
                <section class="step-column">
                    <h3>"Steps"</h3>
                    <div class="inline-actions">
                        <button type="button" on:click=add_command_step>"+ Command"</button>
                        <button type="button" on:click=add_output_step>"+ Output"</button>
                        <button type="button" on:click=add_comment_step>"+ Comment"</button>
                        <button type="button" on:click=add_prompt_step>"+ Prompt"</button>
                        <button type="button" on:click=add_spinner_step>"+ Spinner"</button>
                        <button type="button" on:click=add_cta_step>"+ CTA"</button>
                        <button type="button" on:click=add_pause_step>"+ Pause"</button>
                    </div>

                    <CastImportButton
                        demo_id=demo_id()
                        on_success={Callback::new(move |resp: shared::dto::demo_dto::ImportCastResponse| {
                            set_steps_version.update(|v| *v += 1);
                            set_status.set(resp.message);
                        })}
                    />

                    <p class="text-muted">
                        "Upload a .cast file to automatically append command/output steps below."
                    </p>

                    <StepListEditor steps=steps set_steps=set_steps />
                </section>
                <aside class="preview-column">
                    <LivePreviewPanel
                        steps=steps
                        prompt_string=prompt_string
                        not_found_message=not_found_message
                    />
                </aside>
            </div>

            <Show when=move || !published_slug.get().is_empty()>
                <section class="panel embed-panel">
                    <h3>"Embed Snippet"</h3>
                    <p class="text-muted">"Copy and paste this into your documentation or website."</p>
                    <textarea readonly rows="3" class="code-block">
                        {move || {
                            format!(
                                "<script src=\"{}/static/embed.js\" data-demo=\"{}\"></script>",
                                api::api_base(),
                                published_slug.get(),
                            )
                        }}
                    </textarea>
                </section>
            </Show>
        </section>
    }
}
