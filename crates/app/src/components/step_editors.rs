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
    let (expanded_steps, set_expanded_steps) = signal(HashSet::<usize>::new());

    view! {
        <For
            each=move || indexed_steps(steps.get())
            key=|entry| format!("{}-{}", entry.0, entry.1.id)
            children=move |(idx, step)| {
                let expanded_steps = expanded_steps;
                let set_expanded_steps = set_expanded_steps;
                let on_remove = {
                    let set_steps = set_steps;
                    let set_expanded_steps = set_expanded_steps;
                    move |_| {
                        set_steps.update(|items| {
                            if idx < items.len() {
                                items.remove(idx);
                            }
                            normalize_step_orders(items);
                        });
                        set_expanded_steps.update(|open| {
                            open.remove(&idx);
                        });
                    }
                };

                let on_move_up = {
                    let set_steps = set_steps;
                    move |_| {
                        set_steps.update(|items| {
                            if idx > 0 && idx < items.len() {
                                *items = reorder(items, idx, idx - 1);
                            }
                            normalize_step_orders(items);
                        });
                    }
                };

                let on_move_down = {
                    let set_steps = set_steps;
                    move |_| {
                        set_steps.update(|items| {
                            if idx + 1 < items.len() {
                                *items = reorder(items, idx, idx + 1);
                            }
                            normalize_step_orders(items);
                        });
                    }
                };

                let step_type_label = format!("{:?}", step.step_type);
                let step_summary = summarize_step(&step);
                let command_value = step.input.clone().unwrap_or_default();
                let match_pattern_value = step
                    .match_pattern
                    .clone()
                    .unwrap_or_else(|| command_value.clone());
                let description_value = step.description.clone().unwrap_or_default();
                let output_text = step
                    .output
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|line| line.text)
                    .collect::<Vec<_>>()
                    .join("\\n");

                let is_expanded = Signal::derive(move || expanded_steps.with(|open| open.contains(&idx)));
                let toggle_expanded = {
                    let set_expanded_steps = set_expanded_steps;
                    move |_| {
                        set_expanded_steps.update(|open| {
                            if !open.insert(idx) {
                                open.remove(&idx);
                            }
                        });
                    }
                };

                view! {
                    <article class="step-card">
                        <button type="button" class="step-card-toggle" on:click=toggle_expanded>
                            <span class="step-card-title">{format!("#{} {}", idx + 1, step_type_label)}</span>
                            <span class="step-card-summary">{step_summary}</span>
                            <span class="step-card-indicator">
                                {move || if is_expanded.get() { "Collapse" } else { "Edit" }}
                            </span>
                        </button>

                        <Show when=move || is_expanded.get()>
                            <div class="step-card-body">
                                <header class="inline-actions">
                                    <button type="button" on:click=on_move_up>"Up"</button>
                                    <button type="button" on:click=on_move_down>"Down"</button>
                                    <button type="button" on:click=on_remove>"Remove"</button>
                                </header>

                                {match step.step_type {
                                    StepType::Command => {
                                        view! {
                                            <label>
                                                "Command"
                                                <input
                                                    prop:value=command_value.clone()
                                                    on:input={
                                                        let set_steps = set_steps;
                                                        move |ev| {
                                                            let next = event_target_value(&ev);
                                                            set_steps.update(|items| {
                                                                if let Some(item) = items.get_mut(idx) {
                                                                    item.input = Some(next.clone());
                                                                }
                                                            });
                                                        }
                                                    }
                                                />
                                            </label>
                                            <label>
                                                "Match pattern"
                                                <input
                                                    prop:value=match_pattern_value.clone()
                                                    on:input={
                                                        let set_steps = set_steps;
                                                        move |ev| {
                                                            let next = event_target_value(&ev);
                                                            set_steps.update(|items| {
                                                                if let Some(item) = items.get_mut(idx) {
                                                                    item.match_pattern = Some(next.clone());
                                                                }
                                                            });
                                                        }
                                                    }
                                                />
                                            </label>
                                        }
                                        .into_any()
                                    }
                                    StepType::Output => {
                                        view! {
                                            <label>
                                                "Output lines"
                                                <textarea
                                                    prop:value=output_text.clone()
                                                    on:input={
                                                        let set_steps = set_steps;
                                                        move |ev| {
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
                                                            set_steps.update(|items| {
                                                                if let Some(item) = items.get_mut(idx) {
                                                                    item.output = Some(lines.clone());
                                                                }
                                                            });
                                                        }
                                                    }
                                                />
                                            </label>
                                        }
                                        .into_any()
                                    }
                                    _ => {
                                        view! {
                                            <label>
                                                "Description"
                                                <input
                                                    prop:value=description_value.clone()
                                                    on:input={
                                                        let set_steps = set_steps;
                                                        move |ev| {
                                                            let next = event_target_value(&ev);
                                                            set_steps.update(|items| {
                                                                if let Some(item) = items.get_mut(idx) {
                                                                    item.description = Some(next.clone());
                                                                }
                                                            });
                                                        }
                                                    }
                                                />
                                            </label>
                                        }
                                        .into_any()
                                    }
                                }}
                            </div>
                        </Show>
                    </article>
                }
            }
        />
    }
}
