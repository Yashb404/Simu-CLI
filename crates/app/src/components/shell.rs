use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{A, Outlet, Redirect};
use leptos_router::hooks::use_location;

use crate::api;
use crate::auth::{SessionState, refresh_session_state, use_auth_context};
use crate::pages::demos::ThemeModeToggle;

#[derive(Clone, Copy)]
pub struct DashboardSearchContext {
    pub search_query: ReadSignal<String>,
    pub set_search_query: WriteSignal<String>,
}

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
    let (sidebar_project_form_open, set_sidebar_project_form_open) = signal(true);
    let (dashboard_search_query, set_dashboard_search_query) = signal(String::new());
    let (mobile_sidebar_open, set_mobile_sidebar_open) = signal(false);

    provide_context(DashboardSearchContext {
        search_query: dashboard_search_query,
        set_search_query: set_dashboard_search_query,
    });

    let editor_route = Signal::derive(move || {
        let path = location.pathname.get();
        let mut segments = path.split('/').filter(|segment| !segment.is_empty());
        let first = segments.next();
        let second = segments.next();
        let third = segments.next();
        let fourth = segments.next();
        let fifth = segments.next();
        let sixth = segments.next();

        // Editor routes (full-bleed layout)
        let dashboard_editor = first == Some("dashboard") && second == Some("demos") && third.is_some();
        let namespaced_demo_editor = first.is_some() && second == Some("demos") && third.is_some();
        let namespaced_project_demo_editor =
            first.is_some() && second == Some("projects") && third.is_some() && fourth == Some("demos") && fifth.is_some();
        
        // Dashboard listing routes (also need full-bleed for new SimuCLI design)
        let dashboard_listing = first == Some("dashboard") && (second == Some("projects") || second.is_none());
        let namespaced_dashboard_listing = first.is_some() && second == Some("projects");

        let has_deep_suffix = sixth.is_some();
        (dashboard_editor || namespaced_demo_editor || namespaced_project_demo_editor || dashboard_listing || namespaced_dashboard_listing)
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
                        set_sidebar_project_form_open.set(false);
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
                    <main class="bg-surface text-on-surface flex h-screen overflow-hidden">
                        <section class="flex-1 overflow-y-auto p-8 custom-scrollbar">
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
                    let username_for_profile = username.clone();
                    let email = user.email.unwrap_or_else(|| "GitHub account".to_string());
                    let avatar_url = user.avatar_url;
                    let initial = username
                        .chars()
                        .next()
                        .unwrap_or('U')
                        .to_ascii_uppercase()
                        .to_string();
                    let render_avatar = {
                        let avatar_url = avatar_url.clone();
                        let initial = initial.clone();
                        move || {
                            if let Some(url) = avatar_url.clone() {
                                view! { <img class="h-8 w-8 rounded-full border border-zinc-700 object-cover" src={url} alt="GitHub User Avatar" /> }.into_any()
                            } else {
                                view! { <span class="grid h-8 w-8 place-items-center rounded-full border border-zinc-700 font-mono text-xs font-bold text-[#4ae176]">{initial.clone()}</span> }.into_any()
                            }
                        }
                    };

                    let mobile_sidebar_classes = move || {
                        if mobile_sidebar_open.get() {
                            "fixed inset-y-0 left-0 z-50 flex w-64 flex-col border-r border-zinc-800 bg-[#131316] transition-transform duration-200 md:static md:translate-x-0"
                        } else {
                            "fixed inset-y-0 left-0 z-50 flex w-64 -translate-x-full flex-col border-r border-zinc-800 bg-[#131316] transition-transform duration-200 md:static md:translate-x-0"
                        }
                    };

                    view! {
                        <main class="bg-surface text-on-surface flex h-screen overflow-hidden">
                            <Show when=move || mobile_sidebar_open.get()>
                                <button
                                    type="button"
                                    class="fixed inset-0 z-40 border-none bg-black/50 md:hidden"
                                    aria-label="Close sidebar"
                                    on:click=move |_| set_mobile_sidebar_open.set(false)
                                />
                            </Show>

                            <aside class=mobile_sidebar_classes>
                                <div class="flex items-center gap-3 px-6 py-5">
                                    <span class="material-symbols-outlined text-[#4ae176]" style="font-variation-settings: 'FILL' 1;">
                                        "terminal"
                                    </span>
                                    <div>
                                        <h1 class="mono-text text-lg font-bold leading-none tracking-tighter text-[#4ae176]">"SimuCLI"</h1>
                                        <p class="label-text mt-1 text-[10px] uppercase tracking-widest text-zinc-500">{"CLI Studio"}</p>
                                    </div>
                                </div>

                                <div class="px-6 pb-4 md:hidden">
                                    <button
                                        type="button"
                                        class="flex w-full items-center justify-between rounded border border-zinc-800 bg-[#19191d] px-4 py-2.5 transition-colors duration-200 hover:border-[#4ae176]"
                                        on:click=move |_| set_sidebar_project_form_open.update(|value| *value = !*value)
                                    >
                                        <span class="font-medium tracking-tight">"New Project"</span>
                                        <span class="material-symbols-outlined text-sm text-[#4ae176]">"add"</span>
                                    </button>
                                </div>

                                <div class="px-6 pb-4">
                                    <button
                                        type="button"
                                        class="flex w-full items-center justify-between rounded border border-zinc-800 bg-[#19191d] px-4 py-2.5 transition-colors duration-200 hover:border-[#4ae176]"
                                        on:click=move |_| set_sidebar_project_form_open.update(|value| *value = !*value)
                                    >
                                        <span class="font-medium tracking-tight">"New Project"</span>
                                        <span class="material-symbols-outlined text-sm text-[#4ae176]">"add"</span>
                                    </button>

                                    <Show when=move || sidebar_project_form_open.get()>
                                        <div class="mt-4 grid gap-3">
                                            <input
                                                class="rounded border border-zinc-800 bg-[#131316] px-4 py-2.5 text-sm text-zinc-200 placeholder:text-zinc-600"
                                                placeholder="Project name"
                                                prop:value=move || sidebar_project_name.get()
                                                on:input=move |ev| set_sidebar_project_name.set(event_target_value(&ev))
                                            />
                                            <textarea
                                                class="min-h-24 rounded border border-zinc-800 bg-[#131316] px-4 py-2.5 text-sm text-zinc-200 placeholder:text-zinc-600"
                                                placeholder="Description (optional)"
                                                prop:value=move || sidebar_project_description.get()
                                                on:input=move |ev| set_sidebar_project_description.set(event_target_value(&ev))
                                            />
                                            <button
                                                type="button"
                                                class="rounded bg-[#4ae176] px-4 py-2.5 text-sm font-bold text-[#004b1e] transition-opacity hover:opacity-90"
                                                disabled=move || sidebar_project_creating.get()
                                                on:click=create_sidebar_project
                                            >
                                                {move || if sidebar_project_creating.get() { "Creating..." } else { "Create Project" }}
                                            </button>
                                            <Show when=move || !sidebar_project_status.get().is_empty()>
                                                <p class="text-[11px] text-zinc-500">{move || sidebar_project_status.get()}</p>
                                            </Show>
                                        </div>
                                    </Show>
                                </div>

                                <nav class="px-3">
                                    <p class="label-text mb-3 px-3 text-[10px] uppercase tracking-widest text-zinc-600">"Main"</p>
                                    <A attr:class="flex items-center gap-3 border-r-2 border-[#4ae176] bg-[#19191d] px-3 py-2 text-sm font-bold tracking-tight text-[#4ae176] transition-colors duration-150" href="/dashboard">
                                        <span class="material-symbols-outlined text-[20px]" style="font-variation-settings: 'FILL' 1;">"terminal"</span>
                                        <span>"Workspace"</span>
                                    </A>
                                    <a class="flex items-center gap-3 px-3 py-2 text-sm tracking-tight text-zinc-500 transition-colors duration-150 hover:bg-[#19191d] hover:text-zinc-300" href="#">
                                        <span class="material-symbols-outlined text-[20px]">"inventory_2"</span>
                                        <span>"Library"</span>
                                    </a>
                                    <a class="flex items-center gap-3 px-3 py-2 text-sm tracking-tight text-zinc-500 transition-colors duration-150 hover:bg-[#19191d] hover:text-zinc-300" href="#">
                                        <span class="material-symbols-outlined text-[20px]">"analytics"</span>
                                        <span>"Analytics"</span>
                                    </a>
                                </nav>

                                <div class="mt-8 px-3">
                                    <div class="mb-3 flex items-center justify-between px-3">
                                        <p class="label-text text-[10px] uppercase tracking-widest text-zinc-600">"Projects"</p>
                                        <button
                                            type="button"
                                            class="rounded border border-zinc-800 bg-[#19191d] px-3 py-1 text-[10px] font-bold uppercase tracking-widest text-zinc-400 transition-colors hover:border-[#4ae176] hover:text-white"
                                            on:click=move |_| set_sidebar_projects_open.update(|value| *value = !*value)
                                        >
                                            {move || if sidebar_projects_open.get() { "Less" } else { "More" }}
                                        </button>
                                    </div>
                                    <div class="space-y-1">
                                        <For
                                            each=move || sidebar_project_links.get()
                                            key=|project| project.id.clone()
                                            children=move |project| {
                                                let path = api::namespaced_project_path(&username_for_projects, &project.name);
                                                view! {
                                                    <A attr:class="group flex items-center gap-3 px-3 py-2 text-sm text-zinc-400 transition-colors hover:text-white" href={path}>
                                                        <span class="h-1.5 w-1.5 rounded-full bg-zinc-700 transition-colors group-hover:bg-[#4ae176]"></span>
                                                        <span class="mono-text truncate">{project.name}</span>
                                                    </A>
                                                }
                                            }
                                        />
                                    </div>
                                </div>

                                <div class="mt-auto border-t border-zinc-800/50 p-6">
                                    <a class="flex items-center gap-3 rounded px-3 py-2 text-sm tracking-tight text-zinc-500 transition-colors duration-150 hover:bg-[#19191d] hover:text-zinc-300" href="/dashboard">
                                        <span class="material-symbols-outlined text-[20px]">"settings"</span>
                                        <span>"Settings"</span>
                                    </a>

                                    <div class="mt-6 flex items-center gap-3">
                                        <div class="grid h-9 w-9 place-items-center overflow-hidden rounded-full border border-zinc-700 bg-[#19191d]">{render_avatar()}</div>
                                        <div class="min-w-0">
                                            <p class="mono-text text-[11px] font-bold leading-none text-[#4ae176]">{format!("@{username_for_profile}")}</p>
                                            <p class="mt-1 text-[9px] uppercase tracking-tighter text-zinc-500">{email}</p>
                                        </div>
                                    </div>

                                    <button
                                        type="button"
                                        class="mt-6 flex w-full items-center justify-center rounded border border-red-500/40 bg-red-500/10 px-4 py-2 text-sm font-bold text-red-300 transition-colors hover:bg-red-500/20"
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
                            </aside>

                            <div class="flex min-w-0 flex-1 flex-col bg-surface">
                                <header class="flex h-16 w-full shrink-0 items-center justify-between border-b border-zinc-900 bg-[#0e0e10] px-6">
                                    <div class="flex flex-1 items-center max-w-2xl gap-3">
                                        <button
                                            type="button"
                                            class="inline-flex h-10 w-10 items-center justify-center rounded border border-zinc-800 bg-[#131316] text-zinc-300 transition-colors hover:border-[#4ae176] hover:text-white md:hidden"
                                            aria-label="Open sidebar"
                                            on:click=move |_| set_mobile_sidebar_open.set(true)
                                        >
                                            <span class="material-symbols-outlined text-[20px]">"menu"</span>
                                        </button>
                                        <div class="relative w-full">
                                            <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-sm text-zinc-500">"search"</span>
                                            <input
                                                class="mono-text w-full rounded border-none bg-[#131316] py-2 pl-10 pr-4 text-sm text-zinc-300 placeholder:text-zinc-600 focus:ring-1 focus:ring-[#4ae176]"
                                                placeholder="Search demos or project namespace..."
                                                prop:value=move || dashboard_search_query.get()
                                                on:input=move |ev| set_dashboard_search_query.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>

                                    <div class="ml-6 flex items-center gap-6">
                                        <ThemeModeToggle />
                                        <div class="flex items-center gap-3 border-l border-zinc-800 pl-4">
                                            <div class="text-right">
                                                <p class="mono-text text-[11px] font-bold leading-none text-[#4ae176]">{username_for_profile}</p>
                                                <p class="mt-1 text-[9px] uppercase tracking-tighter text-zinc-500">{"Developer"}</p>
                                            </div>
                                            <div class="h-8 w-8 overflow-hidden rounded-full border border-zinc-700">{render_avatar()}</div>
                                        </div>
                                    </div>
                                </header>

                                <section class="min-w-0 flex-1 overflow-y-auto overflow-x-hidden">
                                    <Outlet />
                                </section>
                            </div>
                        </main>
                    }
                    .into_any()
                }
            }
        }}
    }
}

#[component]
fn ShellSkeleton(on_retry: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <main class="bg-surface text-on-surface flex h-screen overflow-hidden">
            <aside class="hidden w-64 flex-col border-r border-zinc-800 bg-[#131316] md:flex">
                <div class="flex items-center gap-3 px-6 py-5">
                    <span class="material-symbols-outlined text-[#4ae176]" style="font-variation-settings: 'FILL' 1;">"terminal"</span>
                    <div>
                        <h1 class="mono-text text-lg font-bold leading-none tracking-tighter text-[#4ae176]">"SimuCLI"</h1>
                        <p class="label-text mt-1 text-[10px] uppercase tracking-widest text-zinc-500">"CLI Studio"</p>
                    </div>
                </div>
            </aside>
            <section class="flex-1 overflow-y-auto p-8 custom-scrollbar">
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
