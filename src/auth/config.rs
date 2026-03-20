use axum_extra::extract::cookie::Key;

#[derive(Clone)]
pub struct AuthRuntime {
    pub config: AuthConfig,
    pub http_client: reqwest::Client,
    pub cookie_key: Key,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
    pub logout_return_url: String,
    pub session_secret: String,
    pub scopes: Vec<String>,
    pub secure_cookies: bool,
}

impl AuthConfig {
    /// Returns `Ok(None)` when GitHub OAuth is intentionally unconfigured.
    ///
    /// # Errors
    /// Returns an error when auth is partially configured or contains invalid values.
    pub fn from_env() -> Result<Option<Self>, String> {
        let Some(client_id) = read_env("GITHUB_CLIENT_ID") else {
            return Ok(None);
        };

        let base_url = require_env("PRIOR_WEB_BASE_URL")?;

        Ok(Some(Self {
            client_id,
            client_secret: require_env("GITHUB_CLIENT_SECRET")?,
            callback_url: read_env("GITHUB_CALLBACK_URL")
                .unwrap_or_else(|| format!("{}/auth/callback", base_url.trim_end_matches('/'))),
            logout_return_url: read_env("GITHUB_LOGOUT_RETURN_URL")
                .unwrap_or_else(|| format!("{}/", base_url.trim_end_matches('/'))),
            session_secret: require_env("PRIOR_WEB_SESSION_SECRET")?,
            scopes: read_env("GITHUB_OAUTH_SCOPES")
                .map(|value| {
                    value
                        .split([',', ' '])
                        .map(str::trim)
                        .filter(|scope| !scope.is_empty())
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .filter(|scopes| !scopes.is_empty())
                .unwrap_or_else(|| vec!["read:user".into(), "user:email".into(), "repo".into()]),
            secure_cookies: read_env("PRIOR_WEB_SECURE_COOKIES").map_or_else(
                || !cfg!(debug_assertions) || !base_url.contains("127.0.0.1"),
                |value| value != "0" && !value.eq_ignore_ascii_case("false"),
            ),
        }))
    }
}

/// Loads the runtime GitHub OAuth client when configured.
///
/// # Errors
/// Returns an error when environment validation or HTTP client construction fails.
pub fn load_auth_runtime() -> Result<Option<AuthRuntime>, String> {
    let Some(config) = AuthConfig::from_env()? else {
        tracing::warn!("GitHub OAuth configuration is absent; prior-web auth is disabled");
        return Ok(None);
    };

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("prior-web")
        .build()
        .map_err(|error| format!("build auth HTTP client: {error}"))?;

    Ok(Some(AuthRuntime {
        cookie_key: Key::derive_from(config.session_secret.as_bytes()),
        config,
        http_client,
    }))
}

fn read_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

fn require_env(key: &str) -> Result<String, String> {
    read_env(key).ok_or_else(|| format!("missing required env var {key}"))
}
