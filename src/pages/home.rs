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
                    <form method="get" action="/auth/login">
                        <input type="hidden" name="return_to" value="/app"/>
                        <button class="btn-primary" type="submit">"Log in"</button>
                    </form>
                    <a class="btn-secondary" href="/app">"Open Dashboard"</a>
                </div>
            </div>
        </main>
    }
}
