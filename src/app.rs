use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{StaticSegment, path};

use crate::pages::dashboard::DashboardPage;
use crate::state::gate::GateUiState;

#[derive(Clone, Debug, Default)]
pub struct GateSender {
    #[cfg(feature = "hydrate")]
    pub tx: Option<futures::channel::mpsc::UnboundedSender<crate::net::gate_client::GateCommand>>,
}

impl GateSender {
    pub fn refresh(&self) -> bool {
        #[cfg(feature = "hydrate")]
        {
            if let Some(tx) = &self.tx {
                return tx
                    .unbounded_send(crate::net::gate_client::GateCommand::RefreshRooms)
                    .is_ok();
            }
        }
        false
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    let gate_ws_url = crate::runtime::gate_ws_meta_content();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="prior-gate-ws-url" content=gate_ws_url/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let gate = RwSignal::new(GateUiState::default());
    let sender = RwSignal::new(GateSender::default());
    provide_context(gate);
    provide_context(sender);

    Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            let tx = crate::net::gate_client::spawn_gate_client(gate);
            sender.update(|state| state.tx = Some(tx));
        }
    });

    view! {
        <Stylesheet id="prior-web" href="/site.css"/>
        <Title text="Prior Web"/>
        <Router>
            <Routes fallback=|| view! { <p>"Not found"</p> }.into_view()>
                <Route path=StaticSegment("") view=DashboardPage/>
                <Route path=path!("app") view=DashboardPage/>
            </Routes>
        </Router>
    }
}
