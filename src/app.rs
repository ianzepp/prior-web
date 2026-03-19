#[cfg(feature = "ssr")]
use axum::http::request::Parts;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{StaticSegment, path};

use crate::pages::auth_denied::AuthDeniedPage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::home::HomePage;
use crate::state::auth::AuthState;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
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
    provide_context(initial_auth_state());

    view! {
        <Stylesheet id="prior-web" href="/site.css"/>
        <Title text="Prior Web"/>
        <Router>
            <Routes fallback=|| view! { <p>"Not found"</p> }.into_view()>
                <Route path=StaticSegment("") view=HomePage/>
                <Route path=path!("app") view=DashboardPage/>
                <Route path=path!("auth/denied") view=AuthDeniedPage/>
            </Routes>
        </Router>
    }
}

fn initial_auth_state() -> AuthState {
    #[cfg(feature = "ssr")]
    {
        let parts = use_context::<Parts>();
        let state = use_context::<crate::runtime::AppState>();

        if let (Some(parts), Some(state)) = (parts, state) {
            return crate::auth::user::auth_state_from_parts(&parts, &state);
        }
    }

    AuthState::Anonymous
}
