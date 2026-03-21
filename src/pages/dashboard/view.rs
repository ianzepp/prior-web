use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;
use pulldown_cmark::{CowStr, Event, Options, Parser, html};
use std::collections::BTreeSet;

use crate::net::factory::{
    FactoryDashboardView, FactoryLifecyclePhaseDisplay, FactoryRunDisplay, FactoryRunUpdateDisplay,
};
use crate::net::prior::{RepoEntry, import_repo, send_room_message};
use crate::state::auth::CurrentUser;
use crate::state::gate::{ConnectionStatus, GateUiState};

use super::live::{LiveSocket, connect_live_socket};
use super::state::{
    CenterMode, ChatMessage, DashboardUiState, RepoImportFeedback, RepoSidebarState, SidebarFilter,
};

pub(crate) fn shell_loading() -> impl IntoView {
    let gate = GateUiState::loading("confirming session");
    let (repo_draft, set_repo_draft) = signal(String::new());
    let (repo_importing, _) = signal(false);
    let (repo_feedback, _) = signal(None::<RepoImportFeedback>);
    view! {
        <main class="app-shell">
            <Topbar
                user_label="Loading...".to_string()
                loading=true
                on_refresh=move |_| {}
                login_url=None
            />
            <div class="main-layout">
                <Sidebar
                    all_count=0
                    attention_count=0
                    starred_count=0
                    completed_count=0
                    gate=gate.clone()
                    tracked_repos=vec![]
                    repos_loading=true
                    repo_list_error=None
                    active_filter=SidebarFilter::All
                    repo_draft=repo_draft
                    set_repo_draft=set_repo_draft
                    repo_importing=repo_importing
                    repo_feedback=repo_feedback
                    on_set_filter=move |_| {}
                    general_active=false
                    general_room="general".to_string()
                    selected_room="#general".to_string()
                    selected_repo=None
                    on_select_repo=move |_| {}
                    on_open_general=move || {}
                    on_open_repo_room=move |_| {}
                    on_import_repo=move || {}
                />
                <div class="center">
                    <RunListPane
                        runs=vec![]
                        loading=true
                        selected_run=None
                        active_filter=SidebarFilter::All
                        starred_runs=BTreeSet::new()
                        on_toggle_star=move |_| {}
                        on_select_run=move |_| {}
                    />
                    <ReadingPane run=None/>
                </div>
                <LifecycleSidebar run=None gate=gate/>
            </div>
        </main>
    }
}

pub(crate) fn unauthenticated_shell() -> impl IntoView {
    let gate = GateUiState::disconnected(
        "authentication required",
        "log in to load factory runs".into(),
    );
    let (repo_draft, set_repo_draft) = signal(String::new());
    let (repo_importing, _) = signal(false);
    let (repo_feedback, _) = signal(None::<RepoImportFeedback>);

    view! {
        <main class="app-shell">
            <Topbar
                user_label="Not signed in".to_string()
                loading=false
                on_refresh=move |_| {}
                login_url=Some("/auth/login".to_string())
            />
            <div class="main-layout">
                <Sidebar
                    all_count=0
                    attention_count=0
                    starred_count=0
                    completed_count=0
                    gate=gate.clone()
                    tracked_repos=vec![]
                    repos_loading=false
                    repo_list_error=None
                    active_filter=SidebarFilter::All
                    repo_draft=repo_draft
                    set_repo_draft=set_repo_draft
                    repo_importing=repo_importing
                    repo_feedback=repo_feedback
                    on_set_filter=move |_| {}
                    general_active=false
                    general_room="general".to_string()
                    selected_room="#general".to_string()
                    selected_repo=None
                    on_select_repo=move |_| {}
                    on_open_general=move || {}
                    on_open_repo_room=move |_| {}
                    on_import_repo=move || {}
                />
                <div class="center">
                    <RunListPane
                        runs=vec![]
                        loading=false
                        selected_run=None
                        active_filter=SidebarFilter::All
                        starred_runs=BTreeSet::new()
                        on_toggle_star=move |_| {}
                        on_select_run=move |_| {}
                    />
                    <div class="reading-pane">
                        <div class="empty-state">
                            <div class="empty-state-icon">"?"</div>
                            <p class="empty-state-title">"Sign in to get started"</p>
                            <p class="empty-state-body">"Prior requires authentication before it can load factory runs and lifecycle state."</p>
                            <br/>
                            {login_button("Log in")}
                        </div>
                    </div>
                </div>
                <LifecycleSidebar run=None gate=gate/>
            </div>
        </main>
    }
}

