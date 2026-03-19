use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{ParentRoute, Redirect, Route, Router, Routes},
    path,
};

use crate::auth::{
    provide_auth_context,
};
use crate::components::shell::AppShell;
use crate::pages::{
    analytics::AnalyticsPage,
    demo_editor::DemoEditorPage,
    demo_share::ShareDemoPage,
    demo_view::DemoViewPage,
    demos::DemosPage,
    landing::LandingPage,
    publish::PublishPage,
    projects::ProjectsPage,
    settings::SettingsPage,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Terminal,
    Dark,
    Light,
}

impl ThemeMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "dark" => Self::Dark,
            "light" => Self::Light,
            _ => Self::Terminal,
        }
    }
}

const THEME_STORAGE_KEY: &str = "cli-demo-studio.theme";

fn load_theme_mode() -> ThemeMode {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(THEME_STORAGE_KEY).ok().flatten())
        .map(|value| ThemeMode::from_str(&value))
        .unwrap_or(ThemeMode::Terminal)
}

fn persist_theme_mode(theme: ThemeMode) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item(THEME_STORAGE_KEY, theme.as_str());
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_auth_context();
    let (theme_mode, set_theme_mode) = signal(load_theme_mode());

    Effect::new(move |_| {
        let active_theme = theme_mode.get();
        persist_theme_mode(active_theme);
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(root) = document.document_element() {
                    let _ = root.set_attribute("data-theme", active_theme.as_str());
                }
            }
        }
    });

    view! {
        <Title text="CLI Demo Studio" />
        <Router>
            <div class="theme-switcher" role="group" aria-label="Theme selector">
                <label for="theme-select" class="theme-switcher-label">"Theme"</label>
                <select
                    id="theme-select"
                    class="theme-switcher-select"
                    prop:value=move || theme_mode.get().as_str()
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        set_theme_mode.set(ThemeMode::from_str(&value));
                    }
                >
                    <option value="terminal">"Terminal"</option>
                    <option value="dark">"Dark"</option>
                    <option value="light">"Light"</option>
                </select>
            </div>

            <Routes fallback=|| view! { <p>"Not Found"</p> }>
                <Route path=path!("/") view=LandingPage />
                <Route path=path!("/d/:slug") view=ShareDemoPage />
                <Route path=path!("/demo/view") view=DemoViewPage />

                <ParentRoute path=path!("") view=AppShell>
                    <Route path=path!("/projects") view=ProjectsPage />
                    <Route path=path!("/demos") view=DemosPage />
                    <Route path=path!("/dashboard") view=RedirectToProjects />
                    <Route path=path!("/dashboard/projects") view=ProjectsPage />
                    <Route path=path!("/dashboard/demos") view=DemosPage />
                    <Route path=path!("/dashboard/demos/:id") view=DemoEditorPage />
                    <Route path=path!("/dashboard/demos/:id/settings") view=SettingsPage />
                    <Route path=path!("/dashboard/demos/:id/publish") view=PublishPage />
                    <Route path=path!("/dashboard/demos/:id/analytics") view=AnalyticsPage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}

#[component]
fn RedirectToProjects() -> impl IntoView {
    view! { <Redirect path="/projects" /> }
}
