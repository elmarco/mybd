use crate::models::UserPublic;
use crate::server::auth::Register;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_action = ServerAction::<Register>::new();
    let user_resource = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();
    let navigate = use_navigate();

    Effect::new(move || {
        if let Some(Ok(())) = register_action.value().get() {
            user_resource.refetch();
            navigate("/collection", Default::default());
        }
    });

    let error = Signal::derive(move || {
        register_action
            .value()
            .get()
            .and_then(|r: Result<(), ServerFnError>| r.err())
            .map(|e: ServerFnError| e.to_string())
    });

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
            <div class="max-w-md w-full space-y-8 p-8 bg-white dark:bg-gray-800 rounded-xl shadow-md">
                <div class="text-center">
                    <h2 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Create your "<span class="text-indigo-accent">"mybd"</span>" account"</h2>
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">"Start tracking your collection today"</p>
                </div>

                {move || error.get().map(|e| view! {
                    <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700 text-sm">
                        {e}
                    </div>
                })}

                <a
                    href="/auth/google/start"
                    rel="external"
                    class="w-full flex items-center justify-center gap-3 py-2 px-4 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-accent"
                >
                    <svg class="w-5 h-5" viewBox="0 0 24 24">
                        <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"/>
                        <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                        <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                        <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                    </svg>
                    "Continue with Google"
                </a>

                <div class="relative">
                    <div class="absolute inset-0 flex items-center">
                        <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                    </div>
                    <div class="relative flex justify-center text-sm">
                        <span class="px-2 bg-white dark:bg-gray-800 text-gray-500 dark:text-gray-400">"or register with email"</span>
                    </div>
                </div>

                <ActionForm action=register_action>
                    <div class="space-y-4">
                        <div>
                            <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Username"</label>
                            <input
                                type="text"
                                name="username"
                                id="username"
                                required
                                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                placeholder="your-username"
                            />
                        </div>
                        <div>
                            <label for="display_name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Display Name "<span class="text-gray-400 dark:text-gray-500 font-normal">"(optional)"</span></label>
                            <input
                                type="text"
                                name="display_name"
                                id="display_name"
                                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                placeholder="Defaults to username"
                            />
                        </div>
                        <div>
                            <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Email"</label>
                            <input
                                type="email"
                                name="email"
                                id="email"
                                required
                                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                placeholder="you@example.com"
                            />
                        </div>
                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Password"</label>
                            <input
                                type="password"
                                name="password"
                                id="password"
                                required
                                minlength="8"
                                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                placeholder="••••••••"
                            />
                            <p class="mt-1 text-xs text-gray-500">"At least 8 characters"</p>
                        </div>
                        <button
                            type="submit"
                            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-indigo-accent hover:bg-indigo-accent-dark focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-accent"
                        >
                            "Create Account"
                        </button>
                    </div>
                </ActionForm>

                <p class="text-center text-sm text-gray-600 dark:text-gray-400">
                    "Already have an account? "
                    <button
                        class="font-medium text-indigo-accent hover:text-indigo-accent-dark cursor-pointer"
                        on:click=move |_| {
                            let login_dialog = expect_context::<crate::components::login_dialog::LoginDialogOpen>().0;
                            login_dialog.set(true);
                        }
                    >
                        "Sign in"
                    </button>
                </p>
            </div>
        </div>
    }
}
