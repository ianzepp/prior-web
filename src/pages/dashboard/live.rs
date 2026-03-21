use std::rc::Rc;

use leptos::prelude::*;

use crate::pages::dashboard::state::ChatMessage;

#[derive(Clone)]
pub(crate) struct LiveSocket {
    join_room: Rc<dyn Fn(String)>,
    send_room_message: Rc<dyn Fn(String, String) -> bool>,
}

impl LiveSocket {
    pub(crate) fn disconnected() -> Self {
        Self {
            join_room: Rc::new(|_| {}),
            send_room_message: Rc::new(|_, _| false),
        }
    }

    pub(crate) fn join_room(&self, room: String) {
        (self.join_room)(room);
    }

    pub(crate) fn send_room_message(&self, room: String, content: String) -> bool {
        (self.send_room_message)(room, content)
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn connect_live_socket(
    set_messages: WriteSignal<Vec<ChatMessage>>,
    set_sending: WriteSignal<bool>,
    on_factory_event: impl Fn() + Clone + 'static,
) -> LiveSocket {
    hydrate_live_socket(set_messages, set_sending, on_factory_event)
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn connect_live_socket(
    _set_messages: WriteSignal<Vec<ChatMessage>>,
    _set_sending: WriteSignal<bool>,
    _on_factory_event: impl Fn() + Clone + 'static,
) -> LiveSocket {
    LiveSocket::disconnected()
}

#[cfg(feature = "hydrate")]
fn hydrate_live_socket(
    set_messages: WriteSignal<Vec<ChatMessage>>,
    set_sending: WriteSignal<bool>,
    on_factory_event: impl Fn() + Clone + 'static,
) -> LiveSocket {
    use crate::net::live::LiveClientMessage;
    use web_sys::WebSocket;

    let ws = match build_socket() {
        Ok(socket) => socket,
        Err(error) => {
            push_error(&set_messages, &set_sending, &error);
            return LiveSocket::disconnected();
        }
    };

    let socket = Rc::new(ws);
    let socket_ref = Rc::new(std::cell::RefCell::new(Some(Rc::clone(&socket))));
    let on_factory_event: Rc<dyn Fn()> = Rc::new(on_factory_event);

    attach_open_handler(&socket);
    attach_message_handler(
        &socket,
        set_messages,
        set_sending,
        Rc::clone(&on_factory_event),
    );
    attach_error_handler(&socket, set_messages, set_sending);
    attach_close_handler(&socket, set_sending, Rc::clone(&socket_ref));

    LiveSocket {
        join_room: {
            let socket_ref = Rc::clone(&socket_ref);
            Rc::new(move |room: String| {
                let socket_binding = socket_ref.borrow();
                let Some(socket) = socket_binding.as_ref() else {
                    return;
                };
                let Ok(payload) = serde_json::to_string(&LiveClientMessage::JoinRoom { room })
                else {
                    return;
                };
                let _ = socket.send_with_str(&payload);
            })
        },
        send_room_message: {
            let socket_ref = Rc::clone(&socket_ref);
            Rc::new(move |room: String, content: String| {
                let socket_binding = socket_ref.borrow();
                let Some(socket) = socket_binding.as_ref() else {
                    return false;
                };
                if socket.ready_state() != WebSocket::OPEN {
                    return false;
                }
                let Ok(payload) =
                    serde_json::to_string(&LiveClientMessage::SendRoomMessage { room, content })
                else {
                    return false;
                };
                socket.send_with_str(&payload).is_ok()
            })
        },
    }
}

#[cfg(feature = "hydrate")]
fn attach_open_handler(socket: &web_sys::WebSocket) {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::Event;

    let on_open = Closure::<dyn FnMut(Event)>::new(move |_| {});
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();
}

#[cfg(feature = "hydrate")]
fn attach_message_handler(
    socket: &web_sys::WebSocket,
    set_messages: WriteSignal<Vec<ChatMessage>>,
    set_sending: WriteSignal<bool>,
    on_factory_event: Rc<dyn Fn()>,
) {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::MessageEvent;

    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        handle_live_message(
            event,
            set_messages,
            set_sending,
            Rc::clone(&on_factory_event),
        );
    });
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
}

#[cfg(feature = "hydrate")]
fn handle_live_message(
    event: web_sys::MessageEvent,
    set_messages: WriteSignal<Vec<ChatMessage>>,
    set_sending: WriteSignal<bool>,
    on_factory_event: Rc<dyn Fn()>,
) {
    use crate::net::live::LiveServerMessage;

    let Some(text) = event.data().as_string() else {
        return;
    };
    let Ok(message) = serde_json::from_str::<LiveServerMessage>(&text) else {
        return;
    };

    match message {
        LiveServerMessage::Connected { .. } => {}
        LiveServerMessage::RoomEvent {
            topic,
            actor,
            content,
            ..
        } => push_room_event(set_messages, topic, actor, content),
        LiveServerMessage::RoomTurnDone { .. } => set_sending.set(false),
        LiveServerMessage::FactoryEvent { .. } => on_factory_event(),
        LiveServerMessage::Error { message } => {
            push_error(&set_messages, &set_sending, &message);
        }
    }
}

#[cfg(feature = "hydrate")]
fn push_room_event(
    set_messages: WriteSignal<Vec<ChatMessage>>,
    topic: String,
    actor: Option<String>,
    content: String,
) {
    if content.trim().is_empty() {
        return;
    }

    set_messages.update(|messages| {
        messages.push(ChatMessage {
            topic,
            actor,
            content,
            is_human: false,
        });
    });
}

#[cfg(feature = "hydrate")]
fn attach_error_handler(
    socket: &web_sys::WebSocket,
    set_messages: WriteSignal<Vec<ChatMessage>>,
    set_sending: WriteSignal<bool>,
) {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::ErrorEvent;

    let on_error = Closure::<dyn FnMut(ErrorEvent)>::new(move |_| {
        push_error(
            &set_messages,
            &set_sending,
            "Live updates disconnected unexpectedly.",
        );
    });
    socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();
}

#[cfg(feature = "hydrate")]
fn attach_close_handler(
    socket: &web_sys::WebSocket,
    set_sending: WriteSignal<bool>,
    socket_ref: Rc<std::cell::RefCell<Option<Rc<web_sys::WebSocket>>>>,
) {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::CloseEvent;

    let on_close = Closure::<dyn FnMut(CloseEvent)>::new(move |_| {
        set_sending.set(false);
        socket_ref.borrow_mut().take();
    });
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();
}

#[cfg(feature = "hydrate")]
fn build_socket() -> Result<web_sys::WebSocket, String> {
    let Some(window) = web_sys::window() else {
        return Err("browser window is unavailable".into());
    };
    let location = window.location();
    let protocol = location
        .protocol()
        .map_err(|error| format!("browser location protocol failed: {error:?}"))?;
    let host = location
        .host()
        .map_err(|error| format!("browser location host failed: {error:?}"))?;
    let scheme = if protocol == "https:" { "wss" } else { "ws" };
    let url = format!("{scheme}://{host}/ws/live");
    web_sys::WebSocket::new(&url).map_err(|error| format!("live websocket failed: {error:?}"))
}

#[cfg(feature = "hydrate")]
fn push_error(
    set_messages: &WriteSignal<Vec<ChatMessage>>,
    set_sending: &WriteSignal<bool>,
    message: &str,
) {
    set_sending.set(false);
    let content = format!("Error: {message}");
    set_messages.update(move |messages| {
        messages.push(ChatMessage {
            topic: "error".to_string(),
            actor: None,
            content: content.clone(),
            is_human: false,
        });
    });
}
