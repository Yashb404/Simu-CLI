use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::types::Json as SqlxJson;
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::{AuthUser, USER_SESSION_KEY},
    error::{ApiError, HandlerResult},
    handlers::owned_demo::OwnedDemo,
    handlers::sanitize_pagination,
    services,
    state::AppState,
};
use shared::{
    dto::{CreateDemoRequest, PublicDemoResponse, UpdateDemoRequest},
    error::AppError,
    models::demo::{
        Demo, DemoSettings, EngineMode, OutputLine, OutputStyle, Step, StepType, Theme, WindowStyle,
    },
};

#[derive(Debug, Deserialize)]
pub struct ListMyDemosQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub project_id: Option<Uuid>,
    pub published: Option<bool>,
}

fn default_theme() -> Theme {
    Theme {
        window_style: WindowStyle::MacOs,
        window_title: "Terminal".to_string(),
        preset: Some("default".to_string()),
        bg_color: "#111827".to_string(),
        fg_color: "#e5e7eb".to_string(),
        cursor_color: "#f9fafb".to_string(),
        font_family: "JetBrains Mono".to_string(),
        font_size: 14,
        line_height: 1.4,
        prompt_string: "$".to_string(),
    }
}

fn default_settings() -> DemoSettings {
    DemoSettings {
        engine_mode: EngineMode::Sequential,
        autoplay: false,
        loop_demo: false,
        loop_delay_ms: 800,
        show_restart_button: true,
        show_hints: false,
        not_found_message: "command not found".to_string(),
    }
}

fn to_engine_mode_db(engine_mode: &EngineMode) -> &'static str {
    match engine_mode {
        EngineMode::Sequential => "sequential",
        EngineMode::FreePlay => "free_play",
    }
}

fn slugify(value: &str) -> String {
    let mut slug = value
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();

    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    slug.trim_matches('-').chars().take(60).collect()
}

#[derive(Serialize)]
pub struct PublishDemoResponse {
    pub id: Uuid,
    pub slug: String,
    pub version: i32,
    pub public_url: String,
}

pub async fn create_demo(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateDemoRequest>,
) -> HandlerResult<(StatusCode, Json<Demo>)> {
    payload.validate()?;

    if let Some(project_id) = payload.project_id {
        let project_exists: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM projects WHERE id = $1 AND owner_id = $2")
                .bind(project_id)
                .bind(user.id)
                .fetch_optional(&state.db)
                .await?;

        if project_exists.is_none() {
            return Err(ApiError(AppError::Validation(
                "project does not exist or is not owned by user".to_string(),
            )));
        }
    }

    let theme = default_theme();
    let settings = default_settings();
    let steps: Vec<Step> = Vec::new();

    let demo = sqlx::query_as::<_, Demo>(
        r#"
        INSERT INTO demos (owner_id, project_id, title, engine_mode, theme, settings, steps)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
                  published, version, created_at, updated_at
        "#,
    )
    .bind(user.id)
    .bind(payload.project_id)
    .bind(payload.title)
    .bind(to_engine_mode_db(&settings.engine_mode))
    .bind(SqlxJson(theme))
    .bind(SqlxJson(settings))
    .bind(SqlxJson(steps))
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(demo)))
}

pub async fn get_demo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    session: Session,
) -> HandlerResult<Json<Demo>> {
    let demo = sqlx::query_as::<_, Demo>(
        r#"
        SELECT id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
               published, version, created_at, updated_at
        FROM demos
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError(AppError::NotFound))?;

    let maybe_user_id: Option<Uuid> = session.get(USER_SESSION_KEY).await.map_err(|e| {
        tracing::error!("Session read failure: {e:?}");
        ApiError(AppError::Internal)
    })?;

    if !demo.published && maybe_user_id != Some(demo.owner_id) {
        return Err(ApiError(AppError::NotFound));
    }

    Ok(Json(demo))
}

