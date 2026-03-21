use leptos::prelude::*;
use leptos_router::components::Redirect;

use crate::api;
use crate::auth::{SessionState, use_auth_context};

#[component]
pub fn LandingPage() -> impl IntoView {
    let auth = use_auth_context();

    move || match auth.session_state.get() {
        SessionState::LoggedIn(_) => view! { <Redirect path="/dashboard" /> }.into_any(),
        SessionState::Error(message) => view! { <MarketingView auth_error=Some(message) /> }.into_any(),
        SessionState::Checking | SessionState::LoggedOut => {
            view! { <MarketingView auth_error=None /> }.into_any()
        }
    }
}

#[component]
fn MarketingView(auth_error: Option<String>) -> impl IntoView {
    let auth = use_auth_context();

    view! {
        <div class="landing-shell">
            <header class="landing-header">
                <div class="landing-brand">
                    <span class="landing-prompt">">_"</span>
                    <span>"CLI Demo Studio"</span>
                </div>
                <a
                    class="landing-login"
                    href={api::login_url()}
                    on:click=move |_| auth.set_logging_in.set(true)
                >
                    {move || {
                        if auth.is_logging_in.get() {
                            "Redirecting to GitHub..."
                        } else {
                            "Login with GitHub"
                        }
                    }}
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
                        <a
                            class="landing-cta"
                            href={api::login_url()}
                            on:click=move |_| auth.set_logging_in.set(true)
                        >
                            {move || {
                                if auth.is_logging_in.get() {
                                    "Connecting to GitHub..."
                                } else {
                                    "Start Building"
                                }
                            }}
                        </a>
                    </div>

                    {move || {
                        auth_error
                            .as_ref()
                            .map(|message| {
                                view! {
                                    <p class="status" role="status" aria-live="polite">
                                        {format!("Login failed: {message}")}
                                    </p>
                                }
                            })
                    }}
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

