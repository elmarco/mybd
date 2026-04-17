use crate::components::SeriesCard;
use crate::server::series::{get_user_collection, get_user_wishlist};
use leptos::either::EitherOf3;
use leptos::prelude::*;

/// Global keydown listener for tab switching (1/2 keys).
fn use_collection_keynav(set_active_tab: WriteSignal<u8>) {
    use leptos::ev::keydown;
    use leptos::web_sys;

    let _handle = window_event_listener(keydown, move |ev: web_sys::KeyboardEvent| {
        if let Some(tag) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element())
            .map(|el| el.tag_name())
        {
            match tag.as_str() {
                "INPUT" | "TEXTAREA" | "SELECT" => return,
                _ => {}
            }
        }
        match ev.key().as_str() {
            "1" => set_active_tab.set(0),
            "2" => set_active_tab.set(1),
            _ => {}
        }
    });
}

#[component]
pub fn CollectionPage() -> impl IntoView {
    let sort_by: RwSignal<Option<String>> = RwSignal::new(None);
    let (active_tab, set_active_tab) = signal(0u8); // 0 = collection, 1 = wishlist
    use_collection_keynav(set_active_tab);

    let collection = Resource::new(move || sort_by.get(), get_user_collection);
    let wishlist = Resource::new(|| (), |_| get_user_wishlist());

    view! {
        <div class="max-w-6xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-4">"My Collection"</h1>

            // Tab bar
            <div class="border-b border-gray-200 dark:border-gray-700 mb-6">
                <nav class="flex gap-8 -mb-px">
                    <button
                        class="cursor-pointer px-1 py-3 text-sm font-medium border-b-2 transition-colors"
                        class=("text-indigo-accent", move || active_tab.get() == 0)
                        class=("border-indigo-accent", move || active_tab.get() == 0)
                        class=("text-gray-500 dark:text-gray-300", move || active_tab.get() != 0)
                        class=("border-transparent", move || active_tab.get() != 0)
                        on:click=move |_| set_active_tab.set(0)
                    >
                        "Collection"
                        <Suspense fallback=|| ()>
                            {move || Suspend::new(async move {
                                collection.await.ok().map(|items| view! {
                                    <span
                                        class="ml-2 text-xs font-semibold px-2.5 py-0.5 rounded-full inline-block min-w-5 text-center"
                                        class=("bg-indigo-accent text-white", move || active_tab.get() == 0)
                                        class=("bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300", move || active_tab.get() != 0)
                                    >
                                        {items.len()}
                                    </span>
                                })
                            })}
                        </Suspense>
                    </button>
                    <button
                        class="cursor-pointer px-1 py-3 text-sm font-medium border-b-2 transition-colors"
                        class=("text-amber-500", move || active_tab.get() == 1)
                        class=("border-amber-500", move || active_tab.get() == 1)
                        class=("text-gray-500 dark:text-gray-300", move || active_tab.get() != 1)
                        class=("border-transparent", move || active_tab.get() != 1)
                        on:click=move |_| set_active_tab.set(1)
                    >
                        "Wishlist"
                        <Suspense fallback=|| ()>
                            {move || Suspend::new(async move {
                                wishlist.await.ok().map(|items| view! {
                                    <span
                                        class="ml-2 text-xs font-semibold px-2.5 py-0.5 rounded-full inline-block min-w-5 text-center"
                                        class=("bg-amber-500 text-white", move || active_tab.get() == 1)
                                        class=("bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300", move || active_tab.get() != 1)
                                    >
                                        {items.len()}
                                    </span>
                                })
                            })}
                        </Suspense>
                    </button>
                </nav>
            </div>

            // Collection tab
            <div class=("hidden", move || active_tab.get() != 0)>
                <div class="flex flex-wrap gap-4 mb-6">
                    <select
                        class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            sort_by.set(if val.is_empty() { None } else { Some(val) });
                        }
                    >
                        <option value="">"Recently Added"</option>
                        <option value="title">"Title"</option>
                    </select>
                </div>

                <Suspense fallback=|| view! { <p class="text-gray-500 dark:text-gray-400">"Loading collection..."</p> }>
                    {move || Suspend::new(async move {
                        match collection.await {
                            Ok(items) if items.is_empty() => {
                                EitherOf3::A(view! {
                                    <div class="text-center py-12">
                                        <p class="text-gray-500 dark:text-gray-400 text-lg">"Your collection is empty."</p>
                                        <a href="/search" class="text-indigo-accent hover:underline mt-2 inline-block">
                                            "Search for series to get started!"
                                        </a>
                                    </div>
                                })
                            }
                            Ok(items) => {
                                EitherOf3::B(view! {
                                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                                        {items.into_iter().map(|swo| {
                                            let ownership = Some((swo.owned_count, swo.total_albums));
                                            let for_sale_count = swo.for_sale_count;
                                            let is_terminated = swo.is_terminated.unwrap_or(false);
                                            let series: crate::models::Series = swo.into();
                                            view! { <SeriesCard series=series ownership=ownership for_sale_count=for_sale_count is_terminated=is_terminated/> }
                                        }).collect_view()}
                                    </div>
                                })
                            }
                            Err(e) => {
                                EitherOf3::C(view! {
                                    <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                        {format!("Error loading collection: {e}")}
                                    </div>
                                })
                            }
                        }
                    })}
                </Suspense>
            </div>

            // Wishlist tab
            <div class=("hidden", move || active_tab.get() != 1)>
                <Suspense fallback=|| view! { <p class="text-gray-500 dark:text-gray-400">"Loading wishlist..."</p> }>
                    {move || Suspend::new(async move {
                        match wishlist.await {
                            Ok(items) if items.is_empty() => {
                                EitherOf3::A(view! {
                                    <p class="text-gray-500 dark:text-gray-400">"Your wishlist is empty. Browse series and add albums you want!"</p>
                                })
                            }
                            Ok(items) => {
                                EitherOf3::B(view! {
                                    <div class="flex flex-col gap-2">
                                        {items.into_iter().map(|item| {
                                            let href = format!("/album/{}", item.album_slug);
                                            let tome = item.tome.map(|t| format!(" · T{t}")).unwrap_or_default();
                                            view! {
                                                <a href=href class="flex items-center gap-3 px-4 py-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg hover:bg-amber-100 dark:hover:bg-amber-900/40 transition-colors">
                                                    <div class="w-10 h-14 rounded bg-gray-200 dark:bg-gray-700 flex-shrink-0 overflow-hidden">
                                                        {item.cover_url.map(|url| view! {
                                                            <img src=url class="w-full h-full object-cover"/>
                                                        })}
                                                    </div>
                                                    <div class="flex-1 min-w-0">
                                                        <div class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                                            {item.album_title.unwrap_or_else(|| format!("Tome {}", item.tome.unwrap_or(0)))}
                                                        </div>
                                                        <div class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                                            {item.series_title}{tome}
                                                        </div>
                                                    </div>
                                                    <span class="material-symbols-outlined text-amber-500" style="font-size: 20px;">"star"</span>
                                                </a>
                                            }
                                        }).collect_view()}
                                    </div>
                                })
                            }
                            Err(e) => {
                                EitherOf3::C(view! {
                                    <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                        {format!("Error loading wishlist: {e}")}
                                    </div>
                                })
                            }
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
