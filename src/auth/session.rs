use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::state::auth::CurrentUser;

pub const SESSION_COOKIE_NAME: &str = "prior_web_session";
pub const FLOW_COOKIE_NAME: &str = "prior_web_auth_flow";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubToken {
    pub access_token: String,
    pub token_type: String,
    pub scopes: Vec<String>,
    pub expires_at_unix: Option<i64>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_at_unix: Option<i64>,
}

impl GitHubToken {
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at_unix
            .is_some_and(|expires_at| expires_at <= OffsetDateTime::now_utc().unix_timestamp())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCookie {
    pub user: CurrentUser,
    pub github_token: GitHubToken,
    pub expires_at_unix: i64,
}

impl SessionCookie {
    #[must_use]
    pub fn new(user: CurrentUser, github_token: GitHubToken) -> Self {
        let token_expiry = github_token
            .expires_at_unix
            .unwrap_or_else(|| (OffsetDateTime::now_utc() + Duration::hours(8)).unix_timestamp());

        Self {
            user,
            github_token,
            expires_at_unix: token_expiry,
        }
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at_unix <= OffsetDateTime::now_utc().unix_timestamp()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthFlowCookie {
    pub csrf_state: String,
    pub return_to: String,
}

pub fn session_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, value))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::hours(8))
        .build()
}

pub fn session_removal_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .build()
}

pub fn flow_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((FLOW_COOKIE_NAME, value))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(10))
        .build()
}

pub fn flow_removal_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((FLOW_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .build()
}
