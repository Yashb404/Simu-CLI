use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::components::embed_code_generator::EmbedCodeGenerator;

/// Renders the "Share" page for a demo, showing publish status, generating a public share link after publishing, and providing an embed code generator.
///
/// The component reads the route `id` param (falls back to "unknown"), attempts to rehydrate published state on mount, and exposes controls to publish the demo, copy or open the generated share link, and insert an embed snippet when available.
///
/// # Examples
///
/// ```
/// // Create the page view and include it in your app's view tree.
/// let share_view = PublishPage();
/// ```
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

    let editor_path = move || format!("/dashboard/demos/{}", demo_id());

    Effect::new(move |_| {
        let id = demo_id();
        if id == "unknown" {
            return;
        }

        spawn_local_scoped({
            let set_public_url = set_public_url;
            let set_published_slug = set_published_slug;
            let set_status = set_status;
            async move {
                if let Ok(detail) = api::get_demo_detail(&id).await {
                    if detail.published
                        && let Some(slug) = detail.slug
                    {
                        set_published_slug.set(slug.clone());
                        set_public_url.set(format!("{}/d/{slug}", api::browser_origin()));
                        set_status.set("Demo already published. Share link is ready.".to_string());
                    }
                }
            }
        });
    });

    let copy_share_link = move |_| {
        let url = public_url.get();
        if url.trim().is_empty() {
            set_status.set("Publish first to generate a share link".to_string());
            return;
        }

        if let Some(window) = web_sys::window() {
            let _ = window.prompt_with_message_and_default("Copy share link", &url);
            set_status.set("Share link ready to copy".to_string());
        }
    };

    let open_share_link = move |_| {
        let url = public_url.get();
        if url.trim().is_empty() {
            set_status.set("Publish first to generate a share link".to_string());
            return;
        }

        if let Some(window) = web_sys::window() {
            let _ =
                window.open_with_url_and_target_and_features(&url, "_blank", "noopener,noreferrer");
        }
    };

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
        <main class="pt-12 pb-20 max-w-5xl mx-auto px-6">
            <header class="mb-12">
                <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
                    <div>
                        <h1 class="text-4xl font-extrabold tracking-tight mb-2">"Share"</h1>
                        <p class="text-on-surface-variant max-w-md">
                            "Share your terminal demo immediately after publishing. Changes are reflected globally across embed instances."
                        </p>
                    </div>
                    <div class="flex flex-col items-end gap-2">
                        <div class="flex items-center gap-2 bg-surface-container-low px-3 py-1.5 rounded border border-outline-variant/20">
                            <span class="w-2 h-2 rounded-full bg-primary animate-pulse"></span>
                            <span class="label text-xs font-medium text-primary uppercase tracking-widest">{move || format!("Status: {}", status.get())}</span>
                        </div>
                    </div>
                </div>
            </header>

            <div class="flex items-center gap-4 mb-8">
                <A href=editor_path attr:class="flex-1 px-6 py-4 rounded-lg font-bold text-sm border border-outline-variant hover:bg-surface-container-low transition-all uppercase tracking-widest text-on-surface text-center">
                    "Back to Editor"
                </A>
                <button
                    type="button"
                    on:click=on_publish
                    class="flex-1 px-6 py-4 rounded-lg font-bold text-sm bg-primary text-on-primary hover:brightness-110 active:scale-[0.98] transition-all uppercase tracking-widest shadow-[0_0_20px_rgba(74,225,118,0.2)]"
                >
                    "Publish Demo"
                </button>
            </div>

            <Show
                when=move || !public_url.get().is_empty() && !published_slug.get().is_empty()
                fallback=|| view! {
                    <div class="p-4 bg-primary-container/10 border-l-2 border-primary">
                        <p class="text-sm font-medium text-on-primary-container flex items-center gap-2">
                            <span class="material-symbols-outlined text-sm">"info"</span>
                            "Publish the demo to generate share and embed options."
                        </p>
                    </div>
                }
            >
                <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
                    <div class="lg:col-span-7 space-y-8">
                        <section class="bg-surface-container border border-outline-variant/30 rounded p-6">
                            <div class="flex items-center justify-between mb-6">
                                <h2 class="label text-sm font-semibold uppercase tracking-wider text-on-surface-variant">"Public Access"</h2>
                                <span class="material-symbols-outlined text-on-surface-variant text-sm">"public"</span>
                            </div>
                            <div class="bg-surface-container-lowest p-4 rounded mb-6 border border-outline-variant/10">
                                <p class="mono text-sm text-on-primary-container truncate">{move || public_url.get()}</p>
                            </div>
                            <div class="flex flex-wrap gap-4">
                                <button
                                    type="button"
                                    class="flex-1 flex items-center justify-center gap-2 bg-surface-container-highest hover:bg-surface-bright text-on-surface py-3 rounded-lg font-bold text-sm transition-colors border border-outline-variant/20"
                                    on:click=copy_share_link
                                >
                                    <span class="material-symbols-outlined text-base">"content_copy"</span>
                                    "Copy Share Link"
                                </button>
                                <button
                                    type="button"
                                    class="flex-1 flex items-center justify-center gap-2 bg-surface-container-highest hover:bg-surface-bright text-on-surface py-3 rounded-lg font-bold text-sm transition-colors border border-outline-variant/20"
                                    on:click=open_share_link
                                >
                                    <span class="material-symbols-outlined text-base">"open_in_new"</span>
                                    "Open Share Link"
                                </button>
                            </div>
                        </section>
                    </div>

                    <div class="lg:col-span-5 space-y-6">
                        <section class="bg-surface-container-low border border-outline-variant/20 rounded-lg p-6">
                            <div class="mb-4 flex items-center justify-between">
                                <span class="label text-[10px] font-bold uppercase tracking-widest text-on-surface-variant">"Embed Code Generator"</span>
                                <span class="material-symbols-outlined text-on-surface-variant text-sm">"code"</span>
                            </div>
                            <EmbedCodeGenerator
                                demo_url=format!(
                                    "{}/embed-runtime/index.html?demo_id={}&api_base={}",
                                    api::api_base(),
                                    published_slug.get(),
                                    api::api_base()
                                )
                                script_url=format!("{}/static/embed.js", api::api_base())
                                demo_id=published_slug.get()
                            />
                        </section>
                    </div>
                </div>
            </Show>
        </main>
    }
}
