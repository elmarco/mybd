use leptos::prelude::*;

use crate::components::login_dialog::LoginDialogOpen;
use crate::models::UserPublic;

/// Wraps protected page content — opens login dialog when not authenticated.
///
/// Does NOT wrap children in `<Suspense>` or `<Show>` to avoid hydration marker
/// mismatches with the child page's own `<Suspense>`. The child page's server
/// function handles unauthenticated users by returning an error or `None`.
#[component]
pub fn AuthGuard(children: Children) -> impl IntoView {
    let user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();
    let login_dialog = expect_context::<LoginDialogOpen>().0;

    // Open login dialog when user resolves as unauthenticated
    Effect::new(move || {
        if let Some(result) = user.get()
            && result.ok().flatten().is_none()
        {
            login_dialog.set(true);
        }
    });

    children()
}