pub async fn update_demo(
    State(state): State<AppState>,
    OwnedDemo(existing): OwnedDemo,
    Json(payload): Json<UpdateDemoRequest>,
) -> HandlerResult<Json<Demo>> {
    payload.validate()?;

    let mut title = existing.title;
    let mut project_id = existing.project_id;
    let mut slug = existing.slug;
    let mut theme = existing.theme.clone();
    let mut settings = existing.settings.clone();
    let mut steps = existing.steps.clone();

    if let Some(new_title) = payload.title {
        title = new_title;
    }
    if let Some(project_update) = payload.project_id {
        if let Some(target_project_id) = project_update {
            let project_exists: Option<Uuid> =
                sqlx::query_scalar("SELECT id FROM projects WHERE id = $1 AND owner_id = $2")
                    .bind(target_project_id)
                    .bind(existing.owner_id)
                    .fetch_optional(&state.db)
                    .await?;

            if project_exists.is_none() {
                return Err(ApiError(AppError::Validation(
                    "project does not exist or is not owned by user".to_string(),
                )));
            }
        }
        project_id = project_update;
    }
    if let Some(new_slug) = payload.slug {
        slug = Some(new_slug);
    }
    if let Some(new_theme) = payload.theme {
        theme = new_theme;
    }
    if let Some(new_settings) = payload.settings {
        settings = new_settings;
    }
    if let Some(new_steps) = payload.steps {
        steps = new_steps;
    }

    let updated = sqlx::query_as::<_, Demo>(
        r#"
        UPDATE demos
        SET title = $1,
            project_id = $2,
            slug = $3,
            engine_mode = $4,
            theme = $5,
            settings = $6,
            steps = $7,
            updated_at = NOW()
        WHERE id = $8 AND owner_id = $9
        RETURNING id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
                  published, version, created_at, updated_at
        "#,
    )
    .bind(title)
    .bind(project_id)
    .bind(slug)
    .bind(to_engine_mode_db(&settings.engine_mode))
    .bind(SqlxJson(theme))
    .bind(SqlxJson(settings))
    .bind(SqlxJson(steps))
    .bind(existing.id)
    .bind(existing.owner_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated))
}

pub async fn delete_demo(
    State(state): State<AppState>,
    OwnedDemo(demo): OwnedDemo,
) -> HandlerResult<StatusCode> {
    let result = sqlx::query("DELETE FROM demos WHERE id = $1 AND owner_id = $2")
        .bind(demo.id)
        .bind(demo.owner_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError(AppError::NotFound));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_my_demos(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<ListMyDemosQuery>,
) -> HandlerResult<Json<Vec<Demo>>> {
    let (limit, offset) = sanitize_pagination(query.limit, query.offset);

    let rows = sqlx::query_as::<_, Demo>(
        r#"
        SELECT id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
               published, version, created_at, updated_at
        FROM demos
        WHERE owner_id = $1
          AND ($2::uuid IS NULL OR project_id = $2)
          AND ($3::bool IS NULL OR published = $3)
        ORDER BY updated_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(user.id)
    .bind(query.project_id)
    .bind(query.published)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

pub async fn get_public_demo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> HandlerResult<Response> {
    get_public_demo_by_id_or_slug(&state, Some(id), None).await
}

pub async fn get_public_demo_by_reference(
    State(state): State<AppState>,
    Path(reference): Path<String>,
) -> HandlerResult<Response> {
    let parsed_uuid = Uuid::parse_str(reference.trim()).ok();
    get_public_demo_by_id_or_slug(&state, parsed_uuid, Some(reference)).await
}

async fn get_public_demo_by_id_or_slug(
    state: &AppState,
    id: Option<Uuid>,
    slug: Option<String>,
) -> HandlerResult<Response> {
    let demo = sqlx::query_as::<_, Demo>(
        r#"
        SELECT id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
               published, version, created_at, updated_at
        FROM demos
        WHERE published = TRUE
          AND (
            ($1::uuid IS NOT NULL AND id = $1)
            OR
            ($2::text IS NOT NULL AND slug = $2)
          )
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(id)
    .bind(slug)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError(AppError::NotFound))?;

    let payload = PublicDemoResponse {
        id: demo.id,
        slug: demo.slug.clone(),
        version: demo.version,
        theme: demo.theme.clone(),
        settings: demo.settings.clone(),
        steps: demo.steps.clone(),
    };

    let mut response = Json(payload).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );

    let etag = format!("W/\"{}-{}\"", demo.id, demo.version);
    if let Ok(header_value) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(header::ETAG, header_value);
    }

    Ok(response)
}

pub async fn publish_demo(
    State(state): State<AppState>,
    OwnedDemo(existing): OwnedDemo,
) -> HandlerResult<Json<PublishDemoResponse>> {
    let slug = existing
        .slug
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            let base = slugify(&existing.title);
            if base.is_empty() {
                format!("demo-{}", existing.id.simple())
            } else {
                format!("{}-{}", base, &existing.id.simple().to_string()[..8])
            }
        });

    let updated = sqlx::query_as::<_, Demo>(
        r#"
        UPDATE demos
        SET published = TRUE,
            slug = $1,
            version = version + 1,
            updated_at = NOW()
        WHERE id = $2 AND owner_id = $3
        RETURNING id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
                  published, version, created_at, updated_at
        "#,
    )
    .bind(&slug)
    .bind(existing.id)
    .bind(existing.owner_id)
    .fetch_one(&state.db)
    .await?;

    let public_url = format!("{}/d/{}", state.config.frontend_url, slug);

    Ok(Json(PublishDemoResponse {
        id: updated.id,
        slug,
        version: updated.version,
        public_url,
    }))
}

