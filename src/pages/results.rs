use crate::components::{Avatar, SeriesCard};
use crate::server::metadata::{search_authors_api, search_series};
use crate::server::series::search_user_collection;
use crate::server::social::search_users;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use std::collections::HashSet;

/// Global keydown listener for tab switching (1/2/3 keys).
fn use_tab_keynav(set_active_tab: WriteSignal<Tab>) {
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
            "1" => set_active_tab.set(Tab::Series),
            "2" => set_active_tab.set(Tab::Authors),
            "3" => set_active_tab.set(Tab::Users),
            _ => {}
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Series,
    Authors,
    Users,
}

#[derive(Clone, Copy)]
pub struct ResultsTabSetter(pub RwSignal<Option<WriteSignal<Tab>>>);

#[component]
pub fn ResultPage() -> impl IntoView {
    let params = use_query_map();
    let query = Memo::new(move |_| params.get().get("q").unwrap_or_default());
    let (active_tab, set_active_tab) = signal(Tab::Series);
    use_tab_keynav(set_active_tab);
    let tab_setter = expect_context::<ResultsTabSetter>();
    tab_setter.0.set(Some(set_active_tab));
    on_cleanup(move || tab_setter.0.set(None));

    // Pagination state for Series tab
    let (current_page, set_current_page) = signal(0usize);
    let (show_all, set_show_all) = signal(false);

    // Reset pagination when query changes
    Effect::new(move || {
        let _ = query.get();
        set_current_page.set(0);
        set_show_all.set(false);
    });

    // Single resource combining all search data — ensures Suspense tracks
    // refetches and avoids timing races between independent Resources.
    let search_data = Resource::new(
        move || query.get(),
        move |q| async move {
            if q.trim().is_empty() {
                return None;
            }
            let c = search_user_collection(q.clone()).await;
            let e = search_series(q.clone()).await;
            let a = search_authors_api(q.clone()).await;
            let u = search_users(q).await;
            Some((c, e, a, u))
        },
    );

    view! {
        <div class="max-w-6xl mx-auto px-4 py-8">
            <Suspense fallback=|| view! { <p class="text-gray-500">"Searching..."</p> }>
                {move || Suspend::new(async move {
                    let Some((collection_data, external_data, author_data, user_data)) = search_data.await else {
                        return Either::Left(view! { <div></div> });
                    };

                    // Merge series: collection items first, then external not already present
                    let mut items: Vec<(crate::models::Series, Option<(i64, i64)>)> = Vec::new();
                    let mut seen_bubble_ids: HashSet<String> = HashSet::new();
                    let mut search_error: Option<String> = None;

                    if let Ok(collection) = collection_data {
                        for swo in collection {
                            if let Some(bid) = &swo.bubble_id {
                                seen_bubble_ids.insert(bid.clone());
                            }
                            let ownership = (swo.owned_count, swo.total_albums);
                            items.push((swo.into(), Some(ownership)));
                        }
                    }

                    match external_data {
                        Ok(external) => {
                            for s in external {
                                let dominated = s
                                    .bubble_id
                                    .as_ref()
                                    .is_some_and(|id| seen_bubble_ids.contains(id));
                                if !dominated {
                                    items.push((s, None));
                                }
                            }
                        }
                        Err(e) => {
                            search_error = Some(format!("Search error: {e}"));
                        }
                    }

                    let authors = author_data.unwrap_or_default();
                    let users = user_data.unwrap_or_default();

                    let series_count = items.len();
                    let authors_count = authors.len();
                    let users_count = users.len();
                    let items_empty = items.is_empty();
                    let has_search_error = search_error.is_some();

                    Either::Right(view! {
                        // Tab bar
                        <div class="border-b border-gray-200 dark:border-gray-700 mb-6">
                            <nav class="flex gap-8 -mb-px" role="tablist">
                                <button
                                    role="tab"
                                    class="cursor-pointer px-1 py-3 text-sm font-medium border-b-2 transition-colors hover:text-indigo-accent-dark"
                                    class=("text-indigo-accent", move || active_tab.get() == Tab::Series)
                                    class=("border-indigo-accent", move || active_tab.get() == Tab::Series)
                                    class=("text-gray-500 dark:text-gray-200", move || active_tab.get() != Tab::Series)
                                    class=("border-transparent", move || active_tab.get() != Tab::Series)
                                    on:click=move |_| set_active_tab.set(Tab::Series)
                                >
                                    "Series"
                                    <span
                                        class="ml-2 text-xs font-semibold px-2.5 py-0.5 rounded-full inline-block min-w-5 text-center"
                                        class=("bg-indigo-accent", move || active_tab.get() == Tab::Series)
                                        class=("text-white", move || active_tab.get() == Tab::Series)
                                        class=("bg-gray-200 dark:bg-gray-600", move || active_tab.get() != Tab::Series)
                                        class=("text-gray-600 dark:text-gray-200", move || active_tab.get() != Tab::Series)
                                    >
                                        {series_count}
                                    </span>
                                </button>
                                <button
                                    role="tab"
                                    class="cursor-pointer px-1 py-3 text-sm font-medium border-b-2 transition-colors hover:text-indigo-accent-dark"
                                    class=("text-indigo-accent", move || active_tab.get() == Tab::Authors)
                                    class=("border-indigo-accent", move || active_tab.get() == Tab::Authors)
                                    class=("text-gray-500 dark:text-gray-200", move || active_tab.get() != Tab::Authors)
                                    class=("border-transparent", move || active_tab.get() != Tab::Authors)
                                    on:click=move |_| set_active_tab.set(Tab::Authors)
                                >
                                    "Authors"
                                    <span
                                        class="ml-2 text-xs font-semibold px-2.5 py-0.5 rounded-full inline-block min-w-5 text-center"
                                        class=("bg-indigo-accent", move || active_tab.get() == Tab::Authors)
                                        class=("text-white", move || active_tab.get() == Tab::Authors)
                                        class=("bg-gray-200 dark:bg-gray-600", move || active_tab.get() != Tab::Authors)
                                        class=("text-gray-600 dark:text-gray-200", move || active_tab.get() != Tab::Authors)
                                    >
                                        {authors_count}
                                    </span>
                                </button>
                                <button
                                    role="tab"
                                    class="cursor-pointer px-1 py-3 text-sm font-medium border-b-2 transition-colors hover:text-indigo-accent-dark"
                                    class=("text-indigo-accent", move || active_tab.get() == Tab::Users)
                                    class=("border-indigo-accent", move || active_tab.get() == Tab::Users)
                                    class=("text-gray-500 dark:text-gray-200", move || active_tab.get() != Tab::Users)
                                    class=("border-transparent", move || active_tab.get() != Tab::Users)
                                    on:click=move |_| set_active_tab.set(Tab::Users)
                                >
                                    "Users"
                                    <span
                                        class="ml-2 text-xs font-semibold px-2.5 py-0.5 rounded-full inline-block min-w-5 text-center"
                                        class=("bg-indigo-accent", move || active_tab.get() == Tab::Users)
                                        class=("text-white", move || active_tab.get() == Tab::Users)
                                        class=("bg-gray-200 dark:bg-gray-600", move || active_tab.get() != Tab::Users)
                                        class=("text-gray-600 dark:text-gray-200", move || active_tab.get() != Tab::Users)
                                    >
                                        {users_count}
                                    </span>
                                </button>
                            </nav>
                        </div>

                        // Series panel
                        <div role="tabpanel" class=("hidden", move || active_tab.get() != Tab::Series)>
                            {search_error.map(|err| view! {
                                <div class="mb-4 bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                    {err}
                                </div>
                            })}
                            {if !items_empty {
                                let page_size = 20usize;
                                let total_items = items.len();
                                let total_pages = total_items.div_ceil(page_size);
                                let needs_pagination = total_items > page_size;
                                let items_signal = RwSignal::new(items);

                                view! {
                                    {move || {
                                        let all = items_signal.get();
                                        let visible: Vec<_> = if show_all.get() {
                                            all
                                        } else {
                                            let start = current_page.get() * page_size;
                                            let end = (start + page_size).min(all.len());
                                            all[start..end].to_vec()
                                        };
                                        view! {
                                            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                                                {visible.into_iter().map(|(series, ownership)| {
                                                    let is_terminated = series.is_terminated.unwrap_or(false);
                                                    view! { <SeriesCard series=series ownership=ownership is_terminated=is_terminated/> }
                                                }).collect_view()}
                                            </div>
                                        }
                                    }}
                                    {needs_pagination.then(|| view! {
                                        <div class="flex items-center justify-center gap-4 mt-6">
                                            <Show when=move || !show_all.get()>
                                                <button
                                                    class="px-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 cursor-pointer"
                                                    disabled=move || current_page.get() == 0
                                                    on:click=move |_| set_current_page.update(|p| *p = p.saturating_sub(1))
                                                >
                                                    "Previous"
                                                </button>
                                                <span class="text-sm text-gray-600 dark:text-gray-400">
                                                    {move || format!("{} / {}", current_page.get() + 1, total_pages)}
                                                </span>
                                                <button
                                                    class="px-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 cursor-pointer"
                                                    disabled=move || current_page.get() + 1 >= total_pages
                                                    on:click=move |_| set_current_page.update(|p| *p += 1)
                                                >
                                                    "Next"
                                                </button>
                                            </Show>
                                            <button
                                                class="px-4 py-2 text-sm text-indigo-accent hover:underline cursor-pointer"
                                                on:click=move |_| set_show_all.update(|v| *v = !*v)
                                            >
                                                {move || if show_all.get() { "Show pages" } else { "Show all" }}
                                            </button>
                                        </div>
                                    })}
                                }.into_any()
                            } else if !has_search_error {
                                view! {
                                    <p class="text-gray-500">"No series found."</p>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                        </div>

                        // Authors panel
                        <div role="tabpanel" class=("hidden", move || active_tab.get() != Tab::Authors)>
                            {if authors.is_empty() {
                                Either::Left(view! {
                                    <p class="text-gray-500">"No authors found."</p>
                                })
                            } else {
                                Either::Right(view! {
                                    <div class="flex flex-wrap gap-3">
                                        {authors.into_iter().map(|a| {
                                            let slug = crate::server::slug::slugify(&a.display_name);
                                            let href = format!(
                                                "/author/{}",
                                                urlencoding::encode(&slug)
                                            );
                                            let years = match (&a.year_of_birth, &a.year_of_death) {
                                                (Some(b), Some(d)) => format!(" ({b}\u{2013}{d})"),
                                                (Some(b), None) => format!(" ({b}\u{2013})"),
                                                _ => String::new(),
                                            };
                                            view! {
                                                <a href=href data-nav-item=""
                                                   class="flex items-center gap-3 px-4 py-3 bg-white dark:bg-gray-800 rounded-xl shadow-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
                                                    {a.image_url.clone().map(|url| view! {
                                                        <img src=url alt=a.display_name.clone() class="w-10 h-10 rounded-full object-cover flex-shrink-0"/>
                                                    })}
                                                    <div>
                                                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{a.display_name}</span>
                                                        <span class="text-xs text-gray-500 dark:text-gray-400">{years}</span>
                                                    </div>
                                                </a>
                                            }
                                        }).collect_view()}
                                    </div>
                                })
                            }}
                        </div>

                        // Users panel
                        <div role="tabpanel" class=("hidden", move || active_tab.get() != Tab::Users)>
                            {if users.is_empty() {
                                Either::Left(view! {
                                    <p class="text-gray-500">"No users found."</p>
                                })
                            } else {
                                Either::Right(view! {
                                    <div class="flex flex-wrap gap-3">
                                        {users.into_iter().map(|u| {
                                            let href = format!("/profile/{}", urlencoding::encode(&u.username));
                                            view! {
                                                <a href=href data-nav-item=""
                                                   class="flex items-center gap-3 px-4 py-3 bg-white dark:bg-gray-800 rounded-xl shadow-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
                                                    <Avatar url=u.avatar_url.clone() name=u.display_name.clone() size="w-8 h-8" text_size="text-xs"/>
                                                    <div>
                                                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{u.display_name}</span>
                                                        <span class="text-xs text-gray-500 dark:text-gray-400">" @"{u.username}</span>
                                                    </div>
                                                </a>
                                            }
                                        }).collect_view()}
                                    </div>
                                })
                            }}
                        </div>
                    })
                })}
            </Suspense>
        </div>
    }
}
