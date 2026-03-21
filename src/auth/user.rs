#[cfg(feature = "ssr")]
use axum::http::HeaderMap;
#[cfg(feature = "ssr")]
use axum::http::request::Parts;
#[cfg(feature = "ssr")]
use axum_extra::extract::cookie::PrivateCookieJar;
#[cfg(feature = "ssr")]
use leptos::prelude::{expect_context, use_context};
use leptos::server_fn::error::ServerFnError;

#[cfg(feature = "ssr")]
use crate::auth::session::{GitHubToken, SESSION_COOKIE_NAME, SessionCookie};
#[cfg(feature = "ssr")]
use crate::runtime::AppState;
use crate::state::auth::AuthState;
#[cfg(feature = "ssr")]
use crate::state::auth::CurrentUser;

#[cfg(feature = "ssr")]
#[must_use]
pub fn auth_state_from_parts(parts: &Parts, state: &AppState) -> AuthState {
    current_user_from_parts(parts, state).map_or(AuthState::Anonymous, AuthState::Authenticated)
}

#[cfg(feature = "ssr")]
#[must_use]
pub fn current_user_from_parts(parts: &Parts, state: &AppState) -> Option<CurrentUser> {
    current_session_from_parts(parts, state).map(|session| session.user)
}

#[cfg(feature = "ssr")]
#[must_use]
pub fn current_session_from_parts(parts: &Parts, state: &AppState) -> Option<SessionCookie> {
    current_session_from_headers(&parts.headers, state)
}

#[cfg(feature = "ssr")]
#[must_use]
pub fn current_session_from_headers(
    headers: &HeaderMap,
    state: &AppState,
) -> Option<SessionCookie> {
    let auth = state.auth.as_ref()?;
    let jar = PrivateCookieJar::from_headers(headers, auth.cookie_key.clone());
    let cookie = jar.get(SESSION_COOKIE_NAME)?;
    let session = serde_json::from_str::<SessionCookie>(cookie.value()).ok()?;

    if session.is_expired() || session.github_token.is_expired() {
        return None;
    }

    Some(session)
}

/// Returns the authenticated user for the current server request.
///
/// # Errors
/// Returns an error when request context is unavailable or the user is anonymous.
#[cfg(feature = "ssr")]
pub fn require_current_user() -> Result<CurrentUser, String> {
    Ok(require_current_session()?.user)
}

/// Returns the authenticated GitHub-backed session for the current server request.
///
/// # Errors
/// Returns an error when request context is unavailable or the session is invalid.
#[cfg(feature = "ssr")]
pub fn require_current_session() -> Result<SessionCookie, String> {
    let state = expect_context::<AppState>();
    let parts = use_context::<Parts>()
        .ok_or_else(|| "missing request parts in server context".to_string())?;

    current_session_from_parts(&parts, &state).ok_or_else(|| "authentication required".to_string())
}

/// Returns the reusable GitHub access token for the current session.
///
/// # Errors
/// Returns an error when request context is unavailable or the session is invalid.
#[cfg(feature = "ssr")]
pub fn require_github_access_token() -> Result<GitHubToken, String> {
    Ok(require_current_session()?.github_token)
}

#[leptos::server]
#[allow(clippy::unused_async)]
pub async fn current_auth_state() -> Result<AuthState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let state = expect_context::<AppState>();
        let parts = use_context::<Parts>()
            .ok_or_else(|| ServerFnError::new("missing request parts in server context"))?;

        return Ok(auth_state_from_parts(&parts, &state));
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "current_auth_state is only available on the server",
    ))
}
