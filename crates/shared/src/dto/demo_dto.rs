use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::demo::{DemoSettings, Step, Theme};
#[cfg(feature = "backend")]
use crate::validation::{MAX_OUTPUT_LINES_PER_STEP, MAX_STEPS, is_valid_hex_color, is_valid_slug};

#[cfg(feature = "backend")]
use validator::Validate;

#[cfg(feature = "backend")]
fn validate_slug(value: &str) -> Result<(), validator::ValidationError> {
    if is_valid_slug(value) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_slug");
        err.message = Some("slug must be 3-60 chars: lowercase letters, digits, hyphens".into());
        Err(err)
    }
}

#[cfg(feature = "backend")]
fn validate_steps(value: &Vec<Step>) -> Result<(), validator::ValidationError> {
    if value.len() > MAX_STEPS {
        let mut err = validator::ValidationError::new("too_many_steps");
        err.message = Some(format!("steps cannot exceed {} entries", MAX_STEPS).into());
        return Err(err);
    }

    for step in value {
        if let Some(input) = &step.input
            && (input.trim().is_empty() || input.len() > 200)
        {
            let mut err = validator::ValidationError::new("invalid_step_input");
            err.message = Some("step input must be non-empty and <= 200 chars".into());
            return Err(err);
        }

        if let Some(output_lines) = &step.output {
            if output_lines.len() > MAX_OUTPUT_LINES_PER_STEP {
                let mut err = validator::ValidationError::new("too_many_output_lines");
                err.message = Some(
                    format!(
                        "each step output cannot exceed {} lines",
                        MAX_OUTPUT_LINES_PER_STEP
                    )
                    .into(),
                );
                return Err(err);
            }

            for output_line in output_lines {
                if output_line.text.len() > 500 {
                    let mut err = validator::ValidationError::new("output_line_too_long");
                    err.message = Some("output line text must be <= 500 chars".into());
                    return Err(err);
                }

                if let Some(color) = &output_line.color
                    && !is_valid_hex_color(color)
                {
                    let mut err = validator::ValidationError::new("invalid_output_color");
                    err.message = Some("output line color must be a valid hex color".into());
                    return Err(err);
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "backend")]
fn validate_theme(value: &Theme) -> Result<(), validator::ValidationError> {
    if !is_valid_hex_color(&value.bg_color)
        || !is_valid_hex_color(&value.fg_color)
        || !is_valid_hex_color(&value.cursor_color)
    {
        let mut err = validator::ValidationError::new("invalid_theme_color");
        err.message = Some("theme colors must be valid hex values".into());
        return Err(err);
    }

    if value.font_size < 8 || value.font_size > 32 {
        let mut err = validator::ValidationError::new("invalid_font_size");
        err.message = Some("theme font_size must be between 8 and 32".into());
        return Err(err);
    }

    Ok(())
}

#[cfg(feature = "backend")]
fn validate_settings(value: &DemoSettings) -> Result<(), validator::ValidationError> {
    if value.loop_delay_ms > 60_000 {
        let mut err = validator::ValidationError::new("invalid_loop_delay");
        err.message = Some("loop_delay_ms must be <= 60000".into());
        return Err(err);
    }

    if value.not_found_message.trim().is_empty() || value.not_found_message.len() > 200 {
        let mut err = validator::ValidationError::new("invalid_not_found_message");
        err.message = Some("not_found_message must be non-empty and <= 200 chars".into());
        return Err(err);
    }

    if let Some(url) = &value.documentation_url {
        let trimmed = url.trim();
        if trimmed.len() > 500 {
            let mut err = validator::ValidationError::new("invalid_documentation_url");
            err.message = Some("documentation_url must be <= 500 chars".into());
            return Err(err);
        }

        if !(trimmed.is_empty()
            || trimmed.starts_with("https://")
            || trimmed.starts_with("http://"))
        {
            let mut err = validator::ValidationError::new("invalid_documentation_url");
            err.message =
                Some("documentation_url must start with http:// or https:// when provided".into());
            return Err(err);
        }
    }

    Ok(())
}

#[cfg_attr(feature = "backend", derive(Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDemoRequest {
    #[cfg_attr(feature = "backend", validate(length(min = 1, max = 120)))]
    pub title: String,
    pub project_id: Option<Uuid>,
}

#[cfg_attr(feature = "backend", derive(Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDemoRequest {
    #[cfg_attr(feature = "backend", validate(length(min = 1, max = 120)))]
    pub title: Option<String>,
    pub project_id: Option<Option<Uuid>>,
    #[cfg_attr(feature = "backend", validate(custom(function = "validate_slug")))]
    pub slug: Option<String>,
    #[cfg_attr(feature = "backend", validate(custom(function = "validate_theme")))]
    pub theme: Option<Theme>,
    #[cfg_attr(feature = "backend", validate(custom(function = "validate_settings")))]
    pub settings: Option<DemoSettings>,
    #[cfg_attr(feature = "backend", validate(custom(function = "validate_steps")))]
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

#[cfg(all(test, feature = "backend"))]
mod tests {
    use super::*;
    use crate::{
        models::demo::{EngineMode, OutputLine, OutputStyle, WindowStyle},
        validation::MAX_STEPS,
    };

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
            project_id: None,
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
                short_description: None,
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
            project_id: None,
            slug: Some("valid-slug".to_string()),
            theme: None,
            settings: None,
            steps: Some(steps),
        };

        let result = request.validate();
        assert!(result.is_err(), "too many steps should fail validation");
    }

    #[test]
    fn update_demo_request_rejects_invalid_theme_color() {
        let request = UpdateDemoRequest {
            title: None,
            project_id: None,
            slug: Some("valid-slug".to_string()),
            theme: Some(Theme {
                window_style: WindowStyle::MacOs,
                window_title: "Terminal".to_string(),
                preset: None,
                bg_color: "not-a-color".to_string(),
                fg_color: "#ffffff".to_string(),
                cursor_color: "#00ff00".to_string(),
                font_family: "JetBrains Mono".to_string(),
                font_size: 14,
                line_height: 1.4,
                prompt_string: "$".to_string(),
            }),
            settings: None,
            steps: None,
        };

        let result = request.validate();
        assert!(
            result.is_err(),
            "invalid theme colors should fail validation"
        );
    }

    #[test]
    fn update_demo_request_rejects_too_many_output_lines_in_step() {
        let output = (0..(MAX_OUTPUT_LINES_PER_STEP + 1))
            .map(|_| OutputLine {
                text: "line".to_string(),
                style: OutputStyle::Normal,
                color: None,
                prefix: None,
                indent: 0,
            })
            .collect::<Vec<_>>();

        let steps = vec![Step {
            id: Uuid::new_v4(),
            step_type: crate::models::demo::StepType::Output,
            order: 1,
            input: None,
            match_mode: None,
            match_pattern: None,
            short_description: None,
            description: Some("output".to_string()),
            output: Some(output),
            prompt_config: None,
            spinner_config: None,
            cta_config: None,
            delay_ms: 0,
            typing_speed_ms: 0,
            skippable: true,
        }];

        let request = UpdateDemoRequest {
            title: None,
            project_id: None,
            slug: Some("valid-slug".to_string()),
            theme: None,
            settings: Some(DemoSettings {
                engine_mode: EngineMode::Sequential,
                autoplay: false,
                loop_demo: false,
                loop_delay_ms: 500,
                show_restart_button: true,
                show_hints: false,
                not_found_message: "command not found".to_string(),
                documentation_url: None,
            }),
            steps: Some(steps),
        };

        let result = request.validate();
        assert!(
            result.is_err(),
            "too many output lines should fail validation"
        );
    }
}

// ── Cast import DTOs ──────────────────────────────────────────────────────────

/// Query parameters accepted by `POST /api/demos/{id}/import-cast`.
///
/// All fields are optional so the endpoint can be called with defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportCastQuery {
    /// When `true`, the parser will attempt to strip the trailing shell prompt
    /// from each output block. Default is `true`.
    #[serde(default = "default_strip_prompt")]
    pub strip_trailing_prompt: bool,

    /// Optional list of prompt patterns to strip. If empty, activates heuristic
    /// matching (looks for `$`, `#`, `%`, `>`). See [`cast_parser::strip_trailing_prompt`]
    /// for full semantics.
    #[serde(default)]
    pub prompt_patterns: Vec<String>,
}

fn default_strip_prompt() -> bool {
    true
}

/// Response body returned by `POST /api/demos/{id}/import-cast`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCastResponse {
    /// How many command→output pairs were successfully extracted and persisted.
    pub pairs_imported: usize,
    /// User-facing status message.
    pub message: String,
}
