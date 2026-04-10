use leptos::prelude::*;
use shared::models::demo::{DemoSettings, EngineMode, Theme, WindowStyle};

fn window_style_to_value(style: &WindowStyle) -> &'static str {
    match style {
        WindowStyle::MacOs => "macos",
        WindowStyle::Linux => "linux",
        WindowStyle::Windows => "windows",
        WindowStyle::None => "none",
    }
}

fn window_style_from_value(value: &str) -> WindowStyle {
    match value {
        "linux" => WindowStyle::Linux,
        "windows" => WindowStyle::Windows,
        "none" => WindowStyle::None,
        _ => WindowStyle::MacOs,
    }
}

#[component]
pub fn DemoSettingsForm(
    title: ReadSignal<String>,
    set_title: WriteSignal<String>,
    slug: ReadSignal<String>,
    set_slug: WriteSignal<String>,
    settings: ReadSignal<Option<DemoSettings>>,
    set_settings: WriteSignal<Option<DemoSettings>>,
    theme: ReadSignal<Option<Theme>>,
    set_theme: WriteSignal<Option<Theme>>,
) -> impl IntoView {
    let prompt_string = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.prompt_string)
            .unwrap_or_else(|| "$".to_string())
    });

    let window_title = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.window_title)
            .unwrap_or_else(|| "Terminal".to_string())
    });

    let window_style = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| window_style_to_value(&cfg.window_style).to_string())
            .unwrap_or_else(|| "macos".to_string())
    });

    let bg_color = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.bg_color)
            .unwrap_or_else(|| "#090909".to_string())
    });

    let fg_color = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.fg_color)
            .unwrap_or_else(|| "#f5f5f5".to_string())
    });

    let cursor_color = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.cursor_color)
            .unwrap_or_else(|| "#4ae176".to_string())
    });

    let font_family = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.font_family)
            .unwrap_or_else(|| "JetBrains Mono".to_string())
    });

    let font_size = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| cfg.font_size.to_string())
            .unwrap_or_else(|| "14".to_string())
    });

    let line_height = Signal::derive(move || {
        theme
            .get()
            .map(|cfg| format!("{:.2}", cfg.line_height))
            .unwrap_or_else(|| "1.40".to_string())
    });

    let not_found_message = Signal::derive(move || {
        settings
            .get()
            .map(|cfg| cfg.not_found_message)
            .unwrap_or_else(|| "command not found".to_string())
    });

    let engine_mode = Signal::derive(move || {
        settings
            .get()
            .map(|cfg| cfg.engine_mode.clone())
            .unwrap_or(EngineMode::Sequential)
    });

    view! {
        <section class="rounded-[28px] border border-outline-variant bg-surface-container-low/95 p-5 shadow-[0_24px_90px_-48px_rgba(0,0,0,0.95)]">
            <div class="mb-5 flex items-start justify-between gap-4">
                <div>
                    <p class="text-[10px] font-bold uppercase tracking-[0.24em] text-primary">"Project"</p>
                    <h2 class="mt-1 font-headline text-xl font-semibold text-on-surface">"Demo Settings"</h2>
                </div>
                <span class="rounded-full border border-outline bg-surface-container px-3 py-1 text-[10px] font-bold uppercase tracking-[0.18em] text-on-surface-variant">
                    "Theme wired"
                </span>
            </div>

            <div class="grid gap-4 md:grid-cols-2">
                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant md:col-span-2">
                    "Title"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 placeholder:text-on-surface-variant/50 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=move || title.get()
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                        placeholder="Untitled demo"
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Slug"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 placeholder:text-on-surface-variant/50 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=move || slug.get()
                        on:input=move |ev| set_slug.set(event_target_value(&ev))
                        placeholder="my-cli-demo"
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Engine"
                    <select
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=move || match engine_mode.get() {
                            EngineMode::Sequential => "sequential".to_string(),
                            EngineMode::FreePlay => "free_play".to_string(),
                        }
                        on:change=move |ev| {
                            let next_mode = match event_target_value(&ev).as_str() {
                                "free_play" => EngineMode::FreePlay,
                                _ => EngineMode::Sequential,
                            };
                            set_settings.update(|value| {
                                if let Some(settings) = value.as_mut() {
                                    settings.engine_mode = next_mode;
                                }
                            });
                        }
                    >
                        <option value="sequential">"Sequential"</option>
                        <option value="free_play">"Free play"</option>
                    </select>
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Window Title"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=window_title
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.window_title = next.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Chrome"
                    <select
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=window_style
                        on:change=move |ev| {
                            let next = window_style_from_value(&event_target_value(&ev));
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.window_style = next;
                                }
                            });
                        }
                    >
                        <option value="macos">"macOS"</option>
                        <option value="linux">"Linux"</option>
                        <option value="windows">"Windows"</option>
                        <option value="none">"None"</option>
                    </select>
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Prompt"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 font-mono text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=prompt_string
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.prompt_string = next.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Fallback"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=not_found_message
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            set_settings.update(|value| {
                                if let Some(settings) = value.as_mut() {
                                    settings.not_found_message = next.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Background"
                    <input
                        class="h-12 rounded-2xl border border-outline-variant bg-surface-container-high px-3 py-2 outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        type="color"
                        prop:value=bg_color
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.bg_color = next.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Foreground"
                    <input
                        class="h-12 rounded-2xl border border-outline-variant bg-surface-container-high px-3 py-2 outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        type="color"
                        prop:value=fg_color
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.fg_color = next.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Cursor"
                    <input
                        class="h-12 rounded-2xl border border-outline-variant bg-surface-container-high px-3 py-2 outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        type="color"
                        prop:value=cursor_color
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.cursor_color = next.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Font"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        prop:value=font_family
                        on:input=move |ev| {
                            let next = event_target_value(&ev);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.font_family = next.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Size"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        type="number"
                        min="10"
                        max="24"
                        prop:value=font_size
                        on:input=move |ev| {
                            let next = event_target_value(&ev).parse::<u8>().unwrap_or(14).clamp(10, 24);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.font_size = next;
                                }
                            });
                        }
                    />
                </label>

                <label class="flex flex-col gap-2 text-xs font-semibold uppercase tracking-[0.14em] text-on-surface-variant">
                    "Line Height"
                    <input
                        class="rounded-2xl border border-outline-variant bg-surface-container-high px-4 py-3 text-sm normal-case tracking-normal text-on-surface outline-none transition-all duration-200 focus:border-primary focus:ring-2 focus:ring-primary/20"
                        type="number"
                        min="1"
                        max="2"
                        step="0.05"
                        prop:value=line_height
                        on:input=move |ev| {
                            let next = event_target_value(&ev).parse::<f32>().unwrap_or(1.4).clamp(1.0, 2.0);
                            set_theme.update(|value| {
                                if let Some(theme) = value.as_mut() {
                                    theme.line_height = next;
                                }
                            });
                        }
                    />
                </label>
            </div>
        </section>
    }
}
