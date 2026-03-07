use anyhow::{Context, Result};
use std::env;
use time::Duration;

/// Application configuration loaded from environment variables at startup
#[derive(Clone, Debug)]
pub struct Config {
    /// PostgreSQL connection URL
    pub database_url: String,

    /// GitHub OAuth client ID
    pub github_client_id: String,

    /// GitHub OAuth client secret
    pub github_client_secret: String,

    /// Session encryption secret (64-byte hex string)
    pub session_secret: String,

    /// API server URL (used in OAuth redirect URI)
    pub api_url: String,

    /// Frontend URL (redirect after auth)
    pub frontend_url: String,

    /// Server port
    pub port: u16,

    /// Rate limiting: requests per minute per IP
    pub rate_limit_requests_per_minute: u32,

    /// Session timeout duration
    pub session_timeout: Duration,

    /// Whether session cookies must be marked secure
    pub session_cookie_secure: bool,

    /// Logging level (defaults to "server=debug,tower_sessions=debug")
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// Returns an error when required environment variables are missing or invalid.
    /// The caller should fail startup on error so configuration problems are caught early.
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;

        let github_client_id = env::var("GITHUB_CLIENT_ID")
            .context("GITHUB_CLIENT_ID must be set")?;

        let github_client_secret = env::var("GITHUB_CLIENT_SECRET")
            .context("GITHUB_CLIENT_SECRET must be set")?;

        let session_secret = env::var("SESSION_SECRET")
            .context("SESSION_SECRET must be set (generate with: head -c 32 /dev/urandom | xxd -p)")?;

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

        let api_url = env::var("API_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());

        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .context("PORT must be a valid u16")?;

        let rate_limit_requests_per_minute = env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .context("RATE_LIMIT_REQUESTS_PER_MINUTE must be a valid u32")?;

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
            github_client_secret,
            session_secret,
            api_url,
            frontend_url,
            port,
            rate_limit_requests_per_minute,
            session_timeout: Duration::days(session_timeout_days),
            session_cookie_secure,
            log_level,
        })
    }

    /// OAuth redirect URI for GitHub
    pub fn github_redirect_uri(&self) -> String {
        format!("{}/api/auth/github/callback", self.api_url)
    }
}

#[cfg(test)]
mod tests {
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
}
