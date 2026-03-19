use leptos::prelude::*;

/// Home page redirects to the app dashboard.
/// No marketing landing page — the app is the product.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="auth-denied">
            <div class="auth-denied-card">
                <div class="brand" style="justify-content: center; margin-bottom: 24px;">
                    <div class="brand-mark">"P"</div>
                    "Prior"
                </div>
                <h1>"Concurrent software delivery"</h1>
                <p>"Prior is a compiler-style software factory. Rooms are your inbox. Factory runs post as threaded updates. The lifecycle sidebar tracks every phase."</p>
                <div class="auth-denied-actions">
                    <a class="btn-primary" href="/auth/login?return_to=/app">"Log in"</a>
                    <a class="btn-secondary" href="/app">"Open Dashboard"</a>
                </div>
            </div>
        </main>
    }
}
