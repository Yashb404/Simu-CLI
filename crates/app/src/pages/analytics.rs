use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use crate::api;

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let params = use_params_map();
    let demo_id = move || {
        params
            .read()
            .get("id")
            .unwrap_or_else(|| "unknown".to_string())
    };

    let (series, set_series) = signal(Vec::<api::AnalyticsSeriesPoint>::new());
    let (referrers, set_referrers) = signal(Vec::<api::ReferrerCount>::new());
    let (funnel, set_funnel) = signal(Vec::<api::FunnelPoint>::new());
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let id = demo_id();
        if id == "unknown" {
            set_status.set("Invalid demo id".to_string());
            return;
        }

        spawn_local({
            let set_series = set_series;
            let set_referrers = set_referrers;
            let set_funnel = set_funnel;
            let set_status = set_status;
            async move {
                let series_res = api::get_analytics_series(&id).await;
                let referrers_res = api::get_analytics_referrers(&id).await;
                let funnel_res = api::get_analytics_funnel(&id).await;

                match (series_res, referrers_res, funnel_res) {
                    (Ok(series_data), Ok(referrer_data), Ok(funnel_data)) => {
                        set_series.set(series_data);
                        set_referrers.set(referrer_data);
                        set_funnel.set(funnel_data);
                        set_status.set("Analytics loaded".to_string());
                    }
                    _ => set_status.set("Failed to load analytics data".to_string()),
                }
            }
        });
    });

    view! {
        <section class="page analytics-page">
            <h2>"Analytics"</h2>
            <p>"Track views, interactions, and completion trends."</p>
            <p class="status">{move || status.get()}</p>

            <div class="analytics-grid">
                <section class="panel">
                    <h3>"Events Over Time"</h3>
                    <ul class="list">
                        <For
                            each=move || series.get()
                            key=|point| format!("{}:{}", point.bucket, point.event_type)
                            children=move |point| {
                                view! {
                                    <li>
                                        <span>{format!("{} - {}", point.bucket, point.event_type)}</span>
                                        <strong>{point.total}</strong>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </section>

                <section class="panel">
                    <h3>"Top Referrers"</h3>
                    <ul class="list">
                        <For
                            each=move || referrers.get()
                            key=|item| item.referrer.clone()
                            children=move |item| {
                                view! {
                                    <li>
                                        <span>{item.referrer}</span>
                                        <strong>{item.total}</strong>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </section>

                <section class="panel">
                    <h3>"Step Funnel"</h3>
                    <ul class="list">
                        <For
                            each=move || funnel.get()
                            key=|item| item.step_index
                            children=move |item| {
                                view! {
                                    <li>
                                        <span>{format!("Step {}", item.step_index)}</span>
                                        <strong>{item.total}</strong>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </section>
            </div>
        </section>
    }
}
