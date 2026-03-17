use leptos::prelude::*;
use shared::models::demo::{Step, StepType};

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
            <div class="preview-canvas">
                <p>{move || format!("{} demo preview", prompt_string.get())}</p>
                <For
                    each=move || indexed_steps(steps.get())
                    key=|entry| format!("{}-{}", entry.0, entry.1.id)
                    children=move |(_, step)| {
                        match step.step_type {
                            StepType::Command => {
                                view! {
                                    <p class="preview-line command-line">
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

                                view! { <pre class="preview-line output-line">{lines}</pre> }.into_any()
                            }
                            _ => {
                                view! {
                                    <p class="preview-line muted-line">
                                        {step.description.unwrap_or_else(|| "(meta step)".to_string())}
                                    </p>
                                }
                                .into_any()
                            }
                        }
                    }
                />

                <p class="preview-line fallback-line">{move || not_found_message.get()}</p>
            </div>
        </section>
    }
}
