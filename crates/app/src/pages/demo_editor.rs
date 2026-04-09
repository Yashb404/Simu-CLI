use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_params_map;
use shared::{
    dto::UpdateDemoRequest,
    models::demo::{DemoSettings, EngineMode, Step, StepType, Theme, WindowStyle},
};

use crate::api;
use crate::components::cast_import::CastImportButton;
use crate::components::demo_settings_form::DemoSettingsForm;
use crate::components::live_preview::LivePreviewPanel;
use crate::components::step_editors::{
    StepListEditor, add_command_block as add_command_block_step, add_default_step,
};

#[cfg(target_arch = "wasm32")]
const EDITOR_SPLIT_RATIO_KEY: &str = "demo_editor_split_ratio";

#[cfg(target_arch = "wasm32")]
fn load_editor_split_ratio() -> f64 {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|ls| ls.get_item(EDITOR_SPLIT_RATIO_KEY).ok().flatten())
        .and_then(|v| v.parse::<f64>().ok())
        .map(|value| value.clamp(0.34, 0.74))
        .unwrap_or(0.56)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_editor_split_ratio() -> f64 {
    0.56
}

#[cfg(target_arch = "wasm32")]
fn persist_editor_split_ratio(value: f64) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(EDITOR_SPLIT_RATIO_KEY, &value.to_string());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_editor_split_ratio(_value: f64) {}

fn normalize_command_match_patterns(steps: &mut [Step]) {
    for step in steps.iter_mut() {
        if step.step_type != StepType::Command {
            continue;
        }

        let input = step.input.clone().unwrap_or_default();
        if input.trim().is_empty() {
            continue;
        }

        // One-time migration for legacy defaults: if pattern still has the
        // original scaffold value but input was changed, sync to input.
        if let Some(pattern) = step.match_pattern.clone() {
            if pattern.trim() == "echo hello" && input.trim() != "echo hello" {
                step.match_pattern = Some(input);
            }
        }
    }
}

