use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;

use crate::app::GateSender;
use crate::state::gate::{ConnectionStatus, GateUiState};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let gate = expect_context::<RwSignal<GateUiState>>();
    let sender = expect_context::<RwSignal<GateSender>>();

    let on_refresh = move |_| {
        if !sender.get_untracked().refresh() {
            gate.update(|state| state.status = "gate sender unavailable".into());
        }
    };

    view! {
        <main class="dashboard-shell">
            <section class="hero">
                <p class="eyebrow">"Prior Web"</p>
                <h1>"Remote operator surface over gate"</h1>
                <p class="lede">
                    "This skeleton mirrors the gauntlet split: Leptos app root, shared gate state, and a dedicated realtime client boundary."
                </p>
                <div class="actions">
                    <button class="primary" on:click=on_refresh>"Refresh"</button>
                </div>
            </section>

            <section class="panel-grid">
                <ConnectionPanel gate=gate/>
                <RoomsPanel gate=gate/>
                <EventsPanel gate=gate/>
            </section>
        </main>
    }
}

#[component]
fn ConnectionPanel(gate: RwSignal<GateUiState>) -> impl IntoView {
    let connection_label = move || match gate.get().connection {
        ConnectionStatus::Connecting => "connecting",
        ConnectionStatus::Connected => "connected",
        ConnectionStatus::Disconnected => "disconnected",
    };

    view! {
        <section class="panel">
            <h2>"Connection"</h2>
            <p><strong>"Status: "</strong>{connection_label}</p>
            <p><strong>"Gate URL: "</strong>{move || gate.get().gate_url}</p>
            <p><strong>"Server: "</strong>{move || gate.get().server_name.clone().unwrap_or_else(|| "unknown".into())}</p>
            <p><strong>"Status note: "</strong>{move || gate.get().status}</p>
        </section>
    }
}

#[component]
fn RoomsPanel(gate: RwSignal<GateUiState>) -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Rooms"</h2>
            <ul class="room-list">
                <For
                    each=move || gate.get().rooms
                    key=|room| room.clone()
                    children=move |room| view! { <li>{room}</li> }
                />
            </ul>
            {move || {
                if gate.get().rooms.is_empty() {
                    view! { <p class="muted">"No rooms loaded yet."</p> }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </section>
    }
}

#[component]
fn EventsPanel(gate: RwSignal<GateUiState>) -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Last Event"</h2>
            <p class="event-copy">
                {move || gate.get().last_event.unwrap_or_else(|| "No websocket frames observed yet.".into())}
            </p>
        </section>
    }
}
