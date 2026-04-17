use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::models::UserPublic;

/// Signal the TopBar uses to toggle the mobile sidebar drawer.
#[derive(Clone, Copy)]
pub struct MobileDrawerToggle(pub WriteSignal<bool>);

#[component]
pub fn Sidebar() -> impl IntoView {
    let user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();
    let location = use_location();

    // Desktop collapse state (persisted to localStorage)
    let (collapsed, set_collapsed) = signal(false);

    // Mobile drawer state
    let (mobile_open, set_mobile_open) = signal(false);
    provide_context(MobileDrawerToggle(set_mobile_open));

    // Read collapse state from localStorage on mount
    Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window()
                && let Ok(Some(storage)) = window.local_storage()
                && let Ok(Some(val)) = storage.get_item("mybd-sidebar-collapsed")
            {
                set_collapsed.set(val == "true");
            }
        }
    });

    // Close mobile drawer on route change
    let pathname = location.pathname;
    Effect::new(move || {
        let _ = pathname.get();
        set_mobile_open.set(false);
    });

    let toggle_collapsed = move |_| {
        let new_val = !collapsed.get();
        set_collapsed.set(new_val);
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window()
                && let Ok(Some(storage)) = window.local_storage()
            {
                let _ = storage.set_item(
                    "mybd-sidebar-collapsed",
                    if new_val { "true" } else { "false" },
                );
            }
        }
    };

    let album_count_display = expect_context::<RwSignal<Option<i64>>>();
    let following_count_display = expect_context::<crate::app::FollowingCountDisplay>();

    let sidebar_content = move || {
        let current_path = pathname.get();
        view! {
            // Logo + collapse toggle
            <div class="flex items-center px-4 h-14 border-b border-white/10">
                <img src="/mybd.svg" alt="mybd" class="h-7 w-7 rounded-md flex-shrink-0"/>
                <span
                    class="ml-2.5 text-base font-bold text-white overflow-hidden whitespace-nowrap transition-all duration-200"
                    class:hidden=move || collapsed.get()
                >
                    "mybd"
                </span>
                <button
                    class="ml-auto text-indigo-300 hover:text-white hidden md:block flex-shrink-0"
                    on:click=toggle_collapsed
                >
                    {move || if collapsed.get() { "\u{00BB}" } else { "\u{00AB}" }}
                </button>
            </div>

            // Nav links
            <nav class="flex-1 px-2 py-3 space-y-1">
                {
                    let is_active = current_path.starts_with("/collection");
                    view! {
                        <a
                            href="/collection"
                            class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-colors"
                            class=("bg-white/[.12]", is_active)
                            class=("text-white", is_active)
                            class=("text-indigo-200", !is_active)
                            class=("hover:bg-white/[.08]", !is_active)
                            class=("hover:text-white", !is_active)
                        >
                            <span class="material-symbols-outlined text-xl flex-shrink-0">"collections_bookmark"</span>
                            <span
                                class="overflow-hidden whitespace-nowrap transition-all duration-200 flex-1"
                                class:hidden=move || collapsed.get()
                            >
                                "Collection"
                            </span>
                            <span
                                class="text-xs text-indigo-300 tabular-nums"
                                class:hidden=move || collapsed.get()
                            >
                                {move || album_count_display.get().map(|n| n.to_string()).unwrap_or_default()}
                            </span>
                        </a>
                    }
                }
                {
                    let is_active = current_path.starts_with("/friends");
                    view! {
                        <a
                            href="/friends"
                            class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-colors"
                            class=("bg-white/[.12]", is_active)
                            class=("text-white", is_active)
                            class=("text-indigo-200", !is_active)
                            class=("hover:bg-white/[.08]", !is_active)
                            class=("hover:text-white", !is_active)
                        >
                            <span class="material-symbols-outlined text-xl flex-shrink-0">"group"</span>
                            <span
                                class="overflow-hidden whitespace-nowrap transition-all duration-200 flex-1"
                                class:hidden=move || collapsed.get()
                            >
                                "Friends"
                            </span>
                            <span
                                class="text-xs text-indigo-300 tabular-nums"
                                class:hidden=move || collapsed.get()
                            >
                                {move || following_count_display.0.get().map(|n| n.to_string()).unwrap_or_default()}
                            </span>
                        </a>
                    }
                }
                {
                    let is_active = current_path.starts_with("/world");
                    view! {
                        <a
                            href="/world"
                            class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-colors"
                            class=("bg-white/[.12]", is_active)
                            class=("text-white", is_active)
                            class=("text-indigo-200", !is_active)
                            class=("hover:bg-white/[.08]", !is_active)
                            class=("hover:text-white", !is_active)
                        >
                            <span class="material-symbols-outlined text-xl flex-shrink-0">"public"</span>
                            <span
                                class="overflow-hidden whitespace-nowrap transition-all duration-200"
                                class:hidden=move || collapsed.get()
                            >
                                "World"
                            </span>
                        </a>
                    }
                }
            </nav>

            // Bottom links
            <div class="px-2 py-3 border-t border-white/10 space-y-1">
                <a
                    href="https://github.com/elmarco/mybd/issues/new"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm text-indigo-200 hover:bg-white/[.08] hover:text-white transition-colors"
                >
                    <span class="material-symbols-outlined text-xl flex-shrink-0">"bug_report"</span>
                    <span
                        class="overflow-hidden whitespace-nowrap transition-all duration-200"
                        class:hidden=move || collapsed.get()
                    >
                        "Report a bug"
                    </span>
                </a>
            </div>
        }
    };

    let is_authenticated = move || -> bool { user.get().and_then(|r| r.ok()).flatten().is_some() };

    view! {
        <Suspense fallback=|| ()>
        <Show when=is_authenticated>
            // Desktop sidebar
            <aside
                class="hidden md:flex flex-col bg-[#1e1b4b] transition-all duration-200 h-screen sticky top-0"
                class=("w-[200px]", move || !collapsed.get())
                class=("w-14", move || collapsed.get())
            >
                {sidebar_content()}
            </aside>

            // Mobile overlay
            <div
                class="md:hidden fixed inset-0 z-40"
                class:hidden=move || !mobile_open.get()
            >
                // Backdrop
                <div
                    class="absolute inset-0 bg-black/40"
                    on:click=move |_| set_mobile_open.set(false)
                />
                // Drawer
                <aside class="absolute top-0 left-0 bottom-0 w-[240px] bg-[#1e1b4b] flex flex-col shadow-2xl">
                    {sidebar_content()}
                </aside>
            </div>
        </Show>
        </Suspense>
    }
}
