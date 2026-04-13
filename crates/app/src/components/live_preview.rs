use leptos::prelude::*;
use shared::models::demo::{MatchMode, OutputLine, OutputStyle, Step, StepType, Theme};

#[derive(Clone)]
struct GuideEntry {
    command: String,
    description: String,
}

#[derive(Clone)]
enum PreviewLineKind {
    Command,
    Output,
    Success,
    Error,
    Warning,
    Muted,
    Comment,
}

#[derive(Clone)]
struct PreviewLine {
    kind: PreviewLineKind,
    text: String,
}

impl PreviewLine {
    fn new(kind: PreviewLineKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }
}

fn guide_entries(steps: &[Step]) -> Vec<GuideEntry> {
    steps
        .iter()
        .filter(|step| matches!(step.step_type, StepType::Command))
        .filter_map(|step| {
            let command = step
                .input
                .clone()
                .or_else(|| step.match_pattern.clone())
                .filter(|value| !value.trim().is_empty())?;

            let description = step
                .short_description
                .clone()
                .or_else(|| step.description.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "Run this command in the demo flow.".to_string());

            Some(GuideEntry {
                command,
                description,
            })
        })
        .collect()
}

fn wildcard_match(pattern: &str, input: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == input;
    }

    let parts = pattern
        .split('*')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return true;
    }

    let mut remainder = input;
    for (index, part) in parts.iter().enumerate() {
        let Some(position) = remainder.find(part) else {
            return false;
        };

        if index == 0 && !pattern.starts_with('*') && position != 0 {
            return false;
        }

        remainder = &remainder[position + part.len()..];
    }

    pattern.ends_with('*')
        || parts
            .last()
            .map(|last| input.ends_with(last))
            .unwrap_or(true)
}

fn command_matches(step: &Step, command: &str) -> bool {
    if !matches!(step.step_type, StepType::Command) {
        return false;
    }

    let command = command.trim();
    let pattern = step
        .match_pattern
        .as_ref()
        .or(step.input.as_ref())
        .map(|value| value.trim())
        .unwrap_or_default();

    match step.match_mode.clone().unwrap_or(MatchMode::Exact) {
        MatchMode::Exact => command == pattern,
        MatchMode::Fuzzy => {
            !pattern.is_empty()
                && (command.contains(pattern)
                    || pattern.contains(command)
                    || command.eq_ignore_ascii_case(pattern))
        }
        MatchMode::Wildcard => wildcard_match(pattern, command),
        MatchMode::Any => true,
    }
}

fn line_from_output(output: OutputLine) -> PreviewLine {
    let kind = match output.style {
        OutputStyle::Success => PreviewLineKind::Success,
        OutputStyle::Error => PreviewLineKind::Error,
        OutputStyle::Warning => PreviewLineKind::Warning,
        OutputStyle::Muted => PreviewLineKind::Muted,
        OutputStyle::Bold | OutputStyle::Code | OutputStyle::Normal => PreviewLineKind::Output,
    };

    let prefix = output.prefix.unwrap_or_default();
    let indent = " ".repeat(output.indent as usize);
    PreviewLine::new(kind, format!("{indent}{prefix}{}", output.text))
}

