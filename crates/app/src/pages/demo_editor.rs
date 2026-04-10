use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_params_map;
use shared::{
    dto::{UpdateDemoRequest, demo_dto::ImportCastResponse},
    models::demo::{DemoSettings, EngineMode, Step, StepType, Theme, WindowStyle},
};

use crate::api;
use crate::app::{ThemeController, ThemeMode};
use crate::components::cast_import::CastImportButton;
use crate::components::demo_settings_form::DemoSettingsForm;
use crate::components::live_preview::LivePreviewPanel;
use crate::components::step_editors::{
    StepListEditor, add_command_block as add_command_block_step, add_default_step,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CreatorViewMode {
    Developer,
    User,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CanvasState {
    Workspace,
    ImportPublish,
    ScriptBuilder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PublishState {
    Idle,
    Saving,
    Publishing,
    Success,
    Error,
}

fn normalize_command_match_patterns(steps: &mut [Step]) {
    for step in steps.iter_mut() {
        if step.step_type != StepType::Command {
            continue;
        }

        let input = step.input.clone().unwrap_or_default();
        if input.trim().is_empty() {
            continue;
        }

        if let Some(pattern) = step.match_pattern.clone() {
            if pattern.trim() == "echo hello" && input.trim() != "echo hello" {
                step.match_pattern = Some(input);
            }
        }
    }
}

fn command_guide_entries(steps: &[Step]) -> Vec<(String, String)> {
    steps
        .iter()
        .filter(|step| matches!(step.step_type, StepType::Command))
        .filter_map(|step| {
            let command = step
                .input
                .clone()
                .or_else(|| step.match_pattern.clone())
                .filter(|v| !v.trim().is_empty())?;

            let description = step
                .short_description
                .clone()
                .or_else(|| step.description.clone())
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "Run this command in the terminal preview.".to_string());

            Some((command, description))
        })
        .collect()
}

#[component]
fn TopNav(
    title: ReadSignal<String>,
    set_title: WriteSignal<String>,
    status: ReadSignal<String>,
    view_mode: ReadSignal<CreatorViewMode>,
    set_view_mode: WriteSignal<CreatorViewMode>,
    canvas_state: ReadSignal<CanvasState>,
    set_canvas_state: WriteSignal<CanvasState>,
    theme_mode: Signal<ThemeMode>,
    set_theme_mode: WriteSignal<ThemeMode>,
    on_save: Callback<()>,
    on_publish: Callback<()>,
) -> impl IntoView {
    view! {
        <header class="bg-background border-b border-outline-variant flex flex-wrap justify-between items-center px-6 py-3 gap-3 min-h-14 w-full sticky top-0 z-50">
            <div class="flex items-center gap-8 min-w-0">
            <div class="text-lg font-black text-primary tracking-tighter shrink-0">"SimuCLI Demo Creator"</div>
                <input
                    class="bg-transparent border-none outline-none text-on-surface placeholder:text-zinc-500 text-sm md:text-base min-w-[220px]"
                    prop:value=move || title.get()
                    on:input=move |ev| set_title.set(event_target_value(&ev))
                    placeholder="Untitled demo"
                />
                <span class="hidden lg:inline text-[10px] font-mono text-zinc-500 uppercase tracking-widest">
                    {move || {
                        let current = status.get();
                        if current.trim().is_empty() {
                            "STATUS: READY".to_string()
                        } else {
                            format!("STATUS: {current}")
                        }
                    }}
                </span>
            </div>

            <div class="flex items-center gap-3 flex-wrap">
                <div class="flex items-center gap-2 rounded-lg border border-outline-variant/20 bg-surface-container-low px-2 py-1">
                    <span class="text-[10px] font-semibold uppercase tracking-[0.14em] text-on-surface-variant">"Theme"</span>
                    <select
                        class="rounded-md border border-outline-variant bg-surface-container px-2 py-1 text-xs font-semibold text-on-surface outline-none"
                        prop:value=move || theme_mode.get().as_str()
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_theme_mode.set(ThemeMode::from_str(&value));
                        }
                    >
                        <option value="terminal">"Terminal"</option>
                        <option value="dark">"Dark"</option>
                        <option value="light">"Light"</option>
                    </select>
                </div>

                <div class="flex bg-surface-container-low p-1 rounded-lg border border-outline-variant/20">
                    <button
                        class=move || {
                            if matches!(view_mode.get(), CreatorViewMode::Developer) {
                                "px-3 py-1 text-xs font-bold bg-surface-container text-primary rounded-md shadow-sm"
                            } else {
                                "px-3 py-1 text-xs font-bold text-zinc-500 hover:text-zinc-200"
                            }
                        }
                        on:click=move |_| set_view_mode.set(CreatorViewMode::Developer)
                    >
                        "Dev View"
                    </button>
                    <button
                        class=move || {
                            if matches!(view_mode.get(), CreatorViewMode::User) {
                                "px-3 py-1 text-xs font-bold bg-surface-container text-primary rounded-md shadow-sm"
                            } else {
                                "px-3 py-1 text-xs font-bold text-zinc-500 hover:text-zinc-200"
                            }
                        }
                        on:click=move |_| set_view_mode.set(CreatorViewMode::User)
                    >
                        "User View"
                    </button>
                </div>

                <div class="hidden md:flex bg-surface-container-low p-1 rounded-lg border border-outline-variant/20">
                    <button
                        class=move || {
                            if matches!(canvas_state.get(), CanvasState::Workspace) {
                                "px-3 py-1 text-[10px] font-bold bg-surface-container text-primary rounded-md"
                            } else {
                                "px-3 py-1 text-[10px] font-bold text-zinc-500 hover:text-zinc-200"
                            }
                        }
                        on:click=move |_| set_canvas_state.set(CanvasState::Workspace)
                    >
                        "Workspace"
                    </button>
                    <button
                        class=move || {
                            if matches!(canvas_state.get(), CanvasState::ImportPublish) {
                                "px-3 py-1 text-[10px] font-bold bg-surface-container text-primary rounded-md"
                            } else {
                                "px-3 py-1 text-[10px] font-bold text-zinc-500 hover:text-zinc-200"
                            }
                        }
                        on:click=move |_| set_canvas_state.set(CanvasState::ImportPublish)
                    >
                        "Import"
                    </button>
                    <button
                        class=move || {
                            if matches!(canvas_state.get(), CanvasState::ScriptBuilder) {
                                "px-3 py-1 text-[10px] font-bold bg-surface-container text-primary rounded-md"
                            } else {
                                "px-3 py-1 text-[10px] font-bold text-zinc-500 hover:text-zinc-200"
                            }
                        }
                        on:click=move |_| set_canvas_state.set(CanvasState::ScriptBuilder)
                    >
                        "Builder"
                    </button>
                </div>

                <button
                    class="bg-surface-container-highest px-3 py-1.5 rounded text-sm font-bold text-on-surface hover:bg-surface-bright transition-colors"
                    on:click=move |_| on_save.run(())
                >
                    "Save"
                </button>
                <button
                    class="bg-primary px-3 py-1.5 rounded text-sm font-bold text-on-primary hover:opacity-90 active:scale-95 transition-all"
                    on:click=move |_| on_publish.run(())
                >
                    "Publish"
                </button>
            </div>
        </header>
    }
}

#[component]
fn SideNav() -> impl IntoView {
    view! {
        <aside class="bg-surface-container-low w-16 md:w-64 border-r border-outline-variant flex flex-col shrink-0">
            <div class="p-4 flex items-center gap-3 border-b border-outline-variant md:px-6">
                <div class="w-8 h-8 bg-primary-container rounded flex items-center justify-center">
                    <span class="material-symbols-outlined text-primary">"terminal"</span>
                </div>
                <div class="hidden md:block">
                    <p class="font-mono text-primary text-sm font-bold">"SimuCLI"</p>
                    <p class="text-[10px] text-zinc-500 font-mono uppercase tracking-wider">"v1.2.0"</p>
                </div>
            </div>

            <nav class="flex-1 py-4">
                <div class="px-3 mb-4 hidden md:block">
                    <button class="w-full bg-primary text-on-primary font-bold text-xs py-2 rounded-lg flex items-center justify-center gap-2 uppercase tracking-widest">
                        <span class="material-symbols-outlined text-sm">"add"</span>
                        "New Step"
                    </button>
                </div>

                <div class="space-y-1">
                    <a class="flex items-center gap-4 px-4 md:px-6 py-3 bg-surface-container text-primary font-bold border-l-4 border-primary" href="#">
                        <span class="material-symbols-outlined">"terminal"</span>
                        <span class="hidden md:block font-mono text-xs">"Command"</span>
                    </a>
                    <a class="flex items-center gap-4 px-4 md:px-6 py-3 text-zinc-400 hover:bg-surface-container hover:text-zinc-100 transition-colors" href="#">
                        <span class="material-symbols-outlined">"output"</span>
                        <span class="hidden md:block font-mono text-xs">"Output"</span>
                    </a>
                    <a class="flex items-center gap-4 px-4 md:px-6 py-3 text-zinc-400 hover:bg-surface-container hover:text-zinc-100 transition-colors" href="#">
                        <span class="material-symbols-outlined">"timer"</span>
                        <span class="hidden md:block font-mono text-xs">"Delay"</span>
                    </a>
                    <a class="flex items-center gap-4 px-4 md:px-6 py-3 text-zinc-400 hover:bg-surface-container hover:text-zinc-100 transition-colors" href="#">
                        <span class="material-symbols-outlined">"keyboard"</span>
                        <span class="hidden md:block font-mono text-xs">"Input"</span>
                    </a>
                    <a class="flex items-center gap-4 px-4 md:px-6 py-3 text-zinc-400 hover:bg-surface-container hover:text-zinc-100 transition-colors" href="#">
                        <span class="material-symbols-outlined">"pause_circle"</span>
                        <span class="hidden md:block font-mono text-xs">"Wait"</span>
                    </a>
                </div>
            </nav>

            <div class="mt-auto border-t border-outline-variant py-4 space-y-1">
                <a class="flex items-center gap-4 px-4 md:px-6 py-2 bg-surface-container text-primary font-bold border-l-4 border-primary" href="#">
                    <span class="material-symbols-outlined">"help"</span>
                    <span class="hidden md:block font-mono text-xs">"Guide"</span>
                </a>
                <a class="flex items-center gap-4 px-4 md:px-6 py-2 text-zinc-400 hover:bg-surface-container hover:text-zinc-100 transition-colors" href="#">
                    <span class="material-symbols-outlined">"description"</span>
                    <span class="hidden md:block font-mono text-xs">"Logs"</span>
                </a>
            </div>
        </aside>
    }
}

#[component]
fn GuidePanel(steps: ReadSignal<Vec<Step>>, open: ReadSignal<bool>, set_open: WriteSignal<bool>) -> impl IntoView {
    let entries = Signal::derive(move || command_guide_entries(&steps.get()));

    view! {
        <div
            class=move || {
                if open.get() {
                    "w-80 bg-surface-container-low border-l border-outline-variant flex flex-col shadow-2xl relative z-30 transition-transform duration-300"
                } else {
                    "hidden lg:flex w-0"
                }
            }
        >
            <div class="p-6 border-b border-outline-variant flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-primary">"auto_awesome"</span>
                    <h2 class="text-sm font-black uppercase tracking-widest text-on-surface">"Guide"</h2>
                </div>
                <button class="text-zinc-500 hover:text-white transition-colors" on:click=move |_| set_open.set(false)>
                    <span class="material-symbols-outlined">"close"</span>
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-4 space-y-4">
                <For
                    each=move || entries.get()
                    key=|entry| entry.0.clone()
                    children=move |entry| {
                        view! {
                            <div class="bg-surface-container p-4 rounded border-l-4 border-primary">
                                <div class="flex items-center gap-2 mb-2">
                                    <span class="material-symbols-outlined text-primary text-sm">"terminal"</span>
                                    <span class="font-mono text-xs font-bold text-on-surface">{entry.0.clone()}</span>
                                </div>
                                <p class="text-xs text-on-surface-variant leading-relaxed">{entry.1.clone()}</p>
                            </div>
                        }
                    }
                />

                <Show when=move || entries.get().is_empty()>
                    <div class="bg-surface-container p-4 rounded border border-outline-variant/20">
                        <p class="text-xs text-on-surface-variant">"No command guide entries yet. Add command steps or import a cast file."</p>
                    </div>
                </Show>
            </div>

            <div class="p-4 bg-background border-t border-outline-variant">
                <button class="w-full bg-surface-container-highest text-on-surface text-xs font-bold py-2.5 rounded hover:bg-surface-bright transition-all flex items-center justify-center gap-2">
                    <span class="material-symbols-outlined text-sm">"open_in_new"</span>
                    "Full Documentation"
                </button>
            </div>
        </div>
    }
}

#[component]
fn WorkspaceState(
    steps: ReadSignal<Vec<Step>>,
    prompt_string: Signal<String>,
    not_found_message: Signal<String>,
    theme: ReadSignal<Option<Theme>>,
    developer_mode: Signal<bool>,
    guide_open: ReadSignal<bool>,
    set_guide_open: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex-1 flex overflow-hidden">
            <div class="flex-1 flex flex-col p-6 gap-6 relative overflow-y-auto">
                <div class="flex justify-between items-end">
                    <div>
                        <h1 class="text-2xl font-black tracking-tight text-on-surface">"Terminal Environment"</h1>
                        <p class="text-on-surface-variant font-mono text-xs">"simucli_project_0x882.sh"</p>
                    </div>
                    <div class="flex items-center gap-2 px-3 py-1 bg-surface-container rounded text-[10px] font-mono text-zinc-500">
                        <div class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></div>
                        "STATUS: READY"
                    </div>
                </div>

                <div class="flex-1 grid grid-cols-1 xl:grid-cols-2 gap-4 min-h-[520px]">
                    <div class="flex flex-col bg-surface-container-lowest rounded-lg border border-outline-variant/20 overflow-hidden shadow-2xl">
                        <div class="bg-surface-container-low px-4 py-2 flex items-center justify-between border-b border-outline-variant/10">
                            <div class="flex items-center gap-4">
                                <div class="flex gap-1.5">
                                    <div class="w-2.5 h-2.5 rounded-full bg-error"></div>
                                    <div class="w-2.5 h-2.5 rounded-full bg-amber-400"></div>
                                    <div class="w-2.5 h-2.5 rounded-full bg-primary"></div>
                                </div>
                                <span class="text-[10px] font-mono text-on-surface-variant tracking-wider">"bash -- session-01"</span>
                            </div>
                            <button class="text-zinc-600 text-sm" on:click=move |_| set_guide_open.set(!guide_open.get())>
                                <span class="material-symbols-outlined">"help"</span>
                            </button>
                        </div>
                        <div class="flex-1 min-h-0">
                            <LivePreviewPanel
                                steps=steps
                                prompt_string=prompt_string
                                not_found_message=not_found_message
                                theme=theme
                                developer_mode=developer_mode
                                show_header=false
                                show_internal_guide=false
                                show_titlebar=false
                            />
                        </div>
                    </div>

                    <div class="flex flex-col bg-surface-container-lowest rounded-lg border border-outline-variant/20 overflow-hidden opacity-90">
                        <div class="bg-surface-container-low px-4 py-2 flex items-center justify-between border-b border-outline-variant/10">
                            <div class="flex items-center gap-4">
                                <div class="flex gap-1.5">
                                    <div class="w-2.5 h-2.5 rounded-full bg-zinc-700"></div>
                                    <div class="w-2.5 h-2.5 rounded-full bg-zinc-700"></div>
                                    <div class="w-2.5 h-2.5 rounded-full bg-zinc-700"></div>
                                </div>
                                <span class="text-[10px] font-mono text-on-surface-variant tracking-wider">"error-log -- debug-view"</span>
                            </div>
                            <span class="material-symbols-outlined text-zinc-600 text-sm">"close"</span>
                        </div>
                        <div class="flex-1 p-6 font-mono text-xs leading-relaxed overflow-y-auto bg-black/40 text-zinc-400">
                            <div class="mb-2">"2026-04-10 14:02:12 [ERROR] Failed to connect to mirror-primary.simu.dev"</div>
                            <div class="mb-2 text-error">"2026-04-10 14:02:12 [CRITICAL] Package 'cli-utils' not found in current environment."</div>
                            <div class="mb-2">"> Try running 'simu help' to see available packages."</div>
                            <div class="mt-6">"> simu run build-deploy"</div>
                            <div class="text-error mt-1">"zsh: command not found: simu"</div>
                        </div>
                    </div>
                </div>

            </div>

            <Show when=move || matches!(developer_mode.get(), true)>
                <GuidePanel steps=steps open=guide_open set_open=set_guide_open />
            </Show>
        </div>
    }
}

