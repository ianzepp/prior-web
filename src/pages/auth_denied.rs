use leptos::prelude::*;
use leptos_router::hooks::query_signal;

#[component]
pub fn AuthDeniedPage() -> impl IntoView {
    let (reason, _) = query_signal::<String>("reason");

    view! {
        <main class="auth-denied">
            <div class="auth-denied-card">
                <h1>"Authorization not completed"</h1>
                <p>
                    {move || match reason.get().as_deref() {
                        Some("access_denied") => {
                            "Login was canceled before authorization completed.".to_string()
                        }
                        Some("login_failed") => {
                            "Prior could not complete login and establish a session.".to_string()
                        }
                        Some(_) | None => {
                            "Authentication did not complete. Try again, and check server logs if the failure repeats.".to_string()
                        }
                    }}
                </p>
                <div class="auth-denied-actions">
                    <form method="get" action="/auth/login">
                        <input type="hidden" name="return_to" value="/app"/>
                        <button class="btn-primary" type="submit">"Try again"</button>
                    </form>
                    <a class="btn-secondary" href="/">"Back home"</a>
                </div>
            </div>
        </main>
    }
}
