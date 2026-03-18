use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use shared::models::demo::{MatchMode, OutputLine, OutputStyle, Step, StepType};
use shared::validation::{is_valid_slug, MAX_OUTPUT_LINES_PER_STEP, MAX_STEPS};
use uuid::Uuid;

use crate::api;
use crate::components::live_preview::LivePreviewPanel;
use crate::utils::dnd::{reorder, DndReorder};

fn normalize_step_order(items: &mut [Step]) {
    for (index, step) in items.iter_mut().enumerate() {
        step.order = index as i32;
    }
}

fn default_output_line() -> OutputLine {
    OutputLine {
        text: "sample output".to_string(),
        style: OutputStyle::Normal,
        color: None,
        prefix: None,
        indent: 0,
    }
}

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
        output: Some(vec![default_output_line()]),
        prompt_config: None,
        spinner_config: None,
        cta_config: None,
        delay_ms: 0,
        typing_speed_ms: 0,
        skippable: true,
    }
}

fn step_type_value(step_type: &StepType) -> &'static str {
    match step_type {
        StepType::Command => "command",
        StepType::Output => "output",
        StepType::Prompt => "prompt",
        StepType::Spinner => "spinner",
        StepType::Comment => "comment",
        StepType::Clear => "clear",
        StepType::Pause => "pause",
        StepType::Cta => "cta",
    }
}

fn parse_step_type(value: &str) -> StepType {
    match value {
        "output" => StepType::Output,
        "prompt" => StepType::Prompt,
        "spinner" => StepType::Spinner,
        "comment" => StepType::Comment,
        "clear" => StepType::Clear,
        "pause" => StepType::Pause,
        "cta" => StepType::Cta,
        _ => StepType::Command,
    }
}

fn match_mode_value(mode: Option<&MatchMode>) -> &'static str {
    match mode {
        Some(MatchMode::Fuzzy) => "fuzzy",
        Some(MatchMode::Wildcard) => "wildcard",
        Some(MatchMode::Any) => "any",
        _ => "exact",
    }
}

fn parse_match_mode(value: &str) -> MatchMode {
    match value {
        "fuzzy" => MatchMode::Fuzzy,
        "wildcard" => MatchMode::Wildcard,
        "any" => MatchMode::Any,
        _ => MatchMode::Exact,
    }
}

fn output_style_value(style: &OutputStyle) -> &'static str {
    match style {
        OutputStyle::Success => "success",
        OutputStyle::Error => "error",
        OutputStyle::Warning => "warning",
        OutputStyle::Muted => "muted",
        OutputStyle::Bold => "bold",
        OutputStyle::Code => "code",
        OutputStyle::Normal => "normal",
    }
}

fn parse_output_style(value: &str) -> OutputStyle {
    match value {
        "success" => OutputStyle::Success,
        "error" => OutputStyle::Error,
        "warning" => OutputStyle::Warning,
        "muted" => OutputStyle::Muted,
        "bold" => OutputStyle::Bold,
        "code" => OutputStyle::Code,
        _ => OutputStyle::Normal,
    }
}

fn apply_step_type_defaults(step: &mut Step) {
    match step.step_type {
        StepType::Command => {
            if step.input.as_ref().is_none_or(|v| v.trim().is_empty()) {
                step.input = Some("echo hello".to_string());
            }
            if step.match_mode.is_none() {
                step.match_mode = Some(MatchMode::Exact);
            }
            step.output = None;
        }
        StepType::Output => {
            step.input = None;
            step.match_mode = None;
            if step.output.is_none() {
                step.output = Some(vec![default_output_line()]);
            }
        }
        _ => {
            step.match_mode = None;
            if !matches!(step.step_type, StepType::Output) {
                step.output = None;
            }
        }
    }
}

