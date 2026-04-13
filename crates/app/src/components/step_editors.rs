use leptos::prelude::*;
use shared::models::demo::{
    CtaConfig, MatchMode, OutputLine, OutputStyle, PromptConfig, PromptType, SpinnerConfig,
    SpinnerStyle, Step, StepType,
};
use std::collections::HashSet;
use uuid::Uuid;

pub fn indexed_steps(steps: Vec<Step>) -> Vec<(usize, Step)> {
    steps
        .into_iter()
        .enumerate()
        .collect::<Vec<(usize, Step)>>()
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

fn output_lines_to_text(lines: &[OutputLine]) -> String {
    lines
        .iter()
        .map(|line| line.text.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn text_to_output_lines(raw: &str) -> Vec<OutputLine> {
    raw.lines()
        .map(|line| OutputLine {
            text: line.to_string(),
            style: OutputStyle::Normal,
            color: None,
            prefix: None,
            indent: 0,
        })
        .collect::<Vec<_>>()
}

fn default_prompt_config() -> PromptConfig {
    PromptConfig {
        prompt_type: PromptType::Confirm,
        question: "Continue?".to_string(),
        choices: None,
        default_value: Some("yes".to_string()),
        yes_output: Some(vec![OutputLine {
            text: "Continuing...".to_string(),
            style: OutputStyle::Success,
            color: None,
            prefix: None,
            indent: 0,
        }]),
        no_output: Some(vec![OutputLine {
            text: "Cancelled".to_string(),
            style: OutputStyle::Warning,
            color: None,
            prefix: None,
            indent: 0,
        }]),
    }
}

fn default_spinner_config() -> SpinnerConfig {
    SpinnerConfig {
        style: SpinnerStyle::Dots,
        label: "Working...".to_string(),
        duration_ms: 1200,
        finish_text: "Done".to_string(),
        finish_style: OutputStyle::Success,
    }
}

fn default_cta_config() -> CtaConfig {
    CtaConfig {
        message: "Learn more about this demo".to_string(),
        primary_label: "Open Docs".to_string(),
        primary_url: "https://example.com/docs".to_string(),
        secondary_label: Some("GitHub".to_string()),
        secondary_url: Some("https://github.com".to_string()),
    }
}

fn prompt_type_to_str(kind: &PromptType) -> &'static str {
    match kind {
        PromptType::Confirm => "confirm",
        PromptType::Input => "input",
        PromptType::Password => "password",
        PromptType::Select => "select",
    }
}

fn prompt_type_from_str(value: &str) -> PromptType {
    match value {
        "input" => PromptType::Input,
        "password" => PromptType::Password,
        "select" => PromptType::Select,
        _ => PromptType::Confirm,
    }
}

fn spinner_style_to_str(style: &SpinnerStyle) -> &'static str {
    match style {
        SpinnerStyle::Dots => "dots",
        SpinnerStyle::Bar => "bar",
        SpinnerStyle::Braille => "braille",
        SpinnerStyle::Line => "line",
    }
}

fn spinner_style_from_str(value: &str) -> SpinnerStyle {
    match value {
        "bar" => SpinnerStyle::Bar,
        "braille" => SpinnerStyle::Braille,
        "line" => SpinnerStyle::Line,
        _ => SpinnerStyle::Dots,
    }
}

fn output_style_to_str(style: &OutputStyle) -> &'static str {
    match style {
        OutputStyle::Normal => "normal",
        OutputStyle::Success => "success",
        OutputStyle::Error => "error",
        OutputStyle::Warning => "warning",
        OutputStyle::Muted => "muted",
        OutputStyle::Bold => "bold",
        OutputStyle::Code => "code",
    }
}

