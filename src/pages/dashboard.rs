use leptos::prelude::*;
use leptos::tachys::view::any_view::{AnyView, IntoAny};
use std::collections::BTreeSet;

use crate::auth::user::current_auth_state;
use crate::net::factory::FactoryDashboardView;
use crate::state::auth::{AuthState, CurrentUser};

#[path = "dashboard/state.rs"]
mod state;
#[path = "dashboard/view.rs"]
mod view;

use state::{
    CenterMode, ChatMessage, DashboardUiState, RepoImportFeedback, load_dashboard_view,
    load_gate_snapshot, load_tracked_repos, repo_sidebar_state,
};

#[component]
pub fn DashboardPage() -> AnyView {
    let auth = Resource::new(|| (), |()| current_auth_state());
    let ui = dashboard_ui_state();
    let (refresh_tick, set_refresh_tick) = signal(0_u64);

    let gate = Resource::new(
        move || (refresh_tick.get(), auth.get()),
        |(_, auth_state)| load_gate_snapshot(auth_state),
    );
    let dashboard = Resource::new(
        move || (refresh_tick.get(), auth.get()),
        |(_, auth_state)| load_dashboard_view(auth_state),
    );
    let repos = Resource::new(
        move || (refresh_tick.get(), auth.get()),
        |(_, auth_state)| load_tracked_repos(auth_state),
    );

    let on_refresh = move |_| set_refresh_tick.update(|count| *count += 1);
    let refresh_data = move || set_refresh_tick.update(|count| *count += 1);

    view! {
        <Suspense fallback=move || view::shell_loading().into_any()>
            {move || match auth.get() {
                None => view::shell_loading().into_any(),
                Some(Ok(AuthState::Authenticated(current_user))) => {
                    view! {
                        <AuthenticatedDashboard
                            current_user=current_user
                            gate=gate
                            dashboard=dashboard
                            repos=repos
                            on_refresh=on_refresh
                            on_data_changed=refresh_data
                            ui=ui
                        />
                    }
                    .into_any()
                }
                Some(Ok(AuthState::Anonymous)) => view::unauthenticated_shell().into_any(),
                Some(Err(error)) => view::error_shell(error.to_string()).into_any(),
            }}
        </Suspense>
    }
    .into_any()
}

#[component]
fn AuthenticatedDashboard(
    current_user: CurrentUser,
    gate: Resource<
        Option<Result<crate::state::gate::GateUiState, server_fn::error::ServerFnError>>,
    >,
    dashboard: Resource<Option<Result<FactoryDashboardView, server_fn::error::ServerFnError>>>,
    repos: Resource<
        Option<Result<Vec<crate::net::prior::RepoEntry>, server_fn::error::ServerFnError>>,
    >,
    on_refresh: impl FnMut(leptos::ev::MouseEvent) + Copy + Send + 'static,
    on_data_changed: impl Fn() + Copy + Send + 'static,
    ui: DashboardUiState,
) -> impl IntoView {
    let fallback_user = current_user.clone();
    let content_user = current_user.clone();

    view! {
        <Suspense fallback=move || {
            view::app_shell(
                fallback_user.clone(),
                crate::state::gate::GateUiState::loading("loading gate snapshot"),
                FactoryDashboardView { runs: Vec::new() },
                state::RepoSidebarState {
                    repos: Vec::new(),
                    loading: true,
                    error: None,
                },
                true,
                None,
                on_refresh,
                on_data_changed,
                ui,
            )
        }>
            {move || {
                let gate_state = gate.get().map_or_else(
                    || crate::state::gate::GateUiState::loading("loading gate snapshot"),
                    |result| match result {
                        Some(Ok(state)) => state,
                        Some(Err(error)) => crate::state::gate::GateUiState::disconnected(
                            "server function failed",
                            error.to_string(),
                        ),
                        None => crate::state::gate::GateUiState::loading("loading gate snapshot"),
                    },
                );
                let dashboard_state = dashboard.get().map_or_else(
                    || FactoryDashboardView { runs: Vec::new() },
                    |result| match result {
                        Some(Ok(state)) => state,
                        Some(Err(_)) | None => FactoryDashboardView { runs: Vec::new() },
                    },
                );
                let repo_sidebar = repo_sidebar_state(repos.get());

                view::app_shell(
                    content_user.clone(),
                    gate_state,
                    dashboard_state,
                    repo_sidebar,
                    false,
                    None,
                    on_refresh,
                    on_data_changed,
                    ui,
                )
            }}
        </Suspense>
    }
}

fn dashboard_ui_state() -> DashboardUiState {
    let (selected_run, set_selected_run) = signal(None::<i64>);
    let (selected_repo, set_selected_repo) = signal(None::<String>);
    let (center_mode, set_center_mode) = signal(CenterMode::Runs);
    let (sidebar_filter, set_sidebar_filter) = signal(state::SidebarFilter::All);
    let (starred_runs, set_starred_runs) = signal(BTreeSet::<i64>::new());
    let (chat_messages, set_chat_messages) = signal(Vec::<ChatMessage>::new());
    let (chat_sending, set_chat_sending) = signal(false);
    let (repo_draft, set_repo_draft) = signal(String::new());
    let (repo_importing, set_repo_importing) = signal(false);
    let (repo_feedback, set_repo_feedback) = signal(None::<RepoImportFeedback>);

    DashboardUiState {
        selected_run,
        set_selected_run,
        selected_repo,
        set_selected_repo,
        center_mode,
        set_center_mode,
        sidebar_filter,
        set_sidebar_filter,
        starred_runs,
        set_starred_runs,
        chat_messages,
        set_chat_messages,
        chat_sending,
        set_chat_sending,
        repo_draft,
        set_repo_draft,
        repo_importing,
        set_repo_importing,
        repo_feedback,
        set_repo_feedback,
    }
}
