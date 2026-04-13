use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::auth::{SessionState, use_auth_context};

#[derive(Clone, Copy)]
struct DocsSection {
    title: &'static str,
    summary: &'static str,
    details: &'static str,
}

fn docs_section_for(path: &str) -> DocsSection {
    match path {
        "api" => DocsSection {
            title: "API Reference",
            summary: "Browse the public endpoints and client methods exposed by SimuCLI.",
            details: "This section is the entry point for developers who need the request shapes, response contracts, and integration points behind the documentation runtime.",
        },
        "changelog" => DocsSection {
            title: "Changelog",
            summary: "Track product and documentation updates across releases.",
            details: "Use this page to review release notes, feature additions, and any behavior changes that affect published demos or embeds.",
        },
        "community" => DocsSection {
            title: "Community",
            summary: "Find support, examples, and contribution paths.",
            details: "This page collects the collaboration surface around the project so users can get help and stay aligned with the current workflow.",
        },
        "workspace/projects" => DocsSection {
            title: "Creating Projects",
            summary: "Organize demos into named project groups.",
            details: "Projects are optional containers for related demos. Use them to keep launch work, experiments, and product areas separate.",
        },
        "workspace/demos" => DocsSection {
            title: "Managing Demos",
            summary: "Keep drafts, published demos, and analytics organized.",
            details: "This page explains how demos move through the workspace so teams can publish assets without losing track of status or ownership.",
        },
        "editor/recording" => DocsSection {
            title: "Recording Casts",
            summary: "Capture terminal sessions as guided demo flows.",
            details: "Recorded casts are useful when you want to preserve an exact command sequence and replay it consistently for documentation or marketing.",
        },
        "editor/manual-authoring" => DocsSection {
            title: "Manual Authoring",
            summary: "Author precise demo flows by hand.",
            details: "Manual authoring gives you more control over the narrative, timing, and output text without depending on a live recorder.",
        },
        "editor/step-configuration" => DocsSection {
            title: "Step Configuration",
            summary: "Tune prompts, pauses, and command steps.",
            details: "The step model should stay explicit and easy to scan so the player can execute the sequence exactly as authored.",
        },
        "publishing/deploying" => DocsSection {
            title: "Deploying",
            summary: "Promote a draft demo into a stable public asset.",
            details: "Use publishing to turn an internal demo into something you can share, embed, or surface in a campaign landing page.",
        },
        "publishing/analytics" => DocsSection {
            title: "Analytics",
            summary: "Measure engagement and completion trends.",
            details: "Analytics helps you understand where users stop, what they replay, and which demos actually drive engagement.",
        },
        "integration/script-iframe" => DocsSection {
            title: "Script / Iframe",
            summary: "Embed the runtime with a script tag or an iframe.",
            details: "Script-based embedding is the lightest path. If you need isolation, an iframe keeps the runtime sandboxed while remaining interactive.",
        },
        "integration/react-vue" => DocsSection {
            title: "React / Vue Wrappers",
            summary: "Use thin framework wrappers around the same runtime.",
            details: "Wrappers should stay small and forward the same demo, API, and sizing settings into the player surface.",
        },
        _ => DocsSection {
            title: "Documentation Section",
            summary: "The requested page could not be found.",
            details: "Use the sidebar or top navigation to jump to a supported documentation section.",
        },
    }
}

#[derive(Clone, Copy)]
struct DocsTopic {
    path: &'static str,
    title: &'static str,
    summary: &'static str,
}

const DOC_TOPICS: &[DocsTopic] = &[
    DocsTopic {
        path: "/docs/api",
        title: "API Reference",
        summary: "Browse the public endpoints and client methods exposed by SimuCLI.",
    },
    DocsTopic {
        path: "/docs/changelog",
        title: "Changelog",
        summary: "Track product and documentation updates across releases.",
    },
    DocsTopic {
        path: "/docs/community",
        title: "Community",
        summary: "Find support, examples, and contribution paths.",
    },
    DocsTopic {
        path: "/docs/workspace/projects",
        title: "Creating Projects",
        summary: "Organize demos into named project groups.",
    },
    DocsTopic {
        path: "/docs/workspace/demos",
        title: "Managing Demos",
        summary: "Keep drafts, published demos, and analytics organized.",
    },
    DocsTopic {
        path: "/docs/editor/recording",
        title: "Recording Casts",
        summary: "Capture terminal sessions as guided demo flows.",
    },
    DocsTopic {
        path: "/docs/editor/manual-authoring",
        title: "Manual Authoring",
        summary: "Author precise demo flows by hand.",
    },
    DocsTopic {
        path: "/docs/editor/step-configuration",
        title: "Step Configuration",
        summary: "Tune prompts, pauses, and command steps.",
    },
    DocsTopic {
        path: "/docs/publishing/deploying",
        title: "Deploying",
        summary: "Promote a draft demo into a stable public asset.",
    },
    DocsTopic {
        path: "/docs/publishing/analytics",
        title: "Analytics",
        summary: "Measure engagement and completion trends.",
    },
    DocsTopic {
        path: "/docs/integration/script-iframe",
        title: "Script / Iframe",
        summary: "Embed the runtime with a script tag or an iframe.",
    },
    DocsTopic {
        path: "/docs/integration/react-vue",
        title: "React / Vue Wrappers",
        summary: "Use thin framework wrappers around the same runtime.",
    },
];