fn output_style_from_str(value: &str) -> OutputStyle {
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

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn is_valid_url_like(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("https://") || value.starts_with("http://")
}

fn validate_prompt_config(config: &PromptConfig) -> Vec<String> {
    let mut messages = Vec::new();

    if is_blank(&config.question) {
        messages.push("Question is required.".to_string());
    }

    if matches!(config.prompt_type, PromptType::Select) {
        let choices = config.choices.clone().unwrap_or_default();
        if choices.is_empty() {
            messages.push("Select prompts should define at least one choice.".to_string());
        }
        if let Some(default) = &config.default_value
            && !is_blank(default)
            && !choices.iter().any(|choice| choice == default)
        {
            messages.push("Default value should match one of the Select choices.".to_string());
        }
    }

    if matches!(config.prompt_type, PromptType::Confirm) {
        if config.yes_output.clone().unwrap_or_default().is_empty() {
            messages
                .push("Confirm prompts should include at least one yes-output line.".to_string());
        }
        if config.no_output.clone().unwrap_or_default().is_empty() {
            messages
                .push("Confirm prompts should include at least one no-output line.".to_string());
        }
    }

    messages
}

fn validate_spinner_config(config: &SpinnerConfig) -> Vec<String> {
    let mut messages = Vec::new();

    if is_blank(&config.label) {
        messages.push("Spinner label is required.".to_string());
    }
    if config.duration_ms == 0 {
        messages.push("Duration should be greater than 0 ms.".to_string());
    }
    if is_blank(&config.finish_text) {
        messages.push("Finish text is required.".to_string());
    }

    messages
}

fn validate_cta_config(config: &CtaConfig) -> Vec<String> {
    let mut messages = Vec::new();

    if is_blank(&config.message) {
        messages.push("CTA message is required.".to_string());
    }
    if is_blank(&config.primary_label) {
        messages.push("Primary label is required.".to_string());
    }
    if is_blank(&config.primary_url) {
        messages.push("Primary URL is required.".to_string());
    } else if !is_valid_url_like(&config.primary_url) {
        messages.push("Primary URL should start with http:// or https://.".to_string());
    }

    let has_secondary_label = config
        .secondary_label
        .as_ref()
        .map(|value| !is_blank(value))
        .unwrap_or(false);
    let has_secondary_url = config
        .secondary_url
        .as_ref()
        .map(|value| !is_blank(value))
        .unwrap_or(false);

    if has_secondary_label ^ has_secondary_url {
        messages.push(
            "Provide both secondary label and secondary URL, or leave both empty.".to_string(),
        );
    }
    if has_secondary_url
        && let Some(url) = &config.secondary_url
        && !is_valid_url_like(url)
    {
        messages.push("Secondary URL should start with http:// or https://.".to_string());
    }

    messages
}

fn validate_pause_step(step: &Step) -> Vec<String> {
    let mut messages = Vec::new();
    if step.delay_ms == 0 {
        messages.push("Pause duration should be greater than 0 ms.".to_string());
    }
    messages
}

pub fn create_default_step(step_type: StepType, order: i32) -> Step {
    let mut step = Step {
        id: Uuid::new_v4(),
        step_type,
        order,
        input: None,
        match_mode: None,
        match_pattern: None,
        short_description: None,
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
                text: "Hello from SimuCLI".to_string(),
                style: OutputStyle::Normal,
                color: None,
                prefix: None,
                indent: 0,
            }]);
        }
        StepType::Comment => {
            step.description = Some("Narration or hint".to_string());
        }
        StepType::Prompt => {
            step.prompt_config = Some(default_prompt_config());
        }
        StepType::Spinner => {
            step.spinner_config = Some(default_spinner_config());
        }
        StepType::Cta => {
            step.cta_config = Some(default_cta_config());
        }
        StepType::Pause => {
            step.delay_ms = 800;
            step.skippable = false;
        }
        _ => {}
    }

    step
}

pub fn add_default_step(steps: &mut Vec<Step>, step_type: StepType) {
    let order = steps.len() as i32;
    steps.push(create_default_step(step_type, order));
}

/// Appends a new default Command step to the end of the provided steps list.
///
/// The appended step has its `order` set to the index it occupies (current length of the list)
/// and is initialized via `create_default_step` for `StepType::Command`.
///
/// # Examples
///
/// ```
/// let mut steps: Vec<Step> = Vec::new();
/// add_command_block(&mut steps);
/// assert_eq!(steps.len(), 1);
/// assert!(matches!(steps[0].step_type, StepType::Command));
/// ```
pub fn add_command_block(steps: &mut Vec<Step>) {
    let order = steps.len() as i32;
    steps.push(create_default_step(StepType::Command, order));
}

