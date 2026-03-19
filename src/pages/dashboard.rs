use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyView;
use leptos::tachys::view::any_view::IntoAny;
use std::collections::BTreeSet;

use crate::auth::user::current_auth_state;
use crate::net::prior::send_room_message;
use crate::net::prior_gate::refresh_dashboard;
use crate::state::auth::{AuthState, CurrentUser};
use crate::state::gate::{ConnectionStatus, GateUiState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SidebarFilter {
    All,
    NeedsAttention,
    Starred,
    Completed,
}

#[derive(Debug, Clone)]
struct ChatMessage {
    from: String,
    content: String,
    is_human: bool,
}

// ── Page Entry ──────────────────────────────────────────

#[component]
pub fn DashboardPage() -> AnyView {
    let auth = Resource::new(|| (), |_| current_auth_state());

    let (refresh_tick, set_refresh_tick) = signal(0_u64);
    let gate = Resource::new(move || refresh_tick.get(), |_| refresh_dashboard());
    let (selected_room, set_selected_room) = signal(None::<String>);
    let (sidebar_filter, set_sidebar_filter) = signal(SidebarFilter::All);
    let (starred_rooms, set_starred_rooms) = signal(BTreeSet::<String>::new());
    let (chat_open, set_chat_open) = signal(false);
    let (chat_messages, set_chat_messages) = signal(Vec::<ChatMessage>::new());
    let (chat_sending, set_chat_sending) = signal(false);

    let on_refresh = move |_| set_refresh_tick.update(|count| *count += 1);

    view! {
        <Suspense fallback=move || {
            shell_loading().into_any()
        }>
            {move || match auth.get() {
                None => shell_loading().into_any(),
                Some(Ok(AuthState::Authenticated(current_user))) => {
                    let fallback_user = current_user.clone();
                    let content_user = current_user.clone();
                    view! {
                        <Suspense fallback=move || {
                            app_shell(
                                fallback_user.clone(),
                                GateUiState::loading("loading gate snapshot"),
                                true,
                                None,
                                on_refresh,
                                selected_room,
                                set_selected_room,
                                sidebar_filter,
                                set_sidebar_filter,
                                starred_rooms,
                                set_starred_rooms,
                                chat_open,
                                set_chat_open,
                                chat_messages,
                                set_chat_messages,
                                chat_sending,
                                set_chat_sending,
                            )
                        }>
                            {move || {
                                let gate_state = gate.get().map_or_else(
                                    || GateUiState::loading("loading gate snapshot"),
                                    |result| match result {
                                        Ok(state) => state,
                                        Err(error) => GateUiState::disconnected(
                                            "server function failed",
                                            error.to_string(),
                                        ),
                                    },
                                );

                                app_shell(
                                    content_user.clone(),
                                    gate_state,
                                    false,
                                    None,
                                    on_refresh,
                                    selected_room,
                                    set_selected_room,
                                    sidebar_filter,
                                    set_sidebar_filter,
                                    starred_rooms,
                                    set_starred_rooms,
                                    chat_open,
                                    set_chat_open,
                                    chat_messages,
                                    set_chat_messages,
                                    chat_sending,
                                    set_chat_sending,
                                )
                            }}
                        </Suspense>
                    }
                    .into_any()
                }
                Some(Ok(AuthState::Anonymous)) => unauthenticated_shell().into_any(),
                Some(Err(error)) => error_shell(error.to_string()).into_any(),
            }}
        </Suspense>
    }
    .into_any()
}

// ── Shell Variants ──────────────────────────────────────

fn shell_loading() -> impl IntoView {
    let gate = GateUiState::loading("confirming session");
    view! {
        <main class="app-shell">
            <Topbar
                user_label="Loading\u{2026}".to_string()
                gate=gate.clone()
                loading=true
                on_refresh=move |_| {}
                login_url=None
                on_open_chat=move |_| {}
            />
            <div class="main-layout">
                <Sidebar
                    rooms=vec![]
                    gate=gate.clone()
                    selected_room=None
                    active_filter=SidebarFilter::All
                    starred_rooms=BTreeSet::new()
                    on_set_filter=move |_| {}
                    on_select_room=move |_| {}
                />
                <div class="center">
                    <RoomListPane
                        rooms=vec![]
                        loading=true
                        selected_room=None
                        active_filter=SidebarFilter::All
                        starred_rooms=BTreeSet::new()
                        on_toggle_star=move |_| {}
                        on_select_room=move |_| {}
                    />
                    <ReadingPane/>
                </div>
                <LifecycleSidebar gate=gate/>
            </div>
        </main>
    }
}

fn app_shell(
    current_user: CurrentUser,
    gate: GateUiState,
    loading: bool,
    login_url: Option<&'static str>,
    on_refresh: impl FnMut(leptos::ev::MouseEvent) + Copy + 'static,
    selected_room: ReadSignal<Option<String>>,
    set_selected_room: WriteSignal<Option<String>>,
    sidebar_filter: ReadSignal<SidebarFilter>,
    set_sidebar_filter: WriteSignal<SidebarFilter>,
    starred_rooms: ReadSignal<BTreeSet<String>>,
    set_starred_rooms: WriteSignal<BTreeSet<String>>,
    chat_open: ReadSignal<bool>,
    set_chat_open: WriteSignal<bool>,
    chat_messages: ReadSignal<Vec<ChatMessage>>,
    set_chat_messages: WriteSignal<Vec<ChatMessage>>,
    chat_sending: ReadSignal<bool>,
    set_chat_sending: WriteSignal<bool>,
) -> impl IntoView {
    let user_label = current_user.label();
    let filter = sidebar_filter.get();
    let starred = starred_rooms.get();
    let rooms = gate
        .rooms
        .clone()
        .into_iter()
        .filter(|room| match filter {
            SidebarFilter::All => true,
            SidebarFilter::Starred => starred.contains(room),
            SidebarFilter::NeedsAttention | SidebarFilter::Completed => false,
        })
        .collect::<Vec<_>>();

    Effect::new({
        let rooms = rooms.clone();
        move |_| {
            let current = selected_room.get();
            let next_selection = match current {
                Some(current) if rooms.iter().any(|room| room == &current) => Some(current),
                _ => rooms.first().cloned(),
            };
            set_selected_room.set(next_selection);
        }
    });

    view! {
        <main class="app-shell">
            <Topbar
                user_label=user_label
                gate=gate.clone()
                loading=loading
                on_refresh=on_refresh
                login_url=login_url.map(str::to_string)
                on_open_chat=move |_| set_chat_open.set(true)
            />
            <div class="main-layout">
                <Sidebar
                    rooms=rooms.clone()
                    gate=gate.clone()
                    selected_room=selected_room.get()
                    active_filter=filter
                    starred_rooms=starred.clone()
                    on_set_filter=move |value| set_sidebar_filter.set(value)
                    on_select_room=move |room| set_selected_room.set(Some(room))
                />
                <div class="center">
                    <RoomListPane
                        rooms=rooms
                        loading=loading
                        selected_room=selected_room.get()
                        active_filter=filter
                        starred_rooms=starred
                        on_toggle_star=move |room| {
                            set_starred_rooms.update(|rooms| {
                                if !rooms.insert(room.clone()) {
                                    rooms.remove(&room);
                                }
                            });
                        }
                        on_select_room=move |room| set_selected_room.set(Some(room))
                    />
                    <ReadingPane/>
                </div>
                <LifecycleSidebar gate=gate/>
            </div>
            <ChatModal
                open=chat_open
                set_open=set_chat_open
                messages=chat_messages
                set_messages=set_chat_messages
                sending=chat_sending
                set_sending=set_chat_sending
            />
        </main>
    }
}

fn unauthenticated_shell() -> impl IntoView {
    let gate = GateUiState::disconnected(
        "authentication required",
        "log in to connect to the Prior gate".into(),
    );

    view! {
        <main class="app-shell">
            <Topbar
                user_label="Not signed in".to_string()
                gate=gate.clone()
                loading=false
                on_refresh=move |_| {}
                login_url=Some("/auth/login?return_to=/app".to_string())
                on_open_chat=move |_| {}
            />
            <div class="main-layout">
                <Sidebar
                    rooms=vec![]
                    gate=gate.clone()
                    selected_room=None
                    active_filter=SidebarFilter::All
                    starred_rooms=BTreeSet::new()
                    on_set_filter=move |_| {}
                    on_select_room=move |_| {}
                />
                <div class="center">
                    <RoomListPane
                        rooms=vec![]
                        loading=false
                        selected_room=None
                        active_filter=SidebarFilter::All
                        starred_rooms=BTreeSet::new()
                        on_toggle_star=move |_| {}
                        on_select_room=move |_| {}
                    />
                    <div class="reading-pane">
                        <div class="empty-state">
                            <div class="empty-state-icon">"🔒"</div>
                            <p class="empty-state-title">"Sign in to get started"</p>
                            <p class="empty-state-body">"Prior requires authentication to connect to the gate and discover rooms."</p>
                            <br/>
                            <a class="btn-primary" href="/auth/login?return_to=/app">"Log in"</a>
                        </div>
                    </div>
                </div>
                <LifecycleSidebar gate=gate/>
            </div>
        </main>
    }
}

fn error_shell(error: String) -> impl IntoView {
    let gate = GateUiState::disconnected("session lookup failed", error);

    view! {
        <main class="app-shell">
            <Topbar
                user_label="Session error".to_string()
                gate=gate.clone()
                loading=false
                on_refresh=move |_| {}
                login_url=Some("/auth/login?return_to=/app".to_string())
                on_open_chat=move |_| {}
            />
            <div class="main-layout">
                <Sidebar
                    rooms=vec![]
                    gate=gate.clone()
                    selected_room=None
                    active_filter=SidebarFilter::All
                    starred_rooms=BTreeSet::new()
                    on_set_filter=move |_| {}
                    on_select_room=move |_| {}
                />
                <div class="center">
                    <RoomListPane
                        rooms=vec![]
                        loading=false
                        selected_room=None
                        active_filter=SidebarFilter::All
                        starred_rooms=BTreeSet::new()
                        on_toggle_star=move |_| {}
                        on_select_room=move |_| {}
                    />
                    <div class="reading-pane">
                        <div class="empty-state">
                            <div class="empty-state-icon">"⚠"</div>
                            <p class="empty-state-title">"Session error"</p>
                            <p class="empty-state-body">{gate.status.clone()}</p>
                            <br/>
                            <a class="btn-primary" href="/auth/login?return_to=/app">"Try again"</a>
                        </div>
                    </div>
                </div>
                <LifecycleSidebar gate=gate/>
            </div>
        </main>
    }
}

// ── Topbar ──────────────────────────────────────────────

#[component]
fn Topbar<F, G>(
    user_label: String,
    gate: GateUiState,
    loading: bool,
    on_refresh: F,
    login_url: Option<String>,
    on_open_chat: G,
) -> impl IntoView
where
    F: FnMut(leptos::ev::MouseEvent) + Copy + 'static,
    G: Fn(leptos::ev::MouseEvent) + Copy + 'static,
{
    let status_class = match gate.connection {
        ConnectionStatus::Connecting => "status-pill status-pill--loading",
        ConnectionStatus::Connected => "status-pill status-pill--ok",
        ConnectionStatus::Disconnected => "status-pill status-pill--down",
    };

    let auth_action = if let Some(login_href) = login_url {
        view! { <a class="btn-primary" href=login_href>"Log in"</a> }.into_any()
    } else {
        view! { <a class="btn-secondary" href="/auth/logout">"Log out"</a> }.into_any()
    };

    view! {
        <header class="topbar">
            <div class="topbar-left">
                <div class="brand">
                    <div class="brand-mark">"P"</div>
                    "Prior"
                </div>
            </div>

            <button class="prompt-bar" on:click=on_open_chat type="button">
                <span class="prompt-bar-icon">"+"</span>
                <span class="prompt-bar-placeholder">"Describe a task, paste a brief, or ask a question\u{2026}"</span>
            </button>

            <div class="topbar-right">
                <span class=status_class></span>
                <button
                    class="icon-btn"
                    on:click=on_refresh
                    disabled=loading
                    title="Refresh"
                >
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

// ── Left Sidebar (Repos = Folders) ──────────────────────

#[component]
fn Sidebar<F, G>(
    rooms: Vec<String>,
    gate: GateUiState,
    selected_room: Option<String>,
    active_filter: SidebarFilter,
    starred_rooms: BTreeSet<String>,
    on_set_filter: G,
    on_select_room: F,
) -> impl IntoView
where
    F: Fn(String) + Copy + Send + 'static,
    G: Fn(SidebarFilter) + Copy + 'static,
{
    // Filter out #general and other general-purpose rooms from the sidebar.
    // These are accessed via the prompt bar / chat modal instead.
    let filtered_rooms: Vec<String> = rooms
        .into_iter()
        .filter(|room| !room.starts_with("#"))
        .collect();
    let room_count = filtered_rooms.len();
    let starred_count = starred_rooms.len();
    let connection_label = match gate.connection {
        ConnectionStatus::Connecting => "Connecting\u{2026}",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Disconnected => "Disconnected",
    };
    let connection_class = match gate.connection {
        ConnectionStatus::Connecting => "status-pill status-pill--loading",
        ConnectionStatus::Connected => "status-pill status-pill--ok",
        ConnectionStatus::Disconnected => "status-pill status-pill--down",
    };
    let all_class = if active_filter == SidebarFilter::All {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };
    let attention_class = if active_filter == SidebarFilter::NeedsAttention {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };
    let starred_class = if active_filter == SidebarFilter::Starred {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };
    let completed_class = if active_filter == SidebarFilter::Completed {
        "sidebar-item active"
    } else {
        "sidebar-item"
    };

    view! {
        <nav class="sidebar">
            <div class="sidebar-section">
                <button class=all_class type="button" on:click=move |_| on_set_filter(SidebarFilter::All)>
                    <div class="sidebar-item-left">
                        <span class="sidebar-item-icon">"◉"</span>
                        <span class="sidebar-item-label">"All Rooms"</span>
                    </div>
                    <span class="sidebar-count">{room_count}</span>
                </button>
                <button class=attention_class type="button" on:click=move |_| on_set_filter(SidebarFilter::NeedsAttention)>
                    <div class="sidebar-item-left">
                        <span class="sidebar-item-icon">"⚑"</span>
                        <span class="sidebar-item-label">"Needs Attention"</span>
                    </div>
                </button>
                <button class=starred_class type="button" on:click=move |_| on_set_filter(SidebarFilter::Starred)>
                    <div class="sidebar-item-left">
                        <span class="sidebar-item-icon">"★"</span>
                        <span class="sidebar-item-label">"Starred"</span>
                    </div>
                    <span class="sidebar-count">{starred_count}</span>
                </button>
                <button class=completed_class type="button" on:click=move |_| on_set_filter(SidebarFilter::Completed)>
                    <div class="sidebar-item-left">
                        <span class="sidebar-item-icon">"✓"</span>
                        <span class="sidebar-item-label">"Completed"</span>
                    </div>
                </button>
            </div>

            <div class="sidebar-divider"></div>

            <div class="sidebar-section">
                <div class="sidebar-section-label">"Repos"</div>
                {if filtered_rooms.is_empty() {
                    view! {
                        <div class="sidebar-empty-copy">
                            "No repo rooms available."
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <For
                            each=move || filtered_rooms.clone()
                            key=|room| room.clone()
                            children=move |room| {
                                let is_selected = selected_room.as_ref() == Some(&room);
                                let item_class = if is_selected {
                                    "sidebar-item active"
                                } else {
                                    "sidebar-item"
                                };
                                let room_name = room.clone();

                                view! {
                                    <button
                                        class=item_class
                                        type="button"
                                        on:click=move |_| on_select_room(room_name.clone())
                                    >
                                        <div class="sidebar-item-left">
                                            <span class="sidebar-item-icon" style="font-size: 14px;">"▸"</span>
                                            <span class="sidebar-item-label">{room}</span>
                                        </div>
                                    </button>
                                }
                            }
                        />
                    }.into_any()
                }}
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

// ── Room List Pane (Message List) ───────────────────────

#[component]
fn RoomListPane<F, G>(
    rooms: Vec<String>,
    loading: bool,
    selected_room: Option<String>,
    active_filter: SidebarFilter,
    starred_rooms: BTreeSet<String>,
    on_toggle_star: G,
    on_select_room: F,
) -> impl IntoView
where
    F: Fn(String) + Copy + Send + 'static,
    G: Fn(String) + Copy + Send + 'static,
{
    let room_count = rooms.len();
    let display_rooms = rooms.clone();
    let empty_title = match active_filter {
        SidebarFilter::All => "No rooms discovered",
        SidebarFilter::Starred => "No starred rooms",
        SidebarFilter::NeedsAttention => "Needs attention is not available yet",
        SidebarFilter::Completed => "Completed rooms are not available yet",
    };
    let empty_body = match active_filter {
        SidebarFilter::All => {
            "Rooms will appear here when the gate connects and discovers active sessions."
        }
        SidebarFilter::Starred => "Star rooms from the list to pin them here.",
        SidebarFilter::NeedsAttention => {
            "This filter needs room/run status metadata from Prior before it can show anything useful."
        }
        SidebarFilter::Completed => {
            "This filter needs completion metadata from Prior before it can distinguish finished rooms."
        }
    };

    view! {
        <div class="room-list-pane">
            <div class="room-list-toolbar">
                <div class="room-list-toolbar-left">
                    <span class="toolbar-copy">"Rooms"</span>
                </div>
                <span class="room-list-info">
                    {if loading {
                        "Loading…".to_string()
                    } else {
                        format!("{} room(s)", room_count)
                    }}
                </span>
            </div>

            <div class="room-list">
                {if display_rooms.is_empty() && !loading {
                    view! {
                        <div class="empty-state">
                            <p class="empty-state-title">{empty_title}</p>
                            <p class="empty-state-body">{empty_body}</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <For
                            each=move || display_rooms.clone().into_iter().enumerate()
                            key=|(_, room)| room.clone()
                            children=move |(index, room)| {
                                let is_selected = selected_room
                                    .as_ref()
                                    .map_or(index == 0, |selected| selected == &room);
                                let is_starred = starred_rooms.contains(&room);
                                let row_class = if is_selected {
                                    "room-row selected"
                                } else {
                                    "room-row"
                                };
                                let room_name = room.clone();
                                let star_room = room.clone();
                                let star_class = if is_starred {
                                    "room-star starred"
                                } else {
                                    "room-star"
                                };

                                view! {
                                    <div
                                        class=row_class
                                        on:click=move |_| on_select_room(room_name.clone())
                                    >
                                        <div class="room-row-check">
                                            <button
                                                class=star_class
                                                type="button"
                                                on:click=move |event| {
                                                    event.stop_propagation();
                                                    on_toggle_star(star_room.clone());
                                                }
                                                title={if is_starred { "Remove star" } else { "Star room" }}
                                            >
                                                {if is_starred { "★" } else { "☆" }}
                                            </button>
                                        </div>
                                        <div class="room-content">
                                            <span class="room-sender">{room.clone()}</span>
                                            <span class="room-snippet">"Active room"</span>
                                        </div>
                                        <div class="room-meta">
                                            <span class="room-date">"now"</span>
                                            <span class="room-tag info">"Live"</span>
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

// ── Reading Pane (Thread View) ──────────────────────────

#[component]
fn ReadingPane() -> impl IntoView {
    view! {
        <div class="reading-pane">
            <div class="empty-state">
                <div class="empty-state-icon">"◉"</div>
                <p class="empty-state-title">"Select a room"</p>
                <p class="empty-state-body">"Choose a room from the list to see its conversation thread and factory run updates."</p>
            </div>
        </div>
    }
}

// ── Lifecycle Sidebar (Right) ───────────────────────────

#[component]
fn LifecycleSidebar(gate: GateUiState) -> impl IntoView {
    let server = gate.server_name.clone().unwrap_or_else(|| "unknown".into());

    view! {
        <aside class="right-sidebar">
            <div class="right-sidebar-header">
                <span class="right-sidebar-title">"Lifecycle"</span>
            </div>

            <div>
                <label style="font-size: 12px; color: var(--text-muted); display: block; margin-bottom: 4px;">"Active Run"</label>
                <select class="phase-selector">
                    <option>"No active runs"</option>
                </select>
            </div>

            <div class="lifecycle-progress">
                <div class="lifecycle-progress-seg"></div>
                <div class="lifecycle-progress-seg"></div>
                <div class="lifecycle-progress-seg"></div>
                <div class="lifecycle-progress-seg"></div>
                <div class="lifecycle-progress-seg"></div>
                <div class="lifecycle-progress-seg"></div>
                <div class="lifecycle-progress-seg"></div>
            </div>

            <div class="lifecycle-phases">
                <div class="lifecycle-phase future">
                    <div class="phase-icon pending">"○"</div>
                    <div class="phase-info">
                        <div class="phase-name">"Interpret"</div>
                        <div class="phase-detail">"Normalize problem brief"</div>
                    </div>
                </div>
                <div class="lifecycle-phase future">
                    <div class="phase-icon pending">"○"</div>
                    <div class="phase-info">
                        <div class="phase-name">"Resolve"</div>
                        <div class="phase-detail">"Bind to repo context"</div>
                    </div>
                </div>
                <div class="lifecycle-phase future">
                    <div class="phase-icon pending">"○"</div>
                    <div class="phase-info">
                        <div class="phase-name">"Plan"</div>
                        <div class="phase-detail">"Generate stage graph"</div>
                    </div>
                </div>
                <div class="lifecycle-phase future">
                    <div class="phase-icon pending">"○"</div>
                    <div class="phase-info">
                        <div class="phase-name">"Execute"</div>
                        <div class="phase-detail">"Run issues"</div>
                    </div>
                </div>
                <div class="lifecycle-phase future">
                    <div class="phase-icon pending">"○"</div>
                    <div class="phase-info">
                        <div class="phase-name">"Verify"</div>
                        <div class="phase-detail">"Run tests"</div>
                    </div>
                </div>
                <div class="lifecycle-phase future">
                    <div class="phase-icon pending">"○"</div>
                    <div class="phase-info">
                        <div class="phase-name">"Gate"</div>
                        <div class="phase-detail">"Merge readiness"</div>
                    </div>
                </div>
                <div class="lifecycle-phase future">
                    <div class="phase-icon pending">"○"</div>
                    <div class="phase-info">
                        <div class="phase-name">"Complete"</div>
                        <div class="phase-detail">"PR + review"</div>
                    </div>
                </div>
            </div>

            <div class="right-sidebar-divider"></div>

            // Transport info
            <div class="run-card">
                <div class="run-card-header">
                    <span class="run-card-id">"Transport"</span>
                    <span class={match gate.connection {
                        ConnectionStatus::Connecting => "run-card-status warn",
                        ConnectionStatus::Connected => "run-card-status ok",
                        ConnectionStatus::Disconnected => "run-card-status err",
                    }}>
                        {match gate.connection {
                            ConnectionStatus::Connecting => "Connecting",
                            ConnectionStatus::Connected => "Connected",
                            ConnectionStatus::Disconnected => "Disconnected",
                        }}
                    </span>
                </div>
                <div class="run-card-detail">
                    {format!("Server: {}", server)}
                    <br/>
                    {format!("Gate: {}", gate.gate_url)}
                    <br/>
                    {format!("Rooms: {}", gate.rooms.len())}
                </div>
            </div>

            <div class="right-sidebar-divider"></div>

            <div class="sidebar-legend">
                <div class="legend-item"><div class="legend-dot" style="background: var(--ok);"></div>" Passed"</div>
                <div class="legend-item"><div class="legend-dot" style="background: var(--accent);"></div>" Active"</div>
                <div class="legend-item"><div class="legend-dot" style="background: var(--warn);"></div>" Blocked"</div>
                <div class="legend-item"><div class="legend-dot" style="background: var(--err);"></div>" Failed"</div>
            </div>
        </aside>
    }
}

// ── Chat Modal (#general room) ──────────────────────────

#[component]
fn ChatModal(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    messages: ReadSignal<Vec<ChatMessage>>,
    set_messages: WriteSignal<Vec<ChatMessage>>,
    sending: ReadSignal<bool>,
    set_sending: WriteSignal<bool>,
) -> impl IntoView {
    let (draft, set_draft) = signal(String::new());

    let on_send = move || {
        let content = draft.get().trim().to_string();
        if content.is_empty() || sending.get() {
            return;
        }

        set_draft.set(String::new());
        set_chat_messages_with_human(&set_messages, &content);
        set_sending.set(true);

        let content_clone = content.clone();
        leptos::task::spawn_local(async move {
            let result = send_room_message(
                "#general".to_string(),
                content_clone,
            )
            .await;

            match result {
                Ok(entries) => {
                    set_messages.update(|msgs| {
                        for entry in entries {
                            if entry.content.trim().is_empty() {
                                continue;
                            }
                            msgs.push(ChatMessage {
                                from: entry.actor.unwrap_or_else(|| "Prior".into()),
                                content: entry.content,
                                is_human: false,
                            });
                        }
                    });
                }
                Err(error) => {
                    set_messages.update(|msgs| {
                        msgs.push(ChatMessage {
                            from: "System".into(),
                            content: format!("Error: {error}"),
                            is_human: false,
                        });
                    });
                }
            }
            set_sending.set(false);
        });
    };

    let on_send_click = {
        let on_send = on_send.clone();
        move |_: leptos::ev::MouseEvent| on_send()
    };

    let on_keydown = {
        let on_send = on_send.clone();
        move |event: leptos::ev::KeyboardEvent| {
            if event.key() == "Enter" && !event.shift_key() {
                event.prevent_default();
                on_send();
            }
        }
    };

    let on_backdrop_click = move |_: leptos::ev::MouseEvent| {
        set_open.set(false);
    };

    view! {
        <Show when=move || open.get()>
            <div class="chat-overlay" on:click=on_backdrop_click>
                <div class="chat-modal" on:click=|event: leptos::ev::MouseEvent| event.stop_propagation()>
                    <div class="chat-modal-header">
                        <span class="chat-modal-title">"Prior"</span>
                        <span class="chat-modal-room">"#general"</span>
                        <button
                            class="chat-modal-close"
                            type="button"
                            on:click=move |_| set_open.set(false)
                            title="Close"
                        >
                            "\u{00d7}"
                        </button>
                    </div>

                    <div class="chat-modal-messages">
                        {move || {
                            let msgs = messages.get();
                            if msgs.is_empty() {
                                view! {
                                    <div class="chat-modal-empty">
                                        <p class="chat-modal-empty-title">"What would you like to work on?"</p>
                                        <p class="chat-modal-empty-body">"Describe a task, paste a project brief, reference an issue \u{2014} Prior will interpret and can kick off a factory run."</p>
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
                                            } else {
                                                "chat-bubble chat-bubble--assistant"
                                            };
                                            let avatar_class = if msg.is_human {
                                                "chat-bubble-avatar human"
                                            } else {
                                                "chat-bubble-avatar system"
                                            };
                                            let avatar_letter = msg.from.chars().next()
                                                .unwrap_or('?')
                                                .to_uppercase()
                                                .to_string();

                                            view! {
                                                <div class=bubble_class>
                                                    <div class=avatar_class>{avatar_letter}</div>
                                                    <div class="chat-bubble-body">
                                                        <div class="chat-bubble-sender">{msg.from.clone()}</div>
                                                        <div class="chat-bubble-content">{msg.content.clone()}</div>
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
                                <div class="chat-bubble-avatar system">"P"</div>
                                <div class="chat-bubble-body">
                                    <div class="chat-bubble-sender">"Prior"</div>
                                    <div class="chat-bubble-content chat-bubble-typing">"Thinking\u{2026}"</div>
                                </div>
                            </div>
                        })}
                    </div>

                    <div class="chat-modal-input-area">
                        <textarea
                            class="chat-modal-input"
                            prop:value=move || draft.get()
                            on:input=move |event| set_draft.set(event_target_value(&event))
                            on:keydown=on_keydown
                            placeholder="Describe a task, paste a brief, or ask a question\u{2026}"
                            rows="3"
                            disabled=move || sending.get()
                        ></textarea>
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
        </Show>
    }
}

fn set_chat_messages_with_human(set_messages: &WriteSignal<Vec<ChatMessage>>, content: &str) {
    let content = content.to_string();
    set_messages.update(move |msgs| {
        msgs.push(ChatMessage {
            from: "You".into(),
            content,
            is_human: true,
        });
    });
}