fn topic_matches_query(topic: &DocsTopic, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }

    let query = query.trim().to_ascii_lowercase();
    topic.path.to_ascii_lowercase().contains(&query)
        || topic.title.to_ascii_lowercase().contains(&query)
        || topic.summary.to_ascii_lowercase().contains(&query)
}

#[component]
fn DocsSectionRoute() -> impl IntoView {
    let params = use_params_map();
    let path = move || {
        let map = params.read();
        match (
            map.get("category"),
            map.get("section"),
            map.get("subsection"),
        ) {
            (Some(category), Some(section), Some(subsection)) => {
                format!("{}/{}/{}", category, section, subsection)
            }
            (Some(category), Some(section), None) => format!("{}/{}", category, section),
            (Some(section), None, None) => section,
            _ => "unknown".to_string(),
        }
    };

    move || {
        let section = docs_section_for(&path());
        view! {
            <section class="min-h-screen bg-[#0e0e10] px-6 py-24 text-[#e7e4ec]">
                <div class="mx-auto max-w-4xl">
                    <p class="mb-4 font-mono text-xs uppercase tracking-[0.2em] text-[#4ae176]">"Documentation"</p>
                    <h1 class="mb-6 text-5xl font-black tracking-tighter text-white">{section.title}</h1>
                    <p class="mb-6 max-w-2xl text-lg leading-relaxed text-[#acaab1]">{section.summary}</p>
                    <div class="rounded-xl border border-[#47474e] bg-[#19191d] p-8 text-[#e7e4ec]">
                        <p class="leading-relaxed text-[#acaab1]">{section.details}</p>
                    </div>
                    <div class="mt-8 flex flex-wrap gap-4">
                        <A attr:class="rounded bg-[#4ae176] px-4 py-2 font-bold text-[#004b1e] transition-colors hover:bg-[#38d36a]" href="/docs">"Back to overview"</A>
                        <A attr:class="rounded border border-[#47474e] bg-[#25252b] px-4 py-2 font-bold transition-colors hover:bg-[#2b2c32]" href="/dashboard">"Open dashboard"</A>
                    </div>
                </div>
            </section>
        }
    }
}

