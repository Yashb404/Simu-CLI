use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::demo::{DemoSettings, Step, Theme};
use crate::validation::{is_valid_slug, MAX_STEPS};

fn validate_slug(value: &str) -> Result<(), validator::ValidationError> {
    if is_valid_slug(value) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_slug");
        err.message = Some("slug must be 3-60 chars: lowercase letters, digits, hyphens".into());
        Err(err)
    }
}

fn validate_steps(value: &Vec<Step>) -> Result<(), validator::ValidationError> {
    if value.len() <= MAX_STEPS {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("too_many_steps");
        err.message = Some(format!("steps cannot exceed {} entries", MAX_STEPS).into());
        Err(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDemoRequest {
    #[validate(length(min = 1, max = 120))]
    pub title: String,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateDemoRequest {
    #[validate(length(min = 1, max = 120))]
    pub title: Option<String>,
    #[validate(custom(function = "validate_slug"))]
    pub slug: Option<String>,
    pub theme: Option<Theme>,
    pub settings: Option<DemoSettings>,
    #[validate(custom(function = "validate_steps"))]
    pub steps: Option<Vec<Step>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicDemoResponse {
    pub id: Uuid,
    pub slug: Option<String>,
    pub version: i32,
    pub theme: Theme,
    pub settings: DemoSettings,
    pub steps: Vec<Step>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::MAX_STEPS;

    #[test]
    fn create_demo_request_rejects_empty_title() {
        let request = CreateDemoRequest {
            title: "".to_string(),
            project_id: None,
        };

        let result = request.validate();
        assert!(result.is_err(), "empty title should fail validation");
    }

    #[test]
    fn update_demo_request_rejects_invalid_slug() {
        let request = UpdateDemoRequest {
            title: None,
            slug: Some("Invalid Slug".to_string()),
            theme: None,
            settings: None,
            steps: None,
        };

        let result = request.validate();
        assert!(result.is_err(), "invalid slug should fail validation");
    }

    #[test]
    fn update_demo_request_rejects_too_many_steps() {
        let steps = (0..(MAX_STEPS + 1))
            .map(|_| Step {
                id: Uuid::new_v4(),
                step_type: crate::models::demo::StepType::Comment,
                order: 0,
                input: None,
                match_mode: None,
                match_pattern: None,
                description: Some("note".to_string()),
                output: None,
                prompt_config: None,
                spinner_config: None,
                cta_config: None,
                delay_ms: 0,
                typing_speed_ms: 0,
                skippable: true,
            })
            .collect::<Vec<_>>();

        let request = UpdateDemoRequest {
            title: None,
            slug: Some("valid-slug".to_string()),
            theme: None,
            settings: None,
            steps: Some(steps),
        };

        let result = request.validate();
        assert!(result.is_err(), "too many steps should fail validation");
    }
}
