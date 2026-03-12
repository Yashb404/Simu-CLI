use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 80))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 80))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[cfg(test)]
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
        assert!(result.is_err(), "description >500 chars should fail validation");
    }
}
