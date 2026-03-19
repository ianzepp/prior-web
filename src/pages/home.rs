use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="dashboard-shell">
            <section class="hero">
                <p class="eyebrow">"Prior Web"</p>
                <h1>"Authenticated operator surface over gate"</h1>
                <p class="lede">
                    "prior-web is the public multi-user trust boundary. Sign in through Auth0 to open a user-owned web session before any server-side gate work is allowed."
                </p>
                <div class="actions">
                    <a class="primary" href="/auth/login?return_to=/app">"Log in"</a>
                </div>
            </section>
        </main>
    }
}
