use leptos::prelude::*;
use server_fn::error::ServerFnError;
use std::collections::BTreeSet;

use crate::net::factory::{FactoryDashboardView, fetch_factory_dashboard};
use crate::net::prior::{RepoEntry, list_repos};
use crate::net::prior_gate::refresh_dashboard;
use crate::state::auth::AuthState;
use crate::state::gate::GateUiState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SidebarFilter {
    All,
    NeedsAttention,
    Starred,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CenterMode {
    Runs,
    General,
}

#[derive(Debug, Clone)]
pub(crate) struct ChatMessage {
    pub(crate) from: String,
    pub(crate) content: String,
    pub(crate) is_human: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RepoImportFeedback {
    pub(crate) message: String,
    pub(crate) is_error: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RepoSidebarState {
    pub(crate) repos: Vec<RepoEntry>,
    pub(crate) loading: bool,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct DashboardUiState {
    pub(crate) selected_run: ReadSignal<Option<i64>>,
    pub(crate) set_selected_run: WriteSignal<Option<i64>>,
    pub(crate) selected_room: ReadSignal<String>,
    pub(crate) set_selected_room: WriteSignal<String>,
    pub(crate) selected_repo: ReadSignal<Option<String>>,
    pub(crate) set_selected_repo: WriteSignal<Option<String>>,
    pub(crate) center_mode: ReadSignal<CenterMode>,
    pub(crate) set_center_mode: WriteSignal<CenterMode>,
    pub(crate) sidebar_filter: ReadSignal<SidebarFilter>,
    pub(crate) set_sidebar_filter: WriteSignal<SidebarFilter>,
    pub(crate) starred_runs: ReadSignal<BTreeSet<i64>>,
    pub(crate) set_starred_runs: WriteSignal<BTreeSet<i64>>,
    pub(crate) chat_messages: ReadSignal<Vec<ChatMessage>>,
    pub(crate) set_chat_messages: WriteSignal<Vec<ChatMessage>>,
    pub(crate) chat_sending: ReadSignal<bool>,
    pub(crate) set_chat_sending: WriteSignal<bool>,
    pub(crate) repo_draft: ReadSignal<String>,
    pub(crate) set_repo_draft: WriteSignal<String>,
    pub(crate) repo_importing: ReadSignal<bool>,
    pub(crate) set_repo_importing: WriteSignal<bool>,
    pub(crate) repo_feedback: ReadSignal<Option<RepoImportFeedback>>,
    pub(crate) set_repo_feedback: WriteSignal<Option<RepoImportFeedback>>,
}

pub(crate) async fn load_gate_snapshot(
    auth_state: Option<Result<AuthState, ServerFnError>>,
) -> Option<Result<GateUiState, ServerFnError>> {
    match auth_state {
        Some(Ok(AuthState::Authenticated(_))) => Some(refresh_dashboard().await),
        Some(Ok(AuthState::Anonymous) | Err(_)) | None => None,
    }
}

pub(crate) async fn load_dashboard_view(
    auth_state: Option<Result<AuthState, ServerFnError>>,
) -> Option<Result<FactoryDashboardView, ServerFnError>> {
    match auth_state {
        Some(Ok(AuthState::Authenticated(_))) => Some(fetch_factory_dashboard().await),
        Some(Ok(AuthState::Anonymous) | Err(_)) | None => None,
    }
}

pub(crate) async fn load_tracked_repos(
    auth_state: Option<Result<AuthState, ServerFnError>>,
) -> Option<Result<Vec<RepoEntry>, ServerFnError>> {
    match auth_state {
        Some(Ok(AuthState::Authenticated(_))) => Some(list_repos().await),
        Some(Ok(AuthState::Anonymous) | Err(_)) | None => None,
    }
}

#[allow(clippy::option_option)]
pub(crate) fn repo_sidebar_state(
    repos_state: Option<Option<Result<Vec<RepoEntry>, ServerFnError>>>,
) -> RepoSidebarState {
    match repos_state {
        None => RepoSidebarState {
            repos: Vec::new(),
            loading: true,
            error: None,
        },
        Some(Some(Ok(repos))) => RepoSidebarState {
            repos,
            loading: false,
            error: None,
        },
        Some(Some(Err(error))) => RepoSidebarState {
            repos: Vec::new(),
            loading: false,
            error: Some(error.to_string()),
        },
        Some(None) => RepoSidebarState {
            repos: Vec::new(),
            loading: false,
            error: None,
        },
    }
}
