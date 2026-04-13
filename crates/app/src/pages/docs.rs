use leptos::prelude::*;
use leptos_router::components::A;

#[component]
fn DocsHeader() -> impl IntoView {
    view! {
        <header class="fixed top-0 w-full z-50 flex justify-between items-center px-6 h-14 bg-[#0e0e10] border-b border-[#19191d]">
            <div class="flex items-center gap-8">
                <span class="text-xl font-black tracking-tighter text-white uppercase">"TERMINAL_DOCS"</span>
                <nav class="hidden md:flex gap-6">
                    <A attr:class="font-sans tracking-tight text-sm text-[#4ae176] font-bold border-b-2 border-[#4ae176] pb-1" href="/docs">"Guides"</A>
                    <A attr:class="font-sans tracking-tight text-sm text-zinc-500 hover:text-zinc-200 transition-colors" href="/docs/api">"API"</A>
                    <A attr:class="font-sans tracking-tight text-sm text-zinc-500 hover:text-zinc-200 transition-colors" href="/docs/changelog">"Changelog"</A>
                    <A attr:class="font-sans tracking-tight text-sm text-zinc-500 hover:text-zinc-200 transition-colors" href="/docs/community">"Community"</A>
                </nav>
            </div>
            <div class="flex items-center gap-4">
                <div class="hidden md:flex bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 rounded items-center gap-2">
                    <span class="material-symbols-outlined text-sm text-on-surface-variant">"search"</span>
                    <span class="text-xs text-on-surface-variant font-label">"Search documentation..."</span>
                    <span class="text-[10px] text-outline px-1 border border-outline-variant/50 rounded">"Cmd+K"</span>
                </div>
                <button class="flex items-center gap-2 px-3 py-1.5 bg-surface-container-highest hover:bg-surface-bright text-xs font-bold transition-all border border-outline-variant/20 rounded-lg">
                    <span class="material-symbols-outlined text-sm">"terminal"</span>
                    "Deploy"
                </button>
                <button class="flex items-center gap-2 px-4 py-1.5 bg-white text-black text-xs font-bold transition-all rounded hover:bg-zinc-200">
                    <img
                        alt="GitHub Logo"
                        class="w-4 h-4"
                        src="https://lh3.googleusercontent.com/aida-public/AB6AXuDZoB3jLn7hN2woyYRN7frwsvBszEBna9m5L03wKgDjiuvbuY0Ni3zXpa7auNyU3kgLABuWF6lraoC5gtqsSOve_7ETsjSj9rdZDQaudLOHcZZY_XkO2XmRNwmn2jKrkxlHhASgyENIPfZNlkghP7bll0vrTVmRguQTVpMhsmnIY80VRUyarxhk73Wk8jP5ECxDd_GXXJFb-BJbO31ix-tzL9hgXVabXfXDEer55cnf-12UklRaWjBkNtObSde3OwvXspT5AGlrBD4"
                    />
                    "Login with GitHub"
                </button>
            </div>
        </header>
    }
}

