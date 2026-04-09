use leptos::prelude::*;
use shared::models::demo::{Step, StepType};

const META_STEP_FALLBACK: &str = "(meta step)";

fn indexed_steps(steps: Vec<Step>) -> Vec<(usize, Step)> {
    steps.into_iter().enumerate().collect::<Vec<(usize, Step)>>()
}

#[component]
pub fn LivePreviewPanel(
    steps: ReadSignal<Vec<Step>>,
    prompt_string: Signal<String>,
    not_found_message: Signal<String>,
) -> impl IntoView {
    view! {
        <section class="live-preview-panel flex h-full min-h-0 flex-col gap-4">
            <div class="rounded-2xl border border-zinc-800/70 bg-zinc-950/80 p-4 shadow-[0_24px_80px_-38px_rgba(0,0,0,0.95)]">
                <div class="flex items-start justify-between gap-4">
                    <div class="space-y-1">
                        <p class="text-[11px] uppercase tracking-[0.2em] text-zinc-500">"Preview"</p>
                        <h3 class="text-lg font-semibold text-zinc-50">"Interactive Runtime"</h3>
                        <p class="text-sm text-zinc-400">"This mirrors the runtime payload the embed app receives, including guide descriptions and fallback output."</p>
                    </div>
                </div>
            </div>
            <div class="terminal-chrome flex-1 overflow-hidden rounded-[28px] border border-zinc-800/80 bg-[#050505] shadow-[0_26px_100px_-42px_rgba(0,0,0,1)]">
                <div class="terminal-titlebar">
                    <div class="terminal-dots">
                        <span class="terminal-dot red"></span>
                        <span class="terminal-dot yellow"></span>
                        <span class="terminal-dot green"></span>
                    </div>
                    <span class="terminal-titlebar-text">{move || prompt_string.get()}</span>
                    <span class="cursor-blink" style="color:var(--ink);padding-left:4px;">"█"</span>
                </div>
                <div class="terminal-body">
                    <For
                        each=move || indexed_steps(steps.get())
                        key=|entry| format!("{}-{}", entry.0, entry.1.id)
                        children=move |(_, step)| {
                            match step.step_type {
                                StepType::Command => {
                                    let preview_label = step
                                        .short_description
                                        .clone()
                                        .filter(|value| !value.trim().is_empty())
                                        .unwrap_or_default();
                                    let has_preview_label = !preview_label.trim().is_empty();
                                    view! {
                                        <div class="space-y-1">
                                            <p class="terminal-line cmd">
                                                {format!(
                                                    "{} {}",
                                                    prompt_string.get(),
                                                    step.input.unwrap_or_else(|| "<command>".to_string())
                                                )}
                                            </p>
                                            <Show when=move || has_preview_label>
                                                <p class="terminal-line comment text-xs opacity-80">
                                                    {preview_label.clone()}
                                                </p>
                                            </Show>
                                        </div>
                                    }
                                    .into_any()
                                }
                                StepType::Output => {
                                    let lines = step
                                        .output
                                        .unwrap_or_default()
                                        .into_iter()
                                        .map(|line| line.text)
                                        .collect::<Vec<_>>()
                                        .join("\n");

                                    view! { <pre class="terminal-line">{lines}</pre> }.into_any()
                                }
                                _ => {
                                    view! {
                                        <p class="terminal-line comment">
                                            {format!("# {}", step.description.unwrap_or_else(|| META_STEP_FALLBACK.to_string()))}
                                        </p>
                                    }
                                    .into_any()
                                }
                            }
                        }
                    />
                    <p class="terminal-line comment">{move || format!("# {}", not_found_message.get())}</p>
                </div>
            </div>
        </section>
    }
}