/// Produce preview lines representing the configured response for a given command.
///
/// Looks up the first step that matches `command`. If none is found, returns a single
/// `Error` preview line containing `"{command}: {not_found_message}"`. If a matching
/// command step is found, the returned lines are initialized from that step's configured
/// `output` (if any) and then extended by converting subsequent steps (until the next
/// `StepType::Command`) into `PreviewLine`s according to each step's type:
/// - `Output`: converted via `line_from_output` and appended
/// - `Comment`: appended as a `Comment` line `"# {description}"` when `description` exists
/// - `Spinner`: appended as a `Muted` line `"{label} ... {finish_text}"` when configured
/// - `Prompt`: appended as a `Warning` line `"? {question}"` when configured
/// - `Cta`: appended as a `Success` line `"{primary_label}: {primary_url}"` when configured
/// - `Pause`: appended as a `Muted` line `"# pause {delay_ms}ms"`
/// - `Clear`: clears the accumulated lines
///
/// If no lines remain at the end, a single `Comment` line `"# command matched, but no output steps are configured"`
/// is appended.
///
/// # Examples
///
/// ```no_run
/// // Given a set of steps configured elsewhere in the module:
/// let lines = response_for_command(&steps, "build", "command not found");
/// // `lines` now contains the preview output for the "build" command, or an error line if not found.
/// ```
fn response_for_command(
    steps: &[Step],
    command: &str,
    not_found_message: &str,
) -> Vec<PreviewLine> {
    let Some(command_index) = steps.iter().position(|step| command_matches(step, command)) else {
        return vec![PreviewLine::new(
            PreviewLineKind::Error,
            format!("{command}: {not_found_message}"),
        )];
    };

    let mut lines = steps[command_index]
        .output
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(line_from_output)
        .collect::<Vec<_>>();

    for step in steps.iter().skip(command_index + 1) {
        if matches!(step.step_type, StepType::Command) {
            break;
        }

        match step.step_type {
            StepType::Output => {
                lines.extend(
                    step.output
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .map(line_from_output),
                );
            }
            StepType::Comment => {
                if let Some(description) = step.description.clone() {
                    lines.push(PreviewLine::new(
                        PreviewLineKind::Comment,
                        format!("# {description}"),
                    ));
                }
            }
            StepType::Spinner => {
                if let Some(spinner) = step.spinner_config.clone() {
                    lines.push(PreviewLine::new(
                        PreviewLineKind::Muted,
                        format!("{} ... {}", spinner.label, spinner.finish_text),
                    ));
                }
            }
            StepType::Prompt => {
                if let Some(prompt) = step.prompt_config.clone() {
                    lines.push(PreviewLine::new(
                        PreviewLineKind::Warning,
                        format!("? {}", prompt.question),
                    ));
                }
            }
            StepType::Cta => {
                if let Some(cta) = step.cta_config.clone() {
                    lines.push(PreviewLine::new(
                        PreviewLineKind::Success,
                        format!("{}: {}", cta.primary_label, cta.primary_url),
                    ));
                }
            }
            StepType::Pause => lines.push(PreviewLine::new(
                PreviewLineKind::Muted,
                format!("# pause {}ms", step.delay_ms),
            )),
            StepType::Clear => lines.clear(),
            StepType::Command => {}
        }
    }

    if lines.is_empty() {
        lines.push(PreviewLine::new(
            PreviewLineKind::Comment,
            "# command matched, but no output steps are configured",
        ));
    }

    lines
}

fn help_lines(steps: &[Step]) -> Vec<PreviewLine> {
    let entries = guide_entries(steps);
    if entries.is_empty() {
        return vec![PreviewLine::new(
            PreviewLineKind::Comment,
            "# no commands configured yet",
        )];
    }

    let mut lines = vec![PreviewLine::new(
        PreviewLineKind::Success,
        "Available commands:",
    )];
    lines.extend(entries.into_iter().map(|entry| {
        PreviewLine::new(
            PreviewLineKind::Output,
            format!("  {:<24} {}", entry.command, entry.description),
        )
    }));
    lines
}

fn preview_line_class(kind: &PreviewLineKind) -> &'static str {
    match kind {
        PreviewLineKind::Command => "text-primary",
        PreviewLineKind::Output => "text-inherit",
        PreviewLineKind::Success => "text-emerald-300",
        PreviewLineKind::Error => "text-red-300",
        PreviewLineKind::Warning => "text-amber-300",
        PreviewLineKind::Muted => "text-on-surface-variant",
        PreviewLineKind::Comment => "text-on-surface-variant",
    }
}

fn terminal_style(theme: Option<Theme>) -> String {
    let bg_color = theme
        .as_ref()
        .map(|cfg| cfg.bg_color.clone())
        .unwrap_or_else(|| "#050505".to_string());
    let fg_color = theme
        .as_ref()
        .map(|cfg| cfg.fg_color.clone())
        .unwrap_or_else(|| "#f5f5f5".to_string());
    let font_family = theme
        .as_ref()
        .map(|cfg| cfg.font_family.clone())
        .unwrap_or_else(|| "JetBrains Mono".to_string());
    let font_size = theme.as_ref().map(|cfg| cfg.font_size).unwrap_or(14);
    let line_height = theme.as_ref().map(|cfg| cfg.line_height).unwrap_or(1.4);

    format!(
        "background:{bg_color};color:{fg_color};font-family:'{font_family}',monospace;font-size:{font_size}px;line-height:{line_height};"
    )
}

