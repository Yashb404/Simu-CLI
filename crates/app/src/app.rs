use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{A, Route, Router, Routes},
    path,
};
use wasm_bindgen::{closure::Closure, JsCast};

use crate::pages::{
    analytics::AnalyticsPage,
    demo_editor::DemoEditorPage,
    demo_share::ShareDemoPage,
    demo_view::DemoViewPage,
    demos::DemosPage,
    publish::PublishPage,
    projects::ProjectsPage,
    settings::SettingsPage,
};
use crate::api;

#[derive(Clone)]
enum SessionState {
    Checking,
    LoggedOut,
    LoggedIn(api::CurrentUser),
    Error(String),
}

fn refresh_session_state(set_session_state: WriteSignal<SessionState>) {
    spawn_local(async move {
        match api::get_current_user().await {
            Ok(user) => set_session_state.set(SessionState::LoggedIn(user)),
            Err(err) => {
                if err.contains("Not logged in") {
                    set_session_state.set(SessionState::LoggedOut);
                } else {
                    set_session_state.set(SessionState::Error(err));
                }
            }
        }
    });
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (session_state, set_session_state) = signal(SessionState::Checking);
    let (is_logging_in, set_logging_in) = signal(false);
    let (is_logging_out, set_logging_out) = signal(false);

    Effect::new(move |_| {
        refresh_session_state(set_session_state);

        if let Some(window) = web_sys::window() {
            let callback = Closure::wrap(Box::new(move || {
                if matches!(session_state.get_untracked(), SessionState::LoggedOut | SessionState::Checking) {
                    refresh_session_state(set_session_state);
                }
            }) as Box<dyn FnMut()>);

            if window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    4000,
                )
                .is_ok()
            {
                callback.forget();
            }
        }
    });

    let logout = move |_| {
        set_logging_out.set(true);
        spawn_local({
            let set_session_state = set_session_state;
            let set_logging_out = set_logging_out;
            async move {
                match api::logout().await {
                    Ok(_) => set_session_state.set(SessionState::LoggedOut),
                    Err(err) => set_session_state.set(SessionState::Error(format!("Logout failed: {err}"))),
                }
                set_logging_out.set(false);
            }
        });
    };

    view! {
        <Title text="CLI Demo Studio" />
        <Router>
            <main class="dashboard-shell">
                <aside class="app-sidebar">
                    <div class="sidebar-brand">
                        <p class="kicker">"CLI Demo Studio"</p>
                        <h1>"Dashboard"</h1>
                        <p>"Build, publish, and track CLI walkthroughs."</p>
                    </div>

                    <nav class="app-nav">
                        <A href="/dashboard/projects">"Projects"</A>
                        <A href="/dashboard/demos">"Demos"</A>
                    </nav>

                    <div class="sidebar-auth">
                        {move || {
                            match session_state.get() {
                                SessionState::Checking => view! {
                                    <p class="sidebar-auth-status">"Session: checking..."</p>
                                }.into_any(),
                                SessionState::LoggedOut => view! {
                                    <>
                                        <p class="sidebar-auth-status">"Not signed in"</p>
                                        <a
                                            class="button btn-primary button-block"
                                            href={api::login_url()}
                                            on:click=move |_| set_logging_in.set(true)
                                        >
                                            {move || if is_logging_in.get() { "Redirecting..." } else { "Login via GitHub" }}
                                        </a>
                                        <button
                                            type="button"
                                            class="button btn-outline button-block"
                                            on:click=move |_| refresh_session_state(set_session_state)
                                        >
                                            "Sync Session"
                                        </button>
                                    </>
                                }.into_any(),
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
                                            <div class="sidebar-avatar-wrap">
                                                {avatar_node}
                                            </div>
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
                                                {move || if is_logging_out.get() { "Signing out..." } else { "Logout" }}
                                            </button>
                                        </div>
                                    </>
                                    }.into_any()
                                }
                                SessionState::Error(message) => view! {
                                    <>
                                        <p class="sidebar-auth-status sidebar-auth-error">{format!("Auth error: {message}")}</p>
                                        <a class="button btn-primary button-block" href={api::login_url()}>
                                            "Retry Login"
                                        </a>
                                    </>
                                }.into_any(),
                            }
                        }}
                    </div>
                </aside>

                <section class="app-content">
                    <Routes fallback=|| view! { <p>"Not Found"</p> }>
                        <Route path=path!("/") view=ProjectsPage />
                        <Route path=path!("/dashboard") view=ProjectsPage />
                        <Route path=path!("/dashboard/projects") view=ProjectsPage />
                        <Route path=path!("/dashboard/demos") view=DemosPage />
                        <Route path=path!("/dashboard/demos/:id") view=DemoEditorPage />
                        <Route path=path!("/dashboard/demos/:id/settings") view=SettingsPage />
                        <Route path=path!("/dashboard/demos/:id/publish") view=PublishPage />
                        <Route path=path!("/dashboard/demos/:id/analytics") view=AnalyticsPage />
                        <Route path=path!("/d/:slug") view=ShareDemoPage />
                        <Route path=path!("/demo/view") view=DemoViewPage />
                    </Routes>
                </section>
            </main>
        </Router>
    }
}