/// Produces a short, human-readable summary for a step suitable for list or card headers.
///
/// The summary is derived from the step's type and key fields:
/// - Command: uses `input` or `"(empty command)"`; appends `short_description` as `"input - short_description"` when present.
/// - Output: uses the first output line; if multiple lines, appends `"(+N lines)"`; if no lines, returns `"(no output lines)"`.
/// - Comment: uses `description` or `"(no comment)"`.
/// - Prompt: uses the prompt `question` or `"(configure prompt)"`.
/// - Spinner: uses `"label (duration_ms ms)"` or `"(configure spinner)"`.
/// - Cta: uses `"primary_label -> primary_url"` or `"(configure cta)"`.
/// - Pause: returns `"Pause {delay_ms}ms"`.
/// - Other types: uses `description` or `"(configure step)"`.
///
/// # Examples
///
/// ```
/// let step = create_default_step(StepType::Command, 0);
/// let summary = summarize_step(&step);
/// assert!(summary.contains("echo") || summary == "(empty command)");
/// ```
fn summarize_step(step: &Step) -> String {
    match step.step_type {
        StepType::Command => {
            let command = step
                .input
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "(empty command)".to_string());
            let guide = step
                .short_description
                .clone()
                .filter(|value| !value.trim().is_empty());

            guide
                .map(|text| format!("{command} - {text}"))
                .unwrap_or(command)
        }
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
        StepType::Prompt => step
            .prompt_config
            .as_ref()
            .map(|cfg| cfg.question.to_string())
            .unwrap_or_else(|| "(configure prompt)".to_string()),
        StepType::Spinner => step
            .spinner_config
            .as_ref()
            .map(|cfg| format!("{} ({}ms)", cfg.label, cfg.duration_ms))
            .unwrap_or_else(|| "(configure spinner)".to_string()),
        StepType::Cta => step
            .cta_config
            .as_ref()
            .map(|cfg| format!("{} -> {}", cfg.primary_label, cfg.primary_url))
            .unwrap_or_else(|| "(configure cta)".to_string()),
        StepType::Pause => format!("Pause {}ms", step.delay_ms),
        _ => step
            .description
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "(configure step)".to_string()),
    }
}

fn step_badge_class(step_type: &StepType) -> &'static str {
    match step_type {
        StepType::Command => "border-primary/30 bg-primary/10 text-primary",
        StepType::Output => "border-sky-400/30 bg-sky-400/10 text-sky-200",
        StepType::Pause => "border-amber-400/30 bg-amber-400/10 text-amber-200",
        _ => "border-outline bg-surface-container-high text-on-surface-variant",
    }
}

