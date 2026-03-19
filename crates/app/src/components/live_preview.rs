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
        <section class="live-preview-panel">
            <h3>"Live Preview"</h3>
            <div class="terminal-chrome">
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
                                    view! {
                                        <p class="terminal-line cmd">
                                            {format!(
                                                "{} {}",
                                                prompt_string.get(),
                                                step.input.unwrap_or_else(|| "<command>".to_string())
                                            )}
                                        </p>
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
