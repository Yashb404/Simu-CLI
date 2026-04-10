use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

use crate::api;
use crate::components::confirm_dialog::ConfirmDialog;

#[component]
pub fn ProjectsPage() -> impl IntoView {
    let (projects, set_projects) = signal(Vec::<api::DashboardProject>::new());
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (status, set_status) = signal("Loading projects...".to_string());
    let (requires_login, set_requires_login) = signal(false);
    let (deleting_project_id, set_deleting_project_id) = signal(None::<String>);
    let (pending_delete_project_id, set_pending_delete_project_id) = signal(None::<String>);
    let (pending_delete_project_name, set_pending_delete_project_name) = signal(None::<String>);

    Effect::new(move |_| {
        spawn_local_scoped({
            let set_projects = set_projects;
            let set_status = set_status;
            let set_requires_login = set_requires_login;
            async move {
                match api::list_projects().await {
                    Ok(list) => {
                        let count = list.len();
                        set_projects.set(list);
                        set_requires_login.set(false);
                        if count == 0 {
                            set_status.set(
                                "No projects yet. Create your first project below.".to_string(),
                            );
                        } else {
                            set_status.set(format!("Loaded {} project(s).", count));
                        }
                    }
                    Err(err) => {
                        let unauthorized = err.contains("Not logged in");
                        set_requires_login.set(unauthorized);
                        if unauthorized {
                            set_status.set(
                                "You are not logged in. Sign in with GitHub to view projects."
                                    .to_string(),
                            );
                        } else {
                            set_status.set(format!("Failed to load projects: {err}"));
                        }
                    }
                }
            }
        });
    });

    let create_project = move |_| {
        let project_name = name.get();
        let project_description = description.get();
        if project_name.trim().is_empty() {
            set_status.set("Project name is required".to_string());
            return;
        }

        spawn_local_scoped({
            let set_projects = set_projects;
            let set_status = set_status;
            let set_name = set_name;
            let set_description = set_description;
            async move {
                match api::create_project(
                    project_name.trim(),
                    if project_description.trim().is_empty() {
                        None
                    } else {
                        Some(project_description.trim())
                    },
                )
                .await
                {
                    Ok(project) => {
                        set_projects.update(|items| items.insert(0, project));
                        set_name.set(String::new());
                        set_description.set(String::new());
                        set_status.set("Project created".to_string());
                    }
                    Err(err) => set_status.set(format!("Create failed: {err}")),
                }
            }
        });
    };

    let delete_project = move |project_id: String| {
        set_deleting_project_id.set(Some(project_id.clone()));

        spawn_local_scoped({
            let set_projects = set_projects;
            let set_status = set_status;
            let set_deleting_project_id = set_deleting_project_id;
            async move {
                match api::delete_project(&project_id).await {
                    Ok(()) => {
                        set_projects
                            .update(|items| items.retain(|project| project.id != project_id));
                        set_status.set("Project deleted.".to_string());
                    }
                    Err(err) => set_status.set(format!("Delete failed: {err}")),
                }
                set_deleting_project_id.set(None);
                set_pending_delete_project_id.set(None);
                set_pending_delete_project_name.set(None);
            }
        });
    };

    view! {
        <section class="page projects-page">
            <h2>"Projects"</h2>
            <p>"Create project groups and organize your CLI demos."</p>
            <p class="status">{move || status.get()}</p>

            <Show when=move || requires_login.get()>
                <div class="panel">
                    <h3>"Authentication Required"</h3>
                    <p>"Sign in to load and create projects."</p>
                    <a class="button btn-primary" href={api::login_url()}>
                        "Login with GitHub"
                    </a>
                </div>
            </Show>

            <div class="panel">
                <h3>"New Project"</h3>
                <input
                    placeholder="Project name"
                    prop:value=move || name.get()
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                />
                <textarea
                    placeholder="Description (optional)"
                    prop:value=move || description.get()
                    on:input=move |ev| set_description.set(event_target_value(&ev))
                />
                <button type="button" on:click=create_project>"Create Project"</button>
            </div>

            <div class="panel">
                <h3>"Your Projects"</h3>
                <ul class="list">
                    <For
                        each=move || projects.get()
                        key=|project| project.id.clone()
                        children=move |project| {
                            let row_project_id = project.id.clone();
                            let row_project_name = project.name.clone();
                            let disabled_project_id = row_project_id.clone();
                            let status_project_id = row_project_id.clone();
                            let click_project_id = row_project_id.clone();
                            view! {
                                <li>
                                    <div>
                                        <strong>{project.name}</strong>
                                        <p>{project.description.unwrap_or_else(|| "No description".to_string())}</p>
                                    </div>
                                    <div class="inline-actions">
                                        <button
                                            type="button"
                                            class="btn-danger"
                                            disabled=move || deleting_project_id.get().as_deref() == Some(disabled_project_id.as_str())
                                            on:click=move |_| {
                                                set_pending_delete_project_id.set(Some(click_project_id.clone()));
                                                set_pending_delete_project_name.set(Some(row_project_name.clone()));
                                            }
                                        >
                                            {move || {
                                                if deleting_project_id.get().as_deref() == Some(status_project_id.as_str()) {
                                                    "Deleting..."
                                                } else {
                                                    "Delete"
                                                }
                                            }}
                                        </button>
                                    </div>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>

            <div class="inline-actions">
                <a class="button" href="/dashboard/demos">"Open Demos"</a>
            </div>

            <ConfirmDialog
                open=Signal::derive(move || pending_delete_project_id.get().is_some())
                title=Signal::derive(move || "Delete Project".to_string())
                message=Signal::derive(move || {
                    let name = pending_delete_project_name
                        .get()
                        .unwrap_or_else(|| "this project".to_string());
                    format!(
                        "Delete '{name}' and all linked demos? This action cannot be undone."
                    )
                })
                confirm_label="Delete Project"
                processing_label="Deleting..."
                cancel_label="Cancel"
                is_processing=Signal::derive(move || deleting_project_id.get().is_some())
                on_confirm=Callback::new(move |_| {
                    if let Some(project_id) = pending_delete_project_id.get_untracked() {
                        delete_project(project_id);
                    }
                })
                on_cancel=Callback::new(move |_| {
                    if deleting_project_id.get_untracked().is_none() {
                        set_pending_delete_project_id.set(None);
                        set_pending_delete_project_name.set(None);
                    }
                })
            />
        </section>
    }
}
