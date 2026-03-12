use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;

#[component]
pub fn ProjectsPage() -> impl IntoView {
    let (projects, set_projects) = signal(Vec::<api::DashboardProject>::new());
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        spawn_local({
            let set_projects = set_projects;
            let set_status = set_status;
            async move {
                match api::list_projects().await {
                    Ok(list) => set_projects.set(list),
                    Err(err) => set_status.set(format!("Failed to load projects: {err}")),
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

        spawn_local({
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

    view! {
        <section class="page projects-page">
            <h2>"Projects"</h2>
            <p>"Create project groups and organize your CLI demos."</p>
            <p class="status">{move || status.get()}</p>

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
                            view! {
                                <li>
                                    <div>
                                        <strong>{project.name}</strong>
                                        <p>{project.description.unwrap_or_else(|| "No description".to_string())}</p>
                                    </div>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>

            <a class="action-link" href={api::login_url()}>"Login with GitHub"</a>
            <a class="action-link" href="/dashboard/demos">"Open Demos"</a>
        </section>
    }
}
