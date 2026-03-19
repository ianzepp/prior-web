use leptos::prelude::*;
use leptos_router::hooks::query_signal;

#[component]
pub fn AuthDeniedPage() -> impl IntoView {
    let (reason, _) = query_signal::<String>("reason");

    view! {
        <main class="dashboard-shell">
            <section class="hero">
                <p class="eyebrow">"Prior Web"</p>
                <h1>"Authorization not completed"</h1>
                <p class="lede">
                    {move || match reason.get().as_deref() {
                        Some("access_denied") => {
                            "Login was canceled before authorization completed.".to_string()
                        }
                        Some("login_failed") => {
                            "prior-web could not complete login and establish a session.".to_string()
                        }
                        Some(_) | None => {
                            "Authentication did not complete. Try again, and inspect server logs if the failure repeats.".to_string()
                        }
                    }}
                </p>
                <div class="actions">
                    <a class="primary" href="/auth/login?return_to=/app">"Try again"</a>
                    <a href="/">"Back home"</a>
                </div>
            </section>
        </main>
    }
}
