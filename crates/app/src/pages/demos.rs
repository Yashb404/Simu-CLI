use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use shared::client::ClientError;

use crate::api;
use crate::components::confirm_dialog::ConfirmDialog;

#[component]
pub fn DemosPage() -> impl IntoView {
    let (demos, set_demos) = signal(Vec::<api::DashboardDemo>::new());
    let (projects, set_projects) = signal(Vec::<api::DashboardProject>::new());
    let (title, set_title) = signal(String::new());
    let (new_demo_project_id, set_new_demo_project_id) = signal(String::new());
    let (project_filter_id, set_project_filter_id) = signal(String::new());
    let (status, set_status) = signal("Loading demos...".to_string());
    let (requires_login, set_requires_login) = signal(false);
    let (deleting_demo_id, set_deleting_demo_id) = signal(None::<String>);
    let (updating_demo_project_id, set_updating_demo_project_id) = signal(None::<String>);
    let (pending_delete_demo_id, set_pending_delete_demo_id) = signal(None::<String>);
    let (pending_delete_demo_title, set_pending_delete_demo_title) = signal(None::<String>);

    Effect::new(move |_| {
        spawn_local_scoped({
            let set_projects = set_projects;
            async move {
                if let Ok(list) = api::list_projects().await {
                    set_projects.set(list);
                }
            }
        });
    });

    Effect::new(move |_| {
        let active_project_filter = project_filter_id.get();
        spawn_local_scoped({
            let set_demos = set_demos;
            let set_status = set_status;
            let set_requires_login = set_requires_login;
            async move {
                let project_filter = if active_project_filter.trim().is_empty() {
                    None
                } else {
                    Some(active_project_filter.as_str())
                };

                match api::list_demos_with_filters_typed(None, None, project_filter, None).await {
                    Ok(list) => {
                        let count = list.len();
                        set_demos.set(list);
                        set_requires_login.set(false);
                        if count == 0 {
                            set_status
                                .set("No demos yet. Create your first one below.".to_string());
                        } else {
                            set_status.set(format!("Loaded {} demo(s).", count));
                        }
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
                            set_status.set(format!("Failed to load demos: {}", err));
                        }
                    }
                }
            }
        });
    });

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
            let set_demos = set_demos;
            let set_status = set_status;
            let set_title = set_title;
            let project_filter_id = project_filter_id;
            async move {
                match api::create_demo(demo_title.trim(), project_id.as_deref()).await {
                    Ok(demo) => {
                        let filter = project_filter_id.get_untracked();
                        let should_show = filter.trim().is_empty()
                            || demo.project_id.as_deref() == Some(filter.trim());
                        if should_show {
                            set_demos.update(|items| items.insert(0, demo));
                        }
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
            let set_demos = set_demos;
            let set_status = set_status;
            let set_deleting_demo_id = set_deleting_demo_id;
            let set_pending_delete_demo_id = set_pending_delete_demo_id;
            let set_pending_delete_demo_title = set_pending_delete_demo_title;
            async move {
                match api::delete_demo(&id).await {
                    Ok(()) => {
                        set_demos.update(|items| items.retain(|d| d.id != id));
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
            let set_demos = set_demos;
            let set_status = set_status;
            let set_updating_demo_project_id = set_updating_demo_project_id;
            async move {
                match api::update_demo_project(&demo_id, project.as_deref()).await {
                    Ok(updated_demo) => {
                        set_demos.update(|items| {
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
        <section class="page demos-page">
            <h2>"Demos"</h2>
            <p>"Create, edit, publish, and inspect analytics for demos."</p>
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

            <div class="panel">
                <h3>"New Demo"</h3>
                <input
                    placeholder="Demo title"
                    prop:value=move || title.get()
                    on:input=move |ev| set_title.set(event_target_value(&ev))
                />
                <label>
                    "Project"
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
                </label>
                <button type="button" on:click=create_demo>"Create Demo"</button>
            </div>

            <div class="panel">
                <h3>"Your Demos"</h3>
                <div class="inline-actions">
                    <label>
                        "Filter by project"
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
                    </label>
                </div>
                <Show when=move || !demos.get().is_empty() fallback=|| view! {
                    <p class="empty-state">"No demos found yet."</p>
                }>
                <ul class="list">
                    <For
                        each=move || demos.get()
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
                                .and_then(|project_id| {
                                    projects
                                        .get()
                                        .iter()
                                        .find(|project| project.id == *project_id)
                                        .map(|project| project.name.clone())
                                })
                                .unwrap_or_else(|| "Unassigned".to_string());
                            view! {
                                <li>
                                    <div>
                                        <strong>{demo.title}</strong>
                                        <p>
                                            {move || if demo.published { "Published".to_string() } else { "Draft".to_string() }}
                                            " • "
                                            <span class="subtle-badge">{project_name.clone()}</span>
                                        </p>
                                    </div>
                                    <div class="inline-actions">
                                        <label>
                                            "Project"
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
                                        <a href={format!("/dashboard/demos/{}", demo.id)}>"Editor"</a>
                                        <a href={format!("/dashboard/demos/{}/publish", demo.id)}>"Publish"</a>
                                        <a href={format!("/dashboard/demos/{}/analytics", demo.id)}>"Analytics"</a>
                                        <button
                                            type="button"
                                            class="btn-danger"
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
                                </li>
                            }
                        }
                    />
                </ul>
                </Show>
            </div>

            <ConfirmDialog
                open=Signal::derive(move || pending_delete_demo_id.get().is_some())
                title=Signal::derive(move || "Delete Demo".to_string())
                message=Signal::derive(move || {
                    let title = pending_delete_demo_title
                        .get()
                        .unwrap_or_else(|| "this demo".to_string());
                    format!("Delete '{title}'? This action cannot be undone.")
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