#[component]
fn DocsHeader(
    search_query: ReadSignal<String>,
    set_search_query: WriteSignal<String>,
) -> impl IntoView {
    let auth = use_auth_context();

    view! {
        <header class="fixed top-0 w-full z-50 flex justify-between items-center px-6 h-14 bg-[#0e0e10] border-b border-[#19191d]">
            <div class="flex items-center gap-8">
                <A attr:class="text-xl font-black tracking-tighter text-white uppercase" href="/">"TERMINAL_DOCS"</A>
                <nav class="hidden md:flex gap-6">
                    <A attr:class="font-sans tracking-tight text-sm text-zinc-500 hover:text-zinc-200 transition-colors" href="/">"Home"</A>
                    <A attr:class="font-sans tracking-tight text-sm text-[#4ae176] font-bold border-b-2 border-[#4ae176] pb-1" href="/docs">"Guides"</A>
                    <A attr:class="font-sans tracking-tight text-sm text-zinc-500 hover:text-zinc-200 transition-colors" href="/docs/api">"API"</A>
                    <A attr:class="font-sans tracking-tight text-sm text-zinc-500 hover:text-zinc-200 transition-colors" href="/docs/changelog">"Changelog"</A>
                    <A attr:class="font-sans tracking-tight text-sm text-zinc-500 hover:text-zinc-200 transition-colors" href="/docs/community">"Community"</A>
                </nav>
            </div>
            <div class="flex items-center gap-4">
                <label class="hidden md:flex bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 rounded items-center gap-2">
                    <span class="material-symbols-outlined text-sm text-on-surface-variant">"search"</span>
                    <input
                        class="w-64 bg-transparent text-xs text-on-surface outline-none placeholder:text-on-surface-variant"
                        placeholder="Search docs..."
                        prop:value=move || search_query.get()
                        on:input=move |event| set_search_query.set(event_target_value(&event))
                    />
                </label>
                {move || match auth.session_state.get() {
                    SessionState::LoggedIn(_) => view! {
                        <A attr:class="flex items-center gap-2 px-4 py-1.5 bg-white text-black text-xs font-bold transition-all rounded hover:bg-zinc-200" href="/dashboard">
                            <span class="material-symbols-outlined text-sm">"dashboard"</span>
                            "Go to dashboard"
                        </A>
                    }
                    .into_any(),
                    _ => view! {
                        <a class="flex items-center gap-2 px-4 py-1.5 bg-white text-black text-xs font-bold transition-all rounded hover:bg-zinc-200" href={api::login_url()}>
                            <img
                                alt="GitHub Logo"
                                class="w-4 h-4"
                                src="https://lh3.googleusercontent.com/aida-public/AB6AXuDZoB3jLn7hN2woyYRN7frwsvBszEBna9m5L03wKgDjiuvbuY0Ni3zXpa7auNyU3kgLABuWF6lraoC5gtqsSOve_7ETsjSj9rdZDQaudLOHcZZY_XkO2XmRNwmn2jKrkxlHhASgyENIPfZNlkghP7bll0vrTVmRguQTVpMhsmnIY80VRUyarxhk73Wk8jP5ECxDd_GXXJFb-BJbO31ix-tzL9hgXVabXfXDEer55cnf-12UklRaWjBkNtObSde3OwvXspT5AGlrBD4"
                            />
                            "Login with GitHub"
                        </a>
                    }
                    .into_any(),
                }}
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
            <div class="mt-auto rounded-lg border border-outline-variant/10 bg-surface-container-low p-4 text-[11px] leading-tight text-on-surface-variant">
                "Use the search field above to narrow the docs catalog by title, topic, or route."
            </div>
        </aside>
    }
}

#[component]
pub fn DocsPage() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let filtered_topics = move || {
        DOC_TOPICS
            .iter()
            .copied()
            .filter(|topic| topic_matches_query(topic, &search_query.get()))
            .collect::<Vec<_>>()
    };

    view! {
        <div class="min-h-screen bg-[#0e0e10] text-[#e7e4ec]">
            <DocsHeader search_query set_search_query />
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

                        <section class="mb-20">
                            <h2 class="text-2xl font-bold tracking-tight text-white mb-4 flex items-center gap-2">
                                <span class="w-1 h-6 bg-primary"></span>
                                {move || {
                                    let count = filtered_topics().len();
                                    if search_query.get().trim().is_empty() {
                                        "Docs Index".to_string()
                                    } else {
                                        format!("Search Results ({count})")
                                    }
                                }}
                            </h2>
                            <p class="mb-6 max-w-2xl text-sm text-on-surface-variant">
                                {move || {
                                    let query = search_query.get();
                                    if query.trim().is_empty() {
                                        "Browse the available documentation pages or use search to jump directly to a topic.".to_string()
                                    } else {
                                        format!("Results for \"{}\".", query.trim())
                                    }
                                }}
                            </p>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <For
                                    each=filtered_topics
                                    key=|topic| topic.path
                                    children=move |topic| {
                                        view! {
                                            <A attr:class="group rounded-xl border border-outline-variant/10 bg-surface-container-low p-5 transition-all hover:border-primary/50 hover:bg-surface-container" href=topic.path>
                                                <div class="flex items-start justify-between gap-4 mb-3">
                                                    <div>
                                                        <h3 class="text-lg font-bold text-white group-hover:text-primary transition-colors">{topic.title}</h3>
                                                        <p class="text-xs uppercase tracking-[0.18em] text-zinc-500 mt-1">{topic.path}</p>
                                                    </div>
                                                    <span class="material-symbols-outlined text-zinc-600 group-hover:text-primary transition-colors">"arrow_forward"</span>
                                                </div>
                                                <p class="text-sm leading-relaxed text-on-surface-variant">{topic.summary}</p>
                                            </A>
                                        }
                                    }
                                />
                            </div>
                        </section>

                        <Show when=move || filtered_topics().is_empty()>
                            <div class="mb-20 rounded-xl border border-outline-variant/10 bg-surface-container-low p-6 text-sm text-on-surface-variant">
                                "No documentation pages matched your search. Try a section title, topic name, or route fragment."
                            </div>
                        </Show>

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

#[component]
pub fn DocsSectionPage() -> impl IntoView {
    DocsSectionRoute()
}
