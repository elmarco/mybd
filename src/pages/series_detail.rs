use crate::models::UserPublic;
use crate::server::series::{
    get_or_create_series, get_series, get_series_albums, get_series_albums_for_user,
    get_series_authors, set_all_albums_owned, toggle_album_owned,
};
use leptos::either::{Either, EitherOf3, EitherOf4};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

#[component]
pub fn SeriesDetailPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();

    let owner = Memo::new(move |_| {
        let o = query.get().get("owner").unwrap_or_default();
        if o.is_empty() { None } else { Some(o) }
    });

    let series = Resource::new(
        move || params.read().get("slug"),
        |slug| async move {
            match slug {
                Some(slug) => get_series(slug).await,
                None => Ok(None),
            }
        },
    );

    view! {
        <div class="max-w-7xl mx-auto px-4 py-8">
            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    match series.await {
                        Ok(Some(s)) => {
                            let series_id = s.id;
                            let series_terminated = s.is_terminated.unwrap_or(false);
                            let owner_val = owner.get();
                            let is_logged_in = user.get().and_then(|r| r.ok()).flatten().is_some();

                            // Determine album display mode:
                            // - owner=Some(username) → read-only view of that user's ownership
                            // - logged in, no owner → interactive toggle (current user)
                            // - not logged in, no owner → plain list, no checkmarks
                            let albums = Resource::new(
                                || (),
                                move |_| {
                                    let owner_val = owner_val.clone();
                                    async move {
                                        match &owner_val {
                                            Some(username) => get_series_albums_for_user(series_id, username.clone()).await,
                                            None => get_series_albums(series_id).await,
                                        }
                                    }
                                },
                            );

                            let section_title = StoredValue::new(match &owner.get() {
                                Some(username) => format!("Albums from @{username}"),
                                None => "Albums".to_string(),
                            });
                            let show_checkmarks = owner.get().is_some() || is_logged_in;

                            EitherOf3::A(view! {
                                <div class="flex flex-col md:flex-row gap-8">
                                    // Cover
                                    <div class="w-full md:w-64 flex-shrink-0">
                                        {s.cover_url.clone().map(|url| view! {
                                            <img src=url alt=s.title.clone() class="w-full rounded-xl shadow-md"/>
                                        })}
                                    </div>

                                    // Details
                                    <div class="flex-1">
                                        <div class="flex items-start justify-between gap-2 mt-1">
                                            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">{s.title.clone()}</h1>
                                            <a
                                                href=format!("https://github.com/elmarco/mybd/edit/main/data/series/{}.toml", s.slug)
                                                target="_blank"
                                                rel="noopener"
                                                class="flex-shrink-0 p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                                                title="Edit data on GitHub"
                                            >
                                                <span class="material-symbols-outlined text-xl">"edit"</span>
                                            </a>
                                        </div>
                                        {
                                            let sid = series_id;
                                            let fallback_author = s.author.clone();
                                            let series_authors = Resource::new(
                                                move || sid,
                                                |sid| async move { get_series_authors(sid).await },
                                            );
                                            view! {
                                                <Suspense fallback=move || view! {
                                                    <p class="text-lg text-gray-600 dark:text-gray-400 mt-1">{fallback_author.clone()}</p>
                                                }>
                                                    {move || Suspend::new(async move {
                                                        match series_authors.await {
                                                            Ok(authors) if !authors.is_empty() => {
                                                                Either::Left(view! {
                                                                    <div class="flex flex-wrap gap-2 mt-2">
                                                                        {authors.into_iter().map(|author| {
                                                                            let href = format!(
                                                                                "/author/{}",
                                                                                urlencoding::encode(&author.slug)
                                                                            );
                                                                            view! {
                                                                                <a href=href class="inline-flex items-center gap-1 px-2.5 py-1 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-full text-xs text-gray-700 dark:text-gray-200 transition-colors">
                                                                                    {author.display_name}
                                                                                    {author.role.map(|r| view! {
                                                                                        <span class="text-gray-400">" · "{r}</span>
                                                                                    })}
                                                                                </a>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                })
                                                            }
                                                            _ => Either::Right(()),
                                                        }
                                                    })}
                                                </Suspense>
                                            }
                                        }

                                        <div class="flex flex-wrap gap-4 mt-4 text-sm text-gray-500 dark:text-gray-400">
                                            {s.year.map(|y| view! { <span>"Year: "{y}</span> })}
                                            {s.number_of_albums.map(|n| view! { <span>{n}" albums"</span> })}
                                        </div>

                                        {s.description.clone().map(|desc| view! {
                                            <div class="mt-6 prose prose-gray max-w-none">
                                                <p class="text-gray-700 dark:text-gray-300">{desc}</p>
                                            </div>
                                        })}
                                    </div>
                                </div>

                                // Albums list
                                <section class="mt-10">
                                    <Suspense fallback=|| view! { <p class="text-gray-500">"Loading albums..."</p> }>
                                        {move || Suspend::new(async move {
                                            match albums.await {
                                                Ok(album_list) if album_list.is_empty() => {
                                                    EitherOf3::A(view! {
                                                        <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4">{section_title.get_value()}</h2>
                                                        <p class="text-gray-500">"No albums found."</p>
                                                    })
                                                }
                                                Ok(album_list) => {
                                                    let read_only = owner.get().is_some();
                                                    let interactive = show_checkmarks && !read_only;

                                                    // Collect owned signals for select-all
                                                    let owned_signals: Vec<RwSignal<bool>> =
                                                        album_list.iter().map(|a| RwSignal::new(a.owned)).collect();
                                                    let owned_signals_stored = StoredValue::new(owned_signals.clone());

                                                    let all_owned = Memo::new(move |_| {
                                                        owned_signals_stored.with_value(|sigs| sigs.iter().all(|s| s.get()))
                                                    });
                                                    let toggling_all = RwSignal::new(false);

                                                    let header = if interactive {
                                                        Either::Left(view! {
                                                            <div class="flex items-center gap-3 mb-4">
                                                                <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">{section_title.get_value()}</h2>
                                                                <button
                                                                    class="flex items-center gap-1.5 px-2 py-1 rounded text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors cursor-pointer"
                                                                    prop:disabled=move || toggling_all.get()
                                                                    on:click=move |_| {
                                                                        let new_state = !all_owned.get();
                                                                        toggling_all.set(true);
                                                                        let album_count = expect_context::<Resource<Result<i64, ServerFnError>>>();
                                                                        leptos::task::spawn_local(async move {
                                                                            if set_all_albums_owned(series_id, new_state).await.is_ok() {
                                                                                let delta = owned_signals_stored.with_value(|sigs| {
                                                                                    let changing = sigs.iter().filter(|s| s.get_untracked() != new_state).count() as i64;
                                                                                    for s in sigs {
                                                                                        s.set(new_state);
                                                                                    }
                                                                                    if new_state { changing } else { -changing }
                                                                                });
                                                                                if let Some(d) = use_context::<RwSignal<Option<i64>>>() {
                                                                                    d.update(|n| *n = n.map(|c| c + delta));
                                                                                }
                                                                                album_count.refetch();
                                                                            }
                                                                            toggling_all.set(false);
                                                                        });
                                                                    }
                                                                >
                                                                    <div
                                                                        class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors"
                                                                        class:bg-green-500=move || all_owned.get()
                                                                        class:border-green-500=move || all_owned.get()
                                                                        class:border-gray-300=move || !all_owned.get()
                                                                    >
                                                                        <span
                                                                            class="material-symbols-outlined text-white transition-opacity"
                                                                            class:opacity-100=move || all_owned.get()
                                                                            class:opacity-0=move || !all_owned.get()
                                                                            style="font-size: 16px;"
                                                                        >"check"</span>
                                                                    </div>
                                                                    <span class:text-gray-400=move || toggling_all.get()>
                                                                        {move || if all_owned.get() { "Unselect all" } else { "Select all" }}
                                                                    </span>
                                                                </button>
                                                            </div>
                                                        })
                                                    } else {
                                                        Either::Right(view! {
                                                            <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4">{section_title.get_value()}</h2>
                                                        })
                                                    };

                                                    EitherOf3::B(view! {
                                                        {header}
                                                        <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-x-6">
                                                            {album_list.into_iter().enumerate().map(|(i, album)| {
                                                                render_album_row(album, show_checkmarks, read_only, owned_signals[i])
                                                            }).collect_view()}
                                                            {(!series_terminated).then(|| view! {
                                                                <div class="flex items-center gap-3 py-2 px-2 text-gray-400 dark:text-gray-500 italic">
                                                                    <div class="flex-shrink-0 w-12 h-16 rounded overflow-hidden bg-gray-100 dark:bg-gray-700 flex items-center justify-center">
                                                                        <span class="material-symbols-outlined text-gray-300 dark:text-gray-600">"more_horiz"</span>
                                                                    </div>
                                                                    <span class="text-sm font-medium w-10 text-right">"\u{2014}"</span>
                                                                    <span class="text-sm">"To be continued\u{2026}"</span>
                                                                </div>
                                                            })}
                                                        </div>
                                                    })
                                                }
                                                Err(e) => {
                                                    EitherOf3::C(view! {
                                                        <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                                            {format!("Error loading albums: {e}")}
                                                        </div>
                                                    })
                                                }
                                            }
                                        })}
                                    </Suspense>
                                </section>
                            })
                        }
                        Ok(None) => EitherOf3::B(view! {
                            <div class="text-center py-12">
                                <h2 class="text-2xl font-bold text-gray-300">"Series not found"</h2>
                                <a href="/collection" class="text-indigo-accent hover:underline mt-2 inline-block">"Back to Collection"</a>
                            </div>
                        }),
                        Err(e) => EitherOf3::C(view! {
                            <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                {format!("Error: {e}")}
                            </div>
                        }),
                    }
                })}
            </Suspense>
        </div>
    }
}

