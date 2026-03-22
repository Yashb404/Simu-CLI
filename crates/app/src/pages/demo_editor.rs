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
use crate::components::step_editors::{add_command_block, add_default_step, StepListEditor};

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

    let add_command_block = move |_| {
        set_steps.update(|items| {
            add_command_block(items);
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

    let (is_resizing, set_is_resizing) = signal(false);
    let (script_pane_px, set_script_pane_px) = signal(560.0_f64);

    let on_splitter_down = {
        let set_is_resizing = set_is_resizing;
        move |ev: web_sys::PointerEvent| {
            ev.prevent_default();
            set_is_resizing.set(true);
        }
    };

    let on_editor_pointer_move = {
        let is_resizing = is_resizing;
        let set_script_pane_px = set_script_pane_px;

        move |ev: web_sys::PointerEvent| {
            if !is_resizing.get() {
                return;
            }

            let viewport_width = web_sys::window()
                .and_then(|window| window.inner_width().ok())
                .and_then(|width| width.as_f64())
                .unwrap_or(1200.0_f64);

            let min_width = 360.0_f64;
            let max_width = (viewport_width - 360.0_f64).max(min_width);
            let next = (ev.client_x() as f64).clamp(min_width, max_width);
            set_script_pane_px.set(next);
        }
    };

    let on_editor_pointer_up = {
        let set_is_resizing = set_is_resizing;
        move |_ev: web_sys::PointerEvent| {
            set_is_resizing.set(false);
        }
    };

    view! {
        <div class="editor-workspace">
            <header class="editor-topbar">
                <div class="inline-actions">
                    <input
                        class="editor-title-input"
                        prop:value=move || title.get()
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                        placeholder="Untitled demo"
                    />
                    <p class="status">{move || status.get()}</p>
                </div>
                <div class="action-bar">
                    <button type="button" class="btn-primary" on:click=save_demo>
                        "Save Draft"
                    </button>
                    <button type="button" class="btn-outline" on:click=publish_demo>
                        "Publish & Get Code"
                    </button>
                </div>
            </header>

            <main
                class="editor-main"
                on:pointermove=on_editor_pointer_move
                on:pointerup=on_editor_pointer_up
                on:pointercancel=on_editor_pointer_up
                on:pointerleave=on_editor_pointer_up
            >
                <aside class="script-pane" style=move || format!("flex-basis: {}px;", script_pane_px.get())>
                    <div class="script-toolbar">
                        <button type="button" class="btn-primary-light" on:click=add_command_block>"+ Command Block"</button>
                        <button type="button" on:click=add_command_step>"+ Command"</button>
                        <button type="button" on:click=add_output_step>"+ Output"</button>
                        <button type="button" on:click=add_comment_step>"+ Comment"</button>
                        <button type="button" on:click=add_prompt_step>"+ Prompt"</button>
                        <button type="button" on:click=add_spinner_step>"+ Spinner"</button>
                        <button type="button" on:click=add_cta_step>"+ CTA"</button>
                        <button type="button" on:click=add_pause_step>"+ Pause"</button>
                    </div>

                    <div class="script-content">
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

                        <StepListEditor steps=steps set_steps=set_steps />

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
                    </div>
                </aside>

                <div
                    class=move || {
                        if is_resizing.get() {
                            "editor-splitter is-active".to_string()
                        } else {
                            "editor-splitter".to_string()
                        }
                    }
                    on:pointerdown=on_splitter_down
                />

                <section class="stage-pane">
                    <div class="terminal-container">
                        <LivePreviewPanel
                            steps=steps
                            prompt_string=prompt_string
                            not_found_message=not_found_message
                        />
                    </div>
                </section>
            </main>
        </div>
    }
}
