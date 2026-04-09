use serde::{Deserialize, Serialize};

#[cfg(feature = "backend")]
use validator::Validate;

#[cfg_attr(feature = "backend", derive(Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    #[cfg_attr(feature = "backend", validate(length(min = 1, max = 80)))]
    pub name: String,
    #[cfg_attr(feature = "backend", validate(length(max = 500)))]
    pub description: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    #[cfg_attr(feature = "backend", validate(length(min = 1, max = 80)))]
    pub name: Option<String>,
    #[cfg_attr(feature = "backend", validate(length(max = 500)))]
    pub description: Option<String>,
}

#[cfg(all(test, feature = "backend"))]
mod tests {
    use super::*;

    #[test]
    fn create_project_request_rejects_empty_name() {
        let request = CreateProjectRequest {
            name: "".to_string(),
            description: None,
        };

        let result = request.validate();
        assert!(result.is_err(), "empty project name should fail validation");
    }

    #[test]
    fn update_project_request_rejects_long_description() {
        let request = UpdateProjectRequest {
            name: Some("Demo".to_string()),
            description: Some("x".repeat(501)),
        };

        let result = request.validate();
        assert!(
            result.is_err(),
            "description >500 chars should fail validation"
        );
    }
}
