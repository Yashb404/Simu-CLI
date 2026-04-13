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
    let (sidebar_projects, set_sidebar_projects) = signal(Vec::<api::DashboardProject>::new());
    let (sidebar_projects_open, set_sidebar_projects_open) = signal(true);
    let (sidebar_project_name, set_sidebar_project_name) = signal(String::new());
    let (sidebar_project_description, set_sidebar_project_description) = signal(String::new());
    let (sidebar_project_status, set_sidebar_project_status) = signal(String::new());
    let (sidebar_project_creating, set_sidebar_project_creating) = signal(false);
    let editor_route = Signal::derive(move || {
        let path = location.pathname.get();
        let mut segments = path.split('/').filter(|segment| !segment.is_empty());
        let first = segments.next();
        let second = segments.next();
        let third = segments.next();
        let fourth = segments.next();
        let fifth = segments.next();
        let sixth = segments.next();

        let dashboard_editor = first == Some("dashboard") && second == Some("demos") && third.is_some();
        let namespaced_demo_editor = first.is_some() && second == Some("demos") && third.is_some();
        let namespaced_project_demo_editor =
            first.is_some() && second == Some("projects") && third.is_some() && fourth == Some("demos") && fifth.is_some();

        let has_deep_suffix = sixth.is_some();
        (dashboard_editor || namespaced_demo_editor || namespaced_project_demo_editor)
            && !has_deep_suffix
    });

    Effect::new(move |_| {
        match auth.session_state.get() {
            SessionState::LoggedIn(_) => {
                let set_sidebar_projects = set_sidebar_projects;
                let set_sidebar_project_status = set_sidebar_project_status;
                spawn_local(async move {
                    match api::list_projects().await {
                        Ok(projects) => {
                            set_sidebar_projects.set(projects);
                            set_sidebar_project_status.set(String::new());
                        }
                        Err(err) => {
                            set_sidebar_project_status.set(format!("Projects unavailable: {err}"));
                        }
                    }
                });
            }
            _ => {
                set_sidebar_projects.set(Vec::new());
                set_sidebar_project_status.set(String::new());
            }
        }
    });

    let create_sidebar_project = move |_| {
        let name = sidebar_project_name.get();
        let description = sidebar_project_description.get();

        if name.trim().is_empty() {
            set_sidebar_project_status.set("Project name is required".to_string());
            return;
        }

        set_sidebar_project_creating.set(true);
        spawn_local({
            let set_sidebar_projects = set_sidebar_projects;
            let set_sidebar_project_name = set_sidebar_project_name;
            let set_sidebar_project_description = set_sidebar_project_description;
            let set_sidebar_project_status = set_sidebar_project_status;
            let set_sidebar_project_creating = set_sidebar_project_creating;
            async move {
                match api::create_project(
                    name.trim(),
                    if description.trim().is_empty() {
                        None
                    } else {
                        Some(description.trim())
                    },
                )
                .await
                {
                    Ok(project) => {
                        set_sidebar_projects.update(|items| items.insert(0, project));
                        set_sidebar_project_name.set(String::new());
                        set_sidebar_project_description.set(String::new());
                        set_sidebar_project_status.set("Project created.".to_string());
                        set_sidebar_projects_open.set(true);
                    }
                    Err(err) => {
                        set_sidebar_project_status.set(format!("Create failed: {err}"));
                    }
                }
                set_sidebar_project_creating.set(false);
            }
        });
    };

    let sidebar_project_links = Signal::derive(move || {
        let projects = sidebar_projects.get();
        let show_all = sidebar_projects_open.get() || projects.len() <= 5;

        projects
            .into_iter()
            .take(if show_all { usize::MAX } else { 5 })
            .collect::<Vec<_>>()
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
                    let username_for_projects = username.clone();
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
                                    <A href="/dashboard">
                                        {demo_icon()}
                                        <span>"Dashboard"</span>
                                    </A>
                                </nav>

                                <section class="sidebar-projects panel">
                                    <div class="sidebar-projects-header">
                                        <div>
                                            <p class="sidebar-projects-kicker">"Projects"</p>
                                            <p class="muted">"Open or group your demos here."</p>
                                        </div>
                                        <button
                                            type="button"
                                            class="button btn-outline sidebar-projects-toggle"
                                            on:click=move |_| set_sidebar_projects_open.update(|value| *value = !*value)
                                        >
                                            {move || if sidebar_projects_open.get() { "Less" } else { "More" }}
                                        </button>
                                    </div>

                                    <div class="sidebar-projects-create">
                                        <input
                                            class="sidebar-projects-input"
                                            placeholder="New project"
                                            prop:value=move || sidebar_project_name.get()
                                            on:input=move |ev| set_sidebar_project_name.set(event_target_value(&ev))
                                        />
                                        <textarea
                                            class="sidebar-projects-textarea"
                                            placeholder="Description"
                                            prop:value=move || sidebar_project_description.get()
                                            on:input=move |ev| set_sidebar_project_description.set(event_target_value(&ev))
                                        />
                                        <button
                                            type="button"
                                            class="button btn-primary button-block"
                                            disabled=move || sidebar_project_creating.get()
                                            on:click=create_sidebar_project
                                        >
                                            {move || if sidebar_project_creating.get() { "Creating..." } else { "Create Project" }}
                                        </button>
                                        <Show when=move || !sidebar_project_status.get().is_empty()>
                                            <p class="sidebar-projects-status">{move || sidebar_project_status.get()}</p>
                                        </Show>
                                    </div>

                                    <div class="sidebar-projects-list">
                                        <For
                                            each=move || sidebar_project_links.get()
                                            key=|project| project.id.clone()
                                            children=move |project| {
                                                let path = api::namespaced_project_path(&username_for_projects, &project.name);
                                                view! {
                                                    <A href={path}>
                                                        <span class="sidebar-project-dot"></span>
                                                        <span class="sidebar-project-name">{project.name}</span>
                                                    </A>
                                                }
                                            }
                                        />
                                    </div>
                                </section>

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
