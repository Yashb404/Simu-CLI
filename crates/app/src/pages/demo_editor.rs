use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use shared::{
    dto::UpdateDemoRequest,
    models::demo::{
        DemoSettings, MatchMode, OutputLine, OutputStyle, Step, StepType, Theme,
    },
};
use uuid::Uuid;

use crate::api;
use crate::components::live_preview::LivePreviewPanel;

fn indexed_steps(steps: Vec<Step>) -> Vec<(usize, Step)> {
    steps.into_iter().enumerate().collect::<Vec<(usize, Step)>>()
}

fn create_step(step_type: StepType, order: i32) -> Step {
    Step {
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
    }
}

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
    let (settings, set_settings) = signal(Option::<DemoSettings>::None);
    let (theme, set_theme) = signal(Option::<Theme>::None);
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let id = demo_id();
        if id == "unknown" {
            return;
        }

        spawn_local({
            let set_title = set_title;
            let set_slug = set_slug;
            let set_status = set_status;
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
        if next_title.trim().is_empty() {
            set_status.set("Title is required".to_string());
            return;
        }

        spawn_local({
            let set_status = set_status;
            async move {
                match api::update_demo_payload(
                    &id,
                    &UpdateDemoRequest {
                        title: Some(next_title.trim().to_string()),
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

    let add_command_step = move |_| {
        set_steps.update(|items| {
            let order = items.len() as i32;
            let mut step = create_step(StepType::Command, order);
            step.input = Some("echo hello".to_string());
            step.match_mode = Some(MatchMode::Exact);
            step.match_pattern = step.input.clone();
            items.push(step);
        });
    };

    let add_output_step = move |_| {
        set_steps.update(|items| {
            let order = items.len() as i32;
            let mut step = create_step(StepType::Output, order);
            step.output = Some(vec![OutputLine {
                text: "Hello from CLI Demo Studio".to_string(),
                style: OutputStyle::Normal,
                color: None,
                prefix: None,
                indent: 0,
            }]);
            items.push(step);
        });
    };

    let add_comment_step = move |_| {
        set_steps.update(|items| {
            let order = items.len() as i32;
            let mut step = create_step(StepType::Comment, order);
            step.description = Some("Narration or hint".to_string());
            items.push(step);
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
                <button type="button" on:click=save_demo>"Save Demo"</button>
            </section>

            <div class="editor-grid">
                <section class="step-column">
                    <h3>"Steps"</h3>
                    <div class="inline-actions">
                        <button type="button" on:click=add_command_step>"+ Command"</button>
                        <button type="button" on:click=add_output_step>"+ Output"</button>
                        <button type="button" on:click=add_comment_step>"+ Comment"</button>
                    </div>

                    <For
                        each=move || indexed_steps(steps.get())
                        key=|entry| format!("{}-{}", entry.0, entry.1.id)
                        children=move |(idx, step)| {
                            let on_remove = {
                                let set_steps = set_steps;
                                move |_| {
                                    set_steps.update(|items| {
                                        if idx < items.len() {
                                            items.remove(idx);
                                        }
                                        for (order, item) in items.iter_mut().enumerate() {
                                            item.order = order as i32;
                                        }
                                    });
                                }
                            };

                            let on_move_up = {
                                let set_steps = set_steps;
                                move |_| {
                                    set_steps.update(|items| {
                                        if idx > 0 && idx < items.len() {
                                            items.swap(idx, idx - 1);
                                        }
                                        for (order, item) in items.iter_mut().enumerate() {
                                            item.order = order as i32;
                                        }
                                    });
                                }
                            };

                            let on_move_down = {
                                let set_steps = set_steps;
                                move |_| {
                                    set_steps.update(|items| {
                                        if idx + 1 < items.len() {
                                            items.swap(idx, idx + 1);
                                        }
                                        for (order, item) in items.iter_mut().enumerate() {
                                            item.order = order as i32;
                                        }
                                    });
                                }
                            };

                            let step_type_label = format!("{:?}", step.step_type);
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

                            view! {
                                <article class="step-card">
                                    <header class="inline-actions">
                                        <strong>{format!("#{} {}", idx + 1, step_type_label)}</strong>
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
                                </article>
                            }
                        }
                    />
                </section>
                <LivePreviewPanel
                    steps=steps
                    prompt_string=prompt_string
                    not_found_message=not_found_message
                />
            </div>
        </section>
    }
}
