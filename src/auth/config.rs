use axum_extra::extract::cookie::Key;
use openidconnect::IssuerUrl;
use openidconnect::core::CoreProviderMetadata;

#[derive(Clone)]
pub struct AuthRuntime {
    pub config: AuthConfig,
    pub provider_metadata: CoreProviderMetadata,
    pub http_client: reqwest::Client,
    pub cookie_key: Key,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub domain: String,
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
    pub logout_return_url: String,
    pub base_url: String,
    pub session_secret: String,
    pub github_connection: Option<String>,
    pub secure_cookies: bool,
}

impl AuthConfig {
    /// Returns `Ok(None)` when Auth0 is intentionally unconfigured.
    ///
    /// # Errors
    /// Returns an error when auth is partially configured or contains invalid values.
    pub fn from_env() -> Result<Option<Self>, String> {
        let Some(domain) = read_env("AUTH0_DOMAIN") else {
            return Ok(None);
        };

        Ok(Some(Self {
            domain: domain.clone(),
            client_id: require_env("AUTH0_CLIENT_ID")?,
            client_secret: require_env("AUTH0_CLIENT_SECRET")?,
            callback_url: require_env("AUTH0_CALLBACK_URL")?,
            logout_return_url: require_env("AUTH0_LOGOUT_RETURN_URL")?,
            base_url: require_env("PRIOR_WEB_BASE_URL")?,
            session_secret: require_env("PRIOR_WEB_SESSION_SECRET")?,
            github_connection: read_env("AUTH0_GITHUB_CONNECTION"),
            secure_cookies: read_env("PRIOR_WEB_SECURE_COOKIES").map_or_else(
                || !cfg!(debug_assertions) || !domain.contains("localhost"),
                |value| value != "0" && !value.eq_ignore_ascii_case("false"),
            ),
        }))
    }

    fn issuer_url(&self) -> Result<IssuerUrl, String> {
        IssuerUrl::new(format!("https://{}/", self.domain.trim_end_matches('/')))
            .map_err(|error| format!("invalid AUTH0_DOMAIN issuer URL: {error}"))
    }
}

/// Loads the runtime auth client and provider metadata when Auth0 is configured.
///
/// # Errors
/// Returns an error when environment validation or provider discovery fails.
pub async fn load_auth_runtime() -> Result<Option<AuthRuntime>, String> {
    let Some(config) = AuthConfig::from_env()? else {
        tracing::warn!("Auth0 configuration is absent; prior-web auth is disabled");
        return Ok(None);
    };

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| format!("build auth HTTP client: {error}"))?;
    let provider_metadata =
        CoreProviderMetadata::discover_async(config.issuer_url()?, &http_client)
            .await
            .map_err(|error| format!("discover Auth0 provider metadata: {error}"))?;
    Ok(Some(AuthRuntime {
        cookie_key: Key::derive_from(config.session_secret.as_bytes()),
        config,
        provider_metadata,
        http_client,
    }))
}

fn read_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

fn require_env(key: &str) -> Result<String, String> {
    read_env(key).ok_or_else(|| format!("missing required env var {key}"))
}