#[component]
fn DocsSidebar() -> impl IntoView {
    view! {
        <aside class="fixed left-0 top-14 h-[calc(100vh-3.5rem)] w-64 flex flex-col py-4 bg-[#131316] border-r border-[#19191d] overflow-y-auto z-40">
            <div class="px-6 mb-8">
                <h2 class="text-lg font-bold text-white mb-1">"SimuCLI"</h2>
                <p class="font-mono text-[10px] uppercase tracking-widest text-[#4ae176]">"v2.4.0-stable"</p>
            </div>
            <div class="flex flex-col gap-8 px-2">
                <section>
                    <h3 class="px-4 font-mono text-[10px] uppercase tracking-widest text-zinc-500 mb-2">"Getting Started"</h3>
                    <div class="flex flex-col">
                        <a class="text-[#4ae176] bg-[#19191d] border-l-2 border-[#4ae176] pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="#introduction">"Introduction"</a>
                        <a class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="#quick-start">"Quick Start"</a>
                    </div>
                </section>
                <section>
                    <h3 class="px-4 font-mono text-[10px] uppercase tracking-widest text-zinc-500 mb-2">"Workspace"</h3>
                    <div class="flex flex-col">
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/workspace/projects">"Creating Projects"</A>
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/workspace/demos">"Managing Demos"</A>
                    </div>
                </section>
                <section>
                    <h3 class="px-4 font-mono text-[10px] uppercase tracking-widest text-zinc-500 mb-2">"Editor"</h3>
                    <div class="flex flex-col">
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/editor/recording">"Recording Casts"</A>
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/editor/manual-authoring">"Manual Authoring"</A>
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/editor/step-configuration">"Step Configuration"</A>
                    </div>
                </section>
                <section>
                    <h3 class="px-4 font-mono text-[10px] uppercase tracking-widest text-zinc-500 mb-2">"Publishing"</h3>
                    <div class="flex flex-col">
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/publishing/deploying">"Deploying"</A>
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/publishing/analytics">"Analytics"</A>
                    </div>
                </section>
                <section>
                    <h3 class="px-4 font-mono text-[10px] uppercase tracking-widest text-zinc-500 mb-2">"Integration"</h3>
                    <div class="flex flex-col">
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/integration/script-iframe">"Script / Iframe"</A>
                        <A attr:class="text-zinc-500 hover:text-zinc-300 pl-4 py-2 font-mono text-xs uppercase tracking-widest transition-all" href="/docs/integration/react-vue">"React / Vue Wrappers"</A>
                    </div>
                </section>
            </div>
            <div class="mt-auto border-t border-[#19191d] pt-4 px-6 flex flex-col gap-2">
                <a class="flex items-center gap-2 text-zinc-500 hover:text-zinc-300 text-[10px] font-mono uppercase tracking-widest" href="#">
                    <span class="material-symbols-outlined text-sm">"help"</span>
                    "Support"
                </a>
                <a class="flex items-center gap-2 text-zinc-500 hover:text-zinc-300 text-[10px] font-mono uppercase tracking-widest" href="#">
                    <span class="material-symbols-outlined text-sm">"chat_bubble"</span>
                    "Feedback"
                </a>
            </div>
        </aside>
    }
}

#[component]
fn DocsToc() -> impl IntoView {
    view! {
        <aside class="fixed right-0 top-14 h-[calc(100vh-3.5rem)] w-64 flex flex-col py-8 px-6 bg-[#0e0e10] border-l border-[#19191d] hidden lg:flex">
            <h4 class="text-[10px] font-mono uppercase tracking-widest text-zinc-500 mb-6">"On this page"</h4>
            <nav class="flex flex-col gap-4">
                <a class="text-xs text-zinc-400 hover:text-primary transition-colors border-l border-zinc-800 pl-4 py-0.5" href="#introduction">"Introduction"</a>
                <a class="text-xs text-zinc-400 hover:text-primary transition-colors border-l border-zinc-800 pl-4 py-0.5" href="#why-simucli">"Why SimuCLI?"</a>
                <a class="text-xs text-zinc-400 hover:text-primary transition-colors border-l border-zinc-800 pl-4 py-0.5" href="#quick-start">"Quick Start"</a>
                <a class="text-xs text-zinc-400 hover:text-primary transition-colors border-l border-zinc-800 pl-4 py-0.5" href="#architecture">"Architecture"</a>
            </nav>
            <div class="mt-12 p-4 bg-surface-container-low border border-outline-variant/10 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                    <span class="material-symbols-outlined text-primary text-sm">"stars"</span>
                    <span class="text-[10px] font-mono text-white uppercase tracking-widest">"Pro Tip"</span>
                </div>
                <p class="text-[11px] text-on-surface-variant leading-tight">
                    "Use "
                    <code>"--interactive"</code>
                    " when recording to pause for user input during playback."
                </p>
            </div>
            <div class="mt-auto">
                <img
                    class="w-full h-40 object-cover rounded grayscale opacity-40 mix-blend-screen mb-4"
                    alt="minimalist dark terminal interface with glowing green line art showing connection nodes and digital flow"
                    src="https://lh3.googleusercontent.com/aida-public/AB6AXuASF8rTqrfN0IP1aGFVPPdUZphQhcg3h5Gw7uE4TgC0npvv0W-xofkjslI0096J7nXR2qXWXuYhL09XUxzJIAPIFiREbfxfoNf21nlRSev8MAfxRyNRnX1bBHRvcowrUc_81qlnrM4Z8sMswPzKdguRqKUFL4zrqBsIkMuxrfMAsnjn1Wdi2pkyR6tuBsv9cmWhKTPU-Ubja6IQnlFoduDC1-kuHSgDUBCFUJ2Xvw_gDkZJXlFk5HphUZJt2fNCHHEuflDyjXcuuoY"
                />
                <p class="text-[10px] text-zinc-700 italic">"\"Precision at every keystroke.\""</p>
            </div>
        </aside>
    }
}

