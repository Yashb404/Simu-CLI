use leptos::prelude::*;
use shared::{
    dto::PublicDemoResponse,
    models::demo::{
        DemoSettings, EngineMode, MatchMode, OutputLine, OutputStyle, Step, StepType, Theme,
        WindowStyle,
    },
};
use uuid::Uuid;

use crate::{input_handler::normalize_input, matching::command_matches};

fn indexed_lines(lines: Vec<String>) -> Vec<(usize, String)> {
    lines.into_iter().enumerate().collect::<Vec<(usize, String)>>()
}

// TODO: Replace this local fallback with a structured loading/error state once
// embed fetch reliability and retry behavior are finalized.
fn demo_fallback() -> PublicDemoResponse {
    PublicDemoResponse {
        id: Uuid::new_v4(),
        slug: None,
        version: 1,
        theme: Theme {
            window_style: WindowStyle::MacOs,
            window_title: "CLI Demo Runtime".to_string(),
            preset: Some("default".to_string()),
            bg_color: "#111827".to_string(),
            fg_color: "#e5e7eb".to_string(),
            cursor_color: "#f9fafb".to_string(),
            font_family: "JetBrains Mono".to_string(),
            font_size: 14,
            line_height: 1.4,
            prompt_string: "$".to_string(),
        },
        settings: DemoSettings {
            engine_mode: EngineMode::Sequential,
            autoplay: false,
            loop_demo: false,
            loop_delay_ms: 500,
            show_restart_button: true,
            show_hints: true,
            not_found_message: "command not found".to_string(),
        },
        steps: vec![
            Step {
                id: Uuid::new_v4(),
                step_type: StepType::Command,
                order: 0,
                input: Some("help".to_string()),
                match_mode: Some(MatchMode::Exact),
                match_pattern: Some("help".to_string()),
                description: None,
                output: None,
                prompt_config: None,
                spinner_config: None,
                cta_config: None,
                delay_ms: 0,
                typing_speed_ms: 0,
                skippable: true,
            },
            Step {
                id: Uuid::new_v4(),
                step_type: StepType::Output,
                order: 1,
                input: None,
                match_mode: None,
                match_pattern: None,
                description: None,
                output: Some(vec![OutputLine {
                    text: "Available commands: help, run demo".to_string(),
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
            },
            Step {
                id: Uuid::new_v4(),
                step_type: StepType::Command,
                order: 2,
                input: Some("run demo".to_string()),
                match_mode: Some(MatchMode::Exact),
                match_pattern: Some("run demo".to_string()),
                description: None,
                output: None,
                prompt_config: None,
                spinner_config: None,
                cta_config: None,
                delay_ms: 0,
                typing_speed_ms: 0,
                skippable: true,
            },
            Step {
                id: Uuid::new_v4(),
                step_type: StepType::Output,
                order: 3,
                input: None,
                match_mode: None,
                match_pattern: None,
                description: None,
                output: Some(vec![OutputLine {
                    text: "Scripted demo executed successfully.".to_string(),
                    style: OutputStyle::Success,
                    color: None,
                    prefix: Some("✓".to_string()),
                    indent: 0,
                }]),
                prompt_config: None,
                spinner_config: None,
                cta_config: None,
                delay_ms: 0,
                typing_speed_ms: 0,
                skippable: true,
            },
        ],
    }
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

fn next_command_index(steps: &[Step], command: &str, start: usize, mode: EngineMode) -> Option<usize> {
    let range: Box<dyn Iterator<Item = usize>> = match mode {
        EngineMode::Sequential => Box::new(start..steps.len()),
        EngineMode::FreePlay => Box::new(0..steps.len()),
    };

    for idx in range {
        let step = &steps[idx];
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

fn playback_after_command(steps: &[Step], command_idx: usize) -> (Vec<String>, usize) {
    let mut lines = Vec::new();
    let mut idx = command_idx + 1;

    while idx < steps.len() {
        let step = &steps[idx];
        if step.step_type == StepType::Command {
            break;
        }

        match step.step_type {
            StepType::Output => {
                if let Some(output) = &step.output {
                    lines.extend(output.iter().map(line_from_output));
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

fn run_terminal_command(
    demo: ReadSignal<Option<PublicDemoResponse>>,
    input: ReadSignal<String>,
    set_input: WriteSignal<String>,
    history: WriteSignal<Vec<String>>,
    cursor: ReadSignal<usize>,
    set_cursor: WriteSignal<usize>,
) {
    let raw_input = input.get();
    let command = normalize_input(&raw_input);
    if command.is_empty() {
        return;
    }

    // FIXME: This silently falls back to sample data when public demo loading fails.
    // Keep for MVP tonight, but convert to explicit error UI before production release.
    let loaded = demo.get().unwrap_or_else(demo_fallback);
    let mut next_lines = vec![format!("{} {}", loaded.theme.prompt_string, command.clone())];

    if let Some(command_idx) =
        next_command_index(&loaded.steps, &command, cursor.get(), loaded.settings.engine_mode)
    {
        let (playback_lines, next_cursor) = playback_after_command(&loaded.steps, command_idx);
        next_lines.extend(playback_lines);
        set_cursor.set(next_cursor);
    } else {
        next_lines.push(loaded.settings.not_found_message);
    }

    history.update(|lines| lines.extend(next_lines));
    set_input.set(String::new());
}

#[component]
pub fn TerminalUI(demo: ReadSignal<Option<PublicDemoResponse>>) -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (history, set_history) = signal(vec!["Preview runtime initialized.".to_string()]);
    let (cursor, set_cursor) = signal(0usize);

    view! {
        <section class="terminal-ui" aria-label="CLI simulator terminal">
            <header class="terminal-header">"CLI Demo Runtime"</header>
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
        </section>
    }
}
