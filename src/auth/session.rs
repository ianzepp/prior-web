#![cfg(feature = "ssr")]

use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::state::auth::CurrentUser;

pub const SESSION_COOKIE_NAME: &str = "prior_web_session";
pub const FLOW_COOKIE_NAME: &str = "prior_web_auth_flow";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCookie {
    pub user: CurrentUser,
    pub expires_at_unix: i64,
}

impl SessionCookie {
    #[must_use]
    pub fn new(user: CurrentUser) -> Self {
        Self {
            user,
            expires_at_unix: (OffsetDateTime::now_utc() + Duration::hours(8)).unix_timestamp(),
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
    pub nonce: String,
    pub pkce_verifier: String,
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
