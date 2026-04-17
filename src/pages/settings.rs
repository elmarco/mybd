use crate::app::DarkMode;
use crate::models::{LocationSuggestion, UserPublic};
use crate::server::auth::{DeleteAccount, UpdateProfile};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::web_sys;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let update_action = ServerAction::<UpdateProfile>::new();
    let user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();

    // Refetch user data when the update action completes
    Effect::new(move || {
        if update_action.value().get().is_some() {
            user.refetch();
        }
    });

    let error = Signal::derive(move || {
        update_action
            .value()
            .get()
            .and_then(|r: Result<(), ServerFnError>| r.err())
            .map(|e: ServerFnError| e.to_string())
    });
    let success = Signal::derive(move || {
        update_action
            .value()
            .get()
            .map(|r: Result<(), ServerFnError>| r.is_ok())
            .unwrap_or(false)
    });

    view! {
        <div class="max-w-2xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-6">"Settings"</h1>

            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    let user_data = user.await;
                    match user_data {
                        Ok(Some(u)) => Either::Left(view! {
                            <div>
                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm p-6">
                                    {move || error.get().map(|e| view! {
                                        <div class="mb-4 bg-red-50 border border-red-200 rounded-lg p-4 text-red-700 text-sm">
                                            {e}
                                        </div>
                                    })}
                                    {move || success.get().then(|| view! {
                                        <div class="mb-4 bg-green-50 border border-green-200 rounded-lg p-4 text-green-700 text-sm">
                                            "Profile updated successfully!"
                                        </div>
                                    })}

                                    <ActionForm action=update_action>
                                        <div class="space-y-4">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Display Name"</label>
                                                <input
                                                    type="text"
                                                    name="display_name"
                                                    value={u.display_name.clone()}
                                                    required
                                                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Bio"</label>
                                                <textarea
                                                    name="bio"
                                                    rows="3"
                                                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                                    prop:value=u.bio.clone().unwrap_or_default()
                                                ></textarea>
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Avatar URL"</label>
                                                <input
                                                    type="url"
                                                    name="avatar_url"
                                                    value={u.avatar_url.clone().unwrap_or_default()}
                                                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                                    placeholder="https://..."
                                                />
                                                <p class="mt-1 text-xs text-gray-500">
                                                    "Leave empty to use your "
                                                    <a href="https://gravatar.com" target="_blank" class="text-indigo-accent hover:underline">"Gravatar"</a>
                                                    "."
                                                </p>
                                            </div>
                                            <LocationAutocomplete
                                                initial_location=u.location.clone().unwrap_or_default()
                                            />
                                            <div class="flex items-center gap-2">
                                                <input
                                                    type="checkbox"
                                                    name="is_public"
                                                    id="is_public"
                                                    value="true"
                                                    prop:checked=u.is_public
                                                    class="h-4 w-4 rounded border-gray-300 text-indigo-accent focus:ring-indigo-accent"
                                                />
                                                <label for="is_public" class="text-sm text-gray-700 dark:text-gray-300">"Make my profile and collection public"</label>
                                            </div>
                                            <div class="flex items-center gap-2 ml-6">
                                                <input
                                                    type="checkbox"
                                                    name="wishlist_public"
                                                    id="wishlist_public"
                                                    value="true"
                                                    prop:checked=u.wishlist_public
                                                    class="h-4 w-4 rounded border-gray-300 text-amber-500 focus:ring-amber-500"
                                                />
                                                <label for="wishlist_public" class="text-sm text-gray-700 dark:text-gray-300">
                                                    "Show my wishlist on my public profile"
                                                </label>
                                            </div>
                                            <p class="text-xs text-gray-500 dark:text-gray-400 ml-6">
                                                "For-sale albums are always visible on public profiles."
                                            </p>
                                            <button
                                                type="submit"
                                                class="w-full py-2 px-4 rounded-lg text-white bg-indigo-accent hover:bg-indigo-accent-dark font-medium"
                                            >
                                                "Save Changes"
                                            </button>
                                        </div>
                                    </ActionForm>
                                </div>

                                <DarkModeToggle/>
                                <DeleteAccountSection/>
                            </div>
                        }),
                        _ => Either::Right(view! {
                            <p class="text-gray-500">"Please log in to access settings."</p>
                        }),
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn LocationAutocomplete(initial_location: String) -> impl IntoView {
    let (query, set_query) = signal(initial_location.clone());
    let (location, set_location) = signal(initial_location);
    let (dropdown_open, set_dropdown_open) = signal(false);
    let (suggestions, set_suggestions) = signal(Vec::<LocationSuggestion>::new());

    let search_action = Action::new(move |q: &String| {
        let q = q.clone();
        async move {
            use crate::server::profile::search_location_suggestions;
            search_location_suggestions(q).await.unwrap_or_default()
        }
    });

    Effect::new(move || {
        if let Some(results) = search_action.value().get() {
            set_suggestions.set(results);
        }
    });

    let on_input = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        set_query.set(val.clone());
        // Also update the hidden field so manual typing works
        set_location.set(val.clone());
        if val.trim().len() >= 2 {
            search_action.dispatch(val);
            set_dropdown_open.set(true);
        } else {
            set_suggestions.set(vec![]);
            set_dropdown_open.set(false);
        }
    };

    let on_select = move |suggestion: LocationSuggestion| {
        set_query.set(suggestion.display_name.clone());
        set_location.set(suggestion.display_name);
        set_dropdown_open.set(false);
    };

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Location"</label>
            <div class="relative">
                <input
                    type="text"
                    autocomplete="off"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-accent focus:border-indigo-accent dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                    placeholder="Search for a city..."
                    prop:value=move || query.get()
                    on:input=on_input
                    on:focus=move |_| {
                        if !suggestions.get().is_empty() {
                            set_dropdown_open.set(true);
                        }
                    }
                    on:blur=move |_| {
                        #[cfg(feature = "hydrate")]
                        {
                            use wasm_bindgen::prelude::*;
                            let handler = Closure::once(move || {
                                set_dropdown_open.set(false);
                            });
                            if let Some(window) = web_sys::window() {
                                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                    handler.as_ref().unchecked_ref(),
                                    150,
                                );
                            }
                            handler.forget();
                        }
                    }
                />
                <Show when=move || dropdown_open.get() && !suggestions.get().is_empty()>
                    <div class="absolute left-0 right-0 top-full mt-1 bg-white rounded-lg shadow-lg border border-gray-200 z-50 overflow-hidden max-h-64 overflow-y-auto">
                        <For
                            each=move || suggestions.get()
                            key=|s| s.display_name.clone()
                            let:suggestion
                        >
                            {
                                let s = suggestion.clone();
                                view! {
                                    <button
                                        type="button"
                                        class="w-full text-left px-4 py-2.5 text-sm text-gray-700 hover:bg-gray-50 cursor-pointer"
                                        on:mousedown=move |ev| {
                                            ev.prevent_default();
                                            on_select(s.clone());
                                        }
                                    >
                                        <div class="font-medium">{
                                            let city = &suggestion.city;
                                            let country = &suggestion.country;
                                            match (city.as_str(), country.as_str()) {
                                                ("", "") => String::new(),
                                                (c, "") => c.to_string(),
                                                ("", co) => co.to_string(),
                                                (c, co) => format!("{c}, {co}"),
                                            }
                                        }</div>
                                        <div class="text-xs text-gray-400 truncate">{suggestion.display_name.clone()}</div>
                                    </button>
                                }
                            }
                        </For>
                    </div>
                </Show>
            </div>
            <p class="mt-1 text-xs text-gray-500">"Start typing to search for your city."</p>
            <input type="hidden" name="location" prop:value=move || location.get()/>
        </div>
    }
}

