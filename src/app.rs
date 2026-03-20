#[cfg(feature = "ssr")]
use axum::http::request::Parts;
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos::web_sys;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{StaticSegment, path};

use crate::pages::auth_denied::AuthDeniedPage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::home::HomePage;
use crate::state::auth::AuthState;

#[cfg(feature = "hydrate")]
const THEME_STORAGE_KEY: &str = "prior-web-theme";

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
    let theme = RwSignal::new(initial_theme_dark());
    provide_context(theme);

    Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            apply_theme(theme.get());
        }
    });

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

fn initial_theme_dark() -> bool {
    #[cfg(feature = "hydrate")]
    {
        return browser_theme_preference();
    }

    #[allow(unreachable_code)]
    false
}

#[cfg(feature = "hydrate")]
fn browser_theme_preference() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    if let Ok(Some(storage)) = window.local_storage() {
        if let Ok(Some(value)) = storage.get_item(THEME_STORAGE_KEY) {
            return value == "dark";
        }
    }

    window
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches())
}

#[cfg(feature = "hydrate")]
fn apply_theme(dark: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    if let Some(body) = document.body() {
        if dark {
            let _ = body.set_attribute("data-theme", "dark");
        } else {
            let _ = body.remove_attribute("data-theme");
        }
    }

    if let Ok(Some(storage)) = window.local_storage() {
        let _ = storage.set_item(THEME_STORAGE_KEY, if dark { "dark" } else { "light" });
    }
}
