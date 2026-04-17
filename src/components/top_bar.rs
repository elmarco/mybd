use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::components::login_dialog::LoginDialogOpen;
use crate::components::sidebar::MobileDrawerToggle;
use crate::components::{Avatar, BarcodeScanner};
use crate::models::UserPublic;
use crate::server::social::{clear_notifications, get_notifications, mark_notifications_read};

#[component]
pub fn TopBar() -> impl IntoView {
    let user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();
    let navigate = use_navigate();
    let (search_query, set_search_query) = signal(String::new());
    let (scanner_open, set_scanner_open) = signal(false);

    let navigate_for_scan = navigate.clone();
    let on_scan = Callback::new(move |ean: String| {
        set_scanner_open.set(false);
        let encoded = urlencoding::encode(&ean);
        navigate_for_scan(&format!("/search?q={encoded}"), Default::default());
    });
    let on_scanner_close = Callback::new(move |()| {
        set_scanner_open.set(false);
    });

    // Search history signals (set_search_history used only in hydrate mode)
    #[allow(unused_variables)]
    let (search_history, set_search_history) = signal(Vec::<String>::new());
    let (history_open, set_history_open) = signal(false);
    let (selected_index, set_selected_index) = signal(None::<usize>);

    // Filtered history entries — shared between keydown handler and dropdown UI
    let filtered_history = Memo::new(move |_| {
        let q = search_query.get().to_lowercase();
        search_history
            .get()
            .into_iter()
            .filter(|h| q.is_empty() || h.to_lowercase().contains(&q))
            .collect::<Vec<_>>()
    });

    let (input_focused, set_input_focused) = signal(false);

    let navigate_for_search = navigate.clone();
    let on_search_submit = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" {
            set_history_open.set(false);
            set_selected_index.set(None);
            return;
        }
        if ev.key() == "ArrowDown" {
            ev.prevent_default();
            let count = filtered_history.get().len();
            if count > 0 {
                if !history_open.get() {
                    set_history_open.set(true);
                }
                set_selected_index.update(|idx| {
                    *idx = Some(match *idx {
                        None => 0,
                        Some(i) if i + 1 >= count => 0,
                        Some(i) => i + 1,
                    });
                });
            }
            return;
        }
        if ev.key() == "ArrowUp" {
            ev.prevent_default();
            let count = filtered_history.get().len();
            if count > 0 {
                if !history_open.get() {
                    set_history_open.set(true);
                }
                set_selected_index.update(|idx| {
                    *idx = Some(match *idx {
                        None | Some(0) => count - 1,
                        Some(i) => i - 1,
                    });
                });
            }
            return;
        }
        if ev.key() == "Enter" {
            // Use the highlighted history entry if one is selected
            let q = if let Some(idx) = selected_index.get() {
                filtered_history
                    .get()
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| search_query.get())
            } else {
                search_query.get()
            };
            set_history_open.set(false);
            set_selected_index.set(None);
            if !q.trim().is_empty() {
                set_search_query.set(q.clone());
                #[cfg(feature = "hydrate")]
                add_to_search_history(&q, set_search_history);
                let encoded = urlencoding::encode(&q);
                navigate_for_search(&format!("/search?q={encoded}"), Default::default());
            }
        }
    };

    // Sync search bar from URL ?q= param (for direct links, back button, barcode scans).
    // Skip when input is focused to avoid overwriting what the user is typing.
    let params = use_query_map();
    Effect::new(move || {
        let q = params.get().get("q").unwrap_or_default();
        let focused = untrack(|| input_focused.get());
        if !q.is_empty() && !focused {
            set_search_query.set(q);
        }
    });

    let navigate_for_history = navigate.clone();

    // Load search history from localStorage on mount
    #[cfg(feature = "hydrate")]
    Effect::new(move || {
        set_search_history.set(load_search_history());
    });

    let is_authenticated = move || user.get().and_then(|r| r.ok()).flatten().is_some();

    let user_info = move || user.get().and_then(|r| r.ok()).flatten();

    // Create notification signals and resource at component level (not inside <Show>)
    // so they always consume the same serialization IDs during SSR and hydration.
    let (notif_open, set_notif_open) = signal(false);
    let notifications = Resource::new(
        move || notif_open.get(),
        move |open| async move {
            if open {
                get_notifications().await
            } else {
                Ok(vec![])
            }
        },
    );

    view! {
        <header class="h-14 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex items-center px-4 gap-3 sticky top-0 z-30 flex-shrink-0">
            <Suspense fallback=|| ()>
            // Mobile: hamburger (authenticated) or logo (guest)
            <Show
                when=is_authenticated
                fallback=|| view! {
                    <a href="/" class="flex items-center gap-2 mr-2">
                        <img src="/mybd.svg" alt="mybd" class="h-7 w-7 rounded-md"/>
                    </a>
                }
            >
                <button
                    class="md:hidden p-2 text-gray-600 hover:text-gray-900 -ml-2"
                    on:click=move |_| {
                        if let Some(toggle) = use_context::<MobileDrawerToggle>() {
                            toggle.0.update(|v| *v = !*v);
                        }
                    }
                >
                    <span class="material-symbols-outlined text-2xl">"menu"</span>
                </button>
            </Show>

            // Search bar (centered)
            <div class="flex-1 flex justify-center items-center gap-2">
                <div class="relative w-full max-w-lg" data-search-history="">
                    <div class="flex items-center bg-gray-100 dark:bg-gray-700 dark:focus-within:bg-gray-100 rounded-full px-4 py-2 transition-colors">
                        <span class="material-symbols-outlined text-xl text-gray-400 flex-shrink-0">"search"</span>
                        <input
                            type="text"
                            id="search-input"
                            autocomplete="off"
                            class="flex-1 bg-transparent border-none outline-none text-sm text-gray-900 dark:text-gray-100 dark:focus:text-gray-900 placeholder-gray-400 ml-3 transition-colors"
                            placeholder="search"
                            prop:value=move || search_query.get()
                            on:input=move |ev| {
                                let val = event_target_value(&ev);
                                set_search_query.set(val);
                                set_history_open.set(true);
                                set_selected_index.set(None);
                            }
                            on:keydown=on_search_submit
                            on:focus=move |_| {
                                set_input_focused.set(true);
                                if !search_history.get().is_empty() {
                                    set_history_open.set(true);
                                }
                            }
                            on:blur=move |_| {
                                set_input_focused.set(false);
                                // Delay close to allow click-through on dropdown items
                                #[cfg(feature = "hydrate")]
                                {
                                    use wasm_bindgen::prelude::*;
                                    let handler = Closure::once(move || {
                                        set_history_open.set(false);
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
                        // Clear button (visible when there's text)
                        <Show when=move || !search_query.get().is_empty()>
                            <button
                                class="text-gray-400 hover:text-gray-600 flex-shrink-0"
                                on:click=move |_| set_search_query.set(String::new())
                            >
                                <span class="material-symbols-outlined text-xl">"close"</span>
                            </button>
                        </Show>
                    </div>

                    // Search history dropdown
                    {move || {
                        if !history_open.get() {
                            return view! { <div></div> }.into_any();
                        }
                        let filtered = filtered_history.get();
                        if filtered.is_empty() {
                            return view! { <div></div> }.into_any();
                        }
                        let sel = selected_index.get();
                        let navigate_hist = navigate_for_history.clone();
                        view! {
                            <div class="absolute left-0 right-0 top-full mt-1 bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 z-50 overflow-hidden max-h-64 overflow-y-auto">
                                {filtered.into_iter().enumerate().map(|(i, entry)| {
                                    let entry_display = entry.clone();
                                    let entry_click = entry.clone();
                                    let nav = navigate_hist.clone();
                                    let item_class = if sel == Some(i) {
                                        "w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 flex items-center gap-3 cursor-pointer"
                                    } else {
                                        "w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-3 cursor-pointer"
                                    };
                                    view! {
                                        <button
                                            class=item_class
                                            on:mousedown=move |ev| {
                                                ev.prevent_default();
                                                let e = entry_click.clone();
                                                set_search_query.set(e.clone());
                                                set_history_open.set(false);
                                                set_selected_index.set(None);
                                                let encoded = urlencoding::encode(&e);
                                                nav(&format!("/search?q={encoded}"), Default::default());
                                            }
                                        >
                                            <span class="material-symbols-outlined text-lg text-gray-400">"history"</span>
                                            {entry_display}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                </div>
                // Barcode scanner button
                <button
                    class="text-gray-500 hover:text-gray-700 flex-shrink-0 p-2"
                    on:click=move |_| set_scanner_open.set(true)
                >
                    <span class="material-symbols-outlined text-2xl">"barcode_scanner"</span>
                </button>
            </div>

            // Right side: profile or auth links
            <Show
                when=is_authenticated
                fallback=move || {
                    let login_dialog = expect_context::<LoginDialogOpen>().0;
                    view! {
                        <div class="flex items-center gap-3 flex-shrink-0">
                            <button
                                class="text-sm text-gray-600 hover:text-gray-900 cursor-pointer"
                                on:click=move |_| login_dialog.set(true)
                            >
                                "Sign In"
                            </button>
                            <a href="/register" class="text-sm px-4 py-2 rounded-lg text-white bg-indigo-accent hover:bg-indigo-accent-dark">"Sign Up"</a>
                        </div>
                    }
                }
            >
                {move || {
                    let u = user_info();
                    let user_data = u.clone();
                    let (avatar_url, name) = u
                        .map(|u| (u.avatar_url, u.display_name))
                        .unwrap_or_default();
                    let avatar_url_menu = avatar_url.clone();
                    let name_menu = name.clone();
                    let (menu_open, set_menu_open) = signal(false);
                    let title = name.clone();

                    let unread = expect_context::<RwSignal<i64>>();
                    let unread_resource = expect_context::<Resource<Result<i64, ServerFnError>>>();

                    // Close dropdowns on outside click (without blocking overlays)
                    #[cfg(feature = "hydrate")]
                    {
                        use wasm_bindgen::prelude::*;

                        // Notification dropdown
                        Effect::new(move || {
                            if notif_open.get() {
                                let handler = Closure::<dyn Fn(web_sys::Event)>::new(move |ev: web_sys::Event| {
                                    if let Some(target) = ev.target() {
                                        if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                                            if el.closest("[data-notif-dropdown]").ok().flatten().is_none() {
                                                set_notif_open.set(false);
                                            }
                                        }
                                    }
                                });
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    let opts = web_sys::AddEventListenerOptions::new();
                                    opts.set_once(true);
                                    opts.set_capture(true);
                                    let _ = doc.add_event_listener_with_callback_and_add_event_listener_options(
                                        "pointerdown",
                                        handler.as_ref().unchecked_ref(),
                                        &opts,
                                    );
                                    handler.forget();
                                }
                            }
                        });

                        // User menu
                        let menu_open_close = menu_open;
                        Effect::new(move || {
                            if menu_open_close.get() {
                                let handler = Closure::<dyn Fn(web_sys::Event)>::new(move |ev: web_sys::Event| {
                                    if let Some(target) = ev.target() {
                                        if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                                            if el.closest("[data-user-menu]").ok().flatten().is_none() {
                                                set_menu_open.set(false);
                                            }
                                        }
                                    }
                                });
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    let opts = web_sys::AddEventListenerOptions::new();
                                    opts.set_once(true);
                                    opts.set_capture(true);
                                    let _ = doc.add_event_listener_with_callback_and_add_event_listener_options(
                                        "pointerdown",
                                        handler.as_ref().unchecked_ref(),
                                        &opts,
                                    );
                                    handler.forget();
                                }
                            }
                        });
                    }

                    view! {
                        // Notification bell
                        <div class="relative flex-shrink-0" data-notif-dropdown="">
                            <button
                                class="relative p-2 text-gray-500 hover:text-gray-700 cursor-pointer"
                                on:click=move |_| {
                                    let opening = !notif_open.get();
                                    set_notif_open.set(opening);
                                    if opening && unread.get() > 0 {
                                        leptos::task::spawn_local(async move {
                                            if mark_notifications_read().await.is_ok() {
                                                unread.set(0);
                                                unread_resource.refetch();
                                            }
                                        });
                                    }
                                }
                            >
                                <span class="material-symbols-outlined text-xl">"notifications"</span>
                                <Show when=move || { unread.get() > 0 }>
                                    <span class="absolute -top-0.5 -right-0.5 w-4 h-4 bg-red-500 rounded-full text-white text-[10px] font-bold flex items-center justify-center">
                                        {move || unread.get()}
                                    </span>
                                </Show>
                            </button>

                            <Show when=move || notif_open.get()>
                                <div class="absolute right-0 top-full mt-2 w-80 max-w-[calc(100vw-2rem)] bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 z-50 overflow-hidden max-h-96 overflow-y-auto">
                                    <div class="p-3 border-b border-gray-100 dark:border-gray-700 flex items-center justify-between">
                                        <h3 class="font-semibold text-gray-900 dark:text-gray-100 text-sm">"Notifications"</h3>
                                        <button
                                            class="text-xs text-gray-400 hover:text-red-500 cursor-pointer transition-colors"
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    if clear_notifications().await.is_ok() {
                                                        notifications.refetch();
                                                        unread.set(0);
                                                    }
                                                });
                                            }
                                        >
                                            "Clear all"
                                        </button>
                                    </div>
                                    {move || match notifications.get() {
                                        None => view! {
                                            <p class="p-4 text-gray-500 text-sm">"Loading..."</p>
                                        }.into_any(),
                                        Some(Ok(items)) if items.is_empty() => view! {
                                            <p class="p-4 text-gray-500 text-sm text-center">"No notifications"</p>
                                        }.into_any(),
                                        Some(Ok(items)) => view! {
                                            <div class="divide-y divide-gray-100 dark:divide-gray-700">
                                                {items.into_iter().map(|n| {
                                                    render_notification(n)
                                                }).collect_view()}
                                            </div>
                                        }.into_any(),
                                        Some(Err(_)) => view! {
                                            <p class="p-4 text-gray-500 text-sm">"Failed to load"</p>
                                        }.into_any(),
                                    }}
                                </div>
                            </Show>
                        </div>

                        <div class="relative flex-shrink-0" data-user-menu="">
                            <button
                                class="cursor-pointer"
                                title=title
                                on:click=move |_| set_menu_open.update(|v| *v = !*v)
                            >
                                <Avatar url=avatar_url name=name/>
                            </button>

                            <Show when=move || menu_open.get()>
                                <div class="absolute right-0 top-full mt-2 w-72 max-w-[calc(100vw-2rem)] bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 z-50 overflow-hidden">
                                    // User info header
                                    <div class="flex items-center gap-3 p-4 border-b border-gray-100 dark:border-gray-700">
                                        <Avatar
                                            url=avatar_url_menu.clone()
                                            name=name_menu.clone()
                                            size="w-12 h-12"
                                            text_size="text-lg"
                                        />
                                        <div class="min-w-0">
                                            <div class="font-semibold text-gray-900 dark:text-gray-100 truncate">
                                                {name_menu.clone()}
                                            </div>
                                            <div class="text-sm text-gray-500 dark:text-gray-400 truncate">
                                                {"@"}{user_data.as_ref().map(|u| u.username.clone()).unwrap_or_default()}
                                            </div>
                                        </div>
                                    </div>

                                    // Stats
                                    <div class="grid grid-cols-3 divide-x divide-gray-100 dark:divide-gray-700 border-b border-gray-100 dark:border-gray-700 text-center py-3">
                                        <a href="/collection" class="block cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors rounded-lg py-1"
                                            on:click=move |_| set_menu_open.set(false)>
                                            <div class="text-lg font-bold text-gray-900 dark:text-gray-100">
                                                {move || {
                                                    let count = expect_context::<RwSignal<Option<i64>>>();
                                                    count.get().map(|n| n.to_string()).unwrap_or_else(|| "\u{2014}".to_string())
                                                }}
                                            </div>
                                            <div class="text-xs text-gray-500 dark:text-gray-400">"Albums"</div>
                                        </a>
                                        <a href="/friends" class="block cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors rounded-lg py-1"
                                            on:click=move |_| set_menu_open.set(false)>
                                            <div class="text-lg font-bold text-gray-900 dark:text-gray-100">
                                                {move || {
                                                    let count = expect_context::<crate::app::FollowingCountDisplay>();
                                                    count.0.get().map(|n| n.to_string()).unwrap_or_else(|| "\u{2014}".to_string())
                                                }}
                                            </div>
                                            <div class="text-xs text-gray-500 dark:text-gray-400">"Friends"</div>
                                        </a>
                                        <a href="/lent" class="block cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors rounded-lg py-1"
                                            on:click=move |_| set_menu_open.set(false)>
                                            <div class="text-lg font-bold text-gray-900 dark:text-gray-100">
                                                {move || {
                                                    let count = expect_context::<crate::app::LentCountDisplay>();
                                                    count.0.get().map(|n| n.to_string()).unwrap_or_else(|| "\u{2014}".to_string())
                                                }}
                                            </div>
                                            <div class="text-xs text-gray-500 dark:text-gray-400">"Lent"</div>
                                        </a>
                                    </div>

                                    // Menu items
                                    <div class="py-1">
                                        <a
                                            href={format!("/profile/{}", user_data.as_ref().map(|u| u.username.as_str()).unwrap_or("me"))}
                                            class="flex items-center gap-3 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
                                            on:click=move |_| set_menu_open.set(false)
                                        >
                                            <span class="material-symbols-outlined text-xl text-gray-400">"person"</span>
                                            "Public profile"
                                        </a>
                                        <a
                                            href="/settings"
                                            class="flex items-center gap-3 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
                                            on:click=move |_| set_menu_open.set(false)
                                        >
                                            <span class="material-symbols-outlined text-xl text-gray-400">"settings"</span>
                                            "Settings"
                                        </a>
                                    </div>

                                    // Logout
                                    <div class="border-t border-gray-100 dark:border-gray-700 py-1">
                                        <a
                                            href="/auth/logout"
                                            rel="external"
                                            class="flex items-center gap-3 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
                                        >
                                            <span class="material-symbols-outlined text-xl text-gray-400">"logout"</span>
                                            "Logout"
                                        </a>
                                    </div>
                                </div>
                            </Show>
                        </div>
                    }
                }}
            </Show>
            </Suspense>
        </header>

        // Barcode scanner modal
        {move || scanner_open.get().then(|| view! {
            <BarcodeScanner on_scan=on_scan on_close=on_scanner_close/>
        })}
    }
}

fn render_notification(n: crate::models::Notification) -> impl IntoView {
    let payload: serde_json::Value = serde_json::from_str(&n.payload).unwrap_or_default();

    match n.notification_type.as_str() {
        "followed" => {
            let name = payload["from_display_name"]
                .as_str()
                .unwrap_or("Someone")
                .to_string();
            let username = payload["from_username"].as_str().unwrap_or("").to_string();
            let href = format!("/profile/{}", urlencoding::encode(&username));
            view! {
                <a href=href class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                    <span class="material-symbols-outlined text-indigo-accent flex-shrink-0" style="font-size: 20px;">"person_add"</span>
                    <div class="text-sm">
                        <span class="font-medium text-gray-900 dark:text-gray-100">{name}</span>
                        " added you as a friend"
                    </div>
                </a>
            }
            .into_any()
        }
        "album_lent" => {
            let lender = payload["lender_display_name"]
                .as_str()
                .unwrap_or("Someone")
                .to_string();
            let title = payload["album_title"]
                .as_str()
                .unwrap_or("an album")
                .to_string();
            let album_slug = payload["album_slug"].as_str().unwrap_or("");
            let href = if !album_slug.is_empty() {
                format!("/album/{album_slug}")
            } else {
                let album_id = payload["album_id"].as_i64().unwrap_or(0);
                format!("/album/{album_id}")
            };
            view! {
                <a href=href class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                    <span class="material-symbols-outlined text-green-500 flex-shrink-0" style="font-size: 20px;">"book"</span>
                    <div class="text-sm">
                        <span class="font-medium text-gray-900 dark:text-gray-100">{lender}</span>
                        " lent you "
                        <span class="font-medium text-gray-900 dark:text-gray-100">{title}</span>
                    </div>
                </a>
            }
            .into_any()
        }
        _ => view! {
            <div class="px-4 py-3 text-sm text-gray-500">"Unknown notification"</div>
        }
        .into_any(),
    }
}

#[cfg(feature = "hydrate")]
fn load_search_history() -> Vec<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("mybd-search-history").ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

#[cfg(feature = "hydrate")]
fn save_search_history(history: &[String]) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(json) = serde_json::to_string(history) {
            let _ = storage.set_item("mybd-search-history", &json);
        }
    }
}

#[cfg(feature = "hydrate")]
fn add_to_search_history(query: &str, set_history: WriteSignal<Vec<String>>) {
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        return;
    }
    let mut history = load_search_history();
    history.retain(|h| h != &trimmed);
    history.insert(0, trimmed);
    history.truncate(15);
    save_search_history(&history);
    set_history.set(history);
}
