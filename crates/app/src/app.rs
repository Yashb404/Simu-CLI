use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::pages::{
    demo_editor::DemoEditorPage,
    demo_share::ShareDemoPage,
    projects::ProjectsPage,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="CLI Demo Studio" />
        <Router>
            <main class="app-shell">
                <header class="app-header">
                    <h1>"CLI Demo Studio"</h1>
                    <p>"Scriptless terminal demo builder"</p>
                </header>
                <Routes fallback=|| view! { <p>"Not Found"</p> }>
                    <Route path=path!("/") view=ProjectsPage />
                    <Route path=path!("/dashboard") view=ProjectsPage />
                    <Route path=path!("/dashboard/projects") view=ProjectsPage />
                    <Route path=path!("/dashboard/demos/:id") view=DemoEditorPage />
                    <Route path=path!("/d/:slug") view=ShareDemoPage />
                </Routes>
            </main>
        </Router>
    }
}
