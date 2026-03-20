use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::PrivateCookieJar;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::auth::session::{
    AuthFlowCookie, FLOW_COOKIE_NAME, GitHubToken, SessionCookie, flow_cookie, flow_removal_cookie,
    session_cookie, session_removal_cookie,
};
use crate::runtime::AppState;
use crate::state::auth::CurrentUser;

const GITHUB_AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";
const GITHUB_USER_EMAILS_URL: &str = "https://api.github.com/user/emails";

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub return_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubAccessTokenResponse {
    access_token: String,
    token_type: String,
    scope: Option<String>,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct GitHubUserResponse {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubUserEmailResponse {
    email: String,
    primary: bool,
    verified: bool,
}

#[allow(clippy::unused_async)]
pub async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<LoginQuery>,
) -> Response {
    let Some(auth) = state.auth.as_ref() else {
        return auth_redirect_failure("github auth is not configured");
    };

    let return_to = sanitize_return_to(query.return_to.as_deref());
    let flow = AuthFlowCookie {
        csrf_state: random_token(),
        return_to: return_to.to_string(),
    };
    let flow_json = match serde_json::to_string(&flow) {
        Ok(value) => value,
        Err(error) => {
            return auth_redirect_failure(&format!("serialize auth flow cookie: {error}"));
        }
    };

    let auth_url = format!(
        "{GITHUB_AUTHORIZE_URL}?client_id={}&redirect_uri={}&scope={}&state={}",
        urlencoding::encode(&auth.config.client_id),
        urlencoding::encode(&auth.config.callback_url),
        urlencoding::encode(&auth.config.scopes.join(" ")),
        urlencoding::encode(&flow.csrf_state),
    );

    (
        jar.add(flow_cookie(flow_json, auth.config.secure_cookies)),
        Redirect::to(&auth_url),
    )
        .into_response()
}

pub async fn callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let Some(auth) = state.auth.as_ref() else {
        return auth_redirect_failure("github auth is not configured");
    };

    if let Some(error) = query.error {
        let description = query.error_description.unwrap_or_default();
        tracing::warn!(%error, %description, "GitHub callback returned an OAuth error");
        return Redirect::to(&auth_denied_redirect_url(&error)).into_response();
    }

    let Some(flow_cookie_value) = jar.get(FLOW_COOKIE_NAME) else {
        return auth_redirect_failure("missing auth flow state cookie");
    };
    let flow = match serde_json::from_str::<AuthFlowCookie>(flow_cookie_value.value()) {
        Ok(value) => value,
        Err(error) => {
            return auth_redirect_failure(&format!("invalid auth flow state cookie: {error}"));
        }
    };
    let Some(state_param) = query.state else {
        return auth_redirect_failure("missing callback state");
    };
    if state_param != flow.csrf_state {
        return auth_redirect_failure("callback state mismatch");
    }

    let Some(code) = query.code else {
        return auth_redirect_failure("missing authorization code");
    };

    let token_response = match auth
        .http_client
        .post(GITHUB_ACCESS_TOKEN_URL)
        .header(reqwest::header::ACCEPT, "application/json")
        .form(&[
            ("client_id", auth.config.client_id.as_str()),
            ("client_secret", auth.config.client_secret.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", auth.config.callback_url.as_str()),
        ])
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            return auth_redirect_failure(&format!("exchange authorization code: {error}"));
        }
    };
    if !token_response.status().is_success() {
        return auth_redirect_failure(&format!(
            "exchange authorization code: github returned {}",
            token_response.status()
        ));
    }
    let token = match token_response.json::<GitHubAccessTokenResponse>().await {
        Ok(token) => token,
        Err(error) => {
            return auth_redirect_failure(&format!("decode github access token response: {error}"));
        }
    };

    let github_token = build_github_token(token);
    let user = match fetch_github_user(auth, &github_token).await {
        Ok(user) => user,
        Err(error) => return auth_redirect_failure(&error),
    };
    let session_json = match serde_json::to_string(&SessionCookie::new(user, github_token)) {
        Ok(value) => value,
        Err(error) => return auth_redirect_failure(&format!("serialize session cookie: {error}")),
    };

    (
        jar.remove(flow_removal_cookie(auth.config.secure_cookies))
            .add(session_cookie(session_json, auth.config.secure_cookies)),
        Redirect::to(&flow.return_to),
    )
        .into_response()
}

#[allow(clippy::unused_async)]
pub async fn logout(State(state): State<AppState>, jar: PrivateCookieJar) -> Response {
    let Some(auth) = state.auth.as_ref() else {
        return Redirect::to("/").into_response();
    };

    (
        jar.remove(session_removal_cookie(auth.config.secure_cookies))
            .remove(flow_removal_cookie(auth.config.secure_cookies)),
        Redirect::to(&auth.config.logout_return_url),
    )
        .into_response()
}

fn build_github_token(token: GitHubAccessTokenResponse) -> GitHubToken {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    GitHubToken {
        access_token: token.access_token,
        token_type: token.token_type,
        scopes: token
            .scope
            .as_deref()
            .map(|scope| {
                scope
                    .split(',')
                    .map(str::trim)
                    .filter(|scope| !scope.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        expires_at_unix: token.expires_in.map(|seconds| now + seconds),
        refresh_token: token.refresh_token,
        refresh_token_expires_at_unix: token.refresh_token_expires_in.map(|seconds| now + seconds),
    }
}

async fn fetch_github_user(
    auth: &crate::auth::config::AuthRuntime,
    token: &GitHubToken,
) -> Result<CurrentUser, String> {
    let user_response = auth
        .http_client
        .get(GITHUB_USER_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|error| format!("fetch github user: {error}"))?;
    if !user_response.status().is_success() {
        return Err(format!(
            "fetch github user: github returned {}",
            user_response.status()
        ));
    }
    let user = user_response
        .json::<GitHubUserResponse>()
        .await
        .map_err(|error| format!("decode github user response: {error}"))?;

    let email = if let Some(email) = user.email.clone() {
        Some(email)
    } else {
        fetch_primary_email(auth, token).await?
    };

    Ok(CurrentUser {
        sub: format!("github:{}", user.id),
        github_login: user.login.clone(),
        display_name: user.name.or(Some(user.login)),
        email,
        avatar_url: user.avatar_url,
    })
}

async fn fetch_primary_email(
    auth: &crate::auth::config::AuthRuntime,
    token: &GitHubToken,
) -> Result<Option<String>, String> {
    let response = auth
        .http_client
        .get(GITHUB_USER_EMAILS_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|error| format!("fetch github user emails: {error}"))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let emails = response
        .json::<Vec<GitHubUserEmailResponse>>()
        .await
        .map_err(|error| format!("decode github user emails response: {error}"))?;

    Ok(emails
        .into_iter()
        .find(|email| email.primary && email.verified)
        .map(|email| email.email))
}

fn random_token() -> String {
    use std::fmt::Write;

    use rand::RngCore;

    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn auth_redirect_failure(message: &str) -> Response {
    tracing::error!(%message, "auth flow failed");
    Redirect::to(&auth_denied_redirect_url("login_failed")).into_response()
}

fn sanitize_return_to(value: Option<&str>) -> &str {
    match value {
        Some(path) if path.starts_with('/') && !path.starts_with("//") => path,
        _ => "/app",
    }
}

fn auth_denied_redirect_url(reason: &str) -> String {
    format!("/auth/denied?reason={}", urlencoding::encode(reason))
}
