use leptos::prelude::*;
use leptos_router::components::A;

#[derive(Clone, Copy)]
struct DocsLink {
    label: &'static str,
    href: &'static str,
}

#[derive(Clone, Copy)]
struct DocsCategory {
    title: &'static str,
    items: &'static [DocsLink],
}

const GETTING_STARTED: &[DocsLink] = &[
    DocsLink {
        label: "Introduction",
        href: "#introduction",
    },
    DocsLink {
        label: "Quick Start",
        href: "#quick-start",
    },
];

const WORKSPACE: &[DocsLink] = &[
    DocsLink {
        label: "Creating Projects",
        href: "#creating-projects",
    },
    DocsLink {
        label: "Managing Demos",
        href: "#managing-demos",
    },
];

const EDITOR: &[DocsLink] = &[
    DocsLink {
        label: "Recording Casts",
        href: "#recording-casts",
    },
    DocsLink {
        label: "Manual Authoring",
        href: "#manual-authoring",
    },
    DocsLink {
        label: "Step Configuration",
        href: "#step-configuration",
    },
];

const PUBLISHING: &[DocsLink] = &[
    DocsLink {
        label: "Deploying",
        href: "#deploying",
    },
    DocsLink {
        label: "Analytics",
        href: "#analytics",
    },
];

const INTEGRATION: &[DocsLink] = &[
    DocsLink {
        label: "Script / Iframe",
        href: "#script-iframe",
    },
    DocsLink {
        label: "React / Vue Wrappers",
        href: "#react-vue-wrappers",
    },
];

const DOC_CATEGORIES: &[DocsCategory] = &[
    DocsCategory {
        title: "Getting Started",
        items: GETTING_STARTED,
    },
    DocsCategory {
        title: "Workspace",
        items: WORKSPACE,
    },
    DocsCategory {
        title: "Editor",
        items: EDITOR,
    },
    DocsCategory {
        title: "Publishing",
        items: PUBLISHING,
    },
    DocsCategory {
        title: "Integration",
        items: INTEGRATION,
    },
];

const DOC_TOC: &[DocsLink] = &[
    DocsLink {
        label: "Introduction",
        href: "#introduction",
    },
    DocsLink {
        label: "Quick Start",
        href: "#quick-start",
    },
    DocsLink {
        label: "Zero Jitter",
        href: "#zero-jitter",
    },
    DocsLink {
        label: "Fully Interactive",
        href: "#fully-interactive",
    },
    DocsLink {
        label: "Lightweight",
        href: "#lightweight",
    },
    DocsLink {
        label: "Projects",
        href: "#creating-projects",
    },
    DocsLink {
        label: "Demo Management",
        href: "#managing-demos",
    },
    DocsLink {
        label: "Editor Workflows",
        href: "#recording-casts",
    },
    DocsLink {
        label: "Publishing",
        href: "#deploying",
    },
    DocsLink {
        label: "Embedding",
        href: "#script-iframe",
    },
];