#[component]
pub fn DemoEditorPage() -> impl IntoView {
    let params = use_params_map();
    let demo_id =
        move || params.with_untracked(|p| p.get("id").unwrap_or_else(|| "unknown".to_string()));
    let current_demo_id = demo_id();

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
                        let mut steps_value = demo.steps;
                        normalize_command_match_patterns(&mut steps_value);
                        set_title.set(demo.title);
                        set_slug.set(demo.slug.unwrap_or_default());
                        set_steps.set(steps_value);
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
        let mut next_steps = steps.get();
        normalize_command_match_patterns(&mut next_steps);
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
        let mut next_steps = steps.get();
        normalize_command_match_patterns(&mut next_steps);
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
            add_command_block_step(items);
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

    let is_resizing = RwSignal::new(false);
    let split_ratio = RwSignal::new(load_editor_split_ratio());

    let on_splitter_down = {
        move |ev: web_sys::PointerEvent| {
            ev.prevent_default();
            is_resizing.set(true);
        }
    };

    let on_editor_pointer_move = {
        move |ev: web_sys::PointerEvent| {
            if !is_resizing.get() {
                return;
            }

            let viewport_width = web_sys::window()
                .and_then(|window| window.inner_width().ok())
                .and_then(|width| width.as_f64())
                .unwrap_or(1200.0_f64);

            let next = ((ev.client_x() as f64) / viewport_width).clamp(0.34, 0.74);
            split_ratio.set(next);
            persist_editor_split_ratio(next);
        }
    };

    let on_editor_pointer_up = {
        move |_ev: web_sys::PointerEvent| {
            is_resizing.set(false);
        }
    };

    view! {
        <div class="editor-workspace min-h-screen bg-[radial-gradient(circle_at_top_left,_rgba(24,24,27,0.9),_rgba(9,9,11,1)_42%)] text-zinc-100">
            <header class="editor-topbar border-b border-zinc-800/80 bg-zinc-950/85 px-6 py-5 backdrop-blur-xl">
                <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                <div class="inline-actions flex flex-1 flex-col gap-3">
                    <div class="flex items-center gap-3 text-[11px] uppercase tracking-[0.24em] text-zinc-500">
                        <span>"Demo Builder"</span>
                        <span class="h-px flex-1 bg-zinc-800"></span>
                        <span>"Split Workspace"</span>
                    </div>
                    <input
                        class="editor-title-input w-full rounded-2xl border border-zinc-800 bg-zinc-950/90 px-4 py-3 text-2xl font-semibold tracking-tight text-zinc-50 shadow-[0_20px_80px_-32px_rgba(0,0,0,0.8)] transition-all duration-200 ease-in-out placeholder:text-zinc-500 focus:border-zinc-600 focus:outline-none focus:ring-2 focus:ring-zinc-700/40"
                        prop:value=move || title.get()
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                        placeholder="Untitled demo"
                    />
                    <p class="status text-sm text-zinc-400">{move || status.get()}</p>
                </div>
                <div class="action-bar flex items-center gap-3">
                    <button type="button" class="btn-primary rounded-xl border border-zinc-700 bg-zinc-100 px-4 py-2.5 text-sm font-medium text-zinc-950 transition-all duration-200 ease-in-out hover:-translate-y-0.5 hover:bg-white" on:click=save_demo>
                        "Save Draft"
                    </button>
                    <button type="button" class="btn-outline rounded-xl border border-emerald-500/30 bg-emerald-500/10 px-4 py-2.5 text-sm font-medium text-emerald-200 transition-all duration-200 ease-in-out hover:-translate-y-0.5 hover:border-emerald-400/60 hover:bg-emerald-500/20" on:click=publish_demo>
                        "Publish & Get Code"
                    </button>
                </div>
                </div>
            </header>

            <main
                class=move || {
                    if is_resizing.get() {
                        "editor-main flex min-h-[calc(100vh-112px)] touch-none select-none bg-transparent"
                    } else {
                        "editor-main flex min-h-[calc(100vh-112px)] bg-transparent"
                    }
                }
                on:pointermove=on_editor_pointer_move
                on:pointerup=on_editor_pointer_up
                on:pointercancel=on_editor_pointer_up
                on:pointerleave=on_editor_pointer_up
            >
                <aside
                    class="script-pane flex min-w-0 shrink-0 flex-col border-r border-zinc-800/80 bg-zinc-950/55 backdrop-blur-xl"
                    style=move || format!("flex-basis: calc({:.4}% - 6px);", split_ratio.get() * 100.0)
                >
                    <div class="script-toolbar flex flex-wrap gap-2 border-b border-zinc-800/70 px-5 py-4">
                        <button type="button" class="btn-primary-light rounded-xl border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs font-medium text-zinc-100 transition-all duration-200 ease-in-out hover:-translate-y-0.5 hover:border-zinc-600 hover:bg-zinc-800" on:click=add_command_block>"+ Command Block"</button>
                        <button type="button" class="rounded-xl border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-medium text-zinc-300 transition-all duration-200 ease-in-out hover:border-zinc-700 hover:bg-zinc-900" on:click=add_command_step>"+ Command"</button>
                        <button type="button" class="rounded-xl border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-medium text-zinc-300 transition-all duration-200 ease-in-out hover:border-zinc-700 hover:bg-zinc-900" on:click=add_output_step>"+ Output"</button>
                        <button type="button" class="rounded-xl border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-medium text-zinc-300 transition-all duration-200 ease-in-out hover:border-zinc-700 hover:bg-zinc-900" on:click=add_comment_step>"+ Comment"</button>
                        <button type="button" class="rounded-xl border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-medium text-zinc-300 transition-all duration-200 ease-in-out hover:border-zinc-700 hover:bg-zinc-900" on:click=add_prompt_step>"+ Prompt"</button>
                        <button type="button" class="rounded-xl border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-medium text-zinc-300 transition-all duration-200 ease-in-out hover:border-zinc-700 hover:bg-zinc-900" on:click=add_spinner_step>"+ Spinner"</button>
                        <button type="button" class="rounded-xl border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-medium text-zinc-300 transition-all duration-200 ease-in-out hover:border-zinc-700 hover:bg-zinc-900" on:click=add_cta_step>"+ CTA"</button>
                        <button type="button" class="rounded-xl border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-medium text-zinc-300 transition-all duration-200 ease-in-out hover:border-zinc-700 hover:bg-zinc-900" on:click=add_pause_step>"+ Pause"</button>
                    </div>

                    <div class="script-content flex flex-1 flex-col gap-5 overflow-y-auto px-5 py-5">
                        <section class="rounded-2xl border border-zinc-800/70 bg-zinc-950/80 p-4 shadow-[0_24px_80px_-36px_rgba(0,0,0,0.85)]">
                            <div class="flex items-start justify-between gap-4">
                                <div class="space-y-1">
                                    <p class="text-[11px] uppercase tracking-[0.2em] text-zinc-500">"Script Pane"</p>
                                    <h2 class="text-lg font-semibold text-zinc-50">"Scenario Authoring"</h2>
                                    <p class="text-sm text-zinc-400">"Compose commands, narration, prompts, and guided hints for your interactive CLI demo."</p>
                                </div>
                                <div class="rounded-full border border-zinc-800 bg-zinc-900 px-3 py-1 text-[11px] uppercase tracking-[0.18em] text-zinc-400">
                                    {move || format!("{:.0}% width", split_ratio.get() * 100.0)}
                                </div>
                            </div>
                        </section>

                        <CastImportButton
                            demo_id=current_demo_id.clone()
                            on_success={Callback::new(move |resp: shared::dto::demo_dto::ImportCastResponse| {
                                set_steps_version.update(|v| *v += 1);
                                set_status.set(resp.message);
                            })}
                        />

                        <p class="text-muted text-sm text-zinc-500">
                            "Upload a .cast file to automatically append command/output steps below."
                        </p>

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
                            <section class="panel embed-panel rounded-2xl border border-zinc-800/70 bg-zinc-950/80 p-4 shadow-[0_20px_70px_-36px_rgba(0,0,0,0.9)]">
                                <h3 class="text-base font-semibold text-zinc-50">"Embed Snippet"</h3>
                                <p class="text-muted mt-2 text-sm text-zinc-500">"Copy and paste this into your documentation or website."</p>
                                <textarea readonly rows="3" class="code-block mt-3 w-full rounded-xl border border-zinc-800 bg-zinc-950 p-3 font-mono text-xs text-zinc-300">
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
                            "editor-splitter is-active relative z-20 w-3 cursor-col-resize bg-transparent before:absolute before:inset-y-0 before:left-1/2 before:w-px before:-translate-x-1/2 before:bg-zinc-500/80 before:shadow-[0_0_20px_rgba(244,244,245,0.15)]".to_string()
                        } else {
                            "editor-splitter relative z-20 w-3 cursor-col-resize bg-transparent transition-colors duration-200 ease-in-out hover:bg-zinc-800/30 before:absolute before:inset-y-0 before:left-1/2 before:w-px before:-translate-x-1/2 before:bg-zinc-700".to_string()
                        }
                    }
                    on:pointerdown=on_splitter_down
                />

                <section class="stage-pane min-w-0 flex-1 bg-[linear-gradient(180deg,rgba(24,24,27,0.65),rgba(9,9,11,0.9))]">
                    <div class="terminal-container flex h-full flex-col gap-5 px-6 py-5">
                        <section class="rounded-2xl border border-zinc-800/70 bg-zinc-950/70 p-4 shadow-[0_24px_80px_-40px_rgba(0,0,0,0.85)]">
                            <div class="flex items-start justify-between gap-4">
                                <div class="space-y-1">
                                    <p class="text-[11px] uppercase tracking-[0.2em] text-zinc-500">"Runtime Pane"</p>
                                    <h2 class="text-lg font-semibold text-zinc-50">"Live Terminal Preview"</h2>
                                    <p class="text-sm text-zinc-400">"Preview your guided command flow while you edit the script on the left."</p>
                                </div>
                                <div class="rounded-full border border-zinc-800 bg-zinc-900 px-3 py-1 text-[11px] uppercase tracking-[0.18em] text-zinc-400">
                                    "Resizable"
                                </div>
                            </div>
                        </section>
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
