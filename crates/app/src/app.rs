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

    view! {
        <Title text="CLI Demo Studio" />
        <Router>
            <main class="app-shell">
                <header class="app-header">
                    <div>
                        <p class="kicker">"CLI Demo Studio"</p>
                        <h1>"Terminal Demo Dashboard"</h1>
                        <p>"Build, publish, and track CLI walkthroughs."</p>
                    </div>
                    <a class="button ghost" href={api::login_url()}>"Login with GitHub"</a>
                </header>

                <nav class="app-nav">
                    <A href="/dashboard/projects">"Projects"</A>
                    <A href="/dashboard/demos">"Demos"</A>
                </nav>

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
            </main>
        </Router>
    }
}