pub async fn get_demo_og_image(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> HandlerResult<Response> {
    let demo = sqlx::query_as::<_, Demo>(
        r#"
        SELECT id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
               published, version, created_at, updated_at
        FROM demos
        WHERE id = $1 AND published = TRUE
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError(AppError::NotFound))?;

    let svg = services::og_image::generate_og_svg(&demo.title, demo.version);

    let mut response = (StatusCode::OK, svg).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );

    Ok(response)
}

// ── Cast import handler ───────────────────────────────────────────────────────
//
// POST /api/demos/{id}/import-cast
//
// Accepts a `multipart/form-data` body with a single field named `file`
// containing the raw UTF-8 text of an asciinema v2 `.cast` file.
//
// The handler:
//   1. Verifies the demo exists and is owned by the authenticated user.
//   2. Reads and UTF-8-decodes the uploaded file.
//   3. Runs `extract_commands_from_cast` with options derived from the query
//      string.
//   4. Converts each `CommandInteraction` into two `Step` objects
//      (StepType::Command + StepType::Output) and appends them to the demo.
//   5. Returns an `ImportCastResponse` JSON body.

use shared::dto::demo_dto::{ImportCastQuery, ImportCastResponse};

const MAX_CAST_UPLOAD_BYTES: usize = 5 * 1024 * 1024;
pub async fn import_cast(
    State(state): State<AppState>,
    Path(demo_id): Path<Uuid>,
    Query(query): Query<ImportCastQuery>,
    AuthUser(user): AuthUser,
    mut multipart: axum::extract::Multipart,
) -> HandlerResult<impl IntoResponse> {
    let demo = sqlx::query_as::<_, Demo>(
        "SELECT id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps, published, version, created_at, updated_at FROM demos WHERE id = $1 AND owner_id = $2"
    )
    .bind(demo_id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("DB error fetching demo: {:?}", e);
        ApiError(AppError::Internal)
    })?
    .ok_or(ApiError(AppError::NotFound))?;

    // Read the cast file from multipart form
    let cast_text = read_cast_field(&mut multipart).await?;

    // Build parse options from query parameters
    let parse_options = build_parse_options(&query);

    // Parse the cast file
    let interactions = shared::extract_commands_from_cast(&cast_text, &parse_options)
        .map_err(|e| ApiError(AppError::Validation(format!("Cast parsing failed: {}", e))))?;

    // Convert interactions to steps and append to existing steps
    let mut all_steps = demo.steps;
    append_steps_from_interactions(&mut all_steps, &interactions);

    // Enforce step count limit
    if all_steps.len() > shared::validation::MAX_STEPS {
        return Err(ApiError(AppError::Validation(format!(
            "Import would exceed the maximum of {} steps per demo",
            shared::validation::MAX_STEPS
        ))));
    }

    // Update the demo with new steps
    let updated_demo = Demo {
        steps: all_steps,
        ..demo
    };

    sqlx::query(
        "UPDATE demos SET steps = $1, version = version + 1, updated_at = now() WHERE id = $2",
    )
    .bind(SqlxJson(&updated_demo.steps))
    .bind(demo_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("DB error updating demo: {:?}", e);
        ApiError(AppError::Internal)
    })?;

    Ok(Json(ImportCastResponse {
        pairs_imported: interactions.len(),
        message: format!(
            "Successfully imported {} command/output pairs from cast file",
            interactions.len()
        ),
    }))
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Pull the `file` field from the multipart form, decode it as UTF-8.
async fn read_cast_field(multipart: &mut axum::extract::Multipart) -> HandlerResult<String> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Multipart read error in import_cast: {:?}", e);
        ApiError(AppError::Validation("Multipart error".into()))
    })? {
        if let Some(name) = field.name() {
            if name == "file" {
                let file_name = field.file_name().unwrap_or("").to_ascii_lowercase();
                if !file_name.ends_with(".cast") {
                    return Err(ApiError(AppError::Validation(
                        "Only .cast files are accepted".into(),
                    ))
                    .into());
                }

                let data = field.bytes().await.map_err(|e| {
                    tracing::error!("File bytes read error in import_cast: {:?}", e);
                    ApiError(AppError::Validation("Failed to read file data".into()))
                })?;

                if data.len() > MAX_CAST_UPLOAD_BYTES {
                    return Err(ApiError(AppError::Validation(format!(
                        "File too large. Max allowed is {} MB",
                        MAX_CAST_UPLOAD_BYTES / (1024 * 1024)
                    )))
                    .into());
                }
                return String::from_utf8(data.to_vec()).map_err(|e| {
                    ApiError(AppError::Validation(format!("Invalid UTF-8: {}", e))).into()
                });
            }
        }
    }

    Err(ApiError(AppError::Validation(
        "No 'file' field in multipart form".into(),
    ))
    .into())
}

