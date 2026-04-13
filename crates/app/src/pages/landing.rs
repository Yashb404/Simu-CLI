use leptos::prelude::*;
use leptos_router::components::{A, Redirect};

use crate::api;
use crate::auth::{SessionState, use_auth_context};

#[component]
pub fn LandingPage() -> impl IntoView {
    let auth = use_auth_context();

    move || match auth.session_state.get() {
        SessionState::LoggedIn(_) => view! { <Redirect path="/dashboard" /> }.into_any(),
        SessionState::Error(message) => {
            view! { <MarketingView auth_error=Some(message) /> }.into_any()
        }
        SessionState::Checking | SessionState::LoggedOut => {
            view! { <MarketingView auth_error=None /> }.into_any()
        }
    }
}

#[component]
fn MarketingView(auth_error: Option<String>) -> impl IntoView {
    let auth = use_auth_context();

    view! {
        <div class="min-h-screen bg-[#0e0e10] text-[#e7e4ec]">
            <header class="fixed top-0 z-50 flex w-full items-center justify-between gap-4 border-b border-[#2b2c32] bg-[#0e0e10] px-6 py-4 text-sm font-medium tracking-tight backdrop-blur">
                <div class="flex items-center gap-3">
                    <span class="mono rounded border border-[#4ae176]/30 px-2 py-1 text-[10px] font-bold uppercase tracking-[0.16em] text-[#4ae176]">
                        ">_"
                    </span>
                    <span class="text-xl font-black tracking-tighter text-[#4ae176]">"SimuCLI"</span>
                </div>

                <nav class="hidden items-center gap-8 md:flex">
                    <a class="border-b-2 border-[#4ae176] pb-1 text-[#4ae176]" href="#features">
                        "Features"
                    </a>
                    <A attr:class="text-[#9f9da1] transition-colors hover:text-[#e7e4ec]" href="/docs">
                        "Docs"
                    </A>
                    <a class="text-[#9f9da1] transition-colors hover:text-[#e7e4ec]" href="#pricing">
                        "Pricing"
                    </a>
                </nav>

                <a
                    class="rounded-lg bg-[#4ae176] px-4 py-2 font-bold tracking-tight text-[#004b1e] transition-all hover:bg-[#38d36a] active:scale-95"
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

            <main class="pt-24">
                <section class="mx-auto flex max-w-7xl flex-col items-center px-6 py-20 text-center">
                    <p class="label mb-4 text-xs uppercase tracking-[0.18em] text-[#4ae176]">
                        "Terminal Simulations for Product Teams"
                    </p>
                    <h1 class="mb-6 max-w-4xl text-5xl font-black tracking-tighter text-[#e7e4ec] md:text-7xl">
                        "Build and embed terminal "
                        <span class="text-[#4ae176]">"simulations"</span>
                        " in minutes."
                    </h1>
                    <p class="mb-10 max-w-2xl text-lg leading-relaxed text-[#acaab1] md:text-xl">
                        "A stateless, client-side runtime for creating interactive CLI demos. Built in Rust, powered by WebAssembly."
                    </p>
                    <div class="mb-20 flex flex-col gap-4 sm:flex-row">
                        <a
                            class="rounded bg-[#4ae176] px-8 py-4 font-bold text-[#004b1e] transition-all hover:bg-[#38d36a]"
                            href={api::login_url()}
                            on:click=move |_| auth.set_logging_in.set(true)
                        >
                            {move || {
                                if auth.is_logging_in.get() {
                                    "Connecting to GitHub..."
                                } else {
                                    "Get Started for Free"
                                }
                            }}
                        </a>
                        <A
                            attr:class="rounded border border-[#47474e] bg-[#25252b] px-8 py-4 font-bold transition-all hover:bg-[#2b2c32]"
                            href="/docs"
                        >
                            "View Documentation"
                        </A>
                    </div>

                    {move || {
                        auth_error.as_ref().map(|message| {
                            view! {
                                <p class="max-w-2xl text-sm text-[#ed7f64]" role="status" aria-live="polite">
                                    {format!("Login failed: {message}")}
                                </p>
                            }
                        })
                    }}

                    <div class="terminal-glow w-full max-w-5xl overflow-hidden border border-[#47474e] bg-[#000000] rounded-xl">
                        <div class="flex items-center gap-2 bg-[#1f1f24] px-4 py-2">
                            <div class="flex gap-1.5">
                                <div class="h-3 w-3 rounded-full bg-[#ed7f64]/30"></div>
                                <div class="h-3 w-3 rounded-full bg-[#3b3b3e]"></div>
                                <div class="h-3 w-3 rounded-full bg-[#4ae176]/30"></div>
                            </div>
                            <div class="label mx-auto text-xs text-[#acaab1]">"simucli --demo deploy-api"</div>
                        </div>
                        <div class="mono min-h-[400px] bg-[#131316] p-8 text-left text-sm leading-relaxed md:text-base">
                            <div class="mb-2 flex gap-3">
                                <span class="text-[#4ae176]">"➜"</span>
                                <span class="text-[#e7e4ec]">"simucli deploy --env production"</span>
                            </div>
                            <div class="mb-1 text-[#acaab1]">
                                "Checking local environment... "<span class="text-[#4ae176]">"DONE"</span>
                            </div>
                            <div class="mb-1 text-[#acaab1]">
                                "Authenticating with remote... "<span class="text-[#4ae176]">"VERIFIED"</span>
                            </div>
                            <div class="mb-4 text-[#acaab1]">"Optimizing Rust/WASM binaries... "</div>
                            <div class="mb-4">
                                <div class="h-1 w-full overflow-hidden rounded-full bg-[#25252b]">
                                    <div class="h-full w-2/3 bg-[#4ae176]"></div>
                                </div>
                                <div class="label mt-1 flex justify-between text-[10px] uppercase text-[#4ae176]">
                                    <span>"Compressing assets"</span>
                                    <span>"67%"</span>
                                </div>
                            </div>
                            <div class="text-[#acaab1]">
                                "["<span class="text-[#4ae176]">"OK"</span>
                                "] Deployment successful."
                                <br/>
                                "URL: "<span class="text-[#4ae176] underline">"https://api.simucli.dev/v1/deploy/7f2a1"</span>
                                <br/>
                                "Latency: 12ms"
                            </div>
                            <div class="mt-4 flex gap-3">
                                <span class="text-[#4ae176]">"➜"</span>
                                <span class="h-5 w-2 animate-pulse bg-[#4ae176]"></span>
                            </div>
                        </div>
                    </div>
                </section>

                <section id="features" class="bg-[#131316] py-24">
                    <div class="mx-auto max-w-7xl px-6">
                        <div class="mb-16">
                            <h2 class="mb-2 text-3xl font-bold tracking-tight text-[#e7e4ec]">"Architected for Speed"</h2>
                            <p class="text-[#acaab1]">"The power of a native terminal, the reach of the browser."</p>
                        </div>

                        <div class="grid grid-cols-1 gap-4 md:grid-cols-12">
                            <section class="md:col-span-8 rounded-xl border border-[#47474e] bg-[#19191d] p-10 transition-colors hover:bg-[#1f1f24]">
                                <div class="mb-6 flex h-12 w-12 items-center justify-center rounded bg-[#4ae176]/10">
                                    <span class="material-symbols-outlined text-[#4ae176]">"bolt"</span>
                                </div>
                                <h3 class="mb-4 text-2xl font-bold text-[#e7e4ec]">"Rust & WASM Powered"</h3>
                                <p class="text-lg leading-relaxed text-[#acaab1]">
                                    "Engineered in Rust for memory safety and compiled to WebAssembly for near-native execution. Fast, lightweight, and runs entirely in the user's browser without overhead."
                                </p>
                            </section>

                            <section class="md:col-span-4 rounded-xl border border-[#47474e] bg-[#19191d] p-10 transition-colors hover:bg-[#1f1f24]">
                                <div class="mb-6 flex h-12 w-12 items-center justify-center rounded bg-[#4ae176]/10">
                                    <span class="material-symbols-outlined text-[#4ae176]">"cloud_off"</span>
                                </div>
                                <h3 class="mb-4 text-xl font-bold text-[#e7e4ec]">"Zero Compute"</h3>
                                <p class="text-[#acaab1]">
                                    "No server-side execution or WebSockets required. Your demos live and breathe purely client-side."
                                </p>
                            </section>

                            <section class="md:col-span-4 rounded-xl border border-[#47474e] bg-[#19191d] p-10 transition-colors hover:bg-[#1f1f24]">
                                <div class="mb-6 flex h-12 w-12 items-center justify-center rounded bg-[#4ae176]/10">
                                    <span class="material-symbols-outlined text-[#4ae176]">"plumbing"</span>
                                </div>
                                <h3 class="mb-4 text-xl font-bold text-[#e7e4ec]">"Asciinema Pipeline"</h3>
                                <p class="text-[#acaab1]">
                                    "Import existing .cast files and convert them into interactive steps instantly. No manual typing required."
                                </p>
                            </section>

                            <section class="md:col-span-8 rounded-xl border border-[#47474e] bg-[#19191d] p-10 transition-colors hover:bg-[#1f1f24]">
                                <div class="mb-6 flex h-12 w-12 items-center justify-center rounded bg-[#4ae176]/10">
                                    <span class="material-symbols-outlined text-[#4ae176]">"code"</span>
                                </div>
                                <h3 class="mb-4 text-2xl font-bold text-[#e7e4ec]">"Stateless Simulation"</h3>
                                <p class="text-lg leading-relaxed text-[#acaab1]">
                                    "SimuCLI is defined by a simple JSON manifest. Embed your interactive CLI sessions in any website, blog, or documentation via iframe or a custom web component."
                                </p>
                            </section>
                        </div>
                    </div>
                </section>

                <section class="mx-auto max-w-7xl px-6 py-24">
                    <div class="mb-16 text-center">
                        <h2 class="label mb-4 text-3xl font-black uppercase tracking-tight text-[#4ae176]">"Workflow"</h2>
                        <p class="text-xl font-medium text-[#e7e4ec]">"From recording to production in three steps."</p>
                    </div>

                    <div class="grid grid-cols-1 gap-12 md:grid-cols-3">
                        <section class="text-center">
                            <div class="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full border border-[#47474e] bg-[#1f1f24] text-2xl font-bold text-[#4ae176]">
                                "1"
                            </div>
                            <h4 class="mb-2 text-lg font-bold text-[#e7e4ec]">"Author"</h4>
                            <p class="text-[#acaab1]">
                                "Record your CLI session or write a JSON manifest defining each interaction step."
                            </p>
                        </section>

                        <section class="text-center">
                            <div class="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full border border-[#47474e] bg-[#1f1f24] text-2xl font-bold text-[#4ae176]">
                                "2"
                            </div>
                            <h4 class="mb-2 text-lg font-bold text-[#e7e4ec]">"Publish"</h4>
                            <p class="text-[#acaab1]">
                                "Upload to SimuCLI Cloud or host the static WASM artifacts yourself. Completely serverless."
                            </p>
                        </section>

                        <section class="text-center">
                            <div class="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full border border-[#47474e] bg-[#1f1f24] text-2xl font-bold text-[#4ae176]">
                                "3"
                            </div>
                            <h4 class="mb-2 text-lg font-bold text-[#e7e4ec]">"Embed"</h4>
                            <p class="text-[#acaab1]">
                                "Drop a single script tag or iframe into your documentation. Your users get a live playground."
                            </p>
                        </section>
                    </div>
                </section>

                <section id="pricing" class="border-y border-[#47474e]/10 bg-[#0e0e10] py-24">
                    <div class="mx-auto max-w-3xl px-6 text-center">
                        <div class="label mb-4 text-xs uppercase tracking-widest text-[#4ae176]">"Open Standard"</div>
                        <h2 class="mb-8 text-5xl font-black tracking-tighter text-[#e7e4ec]">"Free for everyone."</h2>
                        <p class="mb-10 text-lg text-[#acaab1]">
                            "We believe interactive documentation should be the standard, not a luxury. The SimuCLI runtime and authoring tools are open-source and free to use forever."
                        </p>
                        <div class="inline-flex items-center gap-4 rounded-lg border border-[#4ae176]/20 bg-[#19191d] p-6 text-left">
                            <div class="flex h-10 w-10 items-center justify-center rounded bg-[#4ae176]/20">
                                <span class="material-symbols-outlined text-[#4ae176]">"redeem"</span>
                            </div>
                            <div>
                                <div class="font-bold text-[#e7e4ec]">"No Hidden Fees"</div>
                                <div class="text-sm text-[#acaab1]">"Host it anywhere. Pay nothing for compute."</div>
                            </div>
                        </div>
                    </div>
                </section>

                <section class="relative overflow-hidden py-32">
                    <div class="absolute inset-0 bg-gradient-to-t from-[#4ae176]/5 to-transparent"></div>
                    <div class="relative z-10 mx-auto max-w-4xl px-6 text-center">
                        <h2 class="mb-8 text-4xl font-black tracking-tight text-[#e7e4ec] md:text-5xl">
                            "Ready to upgrade your documentation?"
                        </h2>
                        <a
                            class="mx-auto flex w-fit items-center gap-3 rounded-lg bg-[#4ae176] px-12 py-5 text-lg font-bold text-[#004b1e] transition-all hover:bg-[#38d36a]"
                            href={api::login_url()}
                            on:click=move |_| auth.set_logging_in.set(true)
                        >
                            <span class="material-symbols-outlined">"login"</span>
                            {move || {
                                if auth.is_logging_in.get() {
                                    "Connecting to GitHub..."
                                } else {
                                    "Login with GitHub"
                                }
                            }}
                        </a>
                    </div>
                </section>
            </main>

            <footer class="flex w-full flex-col items-center justify-between gap-4 border-t border-[#25252b]/20 bg-[#0e0e10] px-8 py-12 text-[10px] uppercase tracking-widest text-[#9f9da1] md:flex-row">
                <div class="font-black tracking-tighter text-[#e7e4ec] normal-case text-lg">"SimuCLI"</div>
                <div class="flex gap-8">
                    <a class="transition-colors hover:text-[#4ae176]" href="#">"Privacy"</a>
                    <a class="transition-colors hover:text-[#4ae176]" href="#">"Terms"</a>
                    <a class="transition-colors hover:text-[#4ae176]" href="#">"GitHub"</a>
                    <a class="transition-colors hover:text-[#4ae176]" href="#">"Changelog"</a>
                </div>
                <div>"© 2026 SimuCLI. All rights reserved."</div>
            </footer>
        </div>
    }
}
