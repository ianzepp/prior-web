use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{StaticSegment, path};

use crate::pages::dashboard::DashboardPage;

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
