use leptos::prelude::*;
use std::collections::HashSet;
use shared::models::demo::{
    MatchMode, OutputLine, OutputStyle, Step, StepType,
};
use uuid::Uuid;

pub fn indexed_steps(steps: Vec<Step>) -> Vec<(usize, Step)> {
    steps.into_iter().enumerate().collect::<Vec<(usize, Step)>>()
}

pub fn normalize_step_orders(steps: &mut [Step]) {
    for (order, item) in steps.iter_mut().enumerate() {
        item.order = order as i32;
    }
}

fn reorder<T: Clone>(items: &[T], from_index: usize, to_index: usize) -> Vec<T> {
    if from_index >= items.len() || to_index >= items.len() || from_index == to_index {
        return items.to_vec();
    }

    let mut next = items.to_vec();
    let item = next.remove(from_index);
    next.insert(to_index, item);
    next
}

pub fn create_default_step(step_type: StepType, order: i32) -> Step {
    let mut step = Step {
        id: Uuid::new_v4(),
        step_type,
        order,
        input: None,
        match_mode: None,
        match_pattern: None,
        description: None,
        output: None,
        prompt_config: None,
        spinner_config: None,
        cta_config: None,
        delay_ms: 0,
        typing_speed_ms: 0,
        skippable: true,
    };

    match step.step_type {
        StepType::Command => {
            step.input = Some("echo hello".to_string());
            step.match_mode = Some(MatchMode::Exact);
            step.match_pattern = step.input.clone();
        }
        StepType::Output => {
            step.output = Some(vec![OutputLine {
                text: "Hello from CLI Demo Studio".to_string(),
                style: OutputStyle::Normal,
                color: None,
                prefix: None,
                indent: 0,
            }]);
        }
        StepType::Comment => {
            step.description = Some("Narration or hint".to_string());
        }
        _ => {}
    }

    step
}

pub fn add_default_step(steps: &mut Vec<Step>, step_type: StepType) {
    let order = steps.len() as i32;
    steps.push(create_default_step(step_type, order));
}

fn summarize_step(step: &Step) -> String {
    match step.step_type {
        StepType::Command => step
            .input
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "(empty command)".to_string()),
        StepType::Output => {
            let lines = step.output.clone().unwrap_or_default();
            if lines.is_empty() {
                return "(no output lines)".to_string();
            }

            let first = lines[0].text.clone();
            if lines.len() == 1 {
                first
            } else {
                format!("{} (+{} lines)", first, lines.len() - 1)
            }
        }
        StepType::Comment => step
            .description
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "(no comment)".to_string()),
        _ => step
            .description
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "(configure step)".to_string()),
    }
}

#[component]
pub fn StepListEditor(
    steps: ReadSignal<Vec<Step>>,
    set_steps: WriteSignal<Vec<Step>>,
) -> impl IntoView {
    let (expanded_steps, set_expanded_steps) = signal(HashSet::<Uuid>::new());

    view! {
        <For
            each=move || indexed_steps(steps.get())
            key=|entry| format!("{}-{}", entry.0, entry.1.id)
            children=move |(idx, step)| {
                let step_id = step.id;

                let is_expanded = Signal::derive(move || {
                    expanded_steps.with(|open| open.contains(&step_id))
                });

                let on_toggle = {
                    let set_expanded_steps = set_expanded_steps;
                    Callback::new(move |_| {
                        set_expanded_steps.update(|open| {
                            if !open.insert(step_id) {
                                open.remove(&step_id);
                            }
                        });
                    })
                };

                let on_remove = {
                    let set_steps = set_steps;
                    let set_expanded_steps = set_expanded_steps;
                    Callback::new(move |_| {
                        set_steps.update(|items| {
                            if idx < items.len() {
                                items.remove(idx);
                            }
                            normalize_step_orders(items);
                        });
                        set_expanded_steps.update(|open| {
                            open.remove(&step_id);
                        });
                    })
                };

                let on_move_up = {
                    let set_steps = set_steps;
                    Callback::new(move |_| {
                        set_steps.update(|items| {
                            if idx > 0 && idx < items.len() {
                                *items = reorder(items, idx, idx - 1);
                            }
                            normalize_step_orders(items);
                        });
                    })
                };

                let on_move_down = {
                    let set_steps = set_steps;
                    Callback::new(move |_| {
                        set_steps.update(|items| {
                            if idx + 1 < items.len() {
                                *items = reorder(items, idx, idx + 1);
                            }
                            normalize_step_orders(items);
                        });
                    })
                };

                let on_update = Callback::new({
                    let set_steps = set_steps;
                    move |next: Step| {
                        set_steps.update(|items| {
                            if let Some(item) = items.get_mut(idx) {
                                *item = next;
                            }
                        });
                    }
                });

                view! {
                    <StepCard
                        index=idx
                        step=step.clone()
                        expanded=is_expanded
                        on_toggle=on_toggle
                        on_move_up=on_move_up
                        on_move_down=on_move_down
                        on_remove=on_remove
                        on_update=on_update
                    />
                }
            }
        />
    }
}

