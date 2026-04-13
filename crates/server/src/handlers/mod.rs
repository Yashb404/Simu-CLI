pub mod analytics;
pub mod auth;
pub mod billing;
pub mod common_errors;
pub mod dashboard;
pub mod demos;
pub mod owned_demo;
pub mod projects;

/// Default and maximum page sizes used across list endpoints.
pub const DEFAULT_PAGE_LIMIT: i64 = 50;
pub const MAX_PAGE_LIMIT: i64 = 100;

/// Clamps pagination parameters to safe bounds.
///
/// Both `limit` and `offset` fall back to sensible defaults when absent,
/// and are clamped so callers cannot request unbounded result sets or use
/// negative offsets.
pub fn sanitize_pagination(limit: Option<i64>, offset: Option<i64>) -> (i64, i64) {
    let clamped_limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).clamp(1, MAX_PAGE_LIMIT);
    let clamped_offset = offset.unwrap_or(0).max(0);
    (clamped_limit, clamped_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_pagination_applies_defaults_and_bounds() {
        assert_eq!(sanitize_pagination(None, None), (50, 0));
        assert_eq!(sanitize_pagination(Some(10), Some(5)), (10, 5));
        assert_eq!(sanitize_pagination(Some(1000), Some(-9)), (100, 0));
        assert_eq!(sanitize_pagination(Some(0), Some(0)), (1, 0));
    }
}