#[component]
fn DeleteAccountSection() -> impl IntoView {
    let delete_action = ServerAction::<DeleteAccount>::new();
    let (dialog_open, set_dialog_open) = signal(false);
    let (confirmation, set_confirmation) = signal(String::new());

    let phrase_matches = move || confirmation.get() == "Bachi-bouzouk";

    let delete_error = Signal::derive(move || {
        delete_action
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| e.to_string())
    });

    Effect::new(move || {
        if let Some(Ok(())) = delete_action.value().get() {
            // Hard navigation to avoid stale user resource redirecting to /collection
            if let Some(w) = web_sys::window() {
                let _ = w.location().set_href("/");
            }
        }
    });

    view! {
        <div>
            <div class="mt-8 bg-white dark:bg-gray-800 rounded-xl shadow-sm p-6 border border-red-200 dark:border-red-800">
                <h2 class="text-lg font-semibold text-red-600 mb-2">"Danger Zone"</h2>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
                    "Permanently delete your account and all associated data. This action cannot be undone."
                </p>
                <button
                    class="py-2 px-4 rounded-lg text-white bg-red-600 hover:bg-red-700 font-medium text-sm"
                    on:click=move |_| {
                        set_confirmation.set(String::new());
                        set_dialog_open.set(true);
                    }
                >
                    "Delete Account"
                </button>
            </div>

            <Show when=move || dialog_open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                <div class="bg-white rounded-xl shadow-lg p-6 max-w-md w-full mx-4">
                    <h3 class="text-lg font-bold text-gray-900 mb-2">"Delete your account?"</h3>
                    <p class="text-sm text-gray-600 mb-4">
                        "This will permanently delete your account, collection, and all associated data. This cannot be undone."
                    </p>

                    {move || delete_error.get().map(|e| view! {
                        <div class="mb-4 bg-red-50 border border-red-200 rounded-lg p-3 text-red-700 text-sm">
                            {e}
                        </div>
                    })}

                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Type " <span class="font-bold">"Bachi-bouzouk"</span> " to confirm"
                    </label>
                    <input
                        type="text"
                        class="block w-full px-3 py-2 border border-gray-300 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-red-500 mb-4"
                        placeholder="Bachi-bouzouk"
                        prop:value=move || confirmation.get()
                        on:input=move |ev| set_confirmation.set(event_target_value(&ev))
                    />

                    <div class="flex gap-3 justify-end">
                        <button
                            class="py-2 px-4 rounded-lg text-gray-700 bg-gray-100 hover:bg-gray-200 font-medium text-sm"
                            on:click=move |_| set_dialog_open.set(false)
                        >
                            "Cancel"
                        </button>
                        <ActionForm action=delete_action>
                            <input type="hidden" name="confirmation" prop:value=move || confirmation.get()/>
                            <button
                                type="submit"
                                class="py-2 px-4 rounded-lg text-white font-medium text-sm transition-colors"
                                class=("bg-red-600", move || phrase_matches())
                                class=("hover:bg-red-700", move || phrase_matches())
                                class=("bg-red-300", move || !phrase_matches())
                                class=("cursor-not-allowed", move || !phrase_matches())
                                disabled=move || !phrase_matches()
                            >
                                "Delete my account"
                            </button>
                        </ActionForm>
                    </div>
                </div>
            </div>
            </Show>
        </div>
    }
}