#[component]
pub fn DocsPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-[#0e0e10] text-[#e7e4ec]">
            <DocsHeader />
            <div class="flex pt-14 h-screen overflow-hidden">
                <DocsSidebar />
                <main class="ml-64 mr-64 flex-1 overflow-y-auto bg-[#0e0e10]">
                    <div class="max-w-4xl mx-auto px-12 py-16">
                        <header class="mb-16" id="introduction">
                            <div class="inline-flex items-center gap-2 px-2 py-0.5 bg-primary/10 border border-primary/20 rounded mb-4">
                                <span class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></span>
                                <span class="text-[10px] font-mono text-primary uppercase tracking-widest">"Platform Introduction"</span>
                            </div>
                            <h1 class="text-5xl font-extrabold tracking-tighter text-white mb-6">"Introduction to SimuCLI"</h1>
                            <p class="text-xl text-on-surface-variant leading-relaxed font-body mb-8">
                                "The world's most precise client-side terminal simulation platform. Build interactive terminal walkthroughs, product demos, and CLI tutorials that look real, but run entirely in the browser."
                            </p>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-12">
                                <div class="p-6 bg-surface-container-low border border-outline-variant/10 rounded-lg">
                                    <span class="material-symbols-outlined text-primary mb-3">"speed"</span>
                                    <h4 class="text-white font-bold mb-2">"Zero Latency"</h4>
                                    <p class="text-sm text-on-surface-variant">"SimuCLI runs entirely client-side, ensuring your demos are snappy and offline-capable."</p>
                                </div>
                                <div class="p-6 bg-surface-container-low border border-outline-variant/10 rounded-lg">
                                    <span class="material-symbols-outlined text-primary mb-3">"frame_person"</span>
                                    <h4 class="text-white font-bold mb-2">"High Fidelity"</h4>
                                    <p class="text-sm text-on-surface-variant">"Pixel-perfect terminal rendering with support for themes, colors, and interactive inputs."</p>
                                </div>
                            </div>
                        </header>

                        <section class="mb-20" id="why-simucli">
                            <h2 class="text-2xl font-bold tracking-tight text-white mb-6 flex items-center gap-2">
                                <span class="w-1 h-6 bg-primary"></span>
                                "Why SimuCLI?"
                            </h2>
                            <div class="prose prose-invert max-w-none text-on-surface-variant space-y-4">
                                <p>"Traditional screen recordings are static and hard to maintain. Real SSH environments are dangerous for public demos. "<strong>"SimuCLI"</strong>" bridges the gap by providing a sandbox environment that mimics real terminal behavior without the backend overhead."</p>
                                <p>"Our simulation engine allows you to author scripts that react to user input, pause for explanations, and even \"fake\" network latency to give your product demos a visceral, realistic feel."</p>
                            </div>
                        </section>

                        <section class="mb-20" id="quick-start">
                            <h2 class="text-2xl font-bold tracking-tight text-white mb-6 flex items-center gap-2">
                                <span class="w-1 h-6 bg-primary"></span>
                                "Quick Start"
                            </h2>
                            <p class="text-on-surface-variant mb-6">"Get SimuCLI running in your project in under 60 seconds. Start by installing the core CLI tool."</p>
                            <div class="group relative">
                                <div class="absolute -inset-0.5 bg-gradient-to-r from-primary/20 to-transparent rounded opacity-20 group-hover:opacity-40 transition-opacity"></div>
                                <div class="relative bg-surface-container-lowest border border-outline-variant/30 rounded-lg overflow-hidden">
                                    <div class="flex items-center justify-between px-4 py-2 bg-surface-container border-b border-outline-variant/30">
                                        <div class="flex gap-1.5">
                                            <div class="w-2.5 h-2.5 rounded-full bg-zinc-800"></div>
                                            <div class="w-2.5 h-2.5 rounded-full bg-zinc-800"></div>
                                            <div class="w-2.5 h-2.5 rounded-full bg-zinc-800"></div>
                                        </div>
                                        <span class="text-[10px] font-mono text-on-surface-variant uppercase tracking-widest">"bash"</span>
                                    </div>
                                    <div class="p-5 font-mono text-sm leading-relaxed overflow-x-auto">
                                        <div class="flex">
                                            <span class="text-primary mr-3 opacity-50 select-none">"$"</span>
                                            <span class="text-on-surface">"npm install -g @simucli/core"</span>
                                        </div>
                                        <div class="flex mt-2">
                                            <span class="text-primary mr-3 opacity-50 select-none">"$"</span>
                                            <span class="text-on-surface">"simucli init my-demo"</span>
                                        </div>
                                        <div class="text-zinc-500 mt-2 ml-6">"[ok] Initialized new SimuCLI project in /my-demo"</div>
                                        <div class="flex mt-2">
                                            <span class="text-primary mr-3 opacity-50 select-none">"$"</span>
                                            <span class="text-on-surface">"cd my-demo && simucli start"</span>
                                        </div>
                                        <div class="text-primary mt-2 ml-6">"Demo server running at http://localhost:3000"</div>
                                    </div>
                                </div>
                            </div>
                            <div class="mt-8 flex flex-col gap-4">
                                <h4 class="text-sm font-bold text-white uppercase tracking-wider">"Next steps:"</h4>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <a class="group p-4 bg-surface-container border border-outline-variant/10 hover:border-primary/50 transition-all rounded" href="#">
                                        <div class="flex justify-between items-start mb-2">
                                            <span class="material-symbols-outlined text-zinc-500 group-hover:text-primary transition-colors">"history"</span>
                                            <span class="material-symbols-outlined text-xs text-zinc-600">"arrow_forward"</span>
                                        </div>
                                        <h5 class="text-white text-sm font-bold">"Record your first cast"</h5>
                                        <p class="text-xs text-on-surface-variant">"Learn how to record actual terminal sessions."</p>
                                    </a>
                                    <a class="group p-4 bg-surface-container border border-outline-variant/10 hover:border-primary/50 transition-all rounded" href="#">
                                        <div class="flex justify-between items-start mb-2">
                                            <span class="material-symbols-outlined text-zinc-500 group-hover:text-primary transition-colors">"edit_document"</span>
                                            <span class="material-symbols-outlined text-xs text-zinc-600">"arrow_forward"</span>
                                        </div>
                                        <h5 class="text-white text-sm font-bold">"Manual Authoring"</h5>
                                        <p class="text-xs text-on-surface-variant">"Write demo scripts manually with JSON."</p>
                                    </a>
                                </div>
                            </div>
                        </section>

                        <section class="mb-20" id="architecture">
                            <h2 class="text-2xl font-bold tracking-tight text-white mb-6 flex items-center gap-2">
                                <span class="w-1 h-6 bg-primary"></span>
                                "Architecture"
                            </h2>
                            <div class="bg-surface-container-low border border-outline-variant/10 rounded-xl p-8 overflow-hidden relative">
                                <div class="absolute top-0 right-0 p-8 opacity-10">
                                    <span class="material-symbols-outlined text-[120px] text-primary" style="font-variation-settings: 'FILL' 1;">"architecture"</span>
                                </div>
                                <div class="relative z-10">
                                    <h3 class="text-lg font-bold text-white mb-4">"Pure Client-Side Engine"</h3>
                                    <p class="text-on-surface-variant mb-6 max-w-lg leading-relaxed">"SimuCLI doesn't use a VM. It uses a custom AST parser to translate command strings into virtual terminal states. This results in a package size under 45kb gzipped."</p>
                                    <div class="flex gap-12">
                                        <div>
                                            <p class="text-primary font-mono text-2xl font-bold">"45KB"</p>
                                            <p class="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">"Bundle Size"</p>
                                        </div>
                                        <div>
                                            <p class="text-primary font-mono text-2xl font-bold">"0ms"</p>
                                            <p class="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">"Execution Lag"</p>
                                        </div>
                                        <div>
                                            <p class="text-primary font-mono text-2xl font-bold">"100%"</p>
                                            <p class="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">"WASM Free"</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </section>

                        <footer class="mt-32 pt-12 border-t border-outline-variant/10 flex justify-between text-xs text-zinc-600 font-label">
                            <span>"(c) 2024 SIMU_CORE SYSTEMS"</span>
                            <div class="flex gap-6">
                                <a class="hover:text-primary transition-colors" href="#">"Privacy"</a>
                                <a class="hover:text-primary transition-colors" href="#">"Terms"</a>
                                <a class="hover:text-primary transition-colors" href="#">"Status"</a>
                            </div>
                        </footer>
                    </div>
                </main>
                <DocsToc />
            </div>
        </div>
    }
}