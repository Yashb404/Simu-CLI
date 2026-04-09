use anyhow::{Context, Result};
use std::env;
use time::Duration;

/// Wrapper for secrets that should not appear in debug logs
#[derive(Clone)]
pub struct Secret(pub String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Application configuration loaded from environment variables at startup
#[derive(Clone, Debug)]
pub struct Config {
    /// PostgreSQL connection URL
    pub database_url: String,

    /// GitHub OAuth client ID
    pub github_client_id: String,

    /// GitHub OAuth client secret
    pub github_client_secret: Secret,

    /// Session encryption secret (64-byte hex string)
    pub session_secret: Secret,

    /// API server URL (used in OAuth redirect URI)
    pub api_url: String,

    /// Frontend URL (redirect after auth)
    pub frontend_url: String,

    /// Server port
    pub port: u16,

    /// Rate limiting: requests per minute per IP
    pub rate_limit_requests_per_minute: u32,

    /// Maximum number of DB pool connections
    pub db_max_connections: u32,

    /// Session timeout duration
    pub session_timeout: Duration,

    /// Whether session cookies must be marked secure
    pub session_cookie_secure: bool,

    /// Logging level (defaults to "server=debug,tower_sessions=debug")
    pub log_level: String,

    /// Allowed CORS origins for browser clients.
    pub cors_allowed_origins: Vec<String>,
}

fn normalize_origin(url: &str) -> Result<String> {
    let parsed =
        reqwest::Url::parse(url).with_context(|| format!("invalid URL for origin: {url}"))?;

    let scheme = parsed.scheme();
    let host = parsed
        .host_str()
        .context("origin URL must include a host")?;
    let host = host.trim_end_matches('.');
    if host.is_empty() {
        anyhow::bail!("origin URL host is empty after normalization");
    }

    let origin = match parsed.port() {
        Some(port) => format!("{scheme}://{host}:{port}"),
        None => format!("{scheme}://{host}"),
    };

    Ok(origin)
}

fn loopback_aliases(origin: &str) -> Result<Vec<String>> {
    let parsed =
        reqwest::Url::parse(origin).with_context(|| format!("invalid origin URL: {origin}"))?;

    let scheme = parsed.scheme();
    let host = parsed
        .host_str()
        .context("origin URL must include a host")?;
    let host = host.trim_end_matches('.');

    let port_suffix = parsed
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();

    let mut aliases = vec![origin.to_string()];

    if host == "localhost" {
        aliases.push(format!("{scheme}://localhost{port_suffix}"));
        aliases.push(format!("{scheme}://127.0.0.1{port_suffix}"));
        aliases.push(format!("{scheme}://[::1]{port_suffix}"));
    } else if host == "127.0.0.1" || host == "::1" || host == "[::1]" {
        aliases.push(format!("{scheme}://localhost{port_suffix}"));
    }

    aliases.sort();
    aliases.dedup();
    Ok(aliases)
}

fn parse_cors_allowed_origins(raw: &str) -> Result<Vec<String>> {
    let mut origins = Vec::new();
    for item in raw.split(',') {
        let value = item.trim();
        if value.is_empty() {
            continue;
        }
        let normalized = normalize_origin(value)?;
        origins.extend(loopback_aliases(&normalized)?);
    }

    if origins.is_empty() {
        anyhow::bail!("CORS_ALLOWED_ORIGINS must include at least one origin");
    }

    origins.sort();
    origins.dedup();
    Ok(origins)
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// Returns an error when required environment variables are missing or invalid.
    /// The caller should fail startup on error so configuration problems are caught early.
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

        let github_client_id =
            env::var("GITHUB_CLIENT_ID").context("GITHUB_CLIENT_ID must be set")?;

        let github_client_secret =
            env::var("GITHUB_CLIENT_SECRET").context("GITHUB_CLIENT_SECRET must be set")?;

        let session_secret = env::var("SESSION_SECRET").context(
            "SESSION_SECRET must be set (generate with: head -c 32 /dev/urandom | xxd -p)",
        )?;

        // Validate session secret is valid hex and correct length
        if session_secret.len() != 64 {
            anyhow::bail!(
                "SESSION_SECRET must be 64 hex characters (32 bytes), got {} chars",
                session_secret.len()
            );
        }
        if session_secret.chars().any(|c| !c.is_ascii_hexdigit()) {
            anyhow::bail!("SESSION_SECRET must be valid hex characters");
        }

        let api_url = match env::var("API_URL") {
            Ok(value) => value,
            Err(_) => env::var("RENDER_EXTERNAL_URL")
                .context("API_URL must be set (or RENDER_EXTERNAL_URL when running on Render)")?,
        };

        let frontend_url = match env::var("FRONTEND_URL") {
            Ok(value) => value,
            Err(_) => env::var("RENDER_EXTERNAL_URL").context(
                "FRONTEND_URL must be set (or RENDER_EXTERNAL_URL when running on Render)",
            )?,
        };

        let cors_allowed_origins = match env::var("CORS_ALLOWED_ORIGINS") {
            Ok(value) => parse_cors_allowed_origins(&value)
                .context("CORS_ALLOWED_ORIGINS must be a comma-separated list of valid origins")?,
            Err(_) => {
                let normalized = normalize_origin(&frontend_url)
                    .context("FRONTEND_URL must be a valid origin URL")?;
                loopback_aliases(&normalized)?
            }
        };

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .context("PORT must be a valid u16")?;

        let rate_limit_requests_per_minute = env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .context("RATE_LIMIT_REQUESTS_PER_MINUTE must be a valid u32")?;

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .context("DB_MAX_CONNECTIONS must be a valid u32")?;

        let session_timeout_days = env::var("SESSION_TIMEOUT_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse::<i64>()
            .context("SESSION_TIMEOUT_DAYS must be a valid i64")?;

        let session_cookie_secure = env::var("SESSION_COOKIE_SECURE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .context("SESSION_COOKIE_SECURE must be true or false")?;

        let log_level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "server=debug,tower_sessions=debug".to_string());

        Ok(Config {
            database_url,
            github_client_id,
            github_client_secret: Secret(github_client_secret),
            session_secret: Secret(session_secret),
            api_url,
            frontend_url,
            port,
            rate_limit_requests_per_minute,
            db_max_connections,
            session_timeout: Duration::days(session_timeout_days),
            session_cookie_secure,
            log_level,
            cors_allowed_origins,
        })
    }

    /// OAuth redirect URI for GitHub
    pub fn github_redirect_uri(&self) -> String {
        format!("{}/api/auth/github/callback", self.api_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_secret_validation() {
        // Valid 64-char hex
        let valid_secret = "a".repeat(64);
        assert!(valid_secret.chars().all(|c| c.is_ascii_hexdigit()));

        // Invalid: too short
        let invalid_short = "a".repeat(32);
        assert_ne!(invalid_short.len(), 64);

        // Invalid: contains non-hex
        let invalid_chars = "a".repeat(63) + "z";
        assert!(!invalid_chars.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn normalize_origin_strips_path_and_trailing_slash() {
        let result = normalize_origin("http://localhost:3000/some/path").unwrap();
        assert_eq!(result, "http://localhost:3000");
    }

    #[test]
    fn loopback_aliases_includes_all_localhost_variants_without_port() {
        let aliases = loopback_aliases("http://localhost").unwrap();
        assert!(
            aliases.contains(&"http://localhost".to_string()),
            "must include localhost"
        );
        assert!(
            aliases.contains(&"http://127.0.0.1".to_string()),
            "must include 127.0.0.1"
        );
        assert!(
            aliases.contains(&"http://[::1]".to_string()),
            "must include [::1]"
        );
        // No spurious dots or colons
        for alias in &aliases {
            assert!(
                !alias.contains("localhost."),
                "alias must not contain 'localhost.'"
            );
        }
    }

    #[test]
    fn loopback_aliases_includes_all_localhost_variants_with_port() {
        let aliases = loopback_aliases("http://localhost:3000").unwrap();
        assert!(aliases.contains(&"http://localhost:3000".to_string()));
        assert!(aliases.contains(&"http://127.0.0.1:3000".to_string()));
        assert!(aliases.contains(&"http://[::1]:3000".to_string()));
        // No spurious dots before port
        for alias in &aliases {
            assert!(
                !alias.contains("localhost.:"),
                "alias must not contain 'localhost.:'"
            );
        }
    }

    #[test]
    fn loopback_aliases_for_non_localhost_only_returns_self() {
        let aliases = loopback_aliases("https://example.com").unwrap();
        assert_eq!(aliases, vec!["https://example.com".to_string()]);
    }
}
