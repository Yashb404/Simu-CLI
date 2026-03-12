use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::components::{live_preview::LivePreviewPanel, step_card::StepCard};

#[component]
pub fn DemoEditorPage() -> impl IntoView {
    let params = use_params_map();
    let demo_id = move || {
        params
            .read()
            .get("id")
            .unwrap_or_else(|| "unknown".to_string())
    };

    let (title, set_title) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let id = demo_id();
        if id == "unknown" {
            return;
        }

        spawn_local({
            let set_title = set_title;
            let set_slug = set_slug;
            let set_status = set_status;
            async move {
                match api::get_demo(&id).await {
                    Ok(demo) => {
                        set_title.set(demo.title);
                        set_slug.set(demo.slug.unwrap_or_default());
                    }
                    Err(err) => set_status.set(format!("Failed to load demo: {err}")),
                }
            }
        });
    });

    let save_demo = move |_| {
        let id = demo_id();
        let next_title = title.get();
        let next_slug = slug.get();

        if id == "unknown" {
            set_status.set("Invalid demo id".to_string());
            return;
        }
        if next_title.trim().is_empty() {
            set_status.set("Title is required".to_string());
            return;
        }

        spawn_local({
            let set_status = set_status;
            async move {
                match api::update_demo(
                    &id,
                    Some(next_title.trim()),
                    if next_slug.trim().is_empty() {
                        None
                    } else {
                        Some(next_slug.trim())
                    },
                )
                .await
                {
                    Ok(_) => set_status.set("Saved".to_string()),
                    Err(err) => set_status.set(format!("Save failed: {err}")),
                }
            }
        });
    };

    view! {
        <section class="page demo-editor-page">
            <h2>"Demo Editor"</h2>
            <p>{move || format!("Editing demo: {}", demo_id())}</p>
            <p class="status">{move || status.get()}</p>

            <section class="panel form-grid">
                <label>
                    "Title"
                    <input
                        prop:value=move || title.get()
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                    />
                </label>
                <label>
                    "Slug"
                    <input
                        prop:value=move || slug.get()
                        on:input=move |ev| set_slug.set(event_target_value(&ev))
                    />
                </label>
                <button type="button" on:click=save_demo>"Save Demo"</button>
            </section>

            <div class="editor-grid">
                <section class="step-column">
                    <h3>"Steps"</h3>
                    <StepCard title="Command step" />
                    <StepCard title="Output step" />
                </section>
                <LivePreviewPanel />
            </div>
        </section>
    }
}