/// Build `ParseOptions` from the caller-supplied query parameters.
fn build_parse_options(query: &ImportCastQuery) -> shared::ParseOptions {
    if !query.strip_trailing_prompt {
        return shared::ParseOptions {
            strip_trailing_prompt: None,
        };
    }

    if query.prompt_patterns.is_empty() {
        // Use heuristic matching
        return shared::ParseOptions {
            strip_trailing_prompt: Some(vec!["".to_string()]),
        };
    }

    shared::ParseOptions {
        strip_trailing_prompt: Some(query.prompt_patterns.clone()),
    }
}

/// Convert CommandInteraction pairs into Step objects and append to existing steps.
/// Returns the number of interactions processed (not the number of steps created).
fn append_steps_from_interactions(
    steps: &mut Vec<Step>,
    interactions: &[shared::CommandInteraction],
) {
    let next_order = steps.iter().map(|s| s.order).max().unwrap_or(-1) + 1;

    for (idx, interaction) in interactions.iter().enumerate() {
        let command_order = next_order + (idx as i32 * 2);
        let output_order = command_order + 1;

        // Create Command step
        let command_step = Step {
            id: Uuid::new_v4(),
            step_type: StepType::Command,
            order: command_order,
            input: Some(interaction.command.clone()),
            match_mode: None,
            match_pattern: None,
            short_description: None,
            description: None,
            output: None,
            prompt_config: None,
            spinner_config: None,
            cta_config: None,
            delay_ms: 0,
            typing_speed_ms: 50,
            skippable: true,
        };

        // Create Output step with the command output as normal text
        let output_step = Step {
            id: Uuid::new_v4(),
            step_type: StepType::Output,
            order: output_order,
            input: None,
            match_mode: None,
            match_pattern: None,
            short_description: None,
            description: None,
            output: Some(
                interaction
                    .output
                    .lines()
                    .map(|line| OutputLine {
                        text: line.to_string(),
                        style: OutputStyle::Normal,
                        color: None,
                        prefix: None,
                        indent: 0,
                    })
                    .collect(),
            ),
            prompt_config: None,
            spinner_config: None,
            cta_config: None,
            delay_ms: 0,
            typing_speed_ms: 0,
            skippable: true,
        };

        steps.push(command_step);
        steps.push(output_step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_mode_maps_to_db_values() {
        assert_eq!(to_engine_mode_db(&EngineMode::Sequential), "sequential");
        assert_eq!(to_engine_mode_db(&EngineMode::FreePlay), "free_play");
    }

    #[test]
    fn default_settings_are_safe() {
        let settings = default_settings();
        assert_eq!(settings.engine_mode, EngineMode::Sequential);
        assert!(!settings.autoplay);
        assert!(!settings.loop_demo);
        assert_eq!(settings.not_found_message, "command not found");
    }

    #[test]
    fn default_theme_has_valid_palette_shape() {
        let theme = default_theme();
        assert_eq!(theme.window_title, "Terminal");
        assert!(theme.bg_color.starts_with('#'));
        assert!(theme.fg_color.starts_with('#'));
        assert!(theme.cursor_color.starts_with('#'));
    }
}
