use leptos::prelude::*;
use leptos_router::components::Redirect;

use crate::api;
use crate::auth::{SessionState, use_auth_context};

#[component]
pub fn LandingPage() -> impl IntoView {
    let auth = use_auth_context();

    move || match auth.session_state.get() {
        SessionState::LoggedIn(_) => view! { <Redirect path="/projects" /> }.into_any(),
        SessionState::Checking => view! { <LandingSkeleton /> }.into_any(),
        SessionState::LoggedOut | SessionState::Error(_) => view! { <MarketingView /> }.into_any(),
    }
}

#[component]
fn MarketingView() -> impl IntoView {
    view! {
        <div class="landing-shell">
            <header class="landing-header">
                <div class="landing-brand">
                    <span class="landing-prompt">">_"</span>
                    <span>"CLI Demo Studio"</span>
                </div>
                <a class="landing-login" href={api::login_url()}>
                    "Login with GitHub"
                </a>
            </header>

            <main class="landing-main">
                <section class="landing-copy">
                    <p class="landing-kicker">"Terminal Simulations for Product Teams"</p>
                    <h1>
                        "Build shareable CLI demos with production-grade control."
                    </h1>
                    <p class="landing-description">
                        "Create scripted terminal flows with prompts, pauses, and playback analytics, then ship them with a single embed snippet."
                    </p>

                    <ul class="landing-points">
                        <li>"Script commands and outputs with precise step control"</li>
                        <li>"Publish and embed with script or iframe delivery"</li>
                        <li>"Track replay performance with built-in analytics"</li>
                    </ul>

                    <div class="landing-actions">
                        <a class="landing-cta" href={api::login_url()}>
                            "Start Building"
                        </a>
                    </div>
                </section>

                <section class="landing-preview" aria-label="Terminal preview">
                    <div class="landing-preview-topbar">
                        <span class="dot dot-danger"></span>
                        <span class="dot dot-amber"></span>
                        <span class="dot dot-ink"></span>
                        <span class="landing-preview-title">"preview/demo.sh"</span>
                    </div>
                    <div class="landing-preview-body">
                        <p><span class="prompt">"guest@studio:~$ "</span>"npm create cli-demo"</p>
                        <p class="preview-dim">"Bootstrapping scenario..."</p>
                        <p class="preview-dim">"Loaded 6 scripted steps"</p>
                        <p class="preview-ok">"Ready to publish"</p>
                        <p><span class="prompt">"guest@studio:~$ "</span><span class="cursor"></span></p>
                    </div>
                </section>
            </main>

            <footer class="landing-footer">"2026 CLI Demo Studio"</footer>
        </div>
    }
}

#[component]
fn LandingSkeleton() -> impl IntoView {
    view! {
        <div class="landing-skeleton">
            <p>"Booting terminal..."</p>
        </div>
    }
}
