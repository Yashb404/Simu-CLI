use std::collections::HashMap;

use leptos::prelude::*;
use leptos_meta::Title;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_params_map;
use shared::client::ClientError;
use time::OffsetDateTime;

use crate::api;
use crate::app::{ThemeController, ThemeMode};
use crate::auth::{SessionState, use_auth_context};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::shell::DashboardSearchContext;

fn format_timestamp(value: &OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| value.to_string())
}

fn format_count_label(total: usize, label: &str) -> String {
    let suffix = if total == 1 { "" } else { "s" };
    format!("{total} {label}{suffix}")
}

#[component]
pub fn DemosPage() -> impl IntoView {
    let params = use_params_map();
    let auth = use_auth_context();

    let (all_demos, set_all_demos) = signal(Vec::<api::DashboardDemo>::new());
    let (projects, set_projects) = signal(Vec::<api::DashboardProject>::new());
    let (demo_title, set_demo_title) = signal(String::new());
    let (new_demo_project_id, set_new_demo_project_id) = signal(String::new());
    let (project_name, set_project_name) = signal(String::new());
    let (project_description, set_project_description) = signal(String::new());
    let (project_filter_id, set_project_filter_id) = signal(String::new());
    let (search_query, _set_search_query) = use_context::<DashboardSearchContext>()
        .map(|context| (context.search_query, context.set_search_query))
        .unwrap_or_else(|| signal(String::new()));
    let (published_filter, set_published_filter) = signal("all".to_string());
    let (status, set_status) = signal("Loading dashboard...".to_string());
    let (requires_login, set_requires_login) = signal(false);
    let (deleting_demo_id, set_deleting_demo_id) = signal(None::<String>);
    let (updating_demo_project_id, set_updating_demo_project_id) = signal(None::<String>);
    let (creating_project, set_creating_project) = signal(false);
    let (pending_delete_demo_id, set_pending_delete_demo_id) = signal(None::<String>);
    let (pending_delete_demo_title, set_pending_delete_demo_title) = signal(None::<String>);
    let (is_loading, set_is_loading) = signal(true);
    let (load_nonce, set_load_nonce) = signal(0u32);

    let active_project_slug = Signal::derive(move || {
        params
            .with(|map| map.get("slug"))
            .unwrap_or_default()
            .trim()
            .to_string()
    });

    let username_slug = Signal::derive(move || match auth.session_state.get() {
        SessionState::LoggedIn(user) => api::slugify_segment(&user.username),
        _ => "user".to_string(),
    });

    let active_project_name = Signal::derive(move || {
        let route_slug = active_project_slug.get();
        if route_slug.is_empty() {
            return None;
        }

        projects
            .get()
            .into_iter()
            .find(|project| api::slugify_segment(&project.name) == route_slug)
            .map(|project| project.name)
    });

    let project_lookup = Signal::derive(move || {
        projects
            .get()
            .into_iter()
            .map(|project| (project.id, project.name))
            .collect::<HashMap<_, _>>()
    });

    let filtered_demos = Signal::derive(move || {
        let project_filter = project_filter_id.get();
        let query = search_query.get().trim().to_ascii_lowercase();
        let published = published_filter.get();
        let project_lookup = project_lookup.get();

        all_demos
            .get()
            .into_iter()
            .filter(|demo| {
                if !project_filter.trim().is_empty()
                    && demo.project_id.as_deref() != Some(project_filter.trim())
                {
                    return false;
                }

                if published == "published" && !demo.published {
                    return false;
                }
                if published == "draft" && demo.published {
                    return false;
                }

                if query.is_empty() {
                    return true;
                }

                let title_match = demo.title.to_ascii_lowercase().contains(&query);
                let slug_match = demo
                    .slug
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&query);
                let project_name_match = demo
                    .project_id
                    .as_ref()
                    .and_then(|project_id| project_lookup.get(project_id))
                    .map(|name| name.to_ascii_lowercase().contains(&query))
                    .unwrap_or(false);
                let project_slug_match = demo
                    .project_id
                    .as_ref()
                    .and_then(|project_id| project_lookup.get(project_id))
                    .map(|name| api::slugify_segment(name).contains(&query))
                    .unwrap_or(false);
                title_match || slug_match || project_name_match || project_slug_match
            })
            .collect::<Vec<_>>()
    });

    let dashboard_path = Signal::derive(move || {
        let username = username_slug.get();
        let selected_project_id = project_filter_id.get();

        if selected_project_id.trim().is_empty() {
            if let Some(project_name) = active_project_name.get() {
                format!("/{username}/projects/{}", api::slugify_segment(&project_name))
            } else {
                format!("/{username}/dashboard/projects/all")
            }
        } else {
            let project_name = projects
                .get()
                .into_iter()
                .find(|project| project.id == selected_project_id)
                .map(|project| project.name)
                .unwrap_or_else(|| "all".to_string());
            format!("/{username}/projects/{}", api::slugify_segment(&project_name))
        }
    });

    let dashboard_counts = Signal::derive(move || {
        let demos = all_demos.get();
        let projects = projects.get();
        let published = demos.iter().filter(|demo| demo.published).count();
        let drafts = demos.len().saturating_sub(published);

        (demos.len(), published, drafts, projects.len())
    });

    Effect::new(move |_| {
        let _ = load_nonce.get();
        spawn_local_scoped({
            let set_projects = set_projects;
            let set_all_demos = set_all_demos;
            let set_status = set_status;
            let set_requires_login = set_requires_login;
            let set_is_loading = set_is_loading;
            async move {
                set_is_loading.set(true);
                match api::get_dashboard_snapshot().await {
                    Ok(snapshot) => {
                        let project_count = snapshot.projects.len();
                        let demo_count = snapshot.demos.len();
                        set_projects.set(snapshot.projects);
                        set_all_demos.set(snapshot.demos);
                        set_requires_login.set(false);
                        set_status.set(format!(
                            "Loaded {demo_count} demo(s) across {project_count} project(s)."
                        ));
                    }
                    Err(err) => {
                        let unauthorized = matches!(err, ClientError::Unauthorized);
                        set_requires_login.set(unauthorized);
                        if unauthorized {
                            set_status.set(
                                "You are not logged in. Sign in with GitHub to view demos."
                                    .to_string(),
                            );
                        } else {
                            set_status.set(format!("Failed to load dashboard: {err}"));
                        }
                    }
                }
                set_is_loading.set(false);
            }
        });
    });

    Effect::new(move |_| {
        let route_slug = active_project_slug.get();
        if route_slug.is_empty() {
            set_project_filter_id.set(String::new());
            return;
        }

        if let Some(project) = projects
            .get()
            .into_iter()
            .find(|p| api::slugify_segment(&p.name) == route_slug)
        {
            set_project_filter_id.set(project.id);
        } else {
            set_project_filter_id.set(String::new());
        }
    });

    let refresh_dashboard = move |_| {
        set_load_nonce.update(|value| *value += 1);
    };

    let create_demo = move |_| {
        let demo_title = demo_title.get();
        let selected_project = new_demo_project_id.get();

        if demo_title.trim().is_empty() {
            set_status.set("Demo title is required".to_string());
            return;
        }

        let project_id = if selected_project.trim().is_empty() {
            None
        } else {
            Some(selected_project)
        };

        spawn_local_scoped({
            let set_all_demos = set_all_demos;
            let set_status = set_status;
            let set_demo_title = set_demo_title;
            async move {
                match api::create_demo(demo_title.trim(), project_id.as_deref()).await {
                    Ok(demo) => {
                        set_all_demos.update(|items| items.insert(0, demo));
                        set_demo_title.set(String::new());
                        set_status.set("Demo created.".to_string());
                    }
                    Err(err) => set_status.set(format!("Create failed: {err}")),
                }
            }
        });
    };

    let create_project = move |_| {
        let name = project_name.get();
        let description = project_description.get();

        if name.trim().is_empty() {
            set_status.set("Project name is required".to_string());
            return;
        }

        set_creating_project.set(true);
        spawn_local_scoped({
            let set_projects = set_projects;
            let set_status = set_status;
            let set_project_name = set_project_name;
            let set_project_description = set_project_description;
            let set_new_demo_project_id = set_new_demo_project_id;
            let set_creating_project = set_creating_project;
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
                        set_projects.update(|items| items.insert(0, project.clone()));
                        set_new_demo_project_id.set(project.id.clone());
                        set_project_name.set(String::new());
                        set_project_description.set(String::new());
                        set_status.set("Project created.".to_string());
                    }
                    Err(err) => set_status.set(format!("Create failed: {err}")),
                }
                set_creating_project.set(false);
            }
        });
    };

    let delete_demo = move |id: String| {
        set_deleting_demo_id.set(Some(id.clone()));
        spawn_local_scoped({
            let set_all_demos = set_all_demos;
            let set_status = set_status;
            let set_deleting_demo_id = set_deleting_demo_id;
            let set_pending_delete_demo_id = set_pending_delete_demo_id;
            let set_pending_delete_demo_title = set_pending_delete_demo_title;
            async move {
                match api::delete_demo(&id).await {
                    Ok(()) => {
                        set_all_demos.update(|items| items.retain(|d| d.id != id));
                        set_status.set("Demo deleted.".to_string());
                    }
                    Err(err) => set_status.set(format!("Delete failed: {err}")),
                }
                set_deleting_demo_id.set(None);
                set_pending_delete_demo_id.set(None);
                set_pending_delete_demo_title.set(None);
            }
        });
    };

    let change_demo_project = move |demo_id: String, new_project_id: String| {
        set_updating_demo_project_id.set(Some(demo_id.clone()));
        let project = if new_project_id.trim().is_empty() {
            None
        } else {
            Some(new_project_id)
        };

        spawn_local_scoped({
            let set_all_demos = set_all_demos;
            let set_status = set_status;
            let set_updating_demo_project_id = set_updating_demo_project_id;
            async move {
                match api::update_demo_project(&demo_id, project.as_deref()).await {
                    Ok(updated_demo) => {
                        set_all_demos.update(|items| {
                            if let Some(existing) =
                                items.iter_mut().find(|item| item.id == updated_demo.id)
                            {
                                *existing = updated_demo;
                            }
                        });
                        set_status.set("Demo project updated.".to_string());
                    }
                    Err(err) => set_status.set(format!("Project update failed: {err}")),
                }
                set_updating_demo_project_id.set(None);
            }
        });
    };

    view! {
        <Title text="SimuCLI Dashboard | Precision Engine" />

        // ── Full-bleed shell ──────────────────────────────────────────────────
        <div class="db-shell">

            // ── Sidebar ───────────────────────────────────────────────────────
            <aside class="db-sidebar">
                <div class="db-sidebar-logo">
                    <div class="db-logo-row">
                        // Terminal icon (inline SVG — no external dep)
                        <svg class="db-logo-ico" viewBox="0 0 20 20" fill="currentColor"
                             width="18" height="18">
                            <path d="M2 4a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2
                                     2H4a2 2 0 0 1-2-2V4zm3 3 3.5 3L5 13V7zm4.5 5h5"/>
                        </svg>
                        <div>
                            <div class="db-logo-name">"SimuCLI"</div>
                            <div class="db-logo-sub">"Precision Engine"</div>
                        </div>
                    </div>
                </div>

                <button type="button" class="db-new-project-btn">
                    "New Project"
                    <span>"+"</span>
                </button>

                <nav class="db-nav">
                    <p class="db-nav-label">"Main"</p>
                    <a class="db-nav-item db-nav-item--active" href="#">
                        <svg viewBox="0 0 20 20" width="15" height="15"
                             fill="currentColor">
                            <path d="M2 4a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2
                                     2H4a2 2 0 0 1-2-2V4zm3 3 3.5 3L5 13V7zm4.5 5h5"/>
                        </svg>
                        "Workspace"
                    </a>
                    <a class="db-nav-item" href="#">
                        <svg viewBox="0 0 20 20" width="15" height="15"
                             fill="none" stroke="currentColor" stroke-width="1.5">
                            <rect x="3" y="3" width="6" height="6" rx="1"/>
                            <rect x="11" y="3" width="6" height="6" rx="1"/>
                            <rect x="3" y="11" width="6" height="6" rx="1"/>
                            <rect x="11" y="11" width="6" height="6" rx="1"/>
                        </svg>
                        "Library"
                    </a>
                    <a class="db-nav-item" href="#">
                        <svg viewBox="0 0 20 20" width="15" height="15"
                             fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M3 14l4-4 3 3 4-5 3 3"/>
                        </svg>
                        "Analytics"
                    </a>

                    <p class="db-nav-label">"Projects"</p>
                    <For
                        each=move || projects.get()
                        key=|p| p.id.clone()
                        children=move |p| {
                            let slug = api::slugify_segment(&p.name);
                            let href = format!("/{}/projects/{}", username_slug.get(), slug);
                            view! {
                                <a class="db-nav-item" href={href}>
                                    <span class="db-project-dot"></span>
                                    <span class="db-project-name">{p.name}</span>
                                </a>
                            }
                        }
                    />
                </nav>

                <div class="db-sidebar-footer">
                    <a class="db-nav-item" href="#">
                        <svg viewBox="0 0 20 20" width="15" height="15"
                             fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="10" cy="10" r="3"/>
                            <path d="M10 2v2M10 16v2M2 10h2M16 10h2
                                     M4.2 4.2l1.4 1.4M14.4 14.4l1.4 1.4
                                     M4.2 15.8l1.4-1.4M14.4 5.6l1.4-1.4"/>
                        </svg>
                        "Settings"
                    </a>
                </div>
            </aside>

            // ── Main column ───────────────────────────────────────────────────
            <div class="db-main">

                // ── Top bar ───────────────────────────────────────────────────
                <header class="db-topbar">
                    <div class="db-search-wrap">
                        <svg class="db-search-ico" viewBox="0 0 16 16" width="13" height="13"
                             fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="7" cy="7" r="4"/>
                            <path d="M10.5 10.5l3 3"/>
                        </svg>
                        // NOTE: wire up to DashboardSearchContext here
                        <input class="db-search-input"
                               placeholder="Search demos or project namespace..."
                               type="search" />
                    </div>
                    <div class="db-topbar-right">
                        <Show when=move || false>
                            <ThemeModeToggle />
                        </Show>
                        <div class="db-profile">
                            <div>
                                <div class="db-profile-name">
                                    {move || username_slug.get()}
                                </div>
                                <div class="db-profile-role">"Admin Access"</div>
                            </div>
                            <div class="db-avatar">
                                {move || {
                                    let u = username_slug.get();
                                    u.chars().next()
                                     .map(|c| c.to_uppercase().to_string())
                                     .unwrap_or_else(|| "U".to_string())
                                }}
                            </div>
                        </div>
                    </div>
                </header>

                // ── Canvas ────────────────────────────────────────────────────
                <main class="db-canvas">

                    // Page header
                    <div class="db-page-header">
                        <div>
                            <h2 class="db-page-title">
                                {move || {
                                    if let Some(name) = active_project_name.get() {
                                        format!("{name} demos")
                                    } else {
                                        "Workspace".to_string()
                                    }
                                }}
                            </h2>
                            <p class="db-page-path">{move || dashboard_path.get()}</p>
                        </div>
                        <div class="db-header-actions">
                            <p class="db-status-inline">
                                {move || status.get()}
                            </p>
                            <button type="button" class="db-btn-primary"
                                    on:click=create_demo>
                                "Create Demo"
                            </button>
                        </div>
                    </div>

                    // Auth wall
                    <Show when=move || requires_login.get()>
                        <div class="db-panel db-auth-panel">
                            <h3>"Authentication Required"</h3>
                            <p>"Sign in to load and manage demos."</p>
                            <a class="db-btn-primary" href={api::login_url()}>
                                "Login with GitHub"
                            </a>
                        </div>
                    </Show>

                    // Stats bar
                    <div class="db-stats-bar">
                        <div class="db-stat">
                            <span class="db-stat-label">"Demos"</span>
                            <strong class="db-stat-val">
                                {move || format_count_label(dashboard_counts.get().0, "demo")}
                            </strong>
                        </div>
                        <div class="db-stat">
                            <span class="db-stat-label">"Published"</span>
                            <strong class="db-stat-val db-stat-val--green">
                                {move || format_count_label(dashboard_counts.get().1, "demo")}
                            </strong>
                        </div>
                        <div class="db-stat">
                            <span class="db-stat-label">"Drafts"</span>
                            <strong class="db-stat-val">
                                {move || format_count_label(dashboard_counts.get().2, "demo")}
                            </strong>
                        </div>
                        <div class="db-stat">
                            <span class="db-stat-label">"Projects"</span>
                            <strong class="db-stat-val">
                                {move || format_count_label(dashboard_counts.get().3, "project")}
                            </strong>
                        </div>
                    </div>

                    // Controls / workbench
                    <section class="db-controls">
                        // Filter row
                        <div class="db-filter-row">
                            <select
                                prop:value=move || project_filter_id.get()
                                on:change=move |ev| set_project_filter_id.set(event_target_value(&ev))
                            >
                                <option value="">"All projects"</option>
                                <For
                                    each=move || projects.get()
                                    key=|p| p.id.clone()
                                    children=move |p| view! {
                                        <option value={p.id.clone()}>{p.name}</option>
                                    }
                                />
                            </select>

                            <select
                                prop:value=move || published_filter.get()
                                on:change=move |ev| set_published_filter.set(event_target_value(&ev))
                            >
                                <option value="all">"All states"</option>
                                <option value="published">"Published"</option>
                                <option value="draft">"Draft"</option>
                            </select>

                            <button type="button" class="db-btn-outline"
                                    on:click=refresh_dashboard>
                                "Reload"
                            </button>
                        </div>

                        // Workbench panels
                        <div class="db-workbench">
                            // Create demo panel
                            <div class="db-wb-panel">
                                <h3 class="db-wb-title">"Create Demo"</h3>
                                <div class="db-wb-form">
                                    <input
                                        placeholder="Name your next demo"
                                        prop:value=move || demo_title.get()
                                        on:input=move |ev| set_demo_title.set(event_target_value(&ev))
                                    />
                                    <Show when=move || !projects.get().is_empty()>
                                        <select
                                            prop:value=move || new_demo_project_id.get()
                                            on:change=move |ev| set_new_demo_project_id.set(event_target_value(&ev))
                                        >
                                            <option value="">"No project"</option>
                                            <For
                                                each=move || projects.get()
                                                key=|p| p.id.clone()
                                                children=move |p| view! {
                                                    <option value={p.id.clone()}>{p.name}</option>
                                                }
                                            />
                                        </select>
                                    </Show>
                                    <button type="button" class="db-btn-primary"
                                            on:click=create_demo>
                                        "Create Demo"
                                    </button>
                                </div>
                                <p class="db-wb-note">
                                    "New demos appear immediately in the grid below."
                                </p>
                            </div>

                            // Create project panel
                            <div class="db-wb-panel">
                                <h3 class="db-wb-title">"New Project"</h3>
                                <div class="db-wb-form db-wb-form--col">
                                    <input
                                        placeholder="Project name"
                                        prop:value=move || project_name.get()
                                        on:input=move |ev| set_project_name.set(event_target_value(&ev))
                                    />
                                    <textarea
                                        placeholder="Description (optional)"
                                        prop:value=move || project_description.get()
                                        on:input=move |ev| set_project_description.set(event_target_value(&ev))
                                    />
                                    <button
                                        type="button"
                                        class="db-btn-outline"
                                        disabled=move || creating_project.get()
                                        on:click=create_project
                                    >
                                        {move || if creating_project.get() {
                                            "Creating..."
                                        } else {
                                            "Create Project"
                                        }}
                                    </button>
                                </div>
                                <p class="db-wb-note">
                                    "Projects are optional grouping labels for demos."
                                </p>
                            </div>
                        </div>
                    </section>

                    // Demo grid
                    <Show
                        when=move || !filtered_demos.get().is_empty()
                        fallback=move || {
                            if is_loading.get() {
                                view! {
                                    <div class="db-demos-grid">
                                        <div class="db-demo-card db-demo-card--skeleton"></div>
                                        <div class="db-demo-card db-demo-card--skeleton"></div>
                                        <div class="db-demo-card db-demo-card--skeleton"></div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="db-empty-state">
                                        <h3>"No matching demos"</h3>
                                        <p>"Create a demo or clear the current filters."</p>
                                    </div>
                                }.into_any()
                            }
                        }
                    >
                        <div class="db-demos-grid">
                            <For
                                each=move || filtered_demos.get()
                                key=|demo| demo.id.clone()
                                children=move |demo| {
                                    let demo_id   = demo.id.clone();
                                    let demo_title = demo.title.clone();
                                    let del_id_a  = demo_id.clone();
                                    let del_id_b  = demo_id.clone();
                                    let sel_id_a  = demo_id.clone();
                                    let sel_id_b  = demo_id.clone();

                                    let project_name = demo.project_id.as_ref()
                                        .and_then(|pid| project_lookup.get().get(pid).cloned())
                                        .unwrap_or_else(|| "Unassigned".to_string());

                                    let project_slug = demo.project_id.as_ref()
                                        .and_then(|pid| project_lookup.get().get(pid).cloned())
                                        .map(|n| api::slugify_segment(&n))
                                        .filter(|s| !s.is_empty());

                                    let editor_path = api::namespaced_demo_path(
                                        &username_slug.get(), &demo.id,
                                        project_slug.as_deref(), None);
                                    let publish_path = api::namespaced_demo_path(
                                        &username_slug.get(), &demo.id,
                                        project_slug.as_deref(), Some("publish"));
                                    let analytics_path = api::namespaced_demo_path(
                                        &username_slug.get(), &demo.id,
                                        project_slug.as_deref(), Some("analytics"));

                                    let is_published = demo.published;
                                    let demo_path = format!(
                                        "/{}/demos/{}", username_slug.get(), demo.id);
                                    let version_steps = format!(
                                        "v{} · {} steps", demo.version, demo.steps.len());
                                    let created = format_timestamp(&demo.created_at);
                                    let updated = format_timestamp(&demo.updated_at);
                                    let current_project_id =
                                        demo.project_id.clone().unwrap_or_default();

                                    view! {
                                        <article class="db-demo-card">
                                            // ── Card top ─────────────────────
                                            <div class="db-card-top">
                                                <div>
                                                    <span class="db-card-kicker">
                                                        "Demo Name"
                                                    </span>
                                                    <h3 class="db-card-title">
                                                        {demo.title.clone()}
                                                    </h3>
                                                </div>
                                                <span class=move || {
                                                    if is_published {
                                                        "db-state-pill db-state-pill--published"
                                                    } else {
                                                        "db-state-pill db-state-pill--draft"
                                                    }
                                                }>
                                                    {if is_published { "Published" } else { "Draft" }}
                                                </span>
                                            </div>

                                            // ── Card body ────────────────────
                                            <div class="db-card-body">
                                                <p class="db-card-path">
                                                    {demo_path}
                                                </p>
                                                <p class="db-card-project">
                                                    "Project: "
                                                    <span class="db-subtle-badge">
                                                        {project_name}
                                                    </span>
                                                </p>
                                                <div class="db-card-meta">
                                                    <span>{format!("Created {created}")}</span>
                                                    <span>{format!("Updated {updated}")}</span>
                                                    <span>{version_steps}</span>
                                                </div>
                                            </div>

                                            // ── Card footer ──────────────────
                                            <div class="db-card-footer">
                                                <label class="db-reassign-label">
                                                    "Reassign project"
                                                    <select
                                                        disabled=move || {
                                                            updating_demo_project_id.get()
                                                                .as_deref() == Some(sel_id_b.as_str())
                                                        }
                                                        prop:value={current_project_id}
                                                        on:change=move |ev| {
                                                            change_demo_project(
                                                                sel_id_a.clone(),
                                                                event_target_value(&ev),
                                                            )
                                                        }
                                                    >
                                                        <option value="">"Unassigned"</option>
                                                        <For
                                                            each=move || projects.get()
                                                            key=|p| p.id.clone()
                                                            children=move |p| view! {
                                                                <option value={p.id.clone()}>
                                                                    {p.name}
                                                                </option>
                                                            }
                                                        />
                                                    </select>
                                                </label>

                                                <div class="db-card-actions">
                                                    <a class="db-btn-primary" href={editor_path}>
                                                        "Open Editor"
                                                    </a>
                                                    <a class="db-btn-outline" href={publish_path}>
                                                        "Publish"
                                                    </a>
                                                    <a class="db-btn-outline" href={analytics_path}>
                                                        "Analytics"
                                                    </a>
                                                    <button
                                                        type="button"
                                                        class="db-btn-danger"
                                                        disabled=move || {
                                                            deleting_demo_id.get().as_deref()
                                                                == Some(del_id_a.as_str())
                                                        }
                                                        on:click=move |_| {
                                                            set_pending_delete_demo_id
                                                                .set(Some(demo_id.clone()));
                                                            set_pending_delete_demo_title
                                                                .set(Some(demo_title.clone()));
                                                        }
                                                    >
                                                        {move || {
                                                            if deleting_demo_id.get().as_deref()
                                                                == Some(del_id_b.as_str())
                                                            {
                                                                "Deleting…"
                                                            } else {
                                                                "Delete"
                                                            }
                                                        }}
                                                    </button>
                                                </div>
                                            </div>
                                        </article>
                                    }
                                }
                            />
                        </div>
                    </Show>

                    // Footer void
                    <div class="db-footer-void">
                        <span class="db-heartbeat-dot"></span>
                        <div>
                            <p class="db-footer-line">
                                {move || {
                                    let (demos, published, _, projs) = dashboard_counts.get();
                                    format!(
                                        "{} | {} | {}",
                                        format_count_label(demos, "demo"),
                                        format_count_label(published, "published demo"),
                                        format_count_label(projs, "project"),
                                    )
                                }}
                            </p>
                        </div>
                    </div>

                </main>
            </div>
        </div>

        // Confirm dialog (unchanged)
        <ConfirmDialog
            open=Signal::derive(move || pending_delete_demo_id.get().is_some())
            title=Signal::derive(move || "Delete Demo".to_string())
            message=Signal::derive(move || {
                let t = pending_delete_demo_title.get()
                    .unwrap_or_else(|| "this demo".to_string());
                format!("Delete '{t}'? This action cannot be undone.")
            })
            confirm_label="Delete Demo"
            processing_label="Deleting…"
            cancel_label="Cancel"
            is_processing=Signal::derive(move || deleting_demo_id.get().is_some())
            on_confirm=Callback::new(move |_| {
                if let Some(id) = pending_delete_demo_id.get_untracked() {
                    delete_demo(id);
                }
            })
            on_cancel=Callback::new(move |_| {
                if deleting_demo_id.get_untracked().is_none() {
                    set_pending_delete_demo_id.set(None);
                    set_pending_delete_demo_title.set(None);
                }
            })
        />
    }
}

