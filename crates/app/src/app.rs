use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{A, Route, Router, Routes},
    path,
};

use crate::pages::{
    analytics::AnalyticsPage,
    demo_editor::DemoEditorPage,
    demo_share::ShareDemoPage,
    demo_view::DemoViewPage,
    demos::DemosPage,
    publish::PublishPage,
    projects::ProjectsPage,
    settings::SettingsPage,
};
use crate::api;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (is_logging_in, set_logging_in) = signal(false);

    view! {
        <Title text="CLI Demo Studio" />
        <Router>
            <main class="dashboard-shell">
                <aside class="app-sidebar">
                    <div class="sidebar-brand">
                        <p class="kicker">"CLI Demo Studio"</p>
                        <h1>"Dashboard"</h1>
                        <p>"Build, publish, and track CLI walkthroughs."</p>
                    </div>

                    <nav class="app-nav">
                        <A href="/dashboard/projects">"Projects"</A>
                        <A href="/dashboard/demos">"Demos"</A>
                    </nav>

                    <a
                        class="button btn-primary sidebar-login"
                        href={api::login_url()}
                        on:click=move |_| set_logging_in.set(true)
                    >
                        {move || if is_logging_in.get() { "Redirecting..." } else { "Login via GitHub" }}
                    </a>
                </aside>

                <section class="app-content">
                    <Routes fallback=|| view! { <p>"Not Found"</p> }>
                        <Route path=path!("/") view=ProjectsPage />
                        <Route path=path!("/dashboard") view=ProjectsPage />
                        <Route path=path!("/dashboard/projects") view=ProjectsPage />
                        <Route path=path!("/dashboard/demos") view=DemosPage />
                        <Route path=path!("/dashboard/demos/:id") view=DemoEditorPage />
                        <Route path=path!("/dashboard/demos/:id/settings") view=SettingsPage />
                        <Route path=path!("/dashboard/demos/:id/publish") view=PublishPage />
                        <Route path=path!("/dashboard/demos/:id/analytics") view=AnalyticsPage />
                        <Route path=path!("/d/:slug") view=ShareDemoPage />
                        <Route path=path!("/demo/view") view=DemoViewPage />
                    </Routes>
                </section>
            </main>
        </Router>
    }
}
