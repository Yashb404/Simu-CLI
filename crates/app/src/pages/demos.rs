use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use shared::client::ClientError;

use crate::api;
use crate::app::{ThemeController, ThemeMode};
use crate::components::confirm_dialog::ConfirmDialog;

#[component]
pub fn DemosPage() -> impl IntoView {
    let (all_demos, set_all_demos) = signal(Vec::<api::DashboardDemo>::new());
    let (projects, set_projects) = signal(Vec::<api::DashboardProject>::new());
    let (title, set_title) = signal(String::new());
    let (new_demo_project_id, set_new_demo_project_id) = signal(String::new());
    let (project_filter_id, set_project_filter_id) = signal(String::new());
    let (search_query, set_search_query) = signal(String::new());
    let (published_filter, set_published_filter) = signal("all".to_string());
    let (status, set_status) = signal("Loading demos...".to_string());
    let (requires_login, set_requires_login) = signal(false);
    let (deleting_demo_id, set_deleting_demo_id) = signal(None::<String>);
    let (updating_demo_project_id, set_updating_demo_project_id) = signal(None::<String>);
    let (pending_delete_demo_id, set_pending_delete_demo_id) = signal(None::<String>);
    let (pending_delete_demo_title, set_pending_delete_demo_title) = signal(None::<String>);
    let (is_loading, set_is_loading) = signal(true);
    let (load_nonce, set_load_nonce) = signal(0u32);

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
                title_match || slug_match
            })
            .collect::<Vec<_>>()
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

    let refresh_dashboard = move |_| {
        set_load_nonce.update(|value| *value += 1);
    };

    let create_demo = move |_| {
        let demo_title = title.get();
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
            let set_title = set_title;
            async move {
                match api::create_demo(demo_title.trim(), project_id.as_deref()).await {
                    Ok(demo) => {
                        set_all_demos.update(|items| items.insert(0, demo));
                        set_title.set(String::new());
                        set_status.set("Demo created.".to_string());
                    }
                    Err(err) => set_status.set(format!("Create failed: {err}")),
                }
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
        <section class="page demos-gallery-page">
            <header class="demos-hero">
                <div>
                    <p class="demos-eyebrow">"Command Design Workspace"</p>
                    <h2>"Demos"</h2>
                    <p>
                        "Compose, organize, and publish polished interactive demos without the loading jitter."
                    </p>
                </div>
                <div class="demos-hero-actions">
                    <ThemeModeToggle />
                    <button type="button" class="button btn-outline" on:click=refresh_dashboard>
                        "Refresh"
                    </button>
                </div>
            </header>

            <p class="status">{move || status.get()}</p>

            <Show when=move || requires_login.get()>
                <div class="panel">
                    <h3>"Authentication Required"</h3>
                    <p>"Sign in to load and create demos."</p>
                    <a class="button btn-primary" href={api::login_url()}>
                        "Login with GitHub"
                    </a>
                </div>
            </Show>

            <section class="demos-toolbar panel">
                <div class="demos-create-row">
                    <input
                        placeholder="Name your next demo"
                        prop:value=move || title.get()
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                    />
                    <select
                        prop:value=move || new_demo_project_id.get()
                        on:change=move |ev| set_new_demo_project_id.set(event_target_value(&ev))
                    >
                        <option value="">"No project"</option>
                        <For
                            each=move || projects.get()
                            key=|project| project.id.clone()
                            children=move |project| {
                                view! {
                                    <option value={project.id.clone()}>{project.name}</option>
                                }
                            }
                        />
                    </select>
                    <button type="button" class="button btn-primary" on:click=create_demo>
                        "Create Demo"
                    </button>
                </div>

                <div class="demos-filter-row">
                    <input
                        placeholder="Search by title or slug"
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                    <select
                        prop:value=move || project_filter_id.get()
                        on:change=move |ev| set_project_filter_id.set(event_target_value(&ev))
                    >
                        <option value="">"All projects"</option>
                        <For
                            each=move || projects.get()
                            key=|project| project.id.clone()
                            children=move |project| {
                                view! {
                                    <option value={project.id.clone()}>{project.name}</option>
                                }
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
                </div>
            </section>

            <Show
                when=move || !filtered_demos.get().is_empty()
                fallback=move || {
                    if is_loading.get() {
                        view! {
                            <section class="demos-grid">
                                <article class="demo-card skeleton"></article>
                                <article class="demo-card skeleton"></article>
                                <article class="demo-card skeleton"></article>
                            </section>
                        }
                            .into_any()
                    } else {
                        view! {
                            <p class="empty-state">
                                "No demos match your filters yet. Create one and it will appear here immediately."
                            </p>
                        }
                            .into_any()
                    }
                }
            >
                <section class="demos-grid">
                    <For
                        each=move || filtered_demos.get()
                        key=|demo| demo.id.clone()
                        children=move |demo| {
                            let demo_id = demo.id.clone();
                            let demo_title = demo.title.clone();
                            let deleting_demo_ref_for_disabled = demo_id.clone();
                            let deleting_demo_ref_for_label = demo_id.clone();
                            let project_select_demo_id = demo_id.clone();
                            let project_select_demo_id_for_disabled = demo_id.clone();
                            let project_name = demo
                                .project_id
                                .as_ref()
                                .and_then(|project_id| project_lookup.get().get(project_id).cloned())
                                .unwrap_or_else(|| "Unassigned".to_string());

                            view! {
                                <article class="demo-card">
                                    <div class="demo-card-top">
                                        <h3>{demo.title.clone()}</h3>
                                        <span class=move || {
                                            if demo.published {
                                                "demo-state-pill published"
                                            } else {
                                                "demo-state-pill draft"
                                            }
                                        }>
                                            {move || if demo.published { "Published" } else { "Draft" }}
                                        </span>
                                    </div>

                                    <p class="demo-card-subtitle">
                                        "Project: "
                                        <span class="subtle-badge">{project_name}</span>
                                    </p>

                                    <div class="demo-card-meta">
                                        <span>{format!("Version {}", demo.version)}</span>
                                        <span>{format!("{} steps", demo.steps.len())}</span>
                                    </div>

                                    <label class="demo-project-selector">
                                        "Reassign project"
                                        <select
                                            disabled=move || updating_demo_project_id.get().as_deref() == Some(project_select_demo_id_for_disabled.as_str())
                                            prop:value={demo.project_id.clone().unwrap_or_default()}
                                            on:change=move |ev| {
                                                change_demo_project(
                                                    project_select_demo_id.clone(),
                                                    event_target_value(&ev),
                                                )
                                            }
                                        >
                                            <option value="">"Unassigned"</option>
                                            <For
                                                each=move || projects.get()
                                                key=|project| project.id.clone()
                                                children=move |project| {
                                                    view! {
                                                        <option value={project.id.clone()}>{project.name}</option>
                                                    }
                                                }
                                            />
                                        </select>
                                    </label>

                                    <div class="demo-card-actions">
                                        <a class="button btn-primary" href={format!("/dashboard/demos/{}", demo.id)}>
                                            "Open Editor"
                                        </a>
                                        <a class="button btn-outline" href={format!("/dashboard/demos/{}/publish", demo.id)}>
                                            "Publish"
                                        </a>
                                        <a class="button btn-outline" href={format!("/dashboard/demos/{}/analytics", demo.id)}>
                                            "Analytics"
                                        </a>
                                        <button
                                            type="button"
                                            class="button btn-danger"
                                            disabled=move || deleting_demo_id.get().as_deref() == Some(deleting_demo_ref_for_disabled.as_str())
                                            on:click=move |_| {
                                                set_pending_delete_demo_id.set(Some(demo_id.clone()));
                                                set_pending_delete_demo_title.set(Some(demo_title.clone()));
                                            }
                                        >
                                            {move || {
                                                if deleting_demo_id.get().as_deref() == Some(deleting_demo_ref_for_label.as_str()) {
                                                    "Deleting..."
                                                } else {
                                                    "Delete"
                                                }
                                            }}
                                        </button>
                                    </div>
                                </article>
                            }
                        }
                    />
                </section>
            </Show>

            <ConfirmDialog
                open=Signal::derive(move || pending_delete_demo_id.get().is_some())
                title=Signal::derive(move || "Delete Demo".to_string())
                message=Signal::derive(move || {
                    let pending_title = pending_delete_demo_title
                        .get()
                        .unwrap_or_else(|| "this demo".to_string());
                    format!("Delete '{pending_title}'? This action cannot be undone.")
                })
                confirm_label="Delete Demo"
                processing_label="Deleting..."
                cancel_label="Cancel"
                is_processing=Signal::derive(move || deleting_demo_id.get().is_some())
                on_confirm=Callback::new(move |_| {
                    if let Some(demo_id) = pending_delete_demo_id.get_untracked() {
                        delete_demo(demo_id);
                    }
                })
                on_cancel=Callback::new(move |_| {
                    if deleting_demo_id.get_untracked().is_none() {
                        set_pending_delete_demo_id.set(None);
                        set_pending_delete_demo_title.set(None);
                    }
                })
            />
        </section>
    }
}

#[component]
fn ThemeModeToggle() -> impl IntoView {
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
        <div class="theme-icon-toggle" role="group" aria-label="Theme mode">
            <button
                type="button"
                class=move || {
                    if mode.get() == ThemeMode::Terminal {
                        "theme-icon-btn active"
                    } else {
                        "theme-icon-btn"
                    }
                }
                aria-label="Switch to terminal theme"
                on:click=set_terminal
            >
                {terminal_icon()}
            </button>
            <button
                type="button"
                class=move || {
                    if mode.get() == ThemeMode::Dark {
                        "theme-icon-btn active"
                    } else {
                        "theme-icon-btn"
                    }
                }
                aria-label="Switch to dark theme"
                on:click=set_dark
            >
                {moon_icon()}
            </button>
            <button
                type="button"
                class=move || {
                    if mode.get() == ThemeMode::Light {
                        "theme-icon-btn active"
                    } else {
                        "theme-icon-btn"
                    }
                }
                aria-label="Switch to light theme"
                on:click=set_light
            >
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