#[component]
pub fn DocsPage() -> impl IntoView {
    let (sidebar_open, set_sidebar_open) = signal(false);

    view! {
        <section class="docs-page">
            <header class="docs-header panel">
                <div class="docs-header-copy">
                    <p class="docs-kicker">"Void and Paper Documentation Hub"</p>
                    <h1>"SimuCLI Documentation"</h1>
                    <p class="docs-subtitle">
                        "SimuCLI helps teams author, simulate, publish, and embed interactive command-line demonstrations without video loading jitter."
                    </p>
                </div>

                <div class="docs-header-actions">
                    <A attr:class="docs-header-link" href="/dashboard">
                        "Dashboard"
                    </A>
                    <A
                        attr:class="docs-header-link docs-header-link--solid"
                        href="/docs#quick-start"
                    >
                        "Quick Start"
                    </A>
                    <button
                        type="button"
                        class="docs-menu-button"
                        aria-expanded=move || sidebar_open.get()
                        aria-controls="docs-sidebar"
                        on:click=move |_| set_sidebar_open.update(|value| *value = !*value)
                    >
                        {move || if sidebar_open.get() { "Close Menu" } else { "Menu" }}
                    </button>
                </div>
            </header>

            <div class="docs-layout">
                <Show when=move || sidebar_open.get()>
                    <button
                        type="button"
                        class="docs-backdrop"
                        aria-label="Close documentation navigation"
                        on:click=move |_| set_sidebar_open.set(false)
                    />
                </Show>

                <aside
                    id="docs-sidebar"
                    class=move || {
                        if sidebar_open.get() {
                            "docs-sidebar is-open"
                        } else {
                            "docs-sidebar"
                        }
                    }
                >
                    <div class="docs-sidebar-inner">
                        <div class="docs-sidebar-top">
                            <div>
                                <p class="docs-sidebar-label">"Navigation"</p>
                                <p class="docs-sidebar-title">"Documentation Sections"</p>
                            </div>
                            <button
                                type="button"
                                class="docs-sidebar-close"
                                aria-label="Close documentation navigation"
                                on:click=move |_| set_sidebar_open.set(false)
                            >
                                "×"
                            </button>
                        </div>

                        <nav class="docs-nav" aria-label="Documentation navigation">
                            <For
                                each=move || DOC_CATEGORIES.to_vec()
                                key=|category| category.title
                                children=move |category| {
                                    view! {
                                        <section class="docs-nav-group">
                                            <p class="docs-nav-group-title">{category.title}</p>
                                            <div class="docs-nav-links">
                                                <For
                                                    each=move || category.items.to_vec()
                                                    key=|item| item.label
                                                    children=move |item| {
                                                        let href = format!("/docs{}", item.href);
                                                        view! {
                                                            <A
                                                                attr:class="docs-nav-link"
                                                                href=href
                                                                on:click=move |_| set_sidebar_open.set(false)
                                                            >
                                                                {item.label}
                                                            </A>
                                                        }
                                                    }
                                                />
                                            </div>
                                        </section>
                                    }
                                }
                            />
                        </nav>
                    </div>
                </aside>

                <main class="docs-main">
                    <article class="docs-article">
                        <section id="introduction" class="docs-section docs-hero-section">
                            <p class="docs-section-index">"01"</p>
                            <h2>"Introduction"</h2>
                            <p>
                                "SimuCLI is a Rust-based platform for authors who want terminal demos to behave like real software, not video recordings. You can draft steps, simulate execution, publish stable shares, and embed the final experience into product pages, docs, and launch sites."
                            </p>
                            <p>
                                "The design goal is simple: keep the experience sharp, fast, and readable. No chrome-heavy UI, no unnecessary motion, and no playback jitter when a visitor reaches the demo."
                            </p>

                            <div class="docs-callouts">
                                <article id="zero-jitter" class="docs-feature-card">
                                    <p class="docs-feature-tag">"Zero Jitter"</p>
                                    <h3>"Demos load as interactive runtime content, not pre-rendered video."</h3>
                                    <p>
                                        "That keeps stories crisp even when command output changes, the network is slow, or the demo is embedded in another site."
                                    </p>
                                </article>

                                <article id="fully-interactive" class="docs-feature-card">
                                    <p class="docs-feature-tag">"Fully Interactive"</p>
                                    <h3>"Visitors can type, step through, or follow the scripted flow."</h3>
                                    <p>
                                        "The same authoring model supports guided playback and direct interaction, which makes the content useful for both demos and documentation."
                                    </p>
                                </article>

                                <article id="lightweight" class="docs-feature-card">
                                    <p class="docs-feature-tag">"Lightweight"</p>
                                    <h3>"The runtime is small enough to drop into existing sites."</h3>
                                    <p>
                                        "Teams can reuse the player without shipping a full web app shell or a heavyweight media pipeline."
                                    </p>
                                </article>
                            </div>
                        </section>

                        <section id="quick-start" class="docs-section">
                            <p class="docs-section-index">"02"</p>
                            <h2>"Quick Start"</h2>
                            <p>
                                "Publish a demo, copy the embed bootstrap, and mount the player surface where your site should display the demo. The script loads the runtime; the custom element is the host surface for the published experience."
                            </p>

                            <DocCodeBlock
                                label="embed.html"
                                code="<script\n  src=\"https://cdn.simucli.dev/embed.js\"\n  data-demo=\"your-published-demo\"\n  data-api-base=\"https://api.simucli.dev\"\n  defer></script>\n\n<simu-cli-player\n  demo-id=\"your-published-demo\"\n  api-base=\"https://api.simucli.dev\"></simu-cli-player>"
                            />

                            <div class="docs-note-grid">
                                <article class="docs-note-card">
                                    <h3>"1. Publish first"</h3>
                                    <p>"Only published demos should be shared publicly. The publish step creates the stable reference that embedding uses."</p>
                                </article>
                                <article class="docs-note-card">
                                    <h3>"2. Match the slug"</h3>
                                    <p>"Keep the demo identifier aligned between your dashboard and the embed markup so the player resolves the right runtime content."</p>
                                </article>
                                <article class="docs-note-card">
                                    <h3>"3. Verify the origin"</h3>
                                    <p>"Point the runtime at the correct API base for your environment, especially when moving from local development to production."</p>
                                </article>
                            </div>
                        </section>

                        <section id="creating-projects" class="docs-section">
                            <p class="docs-section-index">"03"</p>
                            <h2>"Workspace: Creating Projects"</h2>
                            <p>
                                "Projects are optional organization labels. Use them to group demos by product area, launch campaign, customer segment, or experiment."
                            </p>
                            <ul class="docs-bullets">
                                <li>"Create a project when you want a clean namespace for related demos."</li>
                                <li>"Use a short name that reads well in URLs and sidebar navigation."</li>
                                <li>"Add a description if your team needs a one-line purpose statement."</li>
                            </ul>
                        </section>

                        <section id="managing-demos" class="docs-section">
                            <p class="docs-section-index">"04"</p>
                            <h2>"Workspace: Managing Demos"</h2>
                            <p>
                                "Each demo can live in a project or remain unassigned. That gives you a simple way to move from an idea to a published asset without reorganizing the entire workspace."
                            </p>
                            <ul class="docs-bullets">
                                <li>"Drafts stay editable until you publish them."</li>
                                <li>"Published demos get stable public references for sharing and embedding."</li>
                                <li>"Analytics stays attached to the demo, so performance data remains tied to the asset people actually see."</li>
                            </ul>
                        </section>

                        <section id="recording-casts" class="docs-section">
                            <p class="docs-section-index">"05"</p>
                            <h2>"Editor: Recording Casts"</h2>
                            <p>
                                "Recorded casts are best when you want to capture a known sequence of terminal commands and responses. The editor helps you step through that flow exactly as a user would experience it."
                            </p>
                        </section>

                        <section id="manual-authoring" class="docs-section">
                            <p class="docs-section-index">"06"</p>
                            <h2>"Editor: Manual Authoring"</h2>
                            <p>
                                "Manual authoring is ideal when you want tighter control over the demo narrative. You can tune pacing, comments, prompts, and outputs without fighting a timeline recorder."
                            </p>
                        </section>

                        <section id="step-configuration" class="docs-section">
                            <p class="docs-section-index">"07"</p>
                            <h2>"Editor: Step Configuration"</h2>
                            <p>
                                "A demo step should tell the player exactly what to do next and exactly what the audience should see. Keep the step definitions small, explicit, and ordered."
                            </p>
                            <ul class="docs-bullets">
                                <li>"Command steps drive the shell interaction."</li>
                                <li>"Output and comment steps explain what changed."</li>
                                <li>"Pause and prompt steps keep the user in control of timing."</li>
                            </ul>
                        </section>

                        <section id="deploying" class="docs-section">
                            <p class="docs-section-index">"08"</p>
                            <h2>"Publishing: Deploying"</h2>
                            <p>
                                "Publishing turns a draft demo into a stable public asset. Once published, you can route visitors to the share page, embed it in a documentation site, or surface it in a campaign landing page."
                            </p>
                        </section>

                        <section id="analytics" class="docs-section">
                            <p class="docs-section-index">"09"</p>
                            <h2>"Publishing: Analytics"</h2>
                            <p>
                                "Analytics helps you understand where users stop, what steps get replayed, and which demos are actually driving engagement. That gives product and marketing teams a practical feedback loop instead of guesswork."
                            </p>
                        </section>

                        <section id="script-iframe" class="docs-section">
                            <p class="docs-section-index">"10"</p>
                            <h2>"Integration: Script / Iframe"</h2>
                            <p>
                                "Script-based embedding is the lightest way to ship the runtime. If your platform is stricter about isolation, an iframe gives you a clean sandbox while keeping the demo interactive."
                            </p>
                            <DocCodeBlock
                                label="iframe.html"
                                code="<iframe\n  src=\"https://docs.example.com/embed/demo-slug\"\n  loading=\"lazy\"\n  title=\"SimuCLI demo\"\n  style=\"width:100%;height:640px;border:0;\"></iframe>"
                            />
                        </section>

                        <section id="react-vue-wrappers" class="docs-section docs-section--last">
                            <p class="docs-section-index">"11"</p>
                            <h2>"Integration: React / Vue Wrappers"</h2>
                            <p>
                                "Framework wrappers are just convenience layers. They should load the same script-backed runtime and forward the demo identifier, API base, and sizing rules into the player surface."
                            </p>
                            <p>
                                "Keep wrappers thin so the platform stays portable and the embed remains easy to reason about during debugging."
                            </p>
                        </section>
                    </article>
                </main>

                <aside class="docs-toc" aria-label="On this page">
                    <div class="docs-toc-inner panel">
                        <p class="docs-toc-label">"On this page"</p>
                        <nav class="docs-toc-nav" aria-label="Documentation table of contents">
                            <For
                                each=move || DOC_TOC.to_vec()
                                key=|item| item.label
                                children=move |item| {
                                    let href = format!("/docs{}", item.href);
                                    view! {
                                        <A attr:class="docs-toc-link" href=href>
                                            {item.label}
                                        </A>
                                    }
                                }
                            />
                        </nav>
                    </div>
                </aside>
            </div>
        </section>
    }
}

#[component]
fn DocCodeBlock(label: &'static str, code: &'static str) -> impl IntoView {
    view! {
        <figure class="doc-code-block">
            <figcaption class="doc-code-header">
                <span class="doc-code-badge">">_"</span>
                <span class="doc-code-label">{label}</span>
            </figcaption>
            <pre class="doc-code-pre"><code class="doc-code">{code}</code></pre>
        </figure>
    }
}