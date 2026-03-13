use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use shared::models::demo::{MatchMode, OutputStyle, Step, StepType};
use uuid::Uuid;

use crate::api;
use crate::components::live_preview::LivePreviewPanel;

fn new_command_step(order: i32) -> Step {
    Step {
        id: Uuid::new_v4(),
        step_type: StepType::Command,
        order,
        input: Some("echo hello".to_string()),
        match_mode: Some(MatchMode::Exact),
        match_pattern: None,
        description: Some("Run a command".to_string()),
        output: None,
        prompt_config: None,
        spinner_config: None,
        cta_config: None,
        delay_ms: 0,
        typing_speed_ms: 0,
        skippable: true,
    }
}

fn new_output_step(order: i32) -> Step {
    Step {
        id: Uuid::new_v4(),
        step_type: StepType::Output,
        order,
        input: None,
        match_mode: None,
        match_pattern: None,
        description: Some("Show output".to_string()),
        output: Some(vec![shared::models::demo::OutputLine {
            text: "sample output".to_string(),
            style: OutputStyle::Normal,
            color: None,
            prefix: None,
            indent: 0,
        }]),
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
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let id = demo_id();
        if id == "unknown" {
            return;
        }

        spawn_local({
            let set_title = set_title;
            let set_slug = set_slug;
            let set_steps = set_steps;
            let set_status = set_status;
            async move {
                match api::get_demo(&id).await {
                    Ok(demo) => {
                        set_title.set(demo.title);
                        set_slug.set(demo.slug.unwrap_or_default());
                        set_steps.set(demo.steps);
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
                match api::update_demo(
                    &id,
                    Some(next_title.trim()),
                    if next_slug.trim().is_empty() {
                        None
                    } else {
                        Some(next_slug.trim())
                    },
                    Some(next_steps),
                )
                .await
                {
                    Ok(_) => set_status.set("Saved".to_string()),
                    Err(err) => set_status.set(format!("Save failed: {err}")),
                }
            }
        });
    };

    let add_command_step = move |_| {
        set_steps.update(|items| {
            let order = items.len() as i32;
            items.push(new_command_step(order));
        });
    };

    let add_output_step = move |_| {
        set_steps.update(|items| {
            let order = items.len() as i32;
            items.push(new_output_step(order));
        });
    };

    let remove_step = move |index: usize| {
        set_steps.update(|items| {
            if index < items.len() {
                items.remove(index);
                for (i, step) in items.iter_mut().enumerate() {
                    step.order = i as i32;
                }
            }
        });
    };

    let update_step_input = move |index: usize, value: String| {
        set_steps.update(|items| {
            if let Some(step) = items.get_mut(index) {
                step.input = if value.trim().is_empty() { None } else { Some(value) };
            }
        });
    };

    let update_step_description = move |index: usize, value: String| {
        set_steps.update(|items| {
            if let Some(step) = items.get_mut(index) {
                step.description = if value.trim().is_empty() { None } else { Some(value) };
            }
        });
    };

    let indexed_steps = move || {
        let mut items = Vec::new();
        for (index, step) in steps.get().into_iter().enumerate() {
            items.push((index, step));
        }
        items
    };

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
                        <button type="button" on:click=add_command_step>"Add Command Step"</button>
                        <button type="button" on:click=add_output_step>"Add Output Step"</button>
                    </div>

                    <For
                        each=indexed_steps
                        key=|(index, step)| format!("{}-{}", index, step.id)
                        children=move |(index, step)| {
                            let kind = format!("{:?}", step.step_type);
                            view! {
                                <article class="step-card">
                                    <div class="inline-actions">
                                        <strong>{format!("#{} {}", index + 1, kind)}</strong>
                                        <button type="button" on:click=move |_| remove_step(index)>
                                            "Remove"
                                        </button>
                                    </div>
                                    <label>
                                        "Command/Input"
                                        <input
                                            prop:value=step.input.unwrap_or_default()
                                            on:input=move |ev| update_step_input(index, event_target_value(&ev))
                                        />
                                    </label>
                                    <label>
                                        "Description"
                                        <input
                                            prop:value=step.description.unwrap_or_default()
                                            on:input=move |ev| update_step_description(index, event_target_value(&ev))
                                        />
                                    </label>
                                </article>
                            }
                        }
                    />
                </section>
                <LivePreviewPanel />
            </div>
        </section>
    }
}
