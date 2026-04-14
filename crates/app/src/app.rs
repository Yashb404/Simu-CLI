use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::{
    components::{ParentRoute, Redirect, Route, Router, Routes},
    path,
};

use crate::auth::provide_auth_context;
use crate::components::shell::AppShell;
use crate::pages::{
    analytics::AnalyticsPage, demo_editor::DemoEditorPage, demo_share::ShareDemoPage,
    demo_view::DemoViewPage, demos::DemosPage, docs::DocsPage, docs::DocsSectionPage,
    embed_fallback::EmbedFallbackPage, landing::LandingPage, publish::PublishPage,
    settings::SettingsPage,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Terminal,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "dark" => Self::Dark,
            "light" => Self::Light,
            _ => Self::Terminal,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ThemeController {
    pub mode: ReadSignal<ThemeMode>,
    pub set_mode: WriteSignal<ThemeMode>,
}

const THEME_STORAGE_KEY: &str = "cli-demo-studio.theme";

fn load_theme_mode() -> ThemeMode {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(THEME_STORAGE_KEY).ok().flatten())
        .map(|value| ThemeMode::parse(&value))
        .unwrap_or(ThemeMode::Terminal)
}

fn persist_theme_mode(theme: ThemeMode) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item(THEME_STORAGE_KEY, theme.as_str());
    }
}

/// Root application component that provides global context, manages persistent theme state, and declares client-side routes.
///
/// This component:
/// - Installs meta and authentication contexts for the app.
/// - Provides a ThemeController into context and keeps the current theme persisted to local storage and mirrored to the document root `data-theme` attribute.
/// - Renders the application's router with public routes (e.g., `/`, `/docs`, `/d/:slug`, `/demo/view`) and authenticated routes nested inside the `AppShell` layout (including `/dashboard`, dashboard demo editor/share routes, and username-scoped demo/project routes).
///
/// # Examples
///
/// ```
/// let _root_view = App();
/// ```
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_auth_context();
    let (theme_mode, set_theme_mode) = signal(load_theme_mode());
    provide_context(ThemeController {
        mode: theme_mode,
        set_mode: set_theme_mode,
    });

    Effect::new(move |_| {
        let active_theme = theme_mode.get();
        persist_theme_mode(active_theme);
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(root) = document.document_element()
        {
            let _ = root.set_attribute("data-theme", active_theme.as_str());
        }
    });

    view! {
        <Title text="SimuCLI" />
        <Router>
            <Routes fallback=|| view! { <p>"Not Found"</p> }>
                <Route path=path!("/") view=LandingPage />
                <Route path=path!("/docs") view=DocsPage />
                <Route path=path!("/docs/:section") view=DocsSectionPage />
                <Route path=path!("/docs/:category/:section") view=DocsSectionPage />
                <Route path=path!("/d/:slug") view=ShareDemoPage />
                <Route path=path!("/embed/:id") view=EmbedFallbackPage />
                <Route path=path!("/demo/view") view=DemoViewPage />

                <ParentRoute path=path!("") view=AppShell>
                    <Route path=path!("/dashboard") view=DemosPage />
                    <Route path=path!("/dashboard/demos") view=RedirectDashboardHome />
                    <Route path=path!("/dashboard/demos/:id") view=DemoEditorPage />
                    <Route path=path!("/dashboard/demos/:id/share") view=PublishPage />
                    <Route path=path!("/:username/projects/:slug") view=DemosPage />
                    <Route path=path!("/:username/demos/:id") view=DemoEditorPage />
                    <Route path=path!("/:username/demos/:id/settings") view=SettingsPage />
                    <Route path=path!("/:username/demos/:id/share") view=PublishPage />
                    <Route path=path!("/:username/demos/:id/analytics") view=AnalyticsPage />
                    <Route path=path!("/:username/projects/:slug/demos/:id") view=DemoEditorPage />
                    <Route path=path!("/:username/projects/:slug/demos/:id/settings") view=SettingsPage />
                    <Route path=path!("/:username/projects/:slug/demos/:id/share") view=PublishPage />
                    <Route path=path!("/:username/projects/:slug/demos/:id/analytics") view=AnalyticsPage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}

/// Redirects the current route to "/dashboard".
///
/// # Examples
///
/// ```
/// let view = RedirectDashboardHome();
/// // renders a Redirect that navigates to "/dashboard"
/// ```
#[component]
fn RedirectDashboardHome() -> impl IntoView {
    view! { <Redirect path="/dashboard" /> }
}