pub(crate) fn error_shell(error: String) -> impl IntoView {
    let gate = GateUiState::disconnected("session lookup failed", error);
    let (repo_draft, set_repo_draft) = signal(String::new());
    let (repo_importing, _) = signal(false);
    let (repo_feedback, _) = signal(None::<RepoImportFeedback>);

    view! {
        <main class="app-shell">
            <Topbar
                user_label="Session error".to_string()
                loading=false
                on_refresh=move |_| {}
                login_url=Some("/auth/login".to_string())
            />
            <div class="main-layout">
                <Sidebar
                    all_count=0
                    attention_count=0
                    starred_count=0
                    completed_count=0
                    gate=gate.clone()
                    tracked_repos=vec![]
                    repos_loading=false
                    repo_list_error=None
                    active_filter=SidebarFilter::All
                    repo_draft=repo_draft
                    set_repo_draft=set_repo_draft
                    repo_importing=repo_importing
                    repo_feedback=repo_feedback
                    on_set_filter=move |_| {}
                    general_active=false
                    general_room="general".to_string()
                    selected_room="#general".to_string()
                    selected_repo=None
                    on_select_repo=move |_| {}
                    on_open_general=move || {}
                    on_open_repo_room=move |_| {}
                    on_import_repo=move || {}
                />
                <div class="center">
                    <RunListPane
                        runs=vec![]
                        loading=false
                        selected_run=None
                        active_filter=SidebarFilter::All
                        starred_runs=BTreeSet::new()
                        on_toggle_star=move |_| {}
                        on_select_run=move |_| {}
                    />
                    <div class="reading-pane">
                        <div class="empty-state">
                            <div class="empty-state-icon">"!"</div>
                            <p class="empty-state-title">"Session error"</p>
                            <p class="empty-state-body">{gate.status.clone()}</p>
                            <br/>
                            {login_button("Try again")}
                        </div>
                    </div>
                </div>
                <LifecycleSidebar run=None gate=gate/>
            </div>
        </main>
    }
}

#[allow(
    clippy::large_types_passed_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub(crate) fn app_shell(
    current_user: CurrentUser,
    gate: GateUiState,
    dashboard: FactoryDashboardView,
    repo_sidebar: RepoSidebarState,
    loading: bool,
    login_url: Option<&'static str>,
    on_refresh: impl FnMut(leptos::ev::MouseEvent) + Copy + 'static,
    on_data_changed: impl Fn() + Copy + 'static,
    ui: DashboardUiState,
) -> impl IntoView {
    let live = connect_live_socket(ui.set_chat_messages, ui.set_chat_sending, move || {
        on_data_changed();
    });
    let live_for_join = live.clone();
    Effect::new(move || {
        let room = ui.selected_room.get();
        if room.is_empty() || room == "#general" {
            return;
        }
        live_for_join.join_room(room);
    });
    let general_room = user_general_room(&current_user);
    let user_label = current_user.label();
    let filter = ui.sidebar_filter.get();
    let starred = ui.starred_runs.get();
    let all_runs = dashboard.runs;
    let attention_count = all_runs.iter().filter(|run| run.needs_attention).count();
    let completed_count = all_runs.iter().filter(|run| run.completed).count();
    let filtered_runs = all_runs
        .iter()
        .filter(|run| match filter {
            SidebarFilter::All => true,
            SidebarFilter::NeedsAttention => run.needs_attention,
            SidebarFilter::Starred => starred.contains(&run.id),
            SidebarFilter::Completed => run.completed,
        })
        .filter(|run| {
            ui.selected_repo
                .get()
                .as_ref()
                .is_none_or(|repo| &run.repo_label == repo)
        })
        .cloned()
        .collect::<Vec<_>>();
    let selected_run_id = match ui.selected_run.get() {
        Some(current) if filtered_runs.iter().any(|run| run.id == current) => Some(current),
        _ => filtered_runs.first().map(|run| run.id),
    };
    let selected_run =
        selected_run_id.and_then(|id| filtered_runs.iter().find(|run| run.id == id).cloned());
    let on_import_repo = move || {
        let repo_spec = ui.repo_draft.get().trim().to_string();
        if repo_spec.is_empty() || ui.repo_importing.get() {
            return;
        }

        let (owner, name, clone_url) = match parse_repo_spec(&repo_spec) {
            Ok(spec) => spec,
            Err(error) => {
                ui.set_repo_feedback.set(Some(RepoImportFeedback {
                    message: error,
                    is_error: true,
                }));
                return;
            }
        };

        ui.set_repo_importing.set(true);
        ui.set_repo_feedback.set(None);

        let set_repo_importing = ui.set_repo_importing;
        let set_repo_feedback = ui.set_repo_feedback;
        let set_repo_draft = ui.set_repo_draft;
        leptos::task::spawn_local(async move {
            let result = import_repo(clone_url, owner.clone(), name.clone(), None).await;

            match result {
                Ok(imported) => {
                    set_repo_draft.set(String::new());
                    set_repo_feedback.set(Some(RepoImportFeedback {
                        message: format!("Tracking {owner}/{name} in {}", imported.room),
                        is_error: false,
                    }));
                    on_data_changed();
                }
                Err(error) => {
                    set_repo_feedback.set(Some(RepoImportFeedback {
                        message: format!("Import failed for {owner}/{name}: {error}"),
                        is_error: true,
                    }));
                }
            }

            set_repo_importing.set(false);
        });
    };
    let open_room = move |room: String| {
        ui.set_selected_room.set(room);
        ui.set_chat_messages.set(Vec::new());
        ui.set_chat_sending.set(false);
        ui.set_center_mode.set(CenterMode::General);
    };

    view! {
        <main class="app-shell">
            <Topbar
                user_label=user_label
                loading=loading
                on_refresh=on_refresh
                login_url=login_url.map(str::to_string)
            />
            <div class="main-layout">
                <Sidebar
                    all_count=all_runs.len()
                    attention_count=attention_count
                    starred_count=starred.len()
                    completed_count=completed_count
                    gate=gate.clone()
                    tracked_repos=repo_sidebar.repos
                    repos_loading=repo_sidebar.loading
                    repo_list_error=repo_sidebar.error
                    active_filter=filter
                    repo_draft=ui.repo_draft
                    set_repo_draft=ui.set_repo_draft
                    repo_importing=ui.repo_importing
                    repo_feedback=ui.repo_feedback
                    on_set_filter=move |value| {
                        ui.set_sidebar_filter.set(value);
                        ui.set_center_mode.set(CenterMode::Runs);
                    }
                    general_active=ui.center_mode.get() == CenterMode::General
                    general_room=general_room.clone()
                    selected_room=ui.selected_room.get()
                    selected_repo=ui.selected_repo.get()
                    on_select_repo=move |repo| {
                        let already_selected = ui.selected_repo.get().as_ref() == Some(&repo);
                        if already_selected {
                            ui.set_selected_repo.set(None);
                        } else {
                            ui.set_selected_repo.set(Some(repo));
                        }
                        ui.set_center_mode.set(CenterMode::Runs);
                    }
                    on_open_general={
                        let general_room = general_room.clone();
                        move || open_room(general_room.clone())
                    }
                    on_open_repo_room=move |room| open_room(room)
                    on_import_repo=on_import_repo
                />
                <div class=if ui.center_mode.get() == CenterMode::General {
                    "center conversation-mode"
                } else {
                    "center"
                }>
                    <RunListPane
                        runs=filtered_runs.clone()
                        loading=loading
                        selected_run=selected_run_id
                        active_filter=filter
                        starred_runs=starred
                        on_toggle_star=move |run_id| {
                            ui.set_starred_runs.update(|runs| {
                                if !runs.insert(run_id) {
                                    runs.remove(&run_id);
                                }
                            });
                        }
                        on_select_run=move |run_id| {
                            ui.set_selected_run.set(Some(run_id));
                            ui.set_center_mode.set(CenterMode::Runs);
                        }
                    />
                    <ReadingPane run=selected_run.clone()/>
                    <GeneralConversationView
                        room=ui.selected_room.get()
                        messages=ui.chat_messages
                        set_messages=ui.set_chat_messages
                        sending=ui.chat_sending
                        set_sending=ui.set_chat_sending
                        live=live.clone()
                    />
                </div>
                <LifecycleSidebar run=selected_run gate=gate/>
            </div>
        </main>
    }
}

