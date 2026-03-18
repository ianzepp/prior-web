use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;

use crate::net::prior_gate::refresh_dashboard;
use crate::state::gate::{ConnectionStatus, GateUiState};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let (refresh_tick, set_refresh_tick) = signal(0_u64);
    let gate = Resource::new(move || refresh_tick.get(), |_| refresh_dashboard());

    let on_refresh = move |_| set_refresh_tick.update(|count| *count += 1);

    view! {
        <main class="dashboard-shell">
            <section class="hero">
                <p class="eyebrow">"Prior Web"</p>
                <h1>"Remote operator surface over gate"</h1>
                <p class="lede">
                    "This surface now treats Prior as a server-side integration boundary. The browser talks to prior-web, and prior-web performs the gate round trip."
                </p>
                <div class="actions">
                    <button class="primary" on:click=on_refresh>"Refresh"</button>
                </div>
            </section>

            <Suspense fallback=move || {
                view! {
                    <section class="panel-grid">
                        <ConnectionPanel gate=GateUiState::loading("loading server-side gate snapshot")/>
                        <RoomsPanel gate=GateUiState::loading("loading server-side gate snapshot")/>
                        <EventsPanel gate=GateUiState::loading("loading server-side gate snapshot")/>
                    </section>
                }
                .into_any()
            }>
                {move || {
                    let gate = gate.get().map_or_else(
                        || GateUiState::loading("loading server-side gate snapshot"),
                        |result| match result {
                            Ok(state) => state,
                            Err(error) => {
                                GateUiState::disconnected("server function failed", error.to_string())
                            }
                        },
                    );

                    view! {
                        <section class="panel-grid">
                            <ConnectionPanel gate=gate.clone()/>
                            <RoomsPanel gate=gate.clone()/>
                            <EventsPanel gate=gate/>
                        </section>
                    }
                    .into_any()
                }}
            </Suspense>
        </main>
    }
}

#[component]
fn ConnectionPanel(gate: GateUiState) -> impl IntoView {
    let connection_label = match gate.connection {
        ConnectionStatus::Connecting => "connecting",
        ConnectionStatus::Connected => "connected",
        ConnectionStatus::Disconnected => "disconnected",
    };

    view! {
        <section class="panel">
            <h2>"Connection"</h2>
            <p><strong>"Status: "</strong>{connection_label}</p>
            <p><strong>"Gate URL: "</strong>{gate.gate_url}</p>
            <p><strong>"Server: "</strong>{gate.server_name.unwrap_or_else(|| "unknown".into())}</p>
            <p><strong>"Status note: "</strong>{gate.status}</p>
        </section>
    }
}

#[component]
fn RoomsPanel(gate: GateUiState) -> impl IntoView {
    let rooms = gate.rooms;
    let listed_rooms = rooms.clone();

    view! {
        <section class="panel">
            <h2>"Rooms"</h2>
            <ul class="room-list">
                <For
                    each=move || listed_rooms.clone()
                    key=|room| room.clone()
                    children=move |room| view! { <li>{room}</li> }
                />
            </ul>
            {if rooms.is_empty() {
                view! { <p class="muted">"No rooms loaded yet."</p> }.into_any()
            } else {
                ().into_any()
            }}
        </section>
    }
}

#[component]
fn EventsPanel(gate: GateUiState) -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Last Event"</h2>
            <p class="event-copy">
                {gate.last_event.unwrap_or_else(|| "No server-side gate events observed during the latest refresh.".into())}
            </p>
        </section>
    }
}