fn render_album_row(
    album: crate::models::AlbumWithOwnership,
    show_checkmarks: bool,
    read_only: bool,
    owned: RwSignal<bool>,
) -> impl IntoView {
    let album_id = album.id;
    let album_href = format!("/album/{}", album.slug);
    let toggling = RwSignal::new(false);
    let wishlisted = RwSignal::new(album.wishlisted);
    let for_sale_price = RwSignal::new(album.for_sale_price);
    let tome_display = album
        .tome
        .map(|t| format!("T{t}"))
        .unwrap_or_else(|| "\u{2014}".to_string());
    let title_display = album.title.clone().unwrap_or_default();
    let cover_url = album.cover_url.clone();

    if album.borrowed && !read_only {
        // Borrowed album — blue badge, not toggleable
        let href = album_href;
        EitherOf4::A(view! {
            <a data-nav-item="" href=href class="flex items-center gap-3 py-2 px-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded">
                <div class="relative flex-shrink-0 w-12 h-16 rounded overflow-hidden bg-gray-100">
                    {cover_url.map(|url| view! {
                        <img src=url class="w-full h-full object-cover"/>
                    })}
                    <div class="absolute -top-0.5 -right-0.5 w-5 h-5 rounded-full bg-blue-500 text-white flex items-center justify-center">
                        <span class="material-symbols-outlined" style="font-size: 14px;">"book"</span>
                    </div>
                </div>
                <span class="text-sm font-medium text-gray-500 w-10 text-right">{tome_display}</span>
                <span class="text-sm text-gray-900 dark:text-gray-100 flex-1">{title_display}</span>
                {move || for_sale_price.get().map(|p| view! {
                    <span class="text-xs text-red-500 font-medium">{format!("{p:.2}€")}</span>
                })}
            </a>
        })
    } else if show_checkmarks && !read_only {
        // Interactive: logged-in user's own albums
        let href = album_href;
        let is_lent = album.lent;
        EitherOf4::B(view! {
            <a data-nav-item="" data-album-id=album_id.to_string() href=href class="flex items-center gap-3 py-2 px-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded">
                <button
                    class="relative flex-shrink-0 w-12 h-16 rounded overflow-hidden cursor-pointer border-0 p-0 bg-gray-100"
                    prop:disabled=move || toggling.get()
                    on:click=move |ev| {
                        ev.prevent_default();
                        ev.stop_propagation();
                        toggling.set(true);
                        let album_count = expect_context::<Resource<Result<i64, ServerFnError>>>();
                        leptos::task::spawn_local(async move {
                            if let Ok(new_state) = toggle_album_owned(album_id).await {
                                owned.set(new_state);
                                if new_state {
                                    wishlisted.set(false);
                                }
                                if let Some(d) = use_context::<RwSignal<Option<i64>>>() {
                                    d.update(|n| *n = n.map(|c| c + if new_state { 1 } else { -1 }));
                                }
                                album_count.refetch();
                            }
                            toggling.set(false);
                        });
                    }
                >
                    {cover_url.map(|url| view! {
                        <img src=url class="w-full h-full object-cover"/>
                    })}
                    // Ownership/wishlist indicator
                    <Show when=move || !wishlisted.get() || owned.get()>
                        <div
                            class="absolute -top-0.5 -right-0.5 w-5 h-5 rounded-full flex items-center justify-center transition-colors"
                            class:bg-green-500=move || owned.get()
                            class:text-white=move || owned.get()
                            class:bg-gray-200=move || !owned.get()
                            class:text-gray-400=move || !owned.get()
                        >
                            <span class="material-symbols-outlined" style="font-size: 14px;">"check"</span>
                        </div>
                    </Show>
                    // Wishlist star (when not owned and wishlisted)
                    <Show when=move || !owned.get() && wishlisted.get()>
                        <div class="absolute -top-0.5 -right-0.5 w-5 h-5 rounded-full bg-amber-500 text-white flex items-center justify-center">
                            <span class="material-symbols-outlined" style="font-size: 14px;">"star"</span>
                        </div>
                    </Show>
                    {is_lent.then(|| view! {
                        <div class="absolute -bottom-0.5 -right-0.5 w-4 h-4 rounded-full bg-amber-500 text-white flex items-center justify-center">
                            <span class="material-symbols-outlined" style="font-size: 10px;">"arrow_forward"</span>
                        </div>
                    })}
                </button>
                <span class="text-sm font-medium text-gray-500 w-10 text-right">{tome_display}</span>
                <span class="text-sm text-gray-900 dark:text-gray-100 flex-1">{title_display}</span>
                {move || for_sale_price.get().map(|p| view! {
                    <span class="text-xs text-red-500 font-medium">{format!("{p:.2}€")}</span>
                })}
            </a>
        })
    } else if show_checkmarks && read_only {
        // Read-only: viewing another user's ownership
        let href = album_href;
        EitherOf4::C(view! {
            <a data-nav-item="" href=href class="flex items-center gap-3 py-2 px-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded">
                <div class="relative flex-shrink-0 w-12 h-16 rounded overflow-hidden bg-gray-100">
                    {cover_url.map(|url| view! {
                        <img src=url class="w-full h-full object-cover"/>
                    })}
                    <div
                        class="absolute -top-0.5 -right-0.5 w-5 h-5 rounded-full flex items-center justify-center"
                        class:bg-green-500=move || owned.get()
                        class:text-white=move || owned.get()
                        class:bg-gray-200=move || !owned.get()
                        class:text-gray-400=move || !owned.get()
                    >
                        <span class="material-symbols-outlined" style="font-size: 14px;">"check"</span>
                    </div>
                    // Wishlist star for read-only view
                    <Show when=move || !owned.get() && wishlisted.get()>
                        <div class="absolute -top-0.5 -right-0.5 w-5 h-5 rounded-full bg-amber-500 text-white flex items-center justify-center">
                            <span class="material-symbols-outlined" style="font-size: 14px;">"star"</span>
                        </div>
                    </Show>
                </div>
                <span class="text-sm font-medium text-gray-500 w-10 text-right">{tome_display}</span>
                <span class="text-sm text-gray-900 dark:text-gray-100 flex-1">{title_display}</span>
                {move || for_sale_price.get().map(|p| view! {
                    <span class="text-xs text-red-500 font-medium">{format!("{p:.2}€")}</span>
                })}
            </a>
        })
    } else {
        // Plain: not logged in, no owner context
        let href = album_href;
        EitherOf4::D(view! {
            <a data-nav-item="" href=href class="flex items-center gap-3 py-2 px-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded">
                <div class="flex-shrink-0 w-12 h-16 rounded overflow-hidden bg-gray-100">
                    {cover_url.map(|url| view! {
                        <img src=url class="w-full h-full object-cover"/>
                    })}
                </div>
                <span class="text-sm font-medium text-gray-500 w-10 text-right">{tome_display}</span>
                <span class="text-sm text-gray-900 dark:text-gray-100 flex-1">{title_display}</span>
            </a>
        })
    }
}

/// Resolves an external series (fetches from API, saves to DB) then redirects to /series/:id.
#[component]
pub fn ExternalSeriesPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    let resolver = Resource::new(
        move || params.read().get("bubble_id").unwrap_or_default(),
        move |bubble_id| async move {
            if bubble_id.is_empty() {
                return Err(ServerFnError::new("Missing bubble_id".to_string()));
            }
            get_or_create_series(bubble_id).await
        },
    );

    Effect::new(move || {
        if let Some(Ok(series)) = resolver.get() {
            navigate(&format!("/series/{}", series.slug), Default::default());
        }
    });

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    match resolver.await {
                        Ok(_) => Either::Left(view! {
                            <p class="text-gray-500">"Redirecting..."</p>
                        }),
                        Err(e) => Either::Right(view! {
                            <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                {format!("Error loading series: {e}")}
                            </div>
                        }),
                    }
                })}
            </Suspense>
        </div>
    }
}