#[component]
pub fn ThemeModeToggle() -> impl IntoView {
    let controller = use_context::<ThemeController>();
    let mode = Signal::derive(move || {
        controller
            .as_ref()
            .map(|theme| theme.mode.get())
            .unwrap_or(ThemeMode::Terminal)
    });

    let set_terminal = {
        let controller = controller;
        move |_| {
            if let Some(theme) = controller {
                theme.set_mode.set(ThemeMode::Terminal);
            }
        }
    };

    let set_dark = {
        let controller = controller;
        move |_| {
            if let Some(theme) = controller {
                theme.set_mode.set(ThemeMode::Dark);
            }
        }
    };

    let set_light = move |_| {
        if let Some(theme) = controller {
            theme.set_mode.set(ThemeMode::Light);
        }
    };

    view! {
        <div class="db-theme-toggle" role="group" aria-label="Theme mode">
            <button type="button"
                class=move || if mode.get() == ThemeMode::Terminal {
                    "db-theme-btn db-theme-btn--active"
                } else { "db-theme-btn" }
                aria-label="Terminal theme"
                on:click=set_terminal>
                {terminal_icon()}
            </button>
            <button type="button"
                class=move || if mode.get() == ThemeMode::Dark {
                    "db-theme-btn db-theme-btn--active"
                } else { "db-theme-btn" }
                aria-label="Dark theme"
                on:click=set_dark>
                {moon_icon()}
            </button>
            <button type="button"
                class=move || if mode.get() == ThemeMode::Light {
                    "db-theme-btn db-theme-btn--active"
                } else { "db-theme-btn" }
                aria-label="Light theme"
                on:click=set_light>
                {sun_icon()}
            </button>
        </div>
    }
}

fn terminal_icon() -> impl IntoView {
    view! {
        <svg class="theme-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7">
            <path d="M3 5.5A2.5 2.5 0 0 1 5.5 3h13A2.5 2.5 0 0 1 21 5.5v13A2.5 2.5 0 0 1 18.5 21h-13A2.5 2.5 0 0 1 3 18.5z" stroke-linecap="round" stroke-linejoin="round" />
            <path d="m8.5 9 2.5 2.5L8.5 14" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M12.5 14h3" stroke-linecap="round" />
        </svg>
    }
}

fn moon_icon() -> impl IntoView {
    view! {
        <svg class="theme-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7">
            <path d="M20 14.4A8.6 8.6 0 1 1 9.6 4a7 7 0 0 0 10.4 10.4z" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
    }
}

fn sun_icon() -> impl IntoView {
    view! {
        <svg class="theme-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7">
            <circle cx="12" cy="12" r="3.5" />
            <path d="M12 2.5v2.5M12 19v2.5M4.9 4.9l1.8 1.8M17.3 17.3l1.8 1.8M2.5 12H5M19 12h2.5M4.9 19.1l1.8-1.8M17.3 6.7l1.8-1.8" stroke-linecap="round" />
        </svg>
    }
}
