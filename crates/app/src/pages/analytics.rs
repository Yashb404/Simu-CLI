use leptos::prelude::*;

use crate::components::charts::{funnel::FunnelChart, timeseries::TimeSeriesChart};

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    view! {
        <section class="page analytics-page">
            <h2>"Analytics"</h2>
            <p>"Track views, interactions, and completion trends."</p>
            <div class="analytics-grid">
                <TimeSeriesChart title="Events Over Time" />
                <FunnelChart title="Step Funnel" />
            </div>
        </section>
    }
}
