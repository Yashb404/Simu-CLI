                            use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{A, Outlet, Redirect};

use crate::api;
use crate::auth::{refresh_session_state, use_auth_context, SessionState};

#[component]
pub fn AppShell() -> impl IntoView {
    let auth = use_auth_context();

    view! {
        {move || {
            match auth.session_state.get() {
                SessionState::Checking => view! { <ShellSkeleton /> }.into_any(),
                SessionState::LoggedOut => view! { <Redirect path="/" /> }.into_any(),
                SessionState::Error(message) => view! {
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

#[component]
fn ShellSkeleton() -> impl IntoView {
    view! {
        <main class="dashboard-shell">
            <aside class="app-sidebar">
                <div class="sidebar-brand">
                    <p class="kicker">"// cli-demo-studio"</p>
                    <h1>"Dashboard"</h1>
                    <p class="muted">"Mounting workspace..."</p>
                </div>
            </aside>
            <section class="app-content">
                <section class="page">
                    <h2>"Loading"</h2>
                    <p class="status">"Mounting filesystem..."</p>
                </section>
            </section>
        </main>
    }
}
