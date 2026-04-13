use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ShareDemoPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || {
        params
            .read()
            .get("slug")
            .unwrap_or_else(|| "unknown".to_string())
    };
    let embed_src = move || {
        format!("/embed/{}", slug())
    };

    view! {
        <section class="page demo-share-page" style="min-height:100vh;padding:24px;background:#0e0e10;color:#e7e4ec;">
            <div style="max-width:1200px;margin:0 auto;display:grid;gap:18px;">
                <header style="display:flex;flex-wrap:wrap;align-items:end;justify-content:space-between;gap:12px;">
                    <div>
                        <p style="margin:0 0 8px;font-size:12px;font-weight:700;letter-spacing:0.18em;text-transform:uppercase;color:#4ae176;">"Shared Demo"</p>
                        <h2 style="margin:0;font-size:clamp(2rem,4vw,3.5rem);line-height:0.95;letter-spacing:-0.05em;">"Terminal render"</h2>
                        <p style="margin:10px 0 0;color:#acaab1;">{move || format!("Public slug: {}", slug())}</p>
                    </div>
                    <a
                        href="/"
                        style="display:inline-flex;align-items:center;gap:8px;border:1px solid rgba(255,255,255,0.12);border-radius:999px;padding:10px 14px;color:#e7e4ec;text-decoration:none;background:rgba(255,255,255,0.03);"
                    >
                        "Back to home"
                    </a>
                </header>

                <div style="overflow:hidden;border:1px solid rgba(255,255,255,0.08);border-radius:20px;background:#11151b;box-shadow:0 30px 80px rgba(0,0,0,0.35);">
                    <iframe
                        src=embed_src
                        title="Shared terminal demo"
                        referrerpolicy="no-referrer"
                        style="display:block;width:100%;height:min(78vh,760px);border:0;background:#000;"
                    />
                </div>
            </div>
        </section>
    }
}
