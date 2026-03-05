use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EngineMode { Sequential, FreePlay }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Command, Output, Prompt, Spinner,
    Comment, Clear, Pause, Cta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode { Exact, Fuzzy, Wildcard, Any }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputStyle {
    Normal, Success, Error, Warning,
    Muted, Bold, Code,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputLine {
    pub text: String,
    pub style: OutputStyle,
    pub color: Option<String>,   // hex override e.g. "#FF6B6B"
    pub prefix: Option<String>,  // "✓", "✗", "→" etc
    pub indent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpinnerStyle { Dots, Bar, Braille, Line }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinnerConfig {
    pub style: SpinnerStyle, 
    pub label: String,
    pub duration_ms: u32,
    pub finish_text: String,
    pub finish_style: OutputStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptType { Confirm, Input, Password, Select }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub prompt_type: PromptType,
    pub question: String,
    pub choices: Option<Vec<String>>,
    pub default_value: Option<String>,
    // For confirm type: what output to show for Y vs N
    pub yes_output: Option<Vec<OutputLine>>,
    pub no_output: Option<Vec<OutputLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtaConfig {
    pub message: String,
    pub primary_label: String,
    pub primary_url: String,
    pub secondary_label: Option<String>,
    pub secondary_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: Uuid,
    pub step_type: StepType,
    pub order: i32,
    // Command steps
    pub input: Option<String>,
    pub match_mode: Option<MatchMode>,
    pub match_pattern: Option<String>,  // compiled wildcard → stored as regex
    pub description: Option<String>,    // shown in auto-generated help
    // Output steps
    pub output: Option<Vec<OutputLine>>,
    // Specialized step configs
    pub prompt_config: Option<PromptConfig>,
    pub spinner_config: Option<SpinnerConfig>,
    pub cta_config: Option<CtaConfig>,
    // Timing
    pub delay_ms: u32,
    pub typing_speed_ms: u32,
    pub skippable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowStyle { MacOs, Linux, Windows, None }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub window_style: WindowStyle, 
    pub window_title: String,
    pub preset: Option<String>,     // "dracula", "monokai", etc.
    pub bg_color: String,
    pub fg_color: String,
    pub cursor_color: String,
    pub font_family: String,
    pub font_size: u8,
    pub line_height: f32,
    pub prompt_string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoSettings {
    pub engine_mode: EngineMode,
    pub autoplay: bool,
    pub loop_demo: bool,
    pub loop_delay_ms: u32,
    pub show_restart_button: bool,
    pub show_hints: bool,
    pub not_found_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demo {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub project_id: Option<Uuid>,
    pub title: String,
    pub slug: Option<String>,
    pub published: bool,
    pub version: i32,
    pub theme: Theme,
    pub settings: DemoSettings,
    pub steps: Vec<Step>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}