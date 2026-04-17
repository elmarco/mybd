use std::collections::HashSet;

use crate::components::SeriesCard;
use crate::server::metadata::{get_author_by_slug, search_series_all};
use crate::server::series::search_series_by_author;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn AuthorPage() -> impl IntoView {
    let params = use_params_map();

    let slug = Memo::new(move |_| params.read().get("slug").unwrap_or_default());

    let author = Resource::new(
        move || slug.get(),
        |slug| async move {
            if slug.is_empty() {
                return Ok(None);
            }
            get_author_by_slug(slug).await
        },
    );

    let owned_series = Resource::new(
        move || slug.get(),
        |slug| async move {
            if slug.trim().is_empty() {
                return Ok(vec![]);
            }
            search_series_by_author(slug).await
        },
    );

    let external_series = Resource::new(
        move || {
            author
                .get()
                .and_then(|r| r.ok().flatten())
                .map(|a| a.display_name)
        },
        |name| async move {
            match name {
                Some(n) if !n.trim().is_empty() => search_series_all(n).await,
                _ => Ok(vec![]),
            }
        },
    );

    view! {
        <div class="max-w-6xl mx-auto px-4 py-8">
            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    let info = author.await;

                    let author_data = match info {
                        Ok(Some(a)) => a,
                        Ok(None) => {
                            return Either::Left(view! {
                                <p class="text-yellow-700">{"Author not found.".to_string()}</p>
                            });
                        }
                        Err(e) => {
                            return Either::Left(view! {
                                <p class="text-red-700">{format!("Error: {e}")}</p>
                            });
                        }
                    };

                    let years = match (&author_data.date_birth, &author_data.date_death) {
                        (Some(b), Some(d)) => Some(format!("{b} – {d}")),
                        (Some(b), None) => Some(format!("{b} –")),
                        _ => None,
                    };

                    Either::Right(view! {
                        <div>
                            <div class="mb-8">
                                <div class="flex items-start justify-between gap-2">
                                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">{author_data.display_name}</h1>
                                    <a
                                        href=format!("https://github.com/elmarco/mybd/edit/main/data/authors/{}.toml", author_data.slug)
                                        target="_blank"
                                        rel="noopener"
                                        class="flex-shrink-0 p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                                        title="Edit data on GitHub"
                                    >
                                        <span class="material-symbols-outlined text-xl">"edit"</span>
                                    </a>
                                </div>
                                {years.map(|y| view! {
                                    <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{y}</p>
                                })}
                            </div>
                            <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4">"Series"</h2>
                            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading series..."</p> }>
                                {move || Suspend::new(async move {
                                    let owned_data = owned_series.await;
                                    let ext_data = external_series.await;

                                    let mut items: Vec<(crate::models::Series, Option<(i64, i64)>)> = Vec::new();
                                    let mut seen: HashSet<String> = HashSet::new();

                                    if let Ok(collection) = owned_data {
                                        for swo in collection {
                                            if let Some(bid) = &swo.bubble_id {
                                                seen.insert(bid.clone());
                                            }
                                            let ownership = (swo.owned_count, swo.total_albums);
                                            items.push((swo.into(), Some(ownership)));
                                        }
                                    }

                                    if let Ok(external) = ext_data {
                                        for s in external {
                                            let dominated = s.bubble_id.as_ref()
                                                .is_some_and(|id| seen.contains(id));
                                            if !dominated {
                                                items.push((s, None));
                                            }
                                        }
                                    }

                                    if items.is_empty() {
                                        Either::Left(view! {
                                            <p class="text-gray-500">"No series found."</p>
                                        })
                                    } else {
                                        Either::Right(view! {
                                            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                                                {items.into_iter().map(|(series, ownership)| {
                                                    let is_terminated = series.is_terminated.unwrap_or(false);
                                                    view! { <SeriesCard series=series ownership=ownership is_terminated=is_terminated/> }
                                                }).collect_view()}
                                            </div>
                                        })
                                    }
                                })}
                            </Suspense>
                        </div>
                    })
                })}
            </Suspense>
        </div>
    }
}
