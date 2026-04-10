use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{A, Outlet, Redirect};
use leptos_router::hooks::use_location;

use crate::api;
use crate::auth::{SessionState, refresh_session_state, use_auth_context};

#[component]
pub fn AppShell() -> impl IntoView {
    let auth = use_auth_context();
    let location = use_location();
    let editor_route = Signal::derive(move || {
        let path = location.pathname.get();
        let mut segments = path.split('/').filter(|segment| !segment.is_empty());
        let first = segments.next();
        let second = segments.next();
        let third = segments.next();
        let fourth = segments.next();

        first == Some("dashboard") && second == Some("demos") && third.is_some() && fourth.is_none()
    });

    view! {
        {move || {
            match auth.session_state.get() {
                SessionState::Checking => view! {
                    <ShellSkeleton on_retry=move || refresh_session_state(auth.set_session_state) />
                }
                .into_any(),
                SessionState::LoggedOut => view! { <Redirect path="/" /> }.into_any(),
                SessionState::Error(message) => view! {
                    <main class="dashboard-shell modern-shell">
                        <section class="app-content">
                            <section class="page">
                                <h2>"Authentication Error"</h2>
                                <p class="status">{message}</p>
                                <div class="inline-actions">
                                    <a class="button btn-primary" href={api::login_url()}>
                                        "Login with GitHub"
                                    </a>
                                    <button
                                        type="button"
                                        class="button btn-outline"
                                        on:click=move |_| refresh_session_state(auth.set_session_state)
                                    >
                                        "Retry"
                                    </button>
                                </div>
                            </section>
                        </section>
                    </main>
                }
                .into_any(),
                SessionState::LoggedIn(user) => {
                    if editor_route.get() {
                        return view! {
                            <main class="modern-shell">
                                <section class="app-content app-content--full">
                                    <Outlet />
                                </section>
                            </main>
                        }
                        .into_any();
                    }

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
                        <main class="dashboard-shell modern-shell">
                            <aside class="app-sidebar modern-sidebar">
                                <div class="sidebar-brand modern-brand">
                                    <div class="brand-row">
                                        <span class="brand-badge">">_"</span>
                                        <h1>"Demo Studio"</h1>
                                    </div>
                                    <p class="muted">"Create, publish, and monitor interactive CLI demos."</p>
                                </div>

                                <nav class="app-nav">
                                    <A href="/projects">
                                        {project_icon()}
                                        <span>"Projects"</span>
                                    </A>
                                    <A href="/demos">
                                        {demo_icon()}
                                        <span>"Demos"</span>
                                    </A>
                                </nav>

                                <div class="sidebar-auth">
                                    <div class="sidebar-auth-profile">
                                        <div class="sidebar-avatar-wrap">{avatar_node}</div>
                                        <div>
                                            <p class="sidebar-auth-status">"Signed in as"</p>
                                            <p class="sidebar-username">{format!("@{username}")}</p>
                                            <p class="muted">{email}</p>
                                        </div>
                                    </div>
                                    <div class="sidebar-auth-actions">
                                        <button
                                            type="button"
                                            class="button btn-danger button-block logout-button"
                                            on:click=move |_| {
                                                auth.set_logging_out.set(true);
                                                spawn_local({
                                                    let auth = auth;
                                                    async move {
                                                        match api::logout().await {
                                                            Ok(_) => {
                                                                auth.set_session_state.set(SessionState::LoggedOut);
                                                                if let Some(window) = web_sys::window() {
                                                                    let _ = window.location().set_href("/");
                                                                }
                                                            }
                                                            Err(err) => auth
                                                                .set_session_state
                                                                .set(SessionState::Error(format!("Logout failed: {err}"))),
                                                        }
                                                        auth.set_logging_out.set(false);
                                                    }
                                                });
                                            }
                                        >
                                            {move || if auth.is_logging_out.get() { "Signing out..." } else { "Logout" }}
                                        </button>
                                    </div>
                                </div>
                            </aside>

                            <section class="app-content">
                                <Outlet />
                            </section>
                        </main>
                    }
                    .into_any()
                }
            }
        }}
    }
}

fn project_icon() -> impl IntoView {
    view! {
        <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
            <path d="M3 7.5a2.5 2.5 0 0 1 2.5-2.5h4l1.8 2h7.2A2.5 2.5 0 0 1 21 9.5v8A2.5 2.5 0 0 1 18.5 20h-13A2.5 2.5 0 0 1 3 17.5z" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
    }
}

fn demo_icon() -> impl IntoView {
    view! {
        <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
            <path d="M4 6.5A2.5 2.5 0 0 1 6.5 4h11A2.5 2.5 0 0 1 20 6.5v11a2.5 2.5 0 0 1-2.5 2.5h-11A2.5 2.5 0 0 1 4 17.5z" stroke-linecap="round" stroke-linejoin="round" />
            <path d="m9 9 3 3-3 3" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M13.5 15H16" stroke-linecap="round" />
        </svg>
    }
}

#[component]
fn ShellSkeleton(on_retry: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <main class="dashboard-shell modern-shell">
            <aside class="app-sidebar modern-sidebar">
                <div class="sidebar-brand modern-brand">
                    <div class="brand-row">
                        <span class="brand-badge">">_"</span>
                        <h1>"Demo Studio"</h1>
                    </div>
                    <p class="muted">"Checking session..."</p>
                </div>
            </aside>
            <section class="app-content">
                <section class="page shell-loading-page">
                    <div class="shell-spinner"></div>
                    <p class="muted">"If you just logged in, this should complete in a moment."</p>
                    <div class="inline-actions">
                        <button type="button" class="button btn-outline" on:click=move |_| on_retry()>
                            "Retry Session Check"
                        </button>
                        <a class="button btn-primary" href={api::login_url()}>
                            "Login with GitHub"
                        </a>
                    </div>
                </section>
            </section>
        </main>
    }
}