#[component]
pub fn StepListEditor(
    steps: ReadSignal<Vec<Step>>,
    set_steps: WriteSignal<Vec<Step>>,
    filter: ReadSignal<String>,
) -> impl IntoView {
    let (expanded_steps, set_expanded_steps) = signal(HashSet::<Uuid>::new());
    let (dragged_idx, set_dragged_idx) = signal(None::<usize>);

    view! {
        <div class="space-y-4">
            <For
                each=move || {
                    let query = filter.get().trim().to_ascii_lowercase();
                    indexed_steps(steps.get())
                        .into_iter()
                        .filter(|(_, step)| {
                            query.is_empty()
                                || summarize_step(step).to_ascii_lowercase().contains(&query)
                                || format!("{:?}", step.step_type).to_ascii_lowercase().contains(&query)
                        })
                        .collect::<Vec<_>>()
                }
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

                let on_drag_start = {
                    let set_dragged_idx = set_dragged_idx;
                    Callback::new(move |_| {
                        set_dragged_idx.set(Some(idx));
                    })
                };

                let on_drag_over = Callback::new(move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                });

                let on_drop = {
                    let set_steps = set_steps;
                    let dragged_idx = dragged_idx;
                    let set_dragged_idx = set_dragged_idx;
                    Callback::new(move |ev: web_sys::DragEvent| {
                        ev.prevent_default();

                        if let Some(from_idx) = dragged_idx.get_untracked()
                            && from_idx != idx
                        {
                            set_steps.update(|items| {
                                if from_idx < items.len() && idx < items.len() {
                                    let moved = items.remove(from_idx);
                                    items.insert(idx, moved);
                                    normalize_step_orders(items);
                                }
                            });
                        }

                        set_dragged_idx.set(None);
                    })
                };

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
                        on_drag_start=on_drag_start
                        on_drag_over=on_drag_over
                        on_drop=on_drop
                    />
                }
                }
            />

            <Show when=move || steps.get().is_empty()>
                <div class="rounded-[28px] border border-dashed border-outline bg-surface-container-low p-8 text-center">
                    <p class="font-headline text-lg font-semibold text-on-surface">"Start with a command block"</p>
                    <p class="mt-2 text-sm text-on-surface-variant">"Add a command and output pair, or import a cast recording to generate steps automatically."</p>
                </div>
            </Show>
        </div>
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
    on_drag_start: Callback<web_sys::DragEvent>,
    on_drag_over: Callback<web_sys::DragEvent>,
    on_drop: Callback<web_sys::DragEvent>,
) -> impl IntoView {
    let step_type_label = format!("{:?}", step.step_type);
    let step_type_class = step_badge_class(&step.step_type);
    let step_summary = summarize_step(&step);
    let step_for_editor = step.clone();

    view! {
        <div
            class="group overflow-hidden rounded-[28px] border border-outline-variant bg-surface-container-low/95 shadow-[0_22px_70px_-42px_rgba(0,0,0,0.95)] backdrop-blur-sm transition-all duration-200 ease-out hover:-translate-y-0.5 hover:border-outline hover:bg-surface-container"
            draggable="true"
            on:dragstart=move |ev| on_drag_start.run(ev)
            on:dragover=move |ev| on_drag_over.run(ev)
            on:drop=move |ev| on_drop.run(ev)
        >
            <button type="button" class="flex w-full items-center gap-4 px-4 py-4 text-left transition-all duration-200 ease-out" on:click=move |ev| on_toggle.run(ev)>
                <div class="grid h-10 w-10 shrink-0 place-items-center rounded-2xl border border-outline-variant bg-background font-mono text-xs font-bold text-on-surface-variant transition-colors duration-200 group-hover:text-primary">
                    {format!("{:02}", index + 1)}
                </div>
                <span class=format!("rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em] {step_type_class}")>{step_type_label}</span>
                <span class="min-w-0 flex-1 truncate text-sm font-medium text-on-surface">{step_summary}</span>
                <span class="text-[10px] font-bold uppercase tracking-[0.18em] text-on-surface-variant">
                    {move || if expanded.get() { "Collapse" } else { "Edit" }}
                </span>
            </button>

            <Show when=move || expanded.get()>
                <div class="space-y-4 border-t border-outline-variant px-4 py-4">
                    <header class="flex flex-wrap items-center gap-2">
                        <button type="button" class="rounded-xl border border-outline-variant bg-surface-container-high px-3 py-1.5 text-xs font-semibold text-on-surface-variant transition-all duration-200 ease-out hover:border-primary/50 hover:text-primary" on:click=move |ev| on_move_up.run(ev)>"Move up"</button>
                        <button type="button" class="rounded-xl border border-outline-variant bg-surface-container-high px-3 py-1.5 text-xs font-semibold text-on-surface-variant transition-all duration-200 ease-out hover:border-primary/50 hover:text-primary" on:click=move |ev| on_move_down.run(ev)>"Move down"</button>
                        <button type="button" class="ml-auto rounded-xl border border-red-500/30 bg-red-500/10 px-3 py-1.5 text-xs font-semibold text-red-200 transition-all duration-200 ease-out hover:border-red-400/60 hover:bg-red-500/20" on:click=move |ev| on_remove.run(ev)>"Remove"</button>
                    </header>
                    <StepEditorRouter step=step_for_editor.clone() on_update=on_update />
                </div>
            </Show>
        </div>
    }
}