fn validate_editor_payload(title: &str, slug: &str, steps: &[Step]) -> Vec<String> {
    let mut errors = Vec::new();

    if title.trim().is_empty() {
        errors.push("Title is required.".to_string());
    }

    if !slug.trim().is_empty() && !is_valid_slug(slug.trim()) {
        errors.push("Slug must be 3-60 chars with lowercase letters, numbers, and hyphens.".to_string());
    }

    if steps.len() > MAX_STEPS {
        errors.push(format!("Steps cannot exceed {} entries.", MAX_STEPS));
    }

    for (index, step) in steps.iter().enumerate() {
        if let Some(input) = &step.input {
            if input.trim().is_empty() || input.len() > 200 {
                errors.push(format!("Step {} input must be non-empty and <= 200 chars.", index + 1));
            }
        }

        if let Some(lines) = &step.output {
            if lines.len() > MAX_OUTPUT_LINES_PER_STEP {
                errors.push(format!("Step {} output lines cannot exceed {}.", index + 1, MAX_OUTPUT_LINES_PER_STEP));
            }

            for line in lines {
                if line.text.len() > 500 {
                    errors.push(format!("Step {} has an output line longer than 500 chars.", index + 1));
                    break;
                }
            }
        }
    }

    errors
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
    let (validation_errors, set_validation_errors) = signal(Vec::<String>::new());
    let (is_saving, set_is_saving) = signal(false);
    let (dragged_step_index, set_dragged_step_index) = signal(Option::<usize>::None);

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
        let mut next_steps = steps.get();

        normalize_step_order(&mut next_steps);

        let errors = validate_editor_payload(&next_title, &next_slug, &next_steps);
        if !errors.is_empty() {
            set_validation_errors.set(errors);
            set_status.set("Fix validation errors before saving.".to_string());
            return;
        }

        set_validation_errors.set(Vec::new());

        if id == "unknown" {
            set_status.set("Invalid demo id".to_string());
            return;
        }

        set_is_saving.set(true);
        set_status.set("Saving...".to_string());

        spawn_local({
            let set_status = set_status;
            let set_is_saving = set_is_saving;
            let set_steps = set_steps;
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
                    Ok(saved) => {
                        set_steps.update(|items| normalize_step_order(items));
                        set_status.set(format!("Saved v{}", saved.version));
                    }
                    Err(err) => set_status.set(format!("Save failed: {err}")),
                }
                set_is_saving.set(false);
            }
        });
    };

    let add_command_step = move |_| {
        set_steps.update(|items| {
            let order = items.len() as i32;
            items.push(new_command_step(order));
            normalize_step_order(items);
        });
        set_status.set("Unsaved changes".to_string());
    };

    let add_output_step = move |_| {
        set_steps.update(|items| {
            let order = items.len() as i32;
            items.push(new_output_step(order));
            normalize_step_order(items);
        });
        set_status.set("Unsaved changes".to_string());
    };

    let remove_step = move |index: usize| {
        set_steps.update(|items| {
            if index < items.len() {
                items.remove(index);
                normalize_step_order(items);
            }
        });
        set_status.set("Unsaved changes".to_string());
    };

    let update_step_input = move |index: usize, value: String| {
        set_steps.update(|items| {
            if let Some(step) = items.get_mut(index) {
                step.input = if value.trim().is_empty() { None } else { Some(value) };
            }
        });
        set_status.set("Unsaved changes".to_string());
    };

    let update_step_description = move |index: usize, value: String| {
        set_steps.update(|items| {
            if let Some(step) = items.get_mut(index) {
                step.description = if value.trim().is_empty() { None } else { Some(value) };
            }
        });
        set_status.set("Unsaved changes".to_string());
    };

    let move_step_up = move |index: usize| {
        set_steps.update(|items| {
            if index > 0 && index < items.len() {
                items.swap(index - 1, index);
                normalize_step_order(items);
            }
        });
        set_status.set("Unsaved changes".to_string());
    };

    let move_step_down = move |index: usize| {
        set_steps.update(|items| {
            if index + 1 < items.len() {
                items.swap(index, index + 1);
                normalize_step_order(items);
            }
        });
        set_status.set("Unsaved changes".to_string());
    };

    let apply_dnd_reorder = move |request: DndReorder| {
        set_steps.update(|items| {
            let next = reorder(items, request.from_index, request.to_index);
            *items = next;
            normalize_step_order(items);
        });
        set_status.set("Unsaved changes (reordered)".to_string());
    };

    let indexed_steps = move || {
        let mut items = Vec::new();
        for (index, step) in steps.get().into_iter().enumerate() {
            items.push((index, step));
        }
        items
    };

    let validation_items = move || {
        validation_errors
            .get()
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>()
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
                <button type="button" disabled=move || is_saving.get() on:click=save_demo>
                    {move || if is_saving.get() { "Saving..." } else { "Save Demo" }}
                </button>
            </section>

            <Show
                when=move || !validation_errors.get().is_empty()
                fallback=|| ()
            >
                <section class="panel">
                    <h3>"Fix These Before Save"</h3>
                    <ul class="list">
                        <For
                            each=validation_items
                            key=|(idx, _)| *idx
                            children=move |(_, msg)| {
                                view! { <li><p>{msg}</p></li> }
                            }
                        />
                    </ul>
                </section>
            </Show>

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
                            let is_command = matches!(step.step_type, StepType::Command);
                            let is_output = matches!(step.step_type, StepType::Output);
                            let output_line_items = move || {
                                steps
                                    .get()
                                    .get(index)
                                    .and_then(|s| s.output.clone())
                                    .unwrap_or_default()
                                    .into_iter()
                                    .enumerate()
                                    .collect::<Vec<_>>()
                            };
                            view! {
                                <article
                                    class="step-card"
                                    draggable="true"
                                    on:dragstart=move |_| set_dragged_step_index.set(Some(index))
                                    on:dragover=move |ev| ev.prevent_default()
                                    on:drop=move |ev| {
                                        ev.prevent_default();
                                        if let Some(from_index) = dragged_step_index.get() {
                                            apply_dnd_reorder(DndReorder { from_index, to_index: index });
                                        }
                                        set_dragged_step_index.set(None);
                                    }
                                    on:dragend=move |_| set_dragged_step_index.set(None)
                                >
                                    <div class="inline-actions">
                                        <strong>{format!("#{} {}", index + 1, kind)}</strong>
                                        <span class="muted">"Drag to reorder"</span>
                                        <button type="button" on:click=move |_| move_step_up(index) disabled=index == 0>
                                            "Move Up"
                                        </button>
                                        <button
                                            type="button"
                                            on:click=move |_| move_step_down(index)
                                            disabled=move || index + 1 >= steps.get().len()
                                        >
                                            "Move Down"
                                        </button>
                                        <button type="button" on:click=move |_| remove_step(index)>
                                            "Remove"
                                        </button>
                                    </div>

                                    <label>
                                        "Step Type"
                                        <select
                                            prop:value=step_type_value(&step.step_type)
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                set_steps.update(|items| {
                                                    if let Some(step) = items.get_mut(index) {
                                                        step.step_type = parse_step_type(&value);
                                                        apply_step_type_defaults(step);
                                                        normalize_step_order(items);
                                                    }
                                                });
                                                set_status.set("Unsaved changes".to_string());
                                            }
                                        >
                                            <option value="command">"Command"</option>
                                            <option value="output">"Output"</option>
                                            <option value="prompt">"Prompt"</option>
                                            <option value="spinner">"Spinner"</option>
                                            <option value="comment">"Comment"</option>
                                            <option value="clear">"Clear"</option>
                                            <option value="pause">"Pause"</option>
                                            <option value="cta">"CTA"</option>
                                        </select>
                                    </label>

                                    <div class="inline-actions">
                                        <label>
                                            "Delay (ms)"
                                            <input
                                                type="number"
                                                min="0"
                                                prop:value=step.delay_ms
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                                                    set_steps.update(|items| {
                                                        if let Some(step) = items.get_mut(index) {
                                                            step.delay_ms = value;
                                                        }
                                                    });
                                                    set_status.set("Unsaved changes".to_string());
                                                }
                                            />
                                        </label>
                                        <label>
                                            "Typing Speed (ms)"
                                            <input
                                                type="number"
                                                min="0"
                                                prop:value=step.typing_speed_ms
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                                                    set_steps.update(|items| {
                                                        if let Some(step) = items.get_mut(index) {
                                                            step.typing_speed_ms = value;
                                                        }
                                                    });
                                                    set_status.set("Unsaved changes".to_string());
                                                }
                                            />
                                        </label>
                                        <label>
                                            "Skippable"
                                            <input
                                                type="checkbox"
                                                prop:checked=step.skippable
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    set_steps.update(|items| {
                                                        if let Some(step) = items.get_mut(index) {
                                                            step.skippable = checked;
                                                        }
                                                    });
                                                    set_status.set("Unsaved changes".to_string());
                                                }
                                            />
                                        </label>
                                    </div>

                                    <label>
                                        "Command/Input"
                                        <input
                                            disabled=!is_command
                                            prop:value=step.input.unwrap_or_default()
                                            on:input=move |ev| update_step_input(index, event_target_value(&ev))
                                        />
                                    </label>

                                    <Show when=move || is_command fallback=|| ()>
                                        <label>
                                            "Match Mode"
                                            <select
                                                prop:value=match_mode_value(step.match_mode.as_ref())
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    set_steps.update(|items| {
                                                        if let Some(step) = items.get_mut(index) {
                                                            step.match_mode = Some(parse_match_mode(&value));
                                                        }
                                                    });
                                                    set_status.set("Unsaved changes".to_string());
                                                }
                                            >
                                                <option value="exact">"Exact"</option>
                                                <option value="fuzzy">"Fuzzy"</option>
                                                <option value="wildcard">"Wildcard"</option>
                                                <option value="any">"Any"</option>
                                            </select>
                                        </label>
                                    </Show>

                                    <label>
                                        "Description"
                                        <input
                                            prop:value=step.description.unwrap_or_default()
                                            on:input=move |ev| update_step_description(index, event_target_value(&ev))
                                        />
                                    </label>

                                    <Show when=move || is_output fallback=|| ()>
                                        <section class="panel">
                                            <div class="inline-actions">
                                                <h4>"Output Lines"</h4>
                                                <button
                                                    type="button"
                                                    on:click=move |_| {
                                                        set_steps.update(|items| {
                                                            if let Some(step) = items.get_mut(index) {
                                                                let lines = step.output.get_or_insert_with(Vec::new);
                                                                if lines.len() < MAX_OUTPUT_LINES_PER_STEP {
                                                                    lines.push(default_output_line());
                                                                }
                                                            }
                                                        });
                                                        set_status.set("Unsaved changes".to_string());
                                                    }
                                                >
                                                    "Add Line"
                                                </button>
                                            </div>

                                            <For
                                                each=output_line_items
                                                key=|(line_index, _)| *line_index
                                                children=move |(line_index, output_line)| {
                                                    view! {
                                                        <div class="panel">
                                                            <div class="inline-actions">
                                                                <strong>{format!("Line {}", line_index + 1)}</strong>
                                                                <button
                                                                    type="button"
                                                                    on:click=move |_| {
                                                                        set_steps.update(|items| {
                                                                            if let Some(step) = items.get_mut(index) {
                                                                                if let Some(lines) = step.output.as_mut() {
                                                                                    if line_index < lines.len() {
                                                                                        lines.remove(line_index);
                                                                                    }
                                                                                    if lines.is_empty() {
                                                                                        lines.push(default_output_line());
                                                                                    }
                                                                                }
                                                                            }
                                                                        });
                                                                        set_status.set("Unsaved changes".to_string());
                                                                    }
                                                                >
                                                                    "Remove"
                                                                </button>
                                                            </div>
                                                            <label>
                                                                "Text"
                                                                <input
                                                                    prop:value=output_line.text
                                                                    on:input=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        set_steps.update(|items| {
                                                                            if let Some(step) = items.get_mut(index) {
                                                                                if let Some(lines) = step.output.as_mut() {
                                                                                    if let Some(current) = lines.get_mut(line_index) {
                                                                                        current.text = value;
                                                                                    }
                                                                                }
                                                                            }
                                                                        });
                                                                        set_status.set("Unsaved changes".to_string());
                                                                    }
                                                                />
                                                            </label>
                                                            <label>
                                                                "Style"
                                                                <select
                                                                    prop:value=output_style_value(&output_line.style)
                                                                    on:change=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        set_steps.update(|items| {
                                                                            if let Some(step) = items.get_mut(index) {
                                                                                if let Some(lines) = step.output.as_mut() {
                                                                                    if let Some(current) = lines.get_mut(line_index) {
                                                                                        current.style = parse_output_style(&value);
                                                                                    }
                                                                                }
                                                                            }
                                                                        });
                                                                        set_status.set("Unsaved changes".to_string());
                                                                    }
                                                                >
                                                                    <option value="normal">"Normal"</option>
                                                                    <option value="success">"Success"</option>
                                                                    <option value="error">"Error"</option>
                                                                    <option value="warning">"Warning"</option>
                                                                    <option value="muted">"Muted"</option>
                                                                    <option value="bold">"Bold"</option>
                                                                    <option value="code">"Code"</option>
                                                                </select>
                                                            </label>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </section>
                                    </Show>
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