#[component]
fn DarkModeToggle() -> impl IntoView {
    let dark_mode = expect_context::<DarkMode>().0;

    let toggle = move |_| {
        let new_val = !dark_mode.get();
        dark_mode.set(new_val);

        #[cfg(feature = "hydrate")]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.document_element() {
                    if new_val {
                        let _ = el.class_list().add_1("dark");
                    } else {
                        let _ = el.class_list().remove_1("dark");
                    }
                }
            }
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item("theme", if new_val { "dark" } else { "light" });
            }
        }
    };

    view! {
        <div class="mt-6 bg-white dark:bg-gray-800 rounded-xl shadow-sm p-6">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">"Appearance"</h2>
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm text-gray-700 dark:text-gray-300">"Dark mode"</p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">"Switch between light and dark themes"</p>
                </div>
                <button
                    type="button"
                    class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors"
                    class=("bg-indigo-accent", move || dark_mode.get())
                    class=("bg-gray-300", move || !dark_mode.get())
                    on:click=toggle
                >
                    <span
                        class="inline-block h-4 w-4 rounded-full bg-white transition-transform"
                        class=("translate-x-6", move || dark_mode.get())
                        class=("translate-x-1", move || !dark_mode.get())
                    />
                </button>
            </div>
        </div>
    }
}