#[component]
fn StepCard(
    index: usize,
    step: Step,
    expanded: Signal<bool>,
    on_toggle: Callback<web_sys::MouseEvent>,
    on_move_up: Callback<web_sys::MouseEvent>,
    on_move_down: Callback<web_sys::MouseEvent>,
    on_remove: Callback<web_sys::MouseEvent>,
    on_update: Callback<Step>,
) -> impl IntoView {
    let step_type_label = format!("{:?}", step.step_type);
    let step_summary = summarize_step(&step);
    let step_for_editor = step.clone();

    view! {
        <article class="step-card">
            <button type="button" class="step-card-toggle" on:click=move |ev| on_toggle.run(ev)>
                <span class="step-card-title">{format!("#{} {}", index + 1, step_type_label)}</span>
                <span class="step-card-summary">{step_summary}</span>
                <span class="step-card-indicator">
                    {move || if expanded.get() { "Collapse" } else { "Edit" }}
                </span>
            </button>

            <Show when=move || expanded.get()>
                <div class="step-card-body">
                    <header class="inline-actions">
                        <button type="button" on:click=move |ev| on_move_up.run(ev)>"Up"</button>
                        <button type="button" on:click=move |ev| on_move_down.run(ev)>"Down"</button>
                        <button type="button" on:click=move |ev| on_remove.run(ev)>"Remove"</button>
                    </header>
                    <StepEditorRouter step=step_for_editor.clone() on_update=on_update />
                </div>
            </Show>
        </article>
    }
}

#[component]
fn StepEditorRouter(step: Step, on_update: Callback<Step>) -> impl IntoView {
    match step.step_type {
        StepType::Command => view! { <CommandEditor step=step on_update=on_update /> }.into_any(),
        StepType::Output => view! { <OutputEditor step=step on_update=on_update /> }.into_any(),
        StepType::Comment => view! { <CommentEditor step=step on_update=on_update /> }.into_any(),
        StepType::Prompt => view! { <PromptEditor step=step on_update=on_update /> }.into_any(),
        StepType::Spinner => view! { <SpinnerEditor step=step on_update=on_update /> }.into_any(),
        StepType::Cta => view! { <CtaEditor step=step on_update=on_update /> }.into_any(),
        StepType::Pause => view! { <PauseEditor step=step on_update=on_update /> }.into_any(),
        StepType::Clear => view! { <ClearEditor step=step on_update=on_update /> }.into_any(),
    }
}

#[component]
fn CommandEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let command_value = step.input.clone().unwrap_or_default();
    let match_pattern_value = step
        .match_pattern
        .clone()
        .unwrap_or_else(|| command_value.clone());
    let step_for_command = step.clone();
    let step_for_pattern = step;

    view! {
        <label>
            "Command"
            <input
                prop:value=command_value
                on:input=move |ev| {
                    let next = event_target_value(&ev);
                    let mut updated = step_for_command.clone();
                    updated.input = Some(next.clone());
                    if updated.match_pattern.is_none() {
                        updated.match_pattern = Some(next);
                    }
                    on_update.run(updated);
                }
            />
        </label>
        <label>
            "Match pattern"
            <input
                prop:value=match_pattern_value
                on:input=move |ev| {
                    let next = event_target_value(&ev);
                    let mut updated = step_for_pattern.clone();
                    updated.match_pattern = Some(next);
                    on_update.run(updated);
                }
            />
        </label>
    }
}

#[component]
fn OutputEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let output_text = step
        .output
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|line| line.text)
        .collect::<Vec<_>>()
        .join("\\n");

    view! {
        <label>
            "Output lines"
            <textarea
                prop:value=output_text
                on:input=move |ev| {
                    let raw = event_target_value(&ev);
                    let lines = raw
                        .lines()
                        .map(|line| OutputLine {
                            text: line.to_string(),
                            style: OutputStyle::Normal,
                            color: None,
                            prefix: None,
                            indent: 0,
                        })
                        .collect::<Vec<_>>();

                    let mut updated = step.clone();
                    updated.output = Some(lines);
                    on_update.run(updated);
                }
            />
        </label>
    }
}

#[component]
fn CommentEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let description_value = step.description.clone().unwrap_or_default();

    view! {
        <label>
            "Description"
            <input
                prop:value=description_value
                on:input=move |ev| {
                    let next = event_target_value(&ev);
                    let mut updated = step.clone();
                    updated.description = Some(next);
                    on_update.run(updated);
                }
            />
        </label>
    }
}

#[component]
fn PromptEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let description_value = step.description.clone().unwrap_or_default();
    view! {
        <label>
            "Prompt step notes"
            <input
                prop:value=description_value
                on:input=move |ev| {
                    let next = event_target_value(&ev);
                    let mut updated = step.clone();
                    updated.description = Some(next);
                    on_update.run(updated);
                }
            />
        </label>
    }
}

#[component]
fn SpinnerEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let description_value = step.description.clone().unwrap_or_default();
    view! {
        <label>
            "Spinner step notes"
            <input
                prop:value=description_value
                on:input=move |ev| {
                    let next = event_target_value(&ev);
                    let mut updated = step.clone();
                    updated.description = Some(next);
                    on_update.run(updated);
                }
            />
        </label>
    }
}

#[component]
fn CtaEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let description_value = step.description.clone().unwrap_or_default();
    view! {
        <label>
            "CTA step notes"
            <input
                prop:value=description_value
                on:input=move |ev| {
                    let next = event_target_value(&ev);
                    let mut updated = step.clone();
                    updated.description = Some(next);
                    on_update.run(updated);
                }
            />
        </label>
    }
}

#[component]
fn PauseEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let delay_value = step.delay_ms.to_string();
    view! {
        <label>
            "Pause duration (ms)"
            <input
                type="number"
                min="0"
                prop:value=delay_value
                on:input=move |ev| {
                    let next = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                    let mut updated = step.clone();
                    updated.delay_ms = next;
                    on_update.run(updated);
                }
            />
        </label>
    }
}

#[component]
fn ClearEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let description_value = step.description.clone().unwrap_or_default();
    view! {
        <label>
            "Clear step notes"
            <input
                prop:value=description_value
                on:input=move |ev| {
                    let next = event_target_value(&ev);
                    let mut updated = step.clone();
                    updated.description = Some(next);
                    on_update.run(updated);
                }
            />
        </label>
    }
}
