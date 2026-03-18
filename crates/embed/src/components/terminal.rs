use leptos::prelude::*;
use shared::{
    dto::PublicDemoResponse,
    models::demo::{
        EngineMode, MatchMode, OutputLine, Step, StepType,
    },
};

use crate::{input_handler::normalize_input, matching::command_matches};

fn indexed_lines(lines: Vec<String>) -> Vec<(usize, String)> {
    lines.into_iter().enumerate().collect::<Vec<(usize, String)>>()
}

struct CliEngine<'a> {
    steps: &'a [Step],
    mode: EngineMode,
    prompt_string: &'a str,
    not_found_message: &'a str,
    cursor: usize,
}

impl<'a> CliEngine<'a> {
    fn new(demo: &'a PublicDemoResponse, cursor: usize) -> Self {
        Self {
            steps: &demo.steps,
            mode: demo.settings.engine_mode.clone(),
            prompt_string: &demo.theme.prompt_string,
            not_found_message: &demo.settings.not_found_message,
            cursor,
        }
    }

    fn run_command(&mut self, raw_input: &str) -> Option<Vec<String>> {
        let command = normalize_input(raw_input);
        if command.is_empty() {
            return None;
        }

        let mut next_lines = vec![format!("{} {}", self.prompt_string, command.clone())];

        if let Some(command_idx) = self.next_command_index(&command) {
            let (playback_lines, next_cursor) = self.playback_after_command(command_idx);
            next_lines.extend(playback_lines);
            self.cursor = next_cursor;
        } else {
            next_lines.push(self.not_found_message.to_string());
        }

        Some(next_lines)
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
    demo: ReadSignal<Option<Result<PublicDemoResponse, String>>>,
    input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    history: WriteSignal<Vec<String>>,
    cursor: ReadSignal<usize>,
    set_cursor: WriteSignal<usize>,
) {
    let raw_input = input.get();

    let Some(state) = demo.get() else {
        history.update(|lines| lines.push("Demo is still loading. Try again in a moment.".to_string()));
        return;
    };

    let loaded = match state {
        Ok(value) => value,
        Err(error) => {
            history.update(|lines| lines.push(format!("Unable to run command: {error}")));
            return;
        }
    };

    let mut engine = CliEngine::new(&loaded, cursor.get());
    let Some(next_lines) = engine.run_command(&raw_input) else {
        return;
    };
    set_cursor.set(engine.cursor);

    history.update(|lines| lines.extend(next_lines));
    set_input.set(String::new());
}

#[component]
pub fn TerminalUI(demo: ReadSignal<Option<Result<PublicDemoResponse, String>>>) -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (history, set_history) = signal(vec!["Preview runtime initialized.".to_string()]);
    let (cursor, set_cursor) = signal(0usize);

    view! {
        <section class="terminal-ui" aria-label="CLI simulator terminal">
            <header class="terminal-header">"CLI Demo Runtime"</header>
            <Show
                when=move || matches!(demo.get(), Some(Ok(_)))
                fallback=move || {
                    view! {
                        <div class="terminal-output">
                            <p>
                                {move || {
                                    match demo.get() {
                                        None => "Loading demo...".to_string(),
                                        Some(Err(error)) => format!("Unable to load demo: {error}"),
                                        Some(Ok(_)) => String::new(),
                                    }
                                }}
                            </p>
                        </div>
                    }
                }
            >
                <div class="terminal-output">
                    <For
                        each=move || indexed_lines(history.get())
                        key=|entry| entry.0
                        children=move |(_, line)| view! { <p>{line}</p> }
                    />
                </div>
                <label class="sr-only" for="terminal-input">"Terminal input"</label>
                <input
                    id="terminal-input"
                    type="text"
                    prop:value=input
                    on:input=move |ev| set_input.set(event_target_value(&ev))
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            run_terminal_command(demo, input, set_input, set_history, cursor, set_cursor);
                        }
                    }
                    placeholder="Type a command"
                />
                <button
                    type="button"
                    on:click=move |_| {
                        run_terminal_command(demo, input, set_input, set_history, cursor, set_cursor);
                    }
                >
                    "Run"
                </button>
            </Show>
        </section>
    }
}