/// Renders a live, interactive terminal preview panel for a sequence of demo steps.
///
/// The component displays a terminal-like UI with an input prompt, a scrollable history of preview lines derived from `steps`, an optional header and titlebar, a developer footer with debug counts, and an optional collapsible mini-guide. It manages local input and history signals, handles the `help` and `clear` commands, resolves command responses from the provided `steps`, and uses `guide_open` / `set_guide_open` to control the guide panel visibility.
///
/// # Examples
///
/// ```no_run
/// use leptos::*;
/// // (Construct appropriate `Step`, `Theme` and signal helpers in your app)
/// let steps = create_rw_signal(Vec::<Step>::new());
/// let prompt_string = create_signal(String::from(">"));
/// let not_found_message = create_signal(String::from("not found"));
/// let theme = create_rw_signal(None::<Theme>);
/// let developer_mode = create_signal(false);
/// let guide_open = create_rw_signal(false);
/// let set_guide_open = guide_open;
///
/// view! {
///     <LivePreviewPanel
///         steps=steps
///         prompt_string=prompt_string
///         not_found_message=not_found_message
///         theme=theme
///         developer_mode=developer_mode
///         guide_open=guide_open
///         set_guide_open=set_guide_open
///     />
/// }
/// ```
#[component]
pub fn LivePreviewPanel(
    steps: ReadSignal<Vec<Step>>,
    prompt_string: Signal<String>,
    not_found_message: Signal<String>,
    theme: ReadSignal<Option<Theme>>,
    developer_mode: Signal<bool>,
    guide_open: ReadSignal<bool>,
    set_guide_open: WriteSignal<bool>,
    #[prop(optional, default = true)] show_header: bool,
    #[prop(optional, default = true)] show_internal_guide: bool,
    #[prop(optional, default = true)] show_titlebar: bool,
) -> impl IntoView {
    let (history, set_history) = signal(vec![PreviewLine::new(
        PreviewLineKind::Comment,
        "# SimuCLI preview ready. Type `help` for the guide or `clear` to reset.",
    )]);
    let (input, set_input) = signal(String::new());

    let title = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.window_title)
            .unwrap_or_else(|| "Terminal".to_string())
    });

    let body_style = Signal::derive(move || terminal_style(theme.get()));

    let submit_command = Callback::new(move |_: ()| {
        let command = input.get().trim().to_string();
        if command.is_empty() {
            return;
        }

        set_input.set(String::new());

        if command.eq_ignore_ascii_case("clear") {
            set_history.set(Vec::new());
            return;
        }

        let steps_snapshot = steps.get();
        let prompt = prompt_string.get();
        let mut next_lines = vec![PreviewLine::new(
            PreviewLineKind::Command,
            format!("{prompt} {command}"),
        )];

        if command.eq_ignore_ascii_case("help") {
            next_lines.extend(help_lines(&steps_snapshot));
        } else {
            next_lines.extend(response_for_command(
                &steps_snapshot,
                &command,
                &not_found_message.get(),
            ));
        }

        set_history.update(|items| items.extend(next_lines));
    });

    view! {
        <section class="relative flex h-full min-h-0 flex-col">
            <Show when=move || show_header>
                <div class="mb-4 flex items-center justify-between gap-3">
                    <div>
                        <p class="text-[10px] font-bold uppercase tracking-[0.24em] text-primary">"Preview"</p>
                        <h2 class="mt-1 font-headline text-2xl font-semibold text-on-surface">"Live Terminal"</h2>
                    </div>
                    <Show when=move || show_internal_guide>
                        <button
                            type="button"
                            class="rounded-full border border-outline bg-surface-container px-4 py-2 text-xs font-bold uppercase tracking-[0.16em] text-on-surface-variant transition-all duration-200 ease-out hover:border-primary/60 hover:text-primary"
                            on:click=move |_| set_guide_open.update(|value| *value = !*value)
                        >
                            "Open Guide"
                        </button>
                    </Show>
                </div>
            </Show>

            <div
                class=move || {
                    if show_titlebar {
                        "relative flex min-h-[520px] flex-1 overflow-hidden rounded-[34px] border border-outline-variant bg-background shadow-[0_40px_140px_-56px_rgba(0,0,0,1)]"
                    } else {
                        "relative flex h-full min-h-0 flex-1 overflow-hidden bg-transparent"
                    }
                }
            >
                <div class="flex min-w-0 flex-1 flex-col">
                    <Show when=move || show_titlebar>
                        <div class="flex h-12 items-center justify-between border-b border-outline-variant bg-surface-container-low px-5">
                            <div class="flex items-center gap-2">
                                <span class="h-3 w-3 rounded-full bg-red-500"></span>
                                <span class="h-3 w-3 rounded-full bg-amber-400"></span>
                                <span class="h-3 w-3 rounded-full bg-primary"></span>
                            </div>
                            <span class="rounded-full border border-outline-variant bg-background px-3 py-1 font-mono text-[11px] text-on-surface-variant">
                                {move || title.get()}
                            </span>
                            <span class="font-mono text-[11px] text-on-surface-variant">"80x24"</span>
                        </div>
                    </Show>

                    <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
                        <div class="min-h-0 flex-1 overflow-y-auto p-5 font-mono" style=body_style>
                            <For
                                each={move || {
                                    history
                                        .get()
                                        .into_iter()
                                        .enumerate()
                                        .collect::<Vec<_>>()
                                }}
                                key=|entry| entry.0
                                children=move |(_, line)| {
                                    let class_name = format!("whitespace-pre-wrap break-words {}", preview_line_class(&line.kind));
                                    view! {
                                        <pre class=class_name>{line.text}</pre>
                                    }
                                }
                            />

                            <div class="mt-4 flex items-center gap-2">
                                <span class="font-mono text-primary">{move || prompt_string.get()}</span>
                                <input
                                    class="min-w-0 flex-1 border-none bg-transparent font-mono text-inherit outline-none placeholder:text-on-surface-variant/60"
                                    prop:value=move || input.get()
                                    on:input=move |ev| set_input.set(event_target_value(&ev))
                                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                                        if ev.key() == "Enter" {
                                            ev.prevent_default();
                                            submit_command.run(());
                                        }
                                    }
                                    placeholder="type a command..."
                                />
                            </div>
                        </div>

                        <Show when=move || developer_mode.get()>
                            <div class="border-t border-outline-variant bg-surface-container-low px-5 py-3">
                                <div class="flex flex-wrap items-center gap-3 text-[11px] font-semibold uppercase tracking-[0.16em] text-on-surface-variant">
                                    <span class="rounded-full bg-primary/10 px-3 py-1 text-primary">"Developer view"</span>
                                    <span>{move || format!("{} steps", steps.get().len())}</span>
                                    <span>{move || format!("{} commands", guide_entries(&steps.get()).len())}</span>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>

                <Show when=move || show_internal_guide>
                    <aside
                        class=move || {
                            if guide_open.get() {
                                "absolute inset-y-0 right-0 z-20 w-full max-w-sm translate-x-0 border-l border-outline-variant bg-surface-container/95 p-5 shadow-[-30px_0_90px_-48px_rgba(0,0,0,1)] backdrop-blur-xl transition-transform duration-300 ease-out"
                            } else {
                                "absolute inset-y-0 right-0 z-20 w-full max-w-sm translate-x-full border-l border-outline-variant bg-surface-container/95 p-5 shadow-[-30px_0_90px_-48px_rgba(0,0,0,1)] backdrop-blur-xl transition-transform duration-300 ease-out"
                            }
                        }
                    >
                        <div class="flex items-center justify-between gap-4">
                            <div>
                                <p class="text-[10px] font-bold uppercase tracking-[0.24em] text-primary">"Mini Guide"</p>
                                <h3 class="mt-1 font-headline text-xl font-semibold text-on-surface">"Commands to try"</h3>
                            </div>
                            <button
                                type="button"
                                class="rounded-full border border-outline bg-background px-3 py-1 text-xs font-bold text-on-surface-variant transition-colors duration-200 hover:text-on-surface"
                                on:click=move |_| set_guide_open.set(false)
                            >
                                "Close"
                            </button>
                        </div>

                        <div class="mt-6 space-y-3">
                            <For
                                each=move || guide_entries(&steps.get())
                                key=|entry| entry.command.clone()
                                children=move |entry| {
                                    view! {
                                        <button
                                            type="button"
                                            class="w-full rounded-2xl border border-outline-variant bg-background/80 p-4 text-left transition-all duration-200 ease-out hover:-translate-y-0.5 hover:border-primary/50 hover:bg-primary/10"
                                            on:click=move |_| set_input.set(entry.command.clone())
                                        >
                                            <p class="font-mono text-sm text-primary">{entry.command.clone()}</p>
                                            <p class="mt-2 text-sm leading-6 text-on-surface-variant">{entry.description.clone()}</p>
                                        </button>
                                    }
                                }
                            />

                            <Show when=move || guide_entries(&steps.get()).is_empty()>
                                <div class="rounded-2xl border border-dashed border-outline bg-background/80 p-5 text-sm text-on-surface-variant">
                                    "No command steps yet. Add a command block or import a cast file to populate the guide."
                                </div>
                            </Show>
                        </div>
                    </aside>
                </Show>
            </div>
        </section>
    }
}
