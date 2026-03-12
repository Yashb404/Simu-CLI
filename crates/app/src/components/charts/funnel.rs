use leptos::prelude::*;

#[component]
pub fn FunnelChart(title: &'static str) -> impl IntoView {
    view! {
        <article class="chart funnel-chart">
            <h4>{title}</h4>
            <div class="chart-placeholder">"Funnel chart placeholder"</div>
        </article>
    }
}
