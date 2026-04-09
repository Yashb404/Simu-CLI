use leptos::prelude::*;
use std::collections::BTreeSet;
use std::sync::Arc;
use shared::{
    dto::PublicDemoResponse,
    models::demo::{EngineMode, MatchMode, OutputLine, Step, StepType},
};

use crate::{
    api::post_analytics_event,
    input_handler::normalize_input,
    matching::command_matches,
    messaging::{post_event_to_parent, EmbedEvent},
    EmbedConfig,
};
use shared::models::analytics::AnalyticsEventType;
use uuid::Uuid;

fn line_css_class(line: &str, prompt_string: &str) -> &'static str {
    if line.starts_with(prompt_string) {
        "terminal-line cmd"
    } else if line.starts_with('#') {
        "terminal-line comment"
    } else {
        "terminal-line"
    }
}

fn indexed_lines(lines: Vec<String>) -> Vec<(usize, String)> {
    lines.into_iter().enumerate().collect::<Vec<(usize, String)>>()
}

fn ordered_command_guide(steps: &[Step]) -> Vec<String> {
    steps
        .iter()
        .filter(|step| step.step_type == StepType::Command)
        .filter_map(|step| {
            step
                .input
                .as_deref()
                .or(step.match_pattern.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .collect()
}

fn command_step_indices(steps: &[Step]) -> Vec<usize> {
    steps
        .iter()
        .enumerate()
        .filter_map(|(idx, step)| (step.step_type == StepType::Command).then_some(idx))
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn should_enable_compact_mode() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let Ok(height) = window.inner_height() else {
        return false;
    };

    height.as_f64().map(|value| value < 620.0).unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn should_enable_compact_mode() -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
const COMPACT_MODE_STORAGE_KEY: &str = "cli_demo_embed_compact_mode";

#[cfg(target_arch = "wasm32")]
fn load_compact_mode_preference() -> Option<bool> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let value = storage.get_item(COMPACT_MODE_STORAGE_KEY).ok()??;
    match value.as_str() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_compact_mode_preference() -> Option<bool> {
    None
}

#[cfg(target_arch = "wasm32")]
fn persist_compact_mode_preference(compact: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let value = if compact { "1" } else { "0" };
    let _ = storage.set_item(COMPACT_MODE_STORAGE_KEY, value);
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_compact_mode_preference(_compact: bool) {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GuideItemState {
    Completed,
    Next,
    Pending,
}

#[derive(Clone)]
struct CliEngine {
    demo_id: String,
    steps: Vec<Step>,
    mode: EngineMode,
    prompt_string: String,
    not_found_message: String,
    cursor: usize,
    command_step_indices: Vec<usize>,
    completed_command_steps: BTreeSet<usize>,
}

impl CliEngine {
    fn new(demo: &PublicDemoResponse) -> Self {
        Self {
            demo_id: demo.id.to_string(),
            steps: demo.steps.clone(),
            mode: demo.settings.engine_mode.clone(),
            prompt_string: demo.theme.prompt_string.clone(),
            not_found_message: demo.settings.not_found_message.clone(),
            cursor: 0,
            command_step_indices: command_step_indices(&demo.steps),
            completed_command_steps: BTreeSet::new(),
        }
    }

    fn run_command(&mut self, raw_input: &str) -> Option<(Vec<String>, bool, Option<i32>)> {
        let command = normalize_input(raw_input);
        if command.is_empty() {
            return None;
        }

        let mut next_lines = vec![format!("{} {}", self.prompt_string, command.clone())];
        let mut completed = false;
        let mut matched_step_index = None;

        if let Some(command_idx) = self.next_command_index(&command) {
            self.completed_command_steps.insert(command_idx);
            let (playback_lines, next_cursor) = self.playback_after_command(command_idx);
            next_lines.extend(playback_lines);
            self.cursor = self.cursor.max(next_cursor);
            completed = self.cursor >= self.steps.len();
            matched_step_index = Some(command_idx as i32);
        } else {
            next_lines.push(self.not_found_message.to_string());
        }

        Some((next_lines, completed, matched_step_index))
    }

    fn next_command_index(&self, command: &str) -> Option<usize> {
        let range: Box<dyn Iterator<Item = usize>> = match self.mode {
            EngineMode::Sequential => Box::new(self.cursor..self.steps.len()),
            EngineMode::FreePlay => Box::new(0..self.steps.len()),
        };

        for idx in range {
            let step = &self.steps[idx];
            if step.step_type != StepType::Command {
                continue;
            }

            let expected = step
                .match_pattern
                .as_deref()
                .filter(|pattern| !pattern.trim().is_empty())
                .or(step.input.as_deref())
                .unwrap_or_default();
            let match_mode = step.match_mode.clone().unwrap_or(MatchMode::Exact);
            if command_matches(&match_mode, expected, command) {
                return Some(idx);
            }
        }

        // Fallback for sequential mode: allow exact repeats of previous commands
        if self.mode == EngineMode::Sequential {
            for idx in 0..self.cursor {
                let step = &self.steps[idx];
                if step.step_type != StepType::Command {
                    continue;
                }

                let expected = step
                    .match_pattern
                    .as_deref()
                    .filter(|pattern| !pattern.trim().is_empty())
                    .or(step.input.as_deref())
                    .unwrap_or_default();
                // Use Exact match mode for fallback repeats (strict matching required)
                if command_matches(&MatchMode::Exact, expected, command) {
                    return Some(idx);
                }
            }
        }

        None
    }

    fn playback_after_command(&self, command_idx: usize) -> (Vec<String>, usize) {
        let mut lines = Vec::new();
        let mut idx = command_idx + 1;

        while idx < self.steps.len() {
            let step = &self.steps[idx];
            if step.step_type == StepType::Command {
                break;
            }

            match step.step_type {
                StepType::Output => {
                    if let Some(output) = &step.output {
                        lines.extend(output.iter().map(Self::line_from_output));
                    }
                }
                StepType::Comment => {
                    if let Some(description) = &step.description {
                        lines.push(format!("# {description}"));
                    }
                }
                StepType::Clear => {
                    lines.push("[screen cleared]".to_string());
                }
                StepType::Pause => {
                    lines.push("[pause]".to_string());
                }
                _ => {}
            }

            idx += 1;
        }

        (lines, idx)
    }

    fn line_from_output(line: &OutputLine) -> String {
        let prefix = line.prefix.clone().unwrap_or_default();
        let indent = " ".repeat(line.indent as usize * 2);
        if prefix.is_empty() {
            format!("{}{}", indent, line.text)
        } else {
            format!("{}{} {}", indent, prefix, line.text)
        }
    }

    fn next_recommended_command_index(&self) -> Option<usize> {
        if self.mode != EngineMode::Sequential {
            return None;
        }

        let mut command_position = 0;
        for idx in 0..self.steps.len() {
            if self.steps[idx].step_type != StepType::Command {
                continue;
            }
            if idx >= self.cursor {
                return Some(command_position);
            }
            command_position += 1;
        }

        None
    }

    fn guide_item_state(&self, guide_idx: usize) -> GuideItemState {
        let Some(step_idx) = self.command_step_indices.get(guide_idx).copied() else {
            return GuideItemState::Pending;
        };

        if self.mode == EngineMode::Sequential
            && self
                .next_recommended_command_index()
                .map(|next_idx| next_idx == guide_idx)
                .unwrap_or(false)
        {
            return GuideItemState::Next;
        }

        if step_idx < self.cursor || self.completed_command_steps.contains(&step_idx) {
            GuideItemState::Completed
        } else {
            GuideItemState::Pending
        }
    }
}

fn run_terminal_command(
    config: EmbedConfig,
    engine: ReadSignal<Option<CliEngine>>,
    set_engine: WriteSignal<Option<CliEngine>>,
    input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    history: WriteSignal<Vec<String>>,
) {
    let raw_input = input.get();

    let mut event_demo_id = None;
    let mut event_demo_uuid = None;
    let mut next_lines = None;
    let mut is_completion = false;
    let mut step_index = None;

    set_engine.update(|maybe_engine| {
        let Some(engine) = maybe_engine.as_mut() else {
            return;
        };

        if let Some((lines, completed, matched_step_index)) = engine.run_command(&raw_input) {
            event_demo_id = Some(engine.demo_id.clone());
            event_demo_uuid = Uuid::parse_str(&engine.demo_id).ok();
            next_lines = Some(lines);
            is_completion = completed;
            step_index = matched_step_index;
        }
    });

    let Some(lines) = next_lines else {
        if engine.get().is_none() {
            history.update(|items| {
                items.push("Demo is still loading. Try again in a moment.".to_string())
            });
        }
        return;
    };

    if let Some(demo_id) = event_demo_id {
        let _ = post_event_to_parent(
            &EmbedEvent::interaction(demo_id.clone(), &raw_input),
            &config.api_base,
        );

        if let Some(demo_uuid) = event_demo_uuid {
            let endpoint = format!("{}/api/analytics/events", config.api_base);
            let interaction_step_index = step_index;
            leptos::task::spawn_local(async move {
                let _ = post_analytics_event(
                    &endpoint,
                    demo_uuid,
                    AnalyticsEventType::Interaction,
                    interaction_step_index,
                )
                .await;
            });
        }

        if is_completion {
            let _ = post_event_to_parent(&EmbedEvent::completion(demo_id), &config.api_base);
            if let Some(demo_uuid) = event_demo_uuid {
                let endpoint = format!("{}/api/analytics/events", config.api_base);
                let completion_step_index = step_index;
                leptos::task::spawn_local(async move {
                    let _ = post_analytics_event(
                        &endpoint,
                        demo_uuid,
                        AnalyticsEventType::Completion,
                        completion_step_index,
                    )
                    .await;
                });
            }
        }
    }

    history.update(|items| items.extend(lines));
    set_input.set(String::new());
}

#[component]
pub fn TerminalUI(demo: PublicDemoResponse, config: EmbedConfig) -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (history, set_history) = signal(vec!["Preview runtime initialized.".to_string()]);
    let (engine, set_engine) = signal(Option::<CliEngine>::None);
    let (view_event_demo_id, set_view_event_demo_id) = signal(Option::<String>::None);
    let (guide_open, set_guide_open) = signal(false);
    let (compact_mode, set_compact_mode) = signal(should_enable_compact_mode());

    let window_title = demo.theme.window_title.clone();
    let prompt_string = demo.theme.prompt_string.clone();
    let guide_commands = Arc::new(ordered_command_guide(&demo.steps));
    let view_config = config.clone();
    let keydown_config = config.clone();
    let click_config = config.clone();

    Effect::new(move |_| {
        if let Some(saved) = load_compact_mode_preference() {
            if saved != compact_mode.get_untracked() {
                set_compact_mode.set(saved);
            }
        }
    });

    Effect::new(move |_| {
        persist_compact_mode_preference(compact_mode.get());
    });

    Effect::new(move |_| {
        let next_demo_id = demo.id.to_string();
        if engine.get().is_none() {
            set_engine.set(Some(CliEngine::new(&demo)));
        }

        if view_event_demo_id.get().as_deref() != Some(next_demo_id.as_str()) {
            let _ = post_event_to_parent(&EmbedEvent::view(next_demo_id.clone()), &view_config.api_base);
            if !view_config.api_base.is_empty() {
                let endpoint = format!("{}/api/analytics/events", view_config.api_base);
                let demo_id = demo.id;
                leptos::task::spawn_local(async move {
                    let _ = post_analytics_event(&endpoint, demo_id, AnalyticsEventType::View, None).await;
                });
            }
            set_view_event_demo_id.set(Some(next_demo_id));
        }
    });

    let prompt_display = prompt_string.clone();

    view! {
        <section class=move || {
            if compact_mode.get() {
                "terminal-chrome is-compact"
            } else {
                "terminal-chrome"
            }
        } aria-label="CLI simulator terminal">
            <div class="terminal-titlebar">
                <div class="terminal-dots">
                    <span class="terminal-dot red"></span>
                    <span class="terminal-dot yellow"></span>
                    <span class="terminal-dot green"></span>
                </div>
                <span class="terminal-titlebar-text">{window_title}</span>
                <button
                    type="button"
                    class="terminal-guide-toggle"
                    aria-expanded=move || guide_open.get().to_string()
                    on:click=move |_| set_guide_open.update(|open| *open = !*open)
                >
                    {move || if guide_open.get() { "Hide Guide" } else { "Show Guide" }}
                </button>
                <button
                    type="button"
                    class="terminal-guide-toggle"
                    aria-pressed=move || compact_mode.get().to_string()
                    on:click=move |_| set_compact_mode.update(|compact| *compact = !*compact)
                >
                    {move || if compact_mode.get() { "Full" } else { "Compact" }}
                </button>
            </div>
            <Show when=move || guide_open.get()>
                <div class="terminal-guide" role="region" aria-label="Recommended command guide">
                    <p class="terminal-guide-heading">"Recommended Command Order"</p>
                    <ol class="terminal-guide-list">
                        {
                            let guide_commands = Arc::clone(&guide_commands);
                            view! {
                        <For
                            each=move || indexed_lines((*guide_commands).clone())
                            key=|entry| entry.0
                            children=move |(idx, command)| {
                                let item_state = move || {
                                    engine
                                        .get()
                                        .as_ref()
                                        .map(|runtime| runtime.guide_item_state(idx))
                                        .unwrap_or(GuideItemState::Pending)
                                };
                                let item_class = move || {
                                    match item_state() {
                                        GuideItemState::Next => {
                                        "terminal-guide-item is-next"
                                        }
                                        GuideItemState::Completed => {
                                            "terminal-guide-item is-done"
                                        }
                                        GuideItemState::Pending => {
                                        "terminal-guide-item"
                                        }
                                    }
                                };
                                view! {
                                    <li class=item_class>
                                        <span class="terminal-guide-marker" aria-hidden="true">
                                            {move || match item_state() {
                                                GuideItemState::Completed => "[x]",
                                                GuideItemState::Next => "[>]",
                                                GuideItemState::Pending => "[ ]",
                                            }}
                                        </span>
                                        <code>{command}</code>
                                    </li>
                                }
                            }
                        />
                            }
                        }
                    </ol>
                </div>
            </Show>
            <div class="terminal-body">
                <For
                    each=move || indexed_lines(history.get())
                    key=|entry| entry.0
                    children=move |(_, line)| {
                        let cls = line_css_class(&line, &prompt_string);
                        view! { <p class={cls}>{line}</p> }
                    }
                />
            </div>
            <div class="terminal-input-row">
                <span class="terminal-prompt-label">{format!("{} ", prompt_display)}</span>
                <label class="sr-only" for="terminal-input">"Terminal input"</label>
                <input
                    id="terminal-input"
                    class="terminal-input"
                    type="text"
                    prop:value=input
                    on:input=move |ev| set_input.set(event_target_value(&ev))
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            run_terminal_command(keydown_config.clone(), engine, set_engine, input, set_input, set_history);
                        }
                    }
                    placeholder="type a command..."
                    autocomplete="off"
                    spellcheck="false"
                />
                <button
                    type="button"
                    class="terminal-run-btn"
                    on:click=move |_| {
                        run_terminal_command(click_config.clone(), engine, set_engine, input, set_input, set_history);
                    }
                >
                    "RUN"
                </button>
            </div>
        </section>
    }
}
