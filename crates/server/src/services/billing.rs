use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PlanLimits {
    pub code: String,
    pub max_demos: i32,
    pub max_monthly_views: i64,
}

pub fn limits_for_plan(code: &str) -> PlanLimits {
    match code {
        "pro" => PlanLimits {
            code: "pro".to_string(),
            max_demos: 100_000,
            max_monthly_views: 100_000_000,
        },
        _ => PlanLimits {
            code: "free".to_string(),
            max_demos: 3,
            max_monthly_views: 10_000,
        },
    }
}
