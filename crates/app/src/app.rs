use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{A, Route, Router, Routes},
    path,
};

use crate::api;
use crate::auth::{
    provide_auth_context,
    refresh_session_state,
    use_auth_context,
    SessionState,
};
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

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_auth_context();

    view! {
        <Title text="CLI Demo Studio" />
        <Router>
            <Routes fallback=|| view! { <p>"Not Found"</p> }>
                <Route path=path!("/") view=LandingPage />
                <Route path=path!("/d/:slug") view=ShareDemoPage />
                <Route path=path!("/demo/view") view=DemoViewPage />

                <Route path=path!("/projects") view=DashboardProjectsRoute />
                <Route path=path!("/demos") view=DashboardDemosRoute />
                <Route path=path!("/dashboard") view=DashboardProjectsRoute />
                <Route path=path!("/dashboard/projects") view=DashboardProjectsRoute />
                <Route path=path!("/dashboard/demos") view=DashboardDemosRoute />
                <Route path=path!("/dashboard/demos/:id") view=DashboardDemoEditorRoute />
                <Route path=path!("/dashboard/demos/:id/settings") view=DashboardSettingsRoute />
                <Route path=path!("/dashboard/demos/:id/publish") view=DashboardPublishRoute />
                <Route path=path!("/dashboard/demos/:id/analytics") view=DashboardAnalyticsRoute />
            </Routes>
        </Router>
    }
}

#[component]
fn DashboardShell(children: Children) -> impl IntoView {
    let auth = use_auth_context();

    let logout = move |_| {
        auth.set_logging_out.set(true);
        spawn_local({
            let auth = auth;
            async move {
                match api::logout().await {
                    Ok(_) => auth.set_session_state.set(SessionState::LoggedOut),
                    Err(err) => auth
                        .set_session_state
                        .set(SessionState::Error(format!("Logout failed: {err}"))),
                }
                auth.set_logging_out.set(false);
            }
        });
    };

    view! {
        <main class="dashboard-shell">
            <aside class="app-sidebar">
                <div class="sidebar-brand">
                    <p class="kicker">"// cli-demo-studio"</p>
                    <h1>"Dashboard"</h1>
                    <p class="muted">"Build, publish, and track CLI walkthroughs."</p>
                </div>

                <nav class="app-nav">
                    <A href="/projects">"Projects"</A>
                    <A href="/demos">"Demos"</A>
                </nav>

                <div class="sidebar-auth">
                    {move || {
                        match auth.session_state.get() {
                            SessionState::Checking => view! {
                                <p class="sidebar-auth-status">"Session: checking..."</p>
                            }
                            .into_any(),
                            SessionState::LoggedOut => view! {
                                <>
                                    <p class="sidebar-auth-status">"Not signed in"</p>
                                    <a
                                        class="button btn-primary button-block"
                                        href={api::login_url()}
                                        on:click=move |_| auth.set_logging_in.set(true)
                                    >
                                        {move || if auth.is_logging_in.get() { "Redirecting..." } else { "Login via GitHub" }}
                                    </a>
                                    <button
                                        type="button"
                                        class="button btn-outline button-block"
                                        on:click=move |_| refresh_session_state(auth.set_session_state)
                                    >
                                        "Sync Session"
                                    </button>
                                </>
                            }
                            .into_any(),
                            SessionState::LoggedIn(user) => {
                                let username = user.username;
                                let email = user.email.unwrap_or_else(|| "GitHub account".to_string());
                                let avatar_url = user.avatar_url;
                                let initial = username
                                    .chars()
                                    .next()
                                    .unwrap_or('U')
                                    .to_ascii_uppercase()
                                    .to_string();
                                let avatar_node = if let Some(url) = avatar_url {
                                    view! { <img class="sidebar-avatar" src={url} alt="User avatar" /> }.into_any()
                                } else {
                                    view! { <span class="sidebar-avatar-fallback">{initial}</span> }.into_any()
                                };
                                view! {
                                    <>
                                        <div class="sidebar-auth-profile">
                                            <div class="sidebar-avatar-wrap">{avatar_node}</div>
                                            <div>
                                                <p class="sidebar-auth-status">{format!("@{username}")}</p>
                                                <p class="muted">{email}</p>
                                            </div>
                                        </div>
                                        <div class="sidebar-auth-actions">
                                            <button
                                                type="button"
                                                class="button btn-danger button-block"
                                                on:click=logout
                                            >
                                                {move || if auth.is_logging_out.get() { "Signing out..." } else { "Logout" }}
                                            </button>
                                        </div>
                                    </>
                                }
                                .into_any()
                            }
                            SessionState::Error(message) => view! {
                                <>
                                    <p class="sidebar-auth-status sidebar-auth-error">{format!("Auth error: {message}")}</p>
                                    <a class="button btn-primary button-block" href={api::login_url()}>
                                        "Retry Login"
                                    </a>
                                </>
                            }
                            .into_any(),
                        }
                    }}
                </div>
            </aside>

            <section class="app-content">{children()}</section>
        </main>
    }
}

#[component]
fn DashboardProjectsRoute() -> impl IntoView {
    view! { <DashboardShell><ProjectsPage /></DashboardShell> }
}

#[component]
fn DashboardDemosRoute() -> impl IntoView {
    view! { <DashboardShell><DemosPage /></DashboardShell> }
}

#[component]
fn DashboardDemoEditorRoute() -> impl IntoView {
    view! { <DashboardShell><DemoEditorPage /></DashboardShell> }
}

#[component]
fn DashboardSettingsRoute() -> impl IntoView {
    view! { <DashboardShell><SettingsPage /></DashboardShell> }
}

#[component]
fn DashboardPublishRoute() -> impl IntoView {
    view! { <DashboardShell><PublishPage /></DashboardShell> }
}

#[component]
fn DashboardAnalyticsRoute() -> impl IntoView {
    view! { <DashboardShell><AnalyticsPage /></DashboardShell> }
}