#[component]
fn Topbar<F>(
    user_label: String,
    loading: bool,
    on_refresh: F,
    login_url: Option<String>,
) -> impl IntoView
where
    F: FnMut(leptos::ev::MouseEvent) + Copy + 'static,
{
    let theme = expect_context::<RwSignal<bool>>();

    let auth_action = if let Some(login_href) = login_url {
        view! {
            <form method="get" action=login_href>
                <input type="hidden" name="return_to" value="/app"/>
                <button class="btn-primary" type="submit">"Log in"</button>
            </form>
        }
        .into_any()
    } else {
        view! {
            <form method="post" action="/auth/logout">
                <button class="btn-secondary" type="submit">"Log out"</button>
            </form>
        }
        .into_any()
    };

    view! {
        <header class="topbar">
            <div class="topbar-left">
                <div class="brand">
                    <div class="brand-mark">"P"</div>
                    "Prior"
                </div>
            </div>

            <div class="topbar-right">
                <button
                    class="icon-btn"
                    type="button"
                    on:click=move |_| theme.update(|dark| *dark = !*dark)
                    title=move || if theme.get() { "Switch to light mode" } else { "Switch to dark mode" }
                >
                    {move || if theme.get() { "☀" } else { "☾" }}
                </button>
                <button class="icon-btn" on:click=on_refresh disabled=loading title="Refresh">
                    {if loading { "⟳" } else { "↻" }}
                </button>
                {auth_action}
                <div class="avatar" title=user_label.clone()>
                    {user_label.chars().next().unwrap_or('?').to_uppercase().to_string()}
                </div>
            </div>
        </header>
    }
}

