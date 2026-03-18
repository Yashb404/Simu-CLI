use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;

#[component]
pub fn DemosPage() -> impl IntoView {
    let (demos, set_demos) = signal(Vec::<api::DashboardDemo>::new());
    let (title, set_title) = signal(String::new());
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        spawn_local({
            let set_demos = set_demos;
            let set_status = set_status;
            async move {
                match api::list_demos().await {
                    Ok(list) => set_demos.set(list),
                    Err(err) => set_status.set(format!("Failed to load demos: {err}")),
                }
            }
        });
    });

    let create_demo = move |_| {
        let demo_title = title.get();
        if demo_title.trim().is_empty() {
            set_status.set("Demo title is required".to_string());
            return;
        }

        spawn_local({
            let set_demos = set_demos;
            let set_status = set_status;
            let set_title = set_title;
            async move {
                match api::create_demo(demo_title.trim(), None).await {
                    Ok(demo) => {
                        set_demos.update(|items| items.insert(0, demo));
                        set_title.set(String::new());
                        set_status.set("Demo created".to_string());
                    }
                    Err(err) => set_status.set(format!("Create failed: {err}")),
                }
            }
        });
    };

    let delete_demo = move |id: String| {
        spawn_local({
            let set_demos = set_demos;
            let set_status = set_status;
            async move {
                match api::delete_demo(&id).await {
                    Ok(()) => {
                        set_demos.update(|items| items.retain(|d| d.id != id));
                        set_status.set("Demo deleted".to_string());
                    }
                    Err(err) => set_status.set(format!("Delete failed: {err}")),
                }
            }
        });
    };

    view! {
        <section class="page demos-page">
            <h2>"Demos"</h2>
            <p>"Create, edit, publish, and inspect analytics for demos."</p>
            <p class="status">{move || status.get()}</p>

            <div class="panel">
                <h3>"New Demo"</h3>
                <input
                    placeholder="Demo title"
                    prop:value=move || title.get()
                    on:input=move |ev| set_title.set(event_target_value(&ev))
                />
                <button type="button" on:click=create_demo>"Create Demo"</button>
            </div>

            <div class="panel">
                <h3>"Your Demos"</h3>
                <ul class="list">
                    <For
                        each=move || demos.get()
                        key=|demo| demo.id.clone()
                        children=move |demo| {
                            view! {
                                <li>
                                    <div>
                                        <strong>{demo.title}</strong>
                                        <p>{move || if demo.published { "Published".to_string() } else { "Draft".to_string() }}</p>
                                    </div>
                                    <div class="inline-actions">
                                        <a href={format!("/dashboard/demos/{}", demo.id)}>"Editor"</a>
                                        <a href={format!("/dashboard/demos/{}/publish", demo.id)}>"Publish"</a>
                                        <a href={format!("/dashboard/demos/{}/analytics", demo.id)}>"Analytics"</a>
                                        <button type="button" on:click=move |_| delete_demo(demo.id.clone())>
                                            "Delete"
                                        </button>
                                    </div>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
        </section>
    }
}
