use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::components::embed_code_generator::EmbedCodeGenerator;

#[component]
pub fn PublishPage() -> impl IntoView {
    let params = use_params_map();
    let demo_id = move || {
        params
            .read()
            .get("id")
            .unwrap_or_else(|| "unknown".to_string())
    };

    let (public_url, set_public_url) = signal(String::new());
    let (published_slug, set_published_slug) = signal(String::new());
    let (status, set_status) = signal("Ready to publish".to_string());

    let on_publish = move |_| {
        let id = demo_id();
        if id == "unknown" {
            set_status.set("Invalid demo id".to_string());
            return;
        }

        spawn_local_scoped({
            let set_public_url = set_public_url;
            let set_published_slug = set_published_slug;
            let set_status = set_status;
            async move {
                match api::publish_demo(&id).await {
                    Ok(result) => {
                        set_public_url.set(result.public_url);
                        set_published_slug.set(result.slug);
                        set_status.set("Demo published".to_string());
                    }
                    Err(err) => set_status.set(format!("Publish failed: {err}")),
                }
            }
        });
    };

    view! {
        <section class="page publish-page">
            <h2>"Publish"</h2>
            <p>"Publish demo and copy the embed snippet."</p>
            <p class="status">{move || status.get()}</p>
            <button type="button" on:click=on_publish>"Publish Demo"</button>

            <Show
                when=move || !public_url.get().is_empty() && !published_slug.get().is_empty()
                fallback=|| view! { <p class="muted">"Publish to generate share and embed snippets."</p> }
            >
                {move || {
                    view! {
                        <EmbedCodeGenerator
                            demo_url=public_url.get()
                            script_url=format!("{}/static/embed.js", api::browser_origin())
                            demo_id=published_slug.get()
                        />
                    }
                }}
            </Show>
        </section>
    }
}