#[component]
fn UserPreviewState(
    steps: ReadSignal<Vec<Step>>,
    prompt_string: Signal<String>,
    not_found_message: Signal<String>,
    theme: ReadSignal<Option<Theme>>,
) -> impl IntoView {
    let user_mode = Signal::derive(|| false);

    view! {
        <div class="flex-1 flex h-[calc(100vh-3.5rem)] overflow-hidden">
            <aside class="bg-surface-container-low w-16 md:w-64 border-r border-outline-variant/10 opacity-40 pointer-events-none grayscale"></aside>
            <div class="flex-1 bg-surface flex items-center justify-center p-8 md:p-16">
                <div class="w-full max-w-5xl bg-surface-container shadow-2xl rounded-xl overflow-hidden flex flex-col min-h-[620px] relative">
                    <div class="h-10 bg-surface-container-highest flex items-center justify-between px-4">
                        <div class="flex items-center gap-2">
                            <div class="flex gap-1.5">
                                <div class="w-3 h-3 rounded-full bg-error/20 border border-error/40"></div>
                                <div class="w-3 h-3 rounded-full bg-zinc-700"></div>
                                <div class="w-3 h-3 rounded-full bg-zinc-700"></div>
                            </div>
                            <div class="ml-4 flex items-center gap-2 text-zinc-500 font-mono text-[11px] uppercase tracking-widest">
                                <span class="material-symbols-outlined text-[14px]">"desktop_windows"</span>
                                "Simulated-Terminal-v1"
                            </div>
                        </div>
                        <div class="flex items-center gap-4 text-zinc-400">
                            <span class="material-symbols-outlined text-[18px]">"add"</span>
                            <span class="material-symbols-outlined text-[18px]">"more_vert"</span>
                        </div>
                    </div>

                    <div class="flex-1 min-h-0">
                        <LivePreviewPanel
                            steps=steps
                            prompt_string=prompt_string
                            not_found_message=not_found_message
                            theme=theme
                            developer_mode=user_mode
                            show_header=false
                            show_internal_guide=false
                            show_titlebar=false
                        />
                    </div>

                    <button class="absolute bottom-6 right-6 flex items-center gap-2 bg-surface-container hover:bg-surface-container-high text-primary px-4 py-2 rounded-lg border border-primary/30 shadow-lg transition-all duration-200">
                        <span class="material-symbols-outlined text-[20px]">"help_center"</span>
                        <span class="text-xs font-bold uppercase tracking-wider">"User Guide"</span>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ImportPublishState(
    demo_id: String,
    published_slug: ReadSignal<String>,
    publish_modal_open: ReadSignal<bool>,
    set_publish_modal_open: WriteSignal<bool>,
    on_import_success: Callback<ImportCastResponse>,
) -> impl IntoView {
    view! {
        <div class="flex-1 overflow-y-auto p-8 bg-background">
            <div class="max-w-5xl mx-auto">
                <header class="mb-8">
                    <h1 class="text-2xl font-black tracking-tight text-on-surface mb-2">"Import Session"</h1>
                    <p class="text-sm text-on-surface-variant max-w-lg">
                        "Transform local terminal sessions into interactive browser demos. Upload a .cast file to begin."
                    </p>
                </header>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <div class="lg:col-span-2 space-y-6">
                        <div class="bg-surface-container border-2 border-dashed border-outline-variant p-8 rounded-xl">
                            <CastImportButton demo_id=demo_id on_success=on_import_success />
                        </div>

                        <div class="bg-surface-container-low p-6 rounded-xl border border-surface-container">
                            <div class="flex justify-between items-center mb-4">
                                <div class="flex items-center gap-3">
                                    <span class="material-symbols-outlined text-primary">"insert_drive_file"</span>
                                    <div>
                                        <p class="text-sm font-bold text-on-surface">"deployment_workflow_v2.cast"</p>
                                        <p class="text-[10px] font-mono text-on-surface-variant">"4.2 MB -- Processing frames..."</p>
                                    </div>
                                </div>
                                <span class="font-mono text-xs text-primary font-bold">"65%"</span>
                            </div>
                            <div class="h-1.5 w-full bg-surface-container-highest rounded-full overflow-hidden">
                                <div class="h-full bg-primary w-[65%] rounded-full"></div>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-6">
                        <div class="bg-surface-container p-5 rounded-xl">
                            <h4 class="font-mono text-[10px] uppercase tracking-widest text-on-surface-variant mb-4">"Validation Checks"</h4>
                            <ul class="space-y-4 text-xs">
                                <li class="text-on-surface">"Format Integrity: Valid JSON-lines structure detected."</li>
                                <li class="text-on-surface">"Timestamp Sync: Wait times normalized for player speed."</li>
                                <li class="text-on-surface-variant">"Metadata Extraction: Waiting for upload completion..."</li>
                            </ul>
                        </div>

                        <div class="bg-black p-4 rounded-xl border border-surface-container-high font-mono text-[10px] text-primary/70 leading-relaxed">
                            <p>"> init ingest_v2"</p>
                            <p>"> loading buffer [0x42A...]"</p>
                            <p>"> frame_count: 1,420"</p>
                            <p>"> duration: 04:12.4"</p>
                            <p class="text-primary font-bold">"> status: streaming_payload"</p>
                        </div>
                    </div>
                </div>
            </div>

            <Show when=move || publish_modal_open.get()>
                <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-background/80 backdrop-blur-md">
                    <div class="bg-surface-container border border-surface-container-highest w-full max-w-xl rounded-xl shadow-2xl overflow-hidden relative">
                        <div class="p-6 border-b border-surface-container-low flex justify-between items-center bg-surface-container-low">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 bg-primary/20 rounded-lg flex items-center justify-center">
                                    <span class="material-symbols-outlined text-primary">"check_circle"</span>
                                </div>
                                <div>
                                    <h2 class="text-lg font-black text-on-surface tracking-tight">"Publish Complete"</h2>
                                    <p class="text-xs text-on-surface-variant">"Your interactive demo is now live."</p>
                                </div>
                            </div>
                            <button class="text-on-surface-variant hover:text-on-surface transition-colors" on:click=move |_| set_publish_modal_open.set(false)>
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>

                        <div class="p-6 space-y-6">
                            <div>
                                <label class="font-mono text-[10px] uppercase tracking-widest text-on-surface-variant block mb-3">"Embed Code Snippet"</label>
                                <pre class="bg-surface-container-lowest p-5 rounded-lg border border-surface-container-high font-mono text-[11px] text-on-surface-variant leading-relaxed overflow-x-auto">{move || {
                                    format!(
                                        "<iframe src=\"{}/embed/{}\" width=\"100%\" height=\"450px\" frameborder=\"0\" allowfullscreen></iframe>",
                                        api::api_base(),
                                        published_slug.get(),
                                    )
                                }}</pre>
                            </div>

                            <div class="grid grid-cols-2 gap-4">
                                <button class="w-full py-3 bg-surface-container-highest hover:bg-surface-bright text-on-surface font-bold text-xs rounded-lg transition-all border border-outline-variant">
                                    "View Demo"
                                </button>
                                <button class="w-full py-3 bg-primary text-on-primary font-bold text-xs rounded-lg transition-all">
                                    "Go to Analytics"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ScriptBuilderState(
    title: ReadSignal<String>,
    set_title: WriteSignal<String>,
    slug: ReadSignal<String>,
    set_slug: WriteSignal<String>,
    settings: ReadSignal<Option<DemoSettings>>,
    set_settings: WriteSignal<Option<DemoSettings>>,
    theme: ReadSignal<Option<Theme>>,
    set_theme: WriteSignal<Option<Theme>>,
    steps: ReadSignal<Vec<Step>>,
    set_steps: WriteSignal<Vec<Step>>,
    step_filter: ReadSignal<String>,
    set_step_filter: WriteSignal<String>,
    prompt_string: Signal<String>,
    not_found_message: Signal<String>,
) -> impl IntoView {
    let developer_mode = Signal::derive(|| true);

    view! {
        <div class="flex-1 flex overflow-hidden bg-background">
            <section class="w-1/2 min-w-0 flex flex-col bg-surface border-r border-surface-container">
                <header class="p-6 border-b border-surface-container-low">
                    <div class="flex justify-between items-center mb-4">
                        <h2 class="text-xl font-black text-on-surface tracking-tight">"Script Steps"</h2>
                        <span class="text-[10px] font-mono text-zinc-500 uppercase tracking-widest bg-surface-container-low px-2 py-1 rounded">
                            {move || format!("{} Nodes", steps.get().len())}
                        </span>
                    </div>
                    <div class="relative">
                        <input
                            class="w-full bg-surface-container-low border-none rounded-lg px-4 py-2 text-xs text-on-surface placeholder:text-zinc-600 focus:ring-1 focus:ring-primary/50 font-mono"
                            placeholder="Filter steps..."
                            prop:value=move || step_filter.get()
                            on:input=move |ev| set_step_filter.set(event_target_value(&ev))
                        />
                    </div>
                </header>

                <div class="flex-1 overflow-y-auto p-6 space-y-6">
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
                    <StepListEditor steps=steps set_steps=set_steps filter=step_filter />
                </div>
            </section>

            <section class="w-1/2 min-w-0 flex flex-col bg-surface-container-lowest p-6 relative">
                <button class="absolute top-10 right-10 z-30 flex items-center gap-2 bg-surface-container/80 px-4 py-2 rounded-full border border-outline-variant/30 text-[10px] font-bold uppercase tracking-widest text-primary shadow-2xl">
                    <span class="material-symbols-outlined text-sm">"help_center"</span>
                    "Open Guide"
                </button>
                <div class="flex-1 min-h-0 rounded-xl overflow-hidden border border-outline-variant/20 shadow-2xl">
                    <LivePreviewPanel
                        steps=steps
                        prompt_string=prompt_string
                        not_found_message=not_found_message
                        theme=theme
                        developer_mode=developer_mode
                        show_header=false
                        show_internal_guide=false
                        show_titlebar=false
                    />
                </div>
            </section>
        </div>
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
        cursor_color: "#4ae176".to_string(),
        font_family: "JetBrains Mono".to_string(),
        font_size: 14,
        line_height: 1.4,
        prompt_string: "$".to_string(),
    }));
    let (status, set_status) = signal(String::new());
    let (published_slug, set_published_slug) = signal(String::new());
    let (step_filter, set_step_filter) = signal(String::new());
    let (view_mode, set_view_mode) = signal(CreatorViewMode::Developer);
    let (canvas_state, set_canvas_state) = signal(CanvasState::Workspace);
    let (publish_modal_open, set_publish_modal_open) = signal(false);
    let (guide_open, set_guide_open) = signal(true);
    let (publish_state, set_publish_state) = signal(PublishState::Idle);
    let theme_controller = use_context::<ThemeController>();
    let (_fallback_theme_mode, fallback_set_theme_mode) = signal(ThemeMode::Terminal);
    let theme_mode = Signal::derive(move || {
        theme_controller
            .as_ref()
            .map(|controller| controller.mode.get())
            .unwrap_or(ThemeMode::Terminal)
    });

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

    let save_demo = Callback::new(move |_| {
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

        set_publish_state.set(PublishState::Saving);
        set_status.set("Saving...".to_string());

        spawn_local_scoped({
            let set_status = set_status;
            let set_steps = set_steps;
            let set_publish_state = set_publish_state;
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
                        set_publish_state.set(PublishState::Idle);
                        set_status.set("Saved".to_string());
                    }
                    Err(err) => {
                        set_publish_state.set(PublishState::Error);
                        set_status.set(format!("Save failed: {err}"));
                    }
                }
            }
        });
    });

    let publish_demo = Callback::new(move |_| {
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

        set_publish_state.set(PublishState::Publishing);
        set_status.set("Publishing...".to_string());

        spawn_local_scoped({
            let set_status = set_status;
            let set_steps = set_steps;
            let set_published_slug = set_published_slug;
            let set_publish_modal_open = set_publish_modal_open;
            let set_publish_state = set_publish_state;
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
                                set_publish_state.set(PublishState::Success);
                                set_publish_modal_open.set(true);
                                set_status.set("Published and embed code ready".to_string());
                            }
                            Err(err) => {
                                set_publish_state.set(PublishState::Error);
                                set_status.set(format!("Publish failed: {err}"));
                            }
                        }
                    }
                    Err(err) => {
                        set_publish_state.set(PublishState::Error);
                        set_status.set(format!("Save failed: {err}"));
                    }
                }
            }
        });
    });

    let on_import_success = Callback::new(move |resp: ImportCastResponse| {
        set_steps_version.update(|v| *v += 1);
        set_status.set(resp.message);
    });

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

    let add_prompt_step = move |_| {
        set_steps.update(|items| {
            add_default_step(items, StepType::Prompt);
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

    let developer_mode = Signal::derive(move || matches!(view_mode.get(), CreatorViewMode::Developer));

    view! {
        <div class="min-h-screen bg-background text-on-surface overflow-hidden">
            <TopNav
                title=title
                set_title=set_title
                status=status
                view_mode=view_mode
                set_view_mode=set_view_mode
                canvas_state=canvas_state
                set_canvas_state=set_canvas_state
                theme_mode=theme_mode
                set_theme_mode=theme_controller
                    .as_ref()
                    .map(|controller| controller.set_mode)
                    .unwrap_or(fallback_set_theme_mode)
                on_save=save_demo
                on_publish=publish_demo
            />

            <div class="flex h-[calc(100vh-3.5rem)] overflow-hidden">
                <Show when=move || !matches!(view_mode.get(), CreatorViewMode::User)>
                    <SideNav />
                </Show>

                <main class="flex-1 overflow-hidden">
                    {move || {
                        if matches!(view_mode.get(), CreatorViewMode::User) {
                            view! {
                                <UserPreviewState
                                    steps=steps
                                    prompt_string=prompt_string
                                    not_found_message=not_found_message
                                    theme=theme
                                />
                            }
                                .into_any()
                        } else {
                            match canvas_state.get() {
                                CanvasState::Workspace => view! {
                                    <WorkspaceState
                                        steps=steps
                                        prompt_string=prompt_string
                                        not_found_message=not_found_message
                                        theme=theme
                                        developer_mode=developer_mode
                                        guide_open=guide_open
                                        set_guide_open=set_guide_open
                                    />
                                }
                                .into_any(),
                                CanvasState::ImportPublish => view! {
                                    <ImportPublishState
                                        demo_id=current_demo_id.clone()
                                        published_slug=published_slug
                                        publish_modal_open=publish_modal_open
                                        set_publish_modal_open=set_publish_modal_open
                                        on_import_success=on_import_success
                                    />
                                }
                                .into_any(),
                                CanvasState::ScriptBuilder => view! {
                                    <ScriptBuilderState
                                        title=title
                                        set_title=set_title
                                        slug=slug
                                        set_slug=set_slug
                                        settings=settings
                                        set_settings=set_settings
                                        theme=theme
                                        set_theme=set_theme
                                        steps=steps
                                        set_steps=set_steps
                                        step_filter=step_filter
                                        set_step_filter=set_step_filter
                                        prompt_string=prompt_string
                                        not_found_message=not_found_message
                                    />
                                }
                                .into_any(),
                            }
                        }
                    }}
                </main>
            </div>

            <div class="md:hidden fixed bottom-0 left-0 right-0 h-16 bg-[#0e0e10] flex items-center justify-around z-50 px-4 border-t border-[#19191d]">
                <button class="flex flex-col items-center gap-1 text-[#4ae176]" on:click=add_command_block>
                    <span class="material-symbols-outlined">"terminal"</span>
                    <span class="text-[10px] font-bold">"Command"</span>
                </button>
                <button class="flex flex-col items-center gap-1 text-zinc-500" on:click=add_output_step>
                    <span class="material-symbols-outlined">"output"</span>
                    <span class="text-[10px] font-bold">"Output"</span>
                </button>
                <button class="flex flex-col items-center gap-1 text-zinc-500" on:click=add_prompt_step>
                    <span class="material-symbols-outlined">"keyboard"</span>
                    <span class="text-[10px] font-bold">"Input"</span>
                </button>
                <button class="flex flex-col items-center gap-1 text-zinc-500" on:click=add_pause_step>
                    <span class="material-symbols-outlined">"pause_circle"</span>
                    <span class="text-[10px] font-bold">"Wait"</span>
                </button>
                <button
                    class="flex flex-col items-center gap-1 text-zinc-500"
                    on:click=move |_| {
                        set_canvas_state.set(CanvasState::ImportPublish);
                        if matches!(publish_state.get(), PublishState::Success) {
                            set_publish_modal_open.set(true);
                        }
                    }
                >
                    <span class="material-symbols-outlined">"publish"</span>
                    <span class="text-[10px] font-bold">"Publish"</span>
                </button>
            </div>

            <div class="hidden">
                <button on:click=add_command_step></button>
            </div>
        </div>
    }
}