#[component]
fn StepEditorRouter(step: Step, on_update: Callback<Step>) -> impl IntoView {
    match step.step_type {
        StepType::Command => {
            view! { <CommandBlockEditor step=step on_update=on_update /> }.into_any()
        }
        StepType::Output => view! { <OutputEditor step=step on_update=on_update /> }.into_any(),
        StepType::Comment => view! { <CommentEditor step=step on_update=on_update /> }.into_any(),
        StepType::Prompt => view! { <PromptEditor step=step on_update=on_update /> }.into_any(),
        StepType::Spinner => view! { <SpinnerEditor step=step on_update=on_update /> }.into_any(),
        StepType::Cta => view! { <CtaEditor step=step on_update=on_update /> }.into_any(),
        StepType::Pause => view! { <PauseEditor step=step on_update=on_update /> }.into_any(),
        StepType::Clear => view! { <ClearEditor step=step on_update=on_update /> }.into_any(),
    }
}

/// Renders an editor UI for a command-type Step and emits updated Step values via the provided callback.
///
/// The editor exposes fields for the command input, an optional short description, a match pattern
/// that determines how user input matches the command, and an optional textual representation of
/// the command's expected output (one line per row). Each field change clones and updates the
/// provided `step` and calls `on_update.run(updated_step)`.
///
/// # Examples
///
/// ```ignore
/// use leptos::Callback;
/// // Construct a minimal command step (fields omitted for brevity)
/// let step = create_default_step(StepType::Command, 0);
/// let on_update = Callback::from(|updated: Step| {
///     // handle updated step (e.g., store in state)
///     dbg!(updated);
/// });
/// // Render the editor component (in a Leptos view context)
/// let _view = CommandBlockEditor(step, on_update);
/// ```
#[component]
fn CommandBlockEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let command_value = step.input.clone().unwrap_or_default();
    let short_description_value = step.short_description.clone().unwrap_or_default();
    let match_pattern_value = step
        .match_pattern
        .clone()
        .unwrap_or_else(|| command_value.clone());
    let output_lines = step.output.clone().unwrap_or_default();
    let output_value = output_lines_to_text(&output_lines);
    let step_for_command = step.clone();
    let step_for_short_description = step.clone();
    let step_for_pattern = step;
    let step_for_output = step_for_short_description.clone();

    view! {
        <div class="command-block-editor flex flex-col gap-4">
            <section class="flex flex-col gap-4 rounded-3xl border border-outline-variant bg-background/70 p-4 shadow-[0_18px_50px_-34px_rgba(0,0,0,0.75)] backdrop-blur-sm transition-all duration-200 ease-out">
                <h4 class="font-headline text-base font-semibold text-on-surface">"Command block"</h4>
                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Command to execute"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 font-mono text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 placeholder:text-on-surface-variant/50 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        placeholder="e.g., npm install"
                        prop:value=command_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_command.clone();
                            let previous_input = updated.input.clone().unwrap_or_default();

                            // Keep match pattern in sync with command by default.
                            // If user has set a custom pattern, preserve it.
                            let should_sync_pattern = updated
                                .match_pattern
                                .as_ref()
                                .map(|pattern| {
                                    pattern.trim().is_empty() || *pattern == previous_input
                                })
                                .unwrap_or(true);

                            updated.input = Some(next.clone());
                            if should_sync_pattern {
                                updated.match_pattern = Some(next);
                            }
                            on_update.run(updated);
                        }
                    />
                </label>
                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Guide description"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 placeholder:text-on-surface-variant/50 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        placeholder="Short explanation shown in the guide and help output"
                        prop:value=short_description_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_short_description.clone();
                            updated.short_description = if next.trim().is_empty() {
                                None
                            } else {
                                Some(next)
                            };
                            on_update.run(updated);
                        }
                    />
                </label>
                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Match pattern (how user input will match this command)"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 font-mono text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 placeholder:text-on-surface-variant/50 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        placeholder="Leave blank to match the command exactly"
                        prop:value=match_pattern_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_pattern.clone();
                            updated.match_pattern = if next.trim().is_empty() {
                                None
                            } else {
                                Some(next)
                            };
                            on_update.run(updated);
                        }
                    />
                </label>
                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Output (optional)"
                    <textarea
                        class="min-h-36 rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 font-mono text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 placeholder:text-on-surface-variant/50 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        placeholder="Write the output for this command here, one line per row. Leave blank if this command has no output."
                        prop:value=output_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_output.clone();
                            updated.output = if next.trim().is_empty() {
                                None
                            } else {
                                Some(text_to_output_lines(&next))
                            };
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>
        </div>
    }
}

