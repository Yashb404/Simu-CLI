use leptos::prelude::*;

#[component]
pub fn TimeSeriesChart(title: &'static str) -> impl IntoView {
    view! {
        <article class="chart timeseries-chart">
            <h4>{title}</h4>
            <div class="chart-placeholder">"Time-series chart placeholder"</div>
        </article>
    }
}