#[component]
fn Sidebar<F, G, H, I, J>(
    all_count: usize,
    attention_count: usize,
    starred_count: usize,
    completed_count: usize,
    gate: GateUiState,
    tracked_repos: Vec<RepoEntry>,
    repos_loading: bool,
    repo_list_error: Option<String>,
    active_filter: SidebarFilter,
    repo_draft: ReadSignal<String>,
    set_repo_draft: WriteSignal<String>,
    repo_importing: ReadSignal<bool>,
    repo_feedback: ReadSignal<Option<RepoImportFeedback>>,
    on_set_filter: F,
    general_active: bool,
    general_room: String,
    selected_room: String,
    selected_repo: Option<String>,
    on_select_repo: G,
    on_open_general: H,
    on_open_repo_room: I,
    on_import_repo: J,
) -> impl IntoView
where
    F: Fn(SidebarFilter) + Copy + 'static,
    G: Fn(String) + Copy + Send + 'static,
    H: Fn() + 'static,
    I: Fn(String) + Copy + Send + 'static,
    J: Fn() + Copy + 'static,
{
    let connection_label = match gate.connection {
        ConnectionStatus::Connecting => "Connecting...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Disconnected => "Disconnected",
    };
    let connection_class = match gate.connection {
        ConnectionStatus::Connecting => "status-pill status-pill--loading",
        ConnectionStatus::Connected => "status-pill status-pill--ok",
        ConnectionStatus::Disconnected => "status-pill status-pill--down",
    };

    view! {
        <nav class="sidebar">
            <div class="sidebar-section">
                <div class="sidebar-section-label">"Rooms"</div>
                <button
                    class=if general_active && selected_room == general_room {
                        "sidebar-item active"
                    } else {
                        "sidebar-item"
                    }
                    type="button"
                    on:click=move |_| on_open_general()
                >
                    <div class="sidebar-item-left">
                        <span class="sidebar-item-icon">"#"</span>
                        <span class="sidebar-item-label">"general"</span>
                    </div>
                </button>
                {tracked_repos.iter().cloned().map(|repo| {
                    let room_label = repo.room;
                    let is_selected = general_active && selected_room == room_label;
                    let room_label_for_click = room_label.clone();

                    view! {
                        <button
                            class=if is_selected { "sidebar-item active" } else { "sidebar-item" }
                            type="button"
                            on:click=move |_| on_open_repo_room(room_label_for_click.clone())
                        >
                            <div class="sidebar-item-left">
                                <span class="sidebar-item-icon">"#"</span>
                                <span class="sidebar-item-label">{room_label}</span>
                            </div>
                        </button>
                    }
                }).collect_view()}
            </div>

            <div class="sidebar-divider"></div>

            <div class="sidebar-section">
                <div class="sidebar-section-label">"Runs"</div>
                <FilterButton
                    label="All Runs"
                    icon="◉"
                    count=Some(all_count)
                    active=!general_active && active_filter == SidebarFilter::All
                    on_click=move |_| on_set_filter(SidebarFilter::All)
                />
                <FilterButton
                    label="Needs Attention"
                    icon="⚑"
                    count=Some(attention_count)
                    active=!general_active && active_filter == SidebarFilter::NeedsAttention
                    on_click=move |_| on_set_filter(SidebarFilter::NeedsAttention)
                />
                <FilterButton
                    label="Starred"
                    icon="★"
                    count=Some(starred_count)
                    active=!general_active && active_filter == SidebarFilter::Starred
                    on_click=move |_| on_set_filter(SidebarFilter::Starred)
                />
                <FilterButton
                    label="Completed"
                    icon="✓"
                    count=Some(completed_count)
                    active=!general_active && active_filter == SidebarFilter::Completed
                    on_click=move |_| on_set_filter(SidebarFilter::Completed)
                />
            </div>

            <div class="sidebar-divider"></div>

            <div class="sidebar-section">
                <div class="sidebar-section-label">"Tracked Repos"</div>
                <div class="sidebar-repo-import">
                    <input
                        class="sidebar-text-input"
                        type="text"
                        placeholder="owner/repo or GitHub URL"
                        prop:value=move || repo_draft.get()
                        on:input=move |event| set_repo_draft.set(event_target_value(&event))
                        disabled=move || repo_importing.get()
                    />
                    <button
                        class="btn-secondary sidebar-import-btn"
                        type="button"
                        on:click=move |_| on_import_repo()
                        disabled=move || repo_importing.get() || repo_draft.get().trim().is_empty()
                    >
                        {move || if repo_importing.get() { "Importing..." } else { "Track Repo" }}
                    </button>
                </div>
                {move || repo_feedback.get().map(|feedback| {
                    let feedback_class = if feedback.is_error {
                        "sidebar-feedback sidebar-feedback--error"
                    } else {
                        "sidebar-feedback sidebar-feedback--ok"
                    };

                    view! { <div class=feedback_class>{feedback.message}</div> }
                })}
                {repo_list_error.as_ref().map(|error| {
                    view! {
                        <div class="sidebar-feedback sidebar-feedback--error">
                            {format!("Could not load tracked repos: {error}")}
                        </div>
                    }
                })}
                <div class="sidebar-repo-list">
                    {if repos_loading {
                        view! { <div class="sidebar-empty-copy">"Loading tracked repositories..."</div> }.into_any()
                    } else if tracked_repos.is_empty() {
                        view! { <div class="sidebar-empty-copy">"No repositories tracked yet. Import one to connect runs back to code." </div> }.into_any()
                    } else {
                        view! {
                            <For
                                each=move || tracked_repos.clone().into_iter()
                                key=|repo| repo.id
                                children=move |repo| {
                                    let repo_label = format!("{}/{}", repo.owner, repo.name);
                                    let repo_label_for_click = repo_label.clone();
                                    let is_selected = selected_repo.as_ref() == Some(&repo_label);
                                    let repo_meta = repo
                                        .default_base_branch
                                        .clone()
                                        .filter(|branch| !branch.trim().is_empty());

                                    view! {
                                        <button
                                            class=if is_selected {
                                                "sidebar-item active sidebar-repo-item"
                                            } else {
                                                "sidebar-item sidebar-repo-item"
                                            }
                                            type="button"
                                            on:click=move |_| on_select_repo(repo_label_for_click.clone())
                                        >
                                            <div class="sidebar-item-left">
                                                <span class="sidebar-item-icon">"⎇"</span>
                                                <div class="sidebar-repo-copy">
                                                    <span class="sidebar-item-label sidebar-repo-label">{repo_label}</span>
                                                    {repo_meta.map(|meta| view! {
                                                        <span class="sidebar-repo-subtle">{meta}</span>
                                                    })}
                                                </div>
                                            </div>
                                        </button>
                                    }
                                }
                            />
                        }.into_any()
                    }}
                </div>
            </div>

            <div class="sidebar-divider"></div>
            <div class="sidebar-section">
                <div class="sidebar-item sidebar-item--static">
                    <div class="sidebar-item-left">
                        <span class=connection_class></span>
                        <span class="sidebar-item-label" style="font-size: 12px; color: var(--text-muted);">
                            {connection_label}
                        </span>
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn FilterButton<F>(
    label: &'static str,
    icon: &'static str,
    count: Option<usize>,
    active: bool,
    on_click: F,
) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + Copy + 'static,
{
    let class = if active {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };

    view! {
        <button class=class type="button" on:click=on_click>
            <div class="sidebar-item-left">
                <span class="sidebar-item-icon">{icon}</span>
                <span class="sidebar-item-label">{label}</span>
            </div>
            {count.map(|value| view! { <span class="sidebar-count">{value}</span> })}
        </button>
    }
}

#[component]
fn RunListPane<F, G>(
    runs: Vec<FactoryRunDisplay>,
    loading: bool,
    selected_run: Option<i64>,
    active_filter: SidebarFilter,
    starred_runs: BTreeSet<i64>,
    on_toggle_star: G,
    on_select_run: F,
) -> impl IntoView
where
    F: Fn(i64) + Copy + Send + 'static,
    G: Fn(i64) + Copy + Send + 'static,
{
    let run_count = runs.len();
    let display_runs = runs.clone();
    let empty_title = match active_filter {
        SidebarFilter::All => "No factory runs found",
        SidebarFilter::Starred => "No starred runs",
        SidebarFilter::NeedsAttention => "No runs need attention",
        SidebarFilter::Completed => "No completed runs yet",
    };
    let empty_body = match active_filter {
        SidebarFilter::All => {
            "Use the prompt bar to start a run, or refresh once Prior has created work."
        }
        SidebarFilter::Starred => "Star runs from the inbox to pin them here.",
        SidebarFilter::NeedsAttention => {
            "Blocked runs, failed verification, and failed checkpoints will surface here."
        }
        SidebarFilter::Completed => {
            "Completed runs will show here when the factory reaches delivery or review-ready states."
        }
    };

    view! {
        <div class="room-list-pane">
            <div class="room-list-toolbar">
                <div class="room-list-toolbar-left">
                    <span class="toolbar-copy">"Runs"</span>
                </div>
                <span class="room-list-info">
                    {if loading { "Loading...".to_string() } else { format!("{run_count} run(s)") }}
                </span>
            </div>

            <div class="room-list">
                {if display_runs.is_empty() && !loading {
                    view! {
                        <div class="empty-state">
                            <p class="empty-state-title">{empty_title}</p>
                            <p class="empty-state-body">{empty_body}</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <For
                            each=move || display_runs.clone().into_iter()
                            key=|run| run.id
                            children=move |run| {
                                let is_selected = selected_run == Some(run.id);
                                let is_starred = starred_runs.contains(&run.id);
                                let row_class = if is_selected { "room-row selected" } else { "room-row" };
                                let star_class = if is_starred { "room-star starred" } else { "room-star" };
                                let run_id = run.id;
                                let star_run_id = run.id;
                                let tag_class = if run.needs_attention {
                                    "room-tag err"
                                } else if run.completed {
                                    "room-tag ok"
                                } else {
                                    "room-tag info"
                                };
                                let tag_text = if run.needs_attention {
                                    "Attention"
                                } else if run.completed {
                                    "Complete"
                                } else {
                                    "Active"
                                };

                                view! {
                                    <div class=row_class on:click=move |_| on_select_run(run_id)>
                                        <div class="room-row-check">
                                            <button
                                                class=star_class
                                                type="button"
                                                on:click=move |event| {
                                                    event.stop_propagation();
                                                    on_toggle_star(star_run_id);
                                                }
                                                title={if is_starred { "Remove star" } else { "Star run" }}
                                            >
                                                {if is_starred { "★" } else { "☆" }}
                                            </button>
                                        </div>
                                        <div class="room-content">
                                            <span class="room-sender">{run.title.clone()}</span>
                                            <span class="room-snippet">{run.summary.clone()}</span>
                                        </div>
                                        <div class="room-meta">
                                            <span class="room-date">{run.current_stage.clone()}</span>
                                            <span class=tag_class>{tag_text}</span>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn ReadingPane(run: Option<FactoryRunDisplay>) -> impl IntoView {
    view! {
        <div class="reading-pane">
            {run.map_or_else(
                || {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"◉"</div>
                            <p class="empty-state-title">"Select a run"</p>
                            <p class="empty-state-body">"Choose a run from the inbox to inspect the factory updates that Prior projects for web display."</p>
                        </div>
                    }
                    .into_any()
                },
                |run| {
                    view! {
                        <div class="reading-pane-header">
                            <div class="reading-pane-title">{run.title.clone()}</div>
                            <div class="reading-pane-subtitle">
                                {format!("{} • {} • {} room(s)", run.repo_label, run.status, run.active_room_count)}
                            </div>
                        </div>
                        <div class="reading-thread">
                            <For
                                each=move || run.updates.clone()
                                key=|update| (update.ts, update.title.clone())
                                children=move |update| view! { <RunUpdateCard update=update/> }
                            />
                        </div>
                    }
                    .into_any()
                },
            )}
        </div>
    }
}

#[component]
fn RunUpdateCard(update: FactoryRunUpdateDisplay) -> impl IntoView {
    let tag_class = match update.kind.as_str() {
        "blocker" => "room-tag err",
        "checkpoint" => "room-tag warn",
        _ => "room-tag info",
    };

    view! {
        <div class="run-card">
            <div class="run-card-header">
                <span class="run-card-id">{update.title}</span>
                <span class=tag_class>{update.kind}</span>
            </div>
            <div class="run-card-detail">{update.body}</div>
        </div>
    }
}

#[component]
fn LifecycleSidebar(run: Option<FactoryRunDisplay>, gate: GateUiState) -> impl IntoView {
    let server = gate.server_name.clone().unwrap_or_else(|| "unknown".into());
    let lifecycle = run
        .as_ref()
        .map_or_else(Vec::new, |run| run.lifecycle.clone());
    let lifecycle_for_progress = lifecycle.clone();
    let lifecycle_for_list = lifecycle.clone();
    let selected_run_label = run.as_ref().map_or_else(
        || "No active runs".to_string(),
        |run| format!("Run {} • {}", run.id, run.current_stage),
    );
    let selected_run_status = run
        .as_ref()
        .map_or_else(|| "None".to_string(), |run| run.status.clone());
    let selected_run_summary = run.as_ref().map_or_else(
        || "No run selected.".to_string(),
        |run| {
            format!(
                "{} issue(s), {} blocker(s), {} room(s)",
                run.issue_count, run.blocker_count, run.active_room_count,
            )
        },
    );
    let room_labels = run
        .as_ref()
        .map_or_else(Vec::new, |run| run.room_labels.clone());

    view! {
        <aside class="right-sidebar">
            <div class="right-sidebar-header">
                <span class="right-sidebar-title">"Lifecycle"</span>
            </div>

            <div>
                <label style="font-size: 12px; color: var(--text-muted); display: block; margin-bottom: 4px;">"Selected Run"</label>
                <select class="phase-selector">
                    <option>{selected_run_label}</option>
                </select>
            </div>

            <div class="lifecycle-progress">
                <For
                    each=move || lifecycle_for_progress.clone()
                    key=|phase| phase.name.clone()
                    children=move |phase| {
                        let class = match phase.state.as_str() {
                            "done" => "lifecycle-progress-seg done",
                            "active" => "lifecycle-progress-seg active",
                            _ => "lifecycle-progress-seg",
                        };
                        view! { <div class=class></div> }
                    }
                />
            </div>

            <div class="lifecycle-phases">
                {if lifecycle.is_empty() {
                    view! {
                        <div class="lifecycle-phase future">
                            <div class="phase-icon pending">"○"</div>
                            <div class="phase-info">
                                <div class="phase-name">"No run selected"</div>
                                <div class="phase-detail">"Choose a run to inspect lifecycle state."</div>
                            </div>
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <For
                            each=move || lifecycle_for_list.clone()
                            key=|phase| phase.name.clone()
                            children=move |phase| view! { <LifecyclePhase phase=phase/> }
                        />
                    }
                    .into_any()
                }}
            </div>

            <div class="right-sidebar-divider"></div>

            <div class="run-card">
                <div class="run-card-header">
                    <span class="run-card-id">"Selected Run"</span>
                    <span class="run-card-status">
                        {selected_run_status}
                    </span>
                </div>
                <div class="run-card-detail">{selected_run_summary}</div>
            </div>

            <div class="right-sidebar-divider"></div>

            <div class="run-card">
                <div class="run-card-header">
                    <span class="run-card-id">"Transport"</span>
                </div>
                <div class="run-card-detail">
                    {format!("Server: {server}")}
                    <br/>
                    {format!("Gate: {}", gate.gate_url)}
                    <br/>
                    {format!("Visible runs: {}", run.as_ref().map_or(0, |_| 1))}
                </div>
            </div>

            {(!room_labels.is_empty()).then(|| view! {
                <>
                    <div class="right-sidebar-divider"></div>
                    <div class="run-card">
                        <div class="run-card-header">
                            <span class="run-card-id">"Run Rooms"</span>
                        </div>
                        <div class="run-card-detail">
                            <For
                                each=move || room_labels.clone()
                                key=|room| room.clone()
                                children=move |room| view! { <div>{room}</div> }
                            />
                        </div>
                    </div>
                </>
            })}
        </aside>
    }
}

#[component]
fn LifecyclePhase(phase: FactoryLifecyclePhaseDisplay) -> impl IntoView {
    let (phase_class, icon_class, icon_text) = match phase.state.as_str() {
        "done" => ("lifecycle-phase done", "phase-icon done", "✓"),
        "active" => ("lifecycle-phase active", "phase-icon active", "◉"),
        _ => ("lifecycle-phase future", "phase-icon pending", "○"),
    };

    view! {
        <div class=phase_class>
            <div class=icon_class>{icon_text}</div>
            <div class="phase-info">
                <div class="phase-name">{phase.name}</div>
                <div class="phase-detail">{phase.detail}</div>
            </div>
        </div>
    }
}

#[component]
fn GeneralConversationView(
    room: String,
    messages: ReadSignal<Vec<ChatMessage>>,
    set_messages: WriteSignal<Vec<ChatMessage>>,
    sending: ReadSignal<bool>,
    set_sending: WriteSignal<bool>,
    live: LiveSocket,
) -> impl IntoView {
    let (draft, set_draft) = signal(String::new());
    let room_for_click = room.clone();
    let room_for_keydown = room.clone();
    let room_label = display_room_name(&room);
    let is_general_room = is_general_room(&room);
    let live_for_click = live.clone();
    let live_for_keydown = live.clone();

    let on_send_click = move |_: leptos::ev::MouseEvent| {
        dispatch_room_message(
            room_for_click.clone(),
            draft,
            set_draft,
            set_messages,
            sending,
            set_sending,
            live_for_click.clone(),
        );
    };

    let on_keydown = move |event: leptos::ev::KeyboardEvent| {
        if event.key() == "Enter" && !event.shift_key() {
            event.prevent_default();
            dispatch_room_message(
                room_for_keydown.clone(),
                draft,
                set_draft,
                set_messages,
                sending,
                set_sending,
                live_for_keydown.clone(),
            );
        }
    };

    view! {
        <div class="conversation-view">
            <div class="conversation-topbar">
                <div class="conversation-room-info">
                    <div class="conversation-room-name">{room_label.clone()}</div>
                    <div class="conversation-room-context">
                        {if is_general_room {
                            "General intake for prompts, briefs, and direct requests to Prior.".to_string()
                        } else {
                            "Repo-specific room for directing work against this repository.".to_string()
                        }}
                    </div>
                </div>
                <div class="conversation-participants">
                    <span class="participant-label">
                        {move || if sending.get() { "Prior is responding" } else { "Ready" }}
                    </span>
                </div>
            </div>

            <div class="conversation-messages">
                {move || {
                    let msgs = messages.get();
                    if msgs.is_empty() {
                        view! {
                            <div class="chat-modal-empty">
                                <p class="chat-modal-empty-title">"What would you like to work on?"</p>
                                <p class="chat-modal-empty-body">"Describe a task, paste a project brief, reference an issue - Prior will interpret and can kick off a factory run."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <For
                                each=move || {
                                    msgs.clone().into_iter().enumerate().collect::<Vec<_>>()
                                }
                                key=|(index, _)| *index
                                children=move |(_, msg)| {
                                    let bubble_class = if msg.is_human {
                                        "chat-bubble chat-bubble--human"
                                    } else if msg.topic == "door:thought" {
                                        "chat-bubble chat-bubble--assistant chat-bubble--thought"
                                    } else if msg.topic == "door:tool" || msg.topic == "door:tool_result" {
                                        "chat-bubble chat-bubble--assistant chat-bubble--tool"
                                    } else {
                                        "chat-bubble chat-bubble--assistant"
                                    };
                                    let meta = chat_message_meta(&msg);
                                    let rendered_content = render_markdown(&msg.content);

                                    view! {
                                        <div class=bubble_class>
                                            <div class="chat-bubble-body">
                                                {meta.map(|meta| view! {
                                                    <div class="chat-bubble-meta">{meta}</div>
                                                })}
                                                <div
                                                    class="chat-bubble-content chat-bubble-markdown"
                                                    inner_html=rendered_content
                                                ></div>
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        }.into_any()
                    }
                }}

                {move || sending.get().then(|| view! {
                    <div class="chat-bubble chat-bubble--assistant">
                        <div class="chat-bubble-body">
                            <div class="chat-bubble-content chat-bubble-typing">"Thinking..."</div>
                        </div>
                    </div>
                })}
            </div>

            <div class="conversation-input-area">
                <div class="conversation-input-wrap">
                    <textarea
                        class="chat-input"
                        prop:value=move || draft.get()
                        on:input=move |event| set_draft.set(event_target_value(&event))
                        on:keydown=on_keydown
                        placeholder="Describe a task, paste a brief, or ask a question..."
                        rows="3"
                        disabled=move || sending.get()
                    ></textarea>
                    <div class="conversation-input-actions">
                        <div class="conversation-input-left">
                            <span class="conversation-room-context">{format!("Messages go to {room_label}.")}</span>
                        </div>
                        <button
                            class="send-btn"
                            type="button"
                            on:click=on_send_click
                            disabled=move || sending.get() || draft.get().trim().is_empty()
                        >
                            "Send"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn set_chat_messages_with_human(set_messages: &WriteSignal<Vec<ChatMessage>>, content: &str) {
    let content = content.to_string();
    set_messages.update(move |msgs| {
        msgs.push(ChatMessage {
            topic: "user".to_string(),
            actor: None,
            content,
            is_human: true,
        });
    });
}

fn render_markdown(content: &str) -> String {
    let parser = Parser::new_ext(content, markdown_options()).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(CowStr::Boxed(
            html_escape::encode_safe(raw.as_ref())
                .into_owned()
                .into_boxed_str(),
        )),
        other => other,
    });

    let mut rendered = String::new();
    html::push_html(&mut rendered, parser);
    rendered
}

fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options
}

fn chat_message_meta(message: &ChatMessage) -> Option<String> {
    if message.is_human {
        return None;
    }

    let label = match message.topic.as_str() {
        "door:thought" => "Thought",
        "door:tool" => "Tool Call",
        "door:tool_result" => "Tool Result",
        "door:chat" | "door:message" => "Prior",
        "error" => "Error",
        _ => "Event",
    };

    match message.actor.as_deref().filter(|actor| !actor.is_empty()) {
        Some(actor) => Some(format!("{label} · {actor}")),
        None => Some(label.to_string()),
    }
}

fn dispatch_room_message(
    room: String,
    draft: ReadSignal<String>,
    set_draft: WriteSignal<String>,
    set_messages: WriteSignal<Vec<ChatMessage>>,
    sending: ReadSignal<bool>,
    set_sending: WriteSignal<bool>,
    live: LiveSocket,
) {
    let content = draft.get().trim().to_string();
    if content.is_empty() || sending.get() {
        return;
    }

    set_draft.set(String::new());
    set_chat_messages_with_human(&set_messages, &content);
    set_sending.set(true);

    if live.send_room_message(room.clone(), content.clone()) {
        return;
    }

    let content_clone = content.clone();
    leptos::task::spawn_local(async move {
        let result = send_room_message(room, content_clone).await;

        match result {
            Ok(entries) => {
                set_messages.update(|msgs| {
                    for entry in entries {
                        if entry.content.trim().is_empty() {
                            continue;
                        }
                        msgs.push(ChatMessage {
                            topic: entry.topic,
                            actor: entry.actor,
                            content: entry.content,
                            is_human: false,
                        });
                    }
                });
            }
            Err(error) => {
                set_messages.update(|msgs| {
                    msgs.push(ChatMessage {
                        topic: "error".to_string(),
                        actor: None,
                        content: format!("Error: {error}"),
                        is_human: false,
                    });
                });
            }
        }
        set_sending.set(false);
    });
}

fn parse_repo_spec(input: &str) -> Result<(String, String, String), String> {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Enter a GitHub repo as owner/name or a full GitHub URL.".into());
    }

    let path = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("git@github.com:"))
        .unwrap_or(trimmed)
        .trim_end_matches(".git");
    let mut parts = path.split('/').filter(|part| !part.is_empty());
    let owner = parts
        .next()
        .ok_or_else(|| "Repo must include an owner.".to_string())?;
    let name = parts
        .next()
        .ok_or_else(|| "Repo must include a repository name.".to_string())?;
    if parts.next().is_some() {
        return Err("Repo format should be owner/name or a direct GitHub repo URL.".into());
    }

    Ok((
        owner.to_string(),
        name.to_string(),
        format!("https://github.com/{owner}/{name}.git"),
    ))
}

fn user_general_room(user: &CurrentUser) -> String {
    format!("user:{}#general", user.sub)
}

fn is_general_room(room: &str) -> bool {
    room.ends_with("#general")
}

fn display_room_name(room: &str) -> String {
    if is_general_room(room) {
        "#general".to_string()
    } else {
        room.to_string()
    }
}

fn login_button(label: &'static str) -> impl IntoView {
    view! {
        <form method="get" action="/auth/login">
            <input type="hidden" name="return_to" value="/app"/>
            <button class="btn-primary" type="submit">{label}</button>
        </form>
    }
}