#[component]
fn OutputEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let output_lines = step.output.clone().unwrap_or_default();
    let output_text = output_lines_to_text(&output_lines);

    view! {
        <label>
            "Output lines"
            <textarea
                prop:value=output_text
                on:input=move |ev| {
                    let raw = event_target_value(&ev);
                    let lines = text_to_output_lines(&raw);

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
    let config = step
        .prompt_config
        .clone()
        .unwrap_or_else(default_prompt_config);
    let prompt_type_value = prompt_type_to_str(&config.prompt_type).to_string();
    let question_value = config.question.clone();
    let default_value = config.default_value.clone().unwrap_or_default();
    let choices_text = config.choices.clone().unwrap_or_default().join("\n");
    let yes_output_text = config
        .yes_output
        .clone()
        .map(|lines| output_lines_to_text(&lines))
        .unwrap_or_default();
    let no_output_text = config
        .no_output
        .clone()
        .map(|lines| output_lines_to_text(&lines))
        .unwrap_or_default();
    let prompt_validation = validate_prompt_config(&config);
    let prompt_validation_view = if prompt_validation.is_empty() {
        None
    } else {
        Some(
            view! {
                <ul class="step-editor-validation">
                    {prompt_validation
                        .iter()
                        .cloned()
                        .map(|message| view! { <li>{message}</li> })
                        .collect_view()}
                </ul>
            }
            .into_any(),
        )
    };
    let step_for_type = step.clone();
    let step_for_question = step.clone();
    let step_for_default = step.clone();
    let step_for_choices = step.clone();
    let step_for_yes = step.clone();
    let step_for_no = step;

    view! {
        <div class="step-editor-grid">
            <section class="step-editor-group">
                <h4>"Core Prompt"</h4>
                <p class="step-editor-help">"Define interaction type, question, and optional default answer."</p>

                <label>
                    "Prompt type"
                    <select
                        prop:value=prompt_type_value
                        on:change=move |ev| {
                            let next = prompt_type_from_str(&event_target_value(&ev));
                            let mut updated = step_for_type.clone();
                            let mut cfg = updated.prompt_config.unwrap_or_else(default_prompt_config);
                            cfg.prompt_type = next;
                            if !matches!(cfg.prompt_type, PromptType::Select) {
                                cfg.choices = None;
                            }
                            if !matches!(cfg.prompt_type, PromptType::Confirm) {
                                cfg.yes_output = None;
                                cfg.no_output = None;
                            }
                            updated.prompt_config = Some(cfg);
                            on_update.run(updated);
                        }
                    >
                        <option value="confirm">"Confirm"</option>
                        <option value="input">"Input"</option>
                        <option value="password">"Password"</option>
                        <option value="select">"Select"</option>
                    </select>
                </label>

                <label>
                    "Question"
                    <input
                        prop:value=question_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_question.clone();
                            let mut cfg = updated.prompt_config.unwrap_or_else(default_prompt_config);
                            cfg.question = next;
                            updated.prompt_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "Default value"
                    <input
                        prop:value=default_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_default.clone();
                            let mut cfg = updated.prompt_config.unwrap_or_else(default_prompt_config);
                            cfg.default_value = if next.trim().is_empty() { None } else { Some(next) };
                            updated.prompt_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>

            <section class="step-editor-group">
                <h4>"Select Choices"</h4>
                <p class="step-editor-help">"Used when prompt type is Select. One choice per line."</p>
                <label>
                    "Choices"
                    <textarea
                        prop:value=choices_text
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            let values = raw
                                .lines()
                                .map(|line| line.trim())
                                .filter(|line| !line.is_empty())
                                .map(|line| line.to_string())
                                .collect::<Vec<_>>();

                            let mut updated = step_for_choices.clone();
                            let mut cfg = updated.prompt_config.unwrap_or_else(default_prompt_config);
                            cfg.choices = if values.is_empty() { None } else { Some(values) };
                            updated.prompt_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>

            <section class="step-editor-group">
                <h4>"Confirm Responses"</h4>
                <p class="step-editor-help">"Used when prompt type is Confirm. Provide output lines for yes/no paths."</p>

                <label>
                    "Yes output"
                    <textarea
                        prop:value=yes_output_text
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            let lines = text_to_output_lines(&raw);
                            let mut updated = step_for_yes.clone();
                            let mut cfg = updated.prompt_config.unwrap_or_else(default_prompt_config);
                            cfg.yes_output = if lines.is_empty() { None } else { Some(lines) };
                            updated.prompt_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "No output"
                    <textarea
                        prop:value=no_output_text
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            let lines = text_to_output_lines(&raw);
                            let mut updated = step_for_no.clone();
                            let mut cfg = updated.prompt_config.unwrap_or_else(default_prompt_config);
                            cfg.no_output = if lines.is_empty() { None } else { Some(lines) };
                            updated.prompt_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>
        </div>

        {prompt_validation_view}
    }
}

#[component]
fn SpinnerEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let config = step
        .spinner_config
        .clone()
        .unwrap_or_else(default_spinner_config);
    let style_value = spinner_style_to_str(&config.style).to_string();
    let label_value = config.label.clone();
    let duration_value = config.duration_ms.to_string();
    let finish_text_value = config.finish_text.clone();
    let finish_style_value = output_style_to_str(&config.finish_style).to_string();
    let spinner_validation = validate_spinner_config(&config);
    let spinner_validation_view = if spinner_validation.is_empty() {
        None
    } else {
        Some(
            view! {
                <ul class="step-editor-validation">
                    {spinner_validation
                        .iter()
                        .cloned()
                        .map(|message| view! { <li>{message}</li> })
                        .collect_view()}
                </ul>
            }
            .into_any(),
        )
    };

    let step_for_style = step.clone();
    let step_for_label = step.clone();
    let step_for_duration = step.clone();
    let step_for_finish_text = step.clone();
    let step_for_finish_style = step;

    view! {
        <div class="step-editor-grid">
            <section class="step-editor-group">
                <h4>"Spinner Behavior"</h4>
                <p class="step-editor-help">"Controls animation style and duration."</p>

                <label>
                    "Spinner style"
                    <select
                        prop:value=style_value
                        on:change=move |ev| {
                            let mut updated = step_for_style.clone();
                            let mut cfg = updated.spinner_config.unwrap_or_else(default_spinner_config);
                            cfg.style = spinner_style_from_str(&event_target_value(&ev));
                            updated.spinner_config = Some(cfg);
                            on_update.run(updated);
                        }
                    >
                        <option value="dots">"Dots"</option>
                        <option value="bar">"Bar"</option>
                        <option value="braille">"Braille"</option>
                        <option value="line">"Line"</option>
                    </select>
                </label>

                <label>
                    "Spinner label"
                    <input
                        prop:value=label_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_label.clone();
                            let mut cfg = updated.spinner_config.unwrap_or_else(default_spinner_config);
                            cfg.label = next;
                            updated.spinner_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "Duration (ms)"
                    <input
                        type="number"
                        min="0"
                        prop:value=duration_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                            let mut updated = step_for_duration.clone();
                            let mut cfg = updated.spinner_config.unwrap_or_else(default_spinner_config);
                            cfg.duration_ms = next;
                            updated.spinner_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>

            <section class="step-editor-group">
                <h4>"Completion Output"</h4>
                <p class="step-editor-help">"Displayed after spinner duration ends."</p>

                <label>
                    "Finish text"
                    <input
                        prop:value=finish_text_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_finish_text.clone();
                            let mut cfg = updated.spinner_config.unwrap_or_else(default_spinner_config);
                            cfg.finish_text = next;
                            updated.spinner_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "Finish style"
                    <select
                        prop:value=finish_style_value
                        on:change=move |ev| {
                            let mut updated = step_for_finish_style.clone();
                            let mut cfg = updated.spinner_config.unwrap_or_else(default_spinner_config);
                            cfg.finish_style = output_style_from_str(&event_target_value(&ev));
                            updated.spinner_config = Some(cfg);
                            on_update.run(updated);
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
            </section>
        </div>

        {spinner_validation_view}
    }
}

#[component]
fn CtaEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let config = step.cta_config.clone().unwrap_or_else(default_cta_config);
    let message_value = config.message.clone();
    let primary_label_value = config.primary_label.clone();
    let primary_url_value = config.primary_url.clone();
    let secondary_label_value = config.secondary_label.clone().unwrap_or_default();
    let secondary_url_value = config.secondary_url.clone().unwrap_or_default();
    let cta_validation = validate_cta_config(&config);
    let cta_validation_view = if cta_validation.is_empty() {
        None
    } else {
        Some(
            view! {
                <ul class="step-editor-validation">
                    {cta_validation
                        .iter()
                        .cloned()
                        .map(|message| view! { <li>{message}</li> })
                        .collect_view()}
                </ul>
            }
            .into_any(),
        )
    };

    let step_for_message = step.clone();
    let step_for_primary_label = step.clone();
    let step_for_primary_url = step.clone();
    let step_for_secondary_label = step.clone();
    let step_for_secondary_url = step;

    view! {
        <div class="step-editor-grid">
            <section class="step-editor-group">
                <h4>"Primary CTA"</h4>
                <p class="step-editor-help">"Message and main action displayed to visitors."</p>

                <label>
                    "Message"
                    <input
                        prop:value=message_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_message.clone();
                            let mut cfg = updated.cta_config.unwrap_or_else(default_cta_config);
                            cfg.message = next;
                            updated.cta_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "Primary label"
                    <input
                        prop:value=primary_label_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_primary_label.clone();
                            let mut cfg = updated.cta_config.unwrap_or_else(default_cta_config);
                            cfg.primary_label = next;
                            updated.cta_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "Primary URL"
                    <input
                        prop:value=primary_url_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_primary_url.clone();
                            let mut cfg = updated.cta_config.unwrap_or_else(default_cta_config);
                            cfg.primary_url = next;
                            updated.cta_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>

            <section class="step-editor-group">
                <h4>"Secondary CTA"</h4>
                <p class="step-editor-help">"Optional. Fill both fields or leave both empty."</p>

                <label>
                    "Secondary label"
                    <input
                        prop:value=secondary_label_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_secondary_label.clone();
                            let mut cfg = updated.cta_config.unwrap_or_else(default_cta_config);
                            cfg.secondary_label = if next.trim().is_empty() { None } else { Some(next) };
                            updated.cta_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "Secondary URL"
                    <input
                        prop:value=secondary_url_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            let mut updated = step_for_secondary_url.clone();
                            let mut cfg = updated.cta_config.unwrap_or_else(default_cta_config);
                            cfg.secondary_url = if next.trim().is_empty() { None } else { Some(next) };
                            updated.cta_config = Some(cfg);
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>
        </div>

        {cta_validation_view}
    }
}

