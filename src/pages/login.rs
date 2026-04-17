use crate::components::login_dialog::LoginDialogOpen;
use leptos::prelude::*;

/// The /login route now just opens the login dialog over the current page.
/// This route is kept for server-side redirects (e.g. Google OAuth errors).
#[component]
pub fn LoginPage() -> impl IntoView {
    let login_dialog = expect_context::<LoginDialogOpen>().0;

    Effect::new(move || {
        login_dialog.set(true);
    });

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
            <p class="text-gray-400">"Redirecting to sign in..."</p>
        </div>
    }
}
