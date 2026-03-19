use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::PrivateCookieJar;
use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreIdToken, CoreIdTokenClaims, CoreTokenResponse,
};
use openidconnect::{
    AccessTokenHash, AuthType, AuthorizationCode, ClientId, ClientSecret, CsrfToken, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RequestTokenError, Scope,
    TokenResponse,
};
use serde::Deserialize;

use crate::auth::session::{
    AuthFlowCookie, FLOW_COOKIE_NAME, SessionCookie, flow_cookie, flow_removal_cookie,
    session_cookie, session_removal_cookie,
};
use crate::runtime::AppState;
use crate::state::auth::CurrentUser;

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

#[allow(clippy::unused_async)]
pub async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<LoginQuery>,
) -> Response {
    let Some(auth) = state.auth.as_ref() else {
        return auth_redirect_failure("auth is not configured");
    };
    let client = match build_oidc_client(auth) {
        Ok(client) => client,
        Err(error) => return auth_redirect_failure(&error),
    };

    let return_to = sanitize_return_to(query.return_to.as_deref());
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut authorize = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("profile".into()))
        .add_scope(Scope::new("email".into()))
        .set_pkce_challenge(pkce_challenge);

    if let Some(connection) = &auth.config.github_connection {
        authorize = authorize.add_extra_param("connection", connection);
    }

    let (auth_url, csrf_state, nonce) = authorize.url();
    let flow = AuthFlowCookie {
        csrf_state: csrf_state.secret().clone(),
        nonce: nonce.secret().clone(),
        pkce_verifier: pkce_verifier.secret().clone(),
        return_to: return_to.to_string(),
    };
    let flow_json = match serde_json::to_string(&flow) {
        Ok(value) => value,
        Err(error) => {
            return auth_redirect_failure(&format!("serialize auth flow cookie: {error}"));
        }
    };

    (
        jar.add(flow_cookie(flow_json, auth.config.secure_cookies)),
        Redirect::to(auth_url.as_str()),
    )
        .into_response()
}

#[allow(clippy::too_many_lines)]
pub async fn callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let Some(auth) = state.auth.as_ref() else {
        return auth_redirect_failure("auth is not configured");
    };
    let client = match build_oidc_client(auth) {
        Ok(client) => client,
        Err(error) => return auth_redirect_failure(&error),
    };

    if let Some(error) = query.error {
        let description = query.error_description.unwrap_or_default();
        tracing::warn!(%error, %description, "Auth0 callback returned an OAuth error");
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

    let token_request = match client.exchange_code(AuthorizationCode::new(code)) {
        Ok(request) => request,
        Err(error) => {
            return auth_redirect_failure(&format!("build token exchange request: {error}"));
        }
    };

    let token_response: CoreTokenResponse = match token_request
        .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
        .request_async(&auth.http_client)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            if let RequestTokenError::Parse(parse_error, body) = &error {
                tracing::error!(
                    parse_path = %parse_error.path(),
                    parse_error = %parse_error,
                    body = %String::from_utf8_lossy(body),
                    "Auth0 token endpoint returned an unparsable response body",
                );
            }
            return auth_redirect_failure(&format!("exchange authorization code: {error}"));
        }
    };

    let Some(id_token): Option<&CoreIdToken> = token_response.id_token() else {
        return auth_redirect_failure("Auth0 did not return an ID token");
    };
    let nonce = Nonce::new(flow.nonce);
    let verifier = client.id_token_verifier();
    let claims: &CoreIdTokenClaims = match id_token.claims(&verifier, &nonce) {
        Ok(claims) => claims,
        Err(error) => return auth_redirect_failure(&format!("verify ID token claims: {error}")),
    };

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let signing_alg = match id_token.signing_alg() {
            Ok(signing_alg) => signing_alg,
            Err(error) => {
                return auth_redirect_failure(&format!("read ID token signing algorithm: {error}"));
            }
        };
        let signing_key = match id_token.signing_key(&verifier) {
            Ok(signing_key) => signing_key,
            Err(error) => {
                return auth_redirect_failure(&format!("read ID token signing key: {error}"));
            }
        };
        let actual_access_token_hash = match AccessTokenHash::from_token(
            token_response.access_token(),
            signing_alg,
            signing_key,
        ) {
            Ok(hash) => hash,
            Err(error) => {
                return auth_redirect_failure(&format!("compute access token hash: {error}"));
            }
        };
        if actual_access_token_hash != *expected_access_token_hash {
            return auth_redirect_failure("access token hash mismatch");
        }
    }

    let user = CurrentUser {
        sub: claims.subject().as_str().to_string(),
        display_name: None,
        email: claims.email().map(|email| email.as_str().to_string()),
        avatar_url: None,
    };
    let session_json = match serde_json::to_string(&SessionCookie::new(user)) {
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

    let logout_url = format!(
        "https://{}/v2/logout?client_id={}&returnTo={}",
        auth.config.domain,
        urlencoding::encode(&auth.config.client_id),
        urlencoding::encode(&auth.config.logout_return_url)
    );

    (
        jar.remove(session_removal_cookie(auth.config.secure_cookies))
            .remove(flow_removal_cookie(auth.config.secure_cookies)),
        Redirect::to(&logout_url),
    )
        .into_response()
}

fn auth_redirect_failure(message: &str) -> Response {
    tracing::error!(%message, "auth flow failed");
    Redirect::to(&auth_denied_redirect_url("login_failed")).into_response()
}

fn build_oidc_client(
    auth: &crate::auth::config::AuthRuntime,
) -> Result<
    CoreClient<
        openidconnect::EndpointSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointMaybeSet,
        openidconnect::EndpointMaybeSet,
    >,
    String,
> {
    Ok(CoreClient::from_provider_metadata(
        auth.provider_metadata.clone(),
        ClientId::new(auth.config.client_id.clone()),
        Some(ClientSecret::new(auth.config.client_secret.clone())),
    )
    .set_auth_type(AuthType::RequestBody)
    .set_redirect_uri(
        openidconnect::RedirectUrl::new(auth.config.callback_url.clone())
            .map_err(|error| format!("invalid AUTH0_CALLBACK_URL: {error}"))?,
    ))
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