#[component]
fn PauseEditor(step: Step, on_update: Callback<Step>) -> impl IntoView {
    let delay_value = step.delay_ms.to_string();
    let skippable_value = step.skippable;
    let pause_validation = validate_pause_step(&step);
    let pause_validation_view = if pause_validation.is_empty() {
        None
    } else {
        Some(
            view! {
                <ul class="step-editor-validation">
                    {pause_validation
                        .iter()
                        .cloned()
                        .map(|message| view! { <li>{message}</li> })
                        .collect_view()}
                </ul>
            }
            .into_any(),
        )
    };
    let step_for_delay = step.clone();
    let step_for_skippable = step;

    view! {
        <div class="step-editor-grid">
            <section class="step-editor-group">
                <h4>"Pause Behavior"</h4>
                <p class="step-editor-help">"Use short pauses to keep playback responsive."</p>

                <label>
                    "Pause duration (ms)"
                    <input
                        type="number"
                        min="0"
                        prop:value=delay_value
                        on:input=move |ev| {
                            let next = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                            let mut updated = step_for_delay.clone();
                            updated.delay_ms = next;
                            on_update.run(updated);
                        }
                    />
                </label>

                <label>
                    "Allow skip"
                    <input
                        type="checkbox"
                        prop:checked=skippable_value
                        on:change=move |ev| {
                            let mut updated = step_for_skippable.clone();
                            updated.skippable = event_target_checked(&ev);
                            on_update.run(updated);
                        }
                    />
                </label>
            </section>
        </div>

        {pause_validation_view}
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
