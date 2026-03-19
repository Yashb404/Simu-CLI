use leptos::prelude::*;
use shared::{
    dto::PublicDemoResponse,
    models::demo::{EngineMode, MatchMode, OutputLine, Step, StepType},
};

use crate::{
    input_handler::normalize_input,
    matching::command_matches,
    messaging::{post_event_to_parent, EmbedEvent},
};

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

#[derive(Clone)]
struct CliEngine {
    demo_id: String,
    steps: Vec<Step>,
    mode: EngineMode,
    prompt_string: String,
    not_found_message: String,
    cursor: usize,
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
        }
    }

    fn run_command(&mut self, raw_input: &str) -> Option<(Vec<String>, bool)> {
        let command = normalize_input(raw_input);
        if command.is_empty() {
            return None;
        }

        let mut next_lines = vec![format!("{} {}", self.prompt_string, command.clone())];
        let mut completed = false;

        if let Some(command_idx) = self.next_command_index(&command) {
            let (playback_lines, next_cursor) = self.playback_after_command(command_idx);
            next_lines.extend(playback_lines);
            self.cursor = next_cursor;
            completed = self.cursor >= self.steps.len();
        } else {
            next_lines.push(self.not_found_message.to_string());
        }

        Some((next_lines, completed))
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
                .or(step.input.as_deref())
                .unwrap_or_default();
            let match_mode = step.match_mode.clone().unwrap_or(MatchMode::Exact);
            if command_matches(&match_mode, expected, command) {
                return Some(idx);
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
}

fn run_terminal_command(
    engine: ReadSignal<Option<CliEngine>>,
    set_engine: WriteSignal<Option<CliEngine>>,
    input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    history: WriteSignal<Vec<String>>,
) {
    let raw_input = input.get();

    let mut event_demo_id = None;
    let mut next_lines = None;
    let mut is_completion = false;

    set_engine.update(|maybe_engine| {
        let Some(engine) = maybe_engine.as_mut() else {
            return;
        };

        if let Some((lines, completed)) = engine.run_command(&raw_input) {
            event_demo_id = Some(engine.demo_id.clone());
            next_lines = Some(lines);
            is_completion = completed;
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
        let _ = post_event_to_parent(&EmbedEvent::interaction(demo_id.clone(), &raw_input));
        if is_completion {
            let _ = post_event_to_parent(&EmbedEvent::completion(demo_id));
        }
    }

    history.update(|items| items.extend(lines));
    set_input.set(String::new());
}

#[component]
pub fn TerminalUI(demo: PublicDemoResponse) -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (history, set_history) = signal(vec!["Preview runtime initialized.".to_string()]);
    let (engine, set_engine) = signal(Option::<CliEngine>::None);
    let (view_event_demo_id, set_view_event_demo_id) = signal(Option::<String>::None);

    let window_title = demo.theme.window_title.clone();
    let prompt_string = demo.theme.prompt_string.clone();

    Effect::new(move |_| {
        let next_demo_id = demo.id.to_string();
        if engine.get().is_none() {
            set_engine.set(Some(CliEngine::new(&demo)));
        }

        if view_event_demo_id.get().as_deref() != Some(next_demo_id.as_str()) {
            let _ = post_event_to_parent(&EmbedEvent::view(next_demo_id.clone()));
            set_view_event_demo_id.set(Some(next_demo_id));
        }
    });

    let prompt_display = prompt_string.clone();

    view! {
        <section class="terminal-chrome" aria-label="CLI simulator terminal">
            <div class="terminal-titlebar">
                <div class="terminal-dots">
                    <span class="terminal-dot red"></span>
                    <span class="terminal-dot yellow"></span>
                    <span class="terminal-dot green"></span>
                </div>
                <span class="terminal-titlebar-text">{window_title}</span>
            </div>
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
                            run_terminal_command(engine, set_engine, input, set_input, set_history);
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
                        run_terminal_command(engine, set_engine, input, set_input, set_history);
                    }
                >
                    "RUN"
                </button>
            </div>
        </section>
    }
}
