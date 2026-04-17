use crate::models::UserPublic;
use crate::server::series::{
    get_album_detail, get_series_albums, set_album_for_sale, toggle_album_owned,
    toggle_album_wishlisted,
};
use crate::server::social::{get_following, get_loan_id, lend_album, return_album};
use crate::utils::format_date;
use leptos::either::{EitherOf3, EitherOf4};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn AlbumDetailPage() -> impl IntoView {
    let params = use_params_map();
    let user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();
    let album = Resource::new(
        move || params.read().get("slug"),
        |slug| async move {
            match slug {
                Some(slug) => get_album_detail(slug).await,
                None => Ok(None),
            }
        },
    );

    // Stable series_id signal: only updates when we have a resolved album,
    // avoids the None→Some cascade that causes series_albums to refetch twice.
    let series_id = RwSignal::new(None::<i64>);
    Effect::new(move || {
        if let Some(Ok(Some(a))) = album.get() {
            series_id.set(Some(a.series_id));
        }
    });

    // Fetch all albums in the series for navigation
    let series_albums = Resource::new(
        move || series_id.get(),
        |series_id| async move {
            match series_id {
                Some(sid) => get_series_albums(sid).await.ok(),
                None => None,
            }
        },
    );

    // Single persistent keyboard listener — reads reactive signals at keypress time,
    // not captured by value, so they're always current.
    #[cfg(feature = "hydrate")]
    {
        use leptos::ev::keydown;
        use leptos::prelude::window_event_listener;
        use leptos_router::hooks::use_navigate;

        let current_album_id =
            Memo::new(move |_| album.get().and_then(|r| r.ok()).flatten().map(|a| a.id));
        let nav_prev = Memo::new(move |_| {
            let cid = current_album_id.get()?;
            let albums = series_albums.get()??;
            let pos = albums.iter().position(|a| a.id == cid)?;
            (pos > 0).then(|| albums[pos - 1].slug.clone())
        });
        let nav_next = Memo::new(move |_| {
            let cid = current_album_id.get()?;
            let albums = series_albums.get()??;
            let pos = albums.iter().position(|a| a.id == cid)?;
            (pos < albums.len() - 1).then(|| albums[pos + 1].slug.clone())
        });

        let nav = use_navigate();
        let _handle = window_event_listener(keydown, move |ev| {
            // Skip when focus is on an input element
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
                "ArrowLeft" => {
                    if let Some(prev) = nav_prev.get_untracked() {
                        ev.prevent_default();
                        nav(&format!("/album/{prev}"), Default::default());
                    }
                }
                "ArrowRight" => {
                    if let Some(next) = nav_next.get_untracked() {
                        ev.prevent_default();
                        nav(&format!("/album/{next}"), Default::default());
                    }
                }
                _ => {}
            }
        });
    }

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8 relative">
            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    match album.await {
                        Ok(Some(a)) => {
                            let album_id = a.id;
                            let is_logged_in = move || user.get().and_then(|r| r.ok()).flatten().is_some();
                            let owned = RwSignal::new(a.owned);
                            let wishlisted = RwSignal::new(a.wishlisted);
                            let for_sale_price = RwSignal::new(a.for_sale_price);
                            let toggling = RwSignal::new(false);

                            let display_title = a.title.clone().unwrap_or_else(|| {
                                a.tome.map(|t| format!("Tome {t}")).unwrap_or_else(|| "Untitled".to_string())
                            });

                            // Reactively compute next/prev album slugs (series_albums loads after album)
                            let nav_ids = Memo::new(move |_| {
                                let albums_list = series_albums.get().and_then(|a| a);
                                if let Some(albums) = albums_list {
                                    let current_pos = albums.iter().position(|alb| alb.id == album_id);
                                    if let Some(pos) = current_pos {
                                        let prev = if pos > 0 {
                                            albums.get(pos - 1).map(|a| a.slug.clone())
                                        } else {
                                            None
                                        };
                                        let next = if pos < albums.len() - 1 {
                                            albums.get(pos + 1).map(|a| a.slug.clone())
                                        } else {
                                            None
                                        };
                                        return (prev, next);
                                    }
                                }
                                (None, None)
                            });

                            let lent_to_init = a.lent_to.clone();
                            let borrowed_from_init = a.borrowed_from.clone();

                            EitherOf3::A(view! {
                                <div class="flex items-start gap-2">
                                // Navigation: prev arrow
                                <div class="hidden md:flex self-center flex-shrink-0">
                                {move || nav_ids.get().0.map(|prev_slug| view! {
                                    <a
                                        href=format!("/album/{prev_slug}")
                                        class="flex items-center justify-center w-10 h-10 bg-white dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 rounded-full shadow-lg transition-colors cursor-pointer"
                                        aria-label="Previous album"
                                    >
                                        <span class="material-symbols-outlined text-gray-700 dark:text-gray-200">"chevron_left"</span>
                                    </a>
                                })}
                                </div>

                                // Content
                                <div class="flex-1 min-w-0 flex flex-col md:flex-row gap-8">
                                    // Cover
                                    <div class="w-full md:w-72 flex-shrink-0">
                                        {a.cover_url.clone().map(|url| view! {
                                            <img src=url alt=display_title.clone() class="w-full rounded-xl shadow-md"/>
                                        })}

                                        // Ownership toggle
                                        {move || is_logged_in().then(|| {
                                            let album_count = expect_context::<Resource<Result<i64, ServerFnError>>>();
                                            view! {
                                                <button
                                                    class="w-full mt-4 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-colors cursor-pointer"
                                                    class=("bg-green-500", move || owned.get())
                                                    class=("text-white", move || owned.get())
                                                    class=("hover:bg-green-600", move || owned.get())
                                                    class=("bg-gray-100 dark:bg-gray-700", move || !owned.get())
                                                    class=("text-gray-700 dark:text-gray-100", move || !owned.get())
                                                    class=("hover:bg-gray-200 dark:hover:bg-gray-600", move || !owned.get())
                                                    prop:disabled=move || toggling.get()
                                                    on:click=move |_| {
                                                        toggling.set(true);
                                                        leptos::task::spawn_local(async move {
                                                            if let Ok(new_state) = toggle_album_owned(album_id).await {
                                                                owned.set(new_state);
                                                                if new_state {
                                                                    wishlisted.set(false);
                                                                } else {
                                                                    for_sale_price.set(None);
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
                                                    <span class="material-symbols-outlined" style="font-size: 20px;">
                                                        {move || if owned.get() { "check_circle" } else { "add_circle_outline" }}
                                                    </span>
                                                    {move || if owned.get() { "In collection" } else { "Add to collection" }}
                                                </button>
                                                // Wishlist toggle (only when not owned)
                                                <Show when=move || !owned.get()>
                                                    <button
                                                        class="w-full mt-2 flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors cursor-pointer"
                                                        class=("bg-amber-50 dark:bg-amber-900/30", move || wishlisted.get())
                                                        class=("text-amber-700 dark:text-amber-400", move || wishlisted.get())
                                                        class=("border border-amber-300 dark:border-amber-700", move || wishlisted.get())
                                                        class=("bg-gray-50 dark:bg-gray-700", move || !wishlisted.get())
                                                        class=("text-gray-600 dark:text-gray-100", move || !wishlisted.get())
                                                        class=("border border-gray-200 dark:border-gray-600", move || !wishlisted.get())
                                                        on:click=move |_| {
                                                            leptos::task::spawn_local(async move {
                                                                if let Ok(new_state) = toggle_album_wishlisted(album_id).await {
                                                                    wishlisted.set(new_state);
                                                                }
                                                            });
                                                        }
                                                    >
                                                        <span class="material-symbols-outlined" style="font-size: 18px;">
                                                            {move || if wishlisted.get() { "star" } else { "star_outline" }}
                                                        </span>
                                                        {move || if wishlisted.get() { "On wishlist" } else { "Add to wishlist" }}
                                                    </button>
                                                </Show>
                                                // For-sale controls (only when owned)
                                                <Show when=move || owned.get()>
                                                    {
                                                        let editing_price = RwSignal::new(false);
                                                        let price_ref = NodeRef::<leptos::html::Input>::new();
                                                        Effect::new(move || {
                                                            if editing_price.get()
                                                                && let Some(el) = price_ref.get()
                                                            {
                                                                let _ = el.focus();
                                                            }
                                                        });
                                                        let price_input = RwSignal::new(
                                                            for_sale_price.get_untracked()
                                                                .map(|p| format!("{p:.2}"))
                                                                .unwrap_or_default()
                                                        );
                                                        view! {
                                                            <Show
                                                                when=move || for_sale_price.get().is_some() && !editing_price.get()
                                                                fallback=move || {
                                                                    view! {
                                                                        <Show when=move || editing_price.get()>
                                                                            <div class="w-full mt-2 flex items-center gap-2 px-4 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                                                                                <span class="text-sm text-red-600 dark:text-red-400 font-medium">"Price:"</span>
                                                                                <div class="flex items-center border border-gray-300 dark:border-gray-600 rounded-md overflow-hidden">
                                                                                    <input
                                                                                        type="number"
                                                                                        step="0.01"
                                                                                        min="0"
                                                                                        node_ref=price_ref
                                                                                        class="w-20 px-2 py-1 text-sm text-right border-none outline-none dark:bg-gray-700 dark:text-gray-100 dark:focus:bg-gray-100 dark:focus:text-gray-900"
                                                                                        prop:value=move || price_input.get()
                                                                                        on:input=move |ev| price_input.set(event_target_value(&ev))
                                                                                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                                                            match ev.key().as_str() {
                                                                                                "Enter" => {
                                                                                                    ev.prevent_default();
                                                                                                    let price_str = price_input.get();
                                                                                                    if let Ok(p) = price_str.parse::<f64>() {
                                                                                                        leptos::task::spawn_local(async move {
                                                                                                            if set_album_for_sale(album_id, Some(p)).await.is_ok() {
                                                                                                                for_sale_price.set(Some(p));
                                                                                                                editing_price.set(false);
                                                                                                            }
                                                                                                        });
                                                                                                    }
                                                                                                }
                                                                                                "Escape" => editing_price.set(false),
                                                                                                _ => {}
                                                                                            }
                                                                                        }
                                                                                    />
                                                                                    <span class="px-2 py-1 bg-gray-100 dark:bg-gray-600 text-gray-500 dark:text-gray-300 text-sm border-l border-gray-300 dark:border-gray-600">"€"</span>
                                                                                </div>
                                                                                <button
                                                                                    class="px-3 py-1 bg-red-500 text-white text-xs font-medium rounded-md cursor-pointer"
                                                                                    on:click=move |_| {
                                                                                        let price_str = price_input.get();
                                                                                        if let Ok(p) = price_str.parse::<f64>() {
                                                                                            leptos::task::spawn_local(async move {
                                                                                                if set_album_for_sale(album_id, Some(p)).await.is_ok() {
                                                                                                    for_sale_price.set(Some(p));
                                                                                                    editing_price.set(false);
                                                                                                }
                                                                                            });
                                                                                        }
                                                                                    }
                                                                                >"Save"</button>
                                                                                <button
                                                                                    class="px-3 py-1 bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-300 text-xs rounded-md cursor-pointer"
                                                                                    on:click=move |_| editing_price.set(false)
                                                                                >"Cancel"</button>
                                                                            </div>
                                                                        </Show>
                                                                        <Show when=move || !editing_price.get() && for_sale_price.get().is_none()>
                                                                            <button
                                                                                class="w-full mt-2 flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm text-gray-500 dark:text-gray-400 border border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors cursor-pointer"
                                                                                on:click=move |_| editing_price.set(true)
                                                                            >
                                                                                <span class="material-symbols-outlined" style="font-size: 18px;">"sell"</span>
                                                                                "Mark for sale"
                                                                            </button>
                                                                        </Show>
                                                                    }
                                                                }
                                                            >
                                                                // Currently for sale — show status
                                                                <div class="w-full mt-2 flex items-center gap-2 px-4 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                                                                    <span class="material-symbols-outlined text-red-500" style="font-size: 18px;">"sell"</span>
                                                                    <span class="text-sm text-red-600 dark:text-red-400 font-medium">
                                                                        {move || format!("For sale at {:.2}€", for_sale_price.get().unwrap_or(0.0))}
                                                                    </span>
                                                                    <button
                                                                        class="ml-auto px-2 py-1 bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded text-xs text-gray-500 dark:text-gray-300 cursor-pointer"
                                                                        on:click=move |_| {
                                                                            price_input.set(for_sale_price.get().map(|p| format!("{p:.2}")).unwrap_or_default());
                                                                            editing_price.set(true);
                                                                        }
                                                                    >"Edit"</button>
                                                                    <button
                                                                        class="px-2 py-1 bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded text-xs text-red-500 cursor-pointer"
                                                                        on:click=move |_| {
                                                                            leptos::task::spawn_local(async move {
                                                                                if set_album_for_sale(album_id, None).await.is_ok() {
                                                                                    for_sale_price.set(None);
                                                                                }
                                                                            });
                                                                        }
                                                                    >"Remove"</button>
                                                                </div>
                                                            </Show>
                                                        }
                                                    }
                                                </Show>
                                            }
                                        })}

                                        // Lending controls
                                        {move || is_logged_in().then(|| {
                                            let lent_to = RwSignal::new(lent_to_init.clone());
                                            let borrowed_from = borrowed_from_init.clone();
                                            let returning = RwSignal::new(false);
                                            let (lend_open, set_lend_open) = signal(false);
                                            let friends = Resource::new(
                                                move || lend_open.get(),
                                                move |open| async move {
                                                    if open { get_following().await } else { Ok(vec![]) }
                                                },
                                            );

                                            view! {
                                                {move || {
                                                    if let Some((_, ref lender_name)) = borrowed_from {
                                                        // Borrower view
                                                        EitherOf4::A(view! {
                                                            <div class="mt-3 flex items-center gap-2 px-4 py-2.5 bg-blue-50 rounded-lg text-sm text-blue-700">
                                                                <span class="material-symbols-outlined" style="font-size: 18px;">"book"</span>
                                                                "Borrowed from "{lender_name.clone()}
                                                            </div>
                                                        })
                                                    } else if let Some((_, ref borrower_name)) = lent_to.get() {
                                                        // Lender view with return button
                                                        let borrower_name = borrower_name.clone();
                                                        EitherOf4::B(view! {
                                                            <div class="mt-3 flex items-center gap-2 px-4 py-2.5 bg-amber-50 rounded-lg text-sm text-amber-700">
                                                                <span class="material-symbols-outlined" style="font-size: 18px;">"book"</span>
                                                                <span class="flex-1">"Lent to "{borrower_name}</span>
                                                                <button
                                                                    class="text-amber-600 hover:text-amber-800 cursor-pointer"
                                                                    prop:disabled=move || returning.get()
                                                                    on:click=move |_| {
                                                                        returning.set(true);
                                                                        let aid = album_id;
                                                                        leptos::task::spawn_local(async move {
                                                                            if let Ok(Some(loan_id)) = get_loan_id(aid).await
                                                                                && return_album(loan_id).await.is_ok()
                                                                            {
                                                                                lent_to.set(None);
                                                                            }
                                                                            returning.set(false);
                                                                        });
                                                                    }
                                                                >
                                                                    "Return"
                                                                </button>
                                                            </div>
                                                        })
                                                    } else if owned.get() {
                                                        // Owner view with lend button
                                                        EitherOf4::C(view! {
                                                            <div class="relative mt-3">
                                                                <button
                                                                    class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg text-sm text-gray-700 dark:text-gray-200 transition-colors cursor-pointer"
                                                                    on:click=move |_| set_lend_open.update(|v| *v = !*v)
                                                                >
                                                                    <span class="material-symbols-outlined" style="font-size: 18px;">"share"</span>
                                                                    "Lend to..."
                                                                </button>
                                                                <Show when=move || lend_open.get()>
                                                                    <div
                                                                        class="fixed inset-0 z-40"
                                                                        on:click=move |_| set_lend_open.set(false)
                                                                    />
                                                                    <div class="absolute left-0 right-0 top-full mt-1 bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 z-50 overflow-hidden max-h-60 overflow-y-auto">
                                                                        <Suspense fallback=|| view! { <p class="p-3 text-gray-500 text-sm">"Loading..."</p> }>
                                                                            {move || Suspend::new(async move {
                                                                                match friends.await {
                                                                                    Ok(fl) if fl.is_empty() => {
                                                                                        EitherOf3::A(view! {
                                                                                            <p class="p-3 text-gray-500 text-sm">"No friends to lend to"</p>
                                                                                        })
                                                                                    }
                                                                                    Ok(fl) => {
                                                                                        EitherOf3::B(view! {
                                                                                            <div class="divide-y divide-gray-100">
                                                                                                {fl.into_iter().map(|f| {
                                                                                                    let fid = f.id;
                                                                                                    let fname = f.display_name.clone();
                                                                                                    view! {
                                                                                                        <button
                                                                                                            class="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-gray-50 dark:hover:bg-gray-700 text-sm text-gray-700 dark:text-gray-200 cursor-pointer"
                                                                                                            on:click=move |_| {
                                                                                                                set_lend_open.set(false);
                                                                                                                let fname = fname.clone();
                                                                                                                leptos::task::spawn_local(async move {
                                                                                                                    if lend_album(album_id, fid).await.is_ok() {
                                                                                                                        lent_to.set(Some((fid, fname)));
                                                                                                                    }
                                                                                                                });
                                                                                                            }
                                                                                                        >
                                                                                                            {fname.clone()}
                                                                                                        </button>
                                                                                                    }
                                                                                                }).collect_view()}
                                                                                            </div>
                                                                                        })
                                                                                    }
                                                                                    Err(_) => EitherOf3::C(view! {
                                                                                        <p class="p-3 text-gray-500 text-sm">"Error loading friends"</p>
                                                                                    }),
                                                                                }
                                                                            })}
                                                                        </Suspense>
                                                                    </div>
                                                                </Show>
                                                            </div>
                                                        })
                                                    } else {
                                                        EitherOf4::D(())
                                                    }
                                                }}
                                            }
                                        })}
                                    </div>

                                    // Details
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-start justify-between gap-2">
                                            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">{display_title}</h1>
                                            <a
                                                href=format!("https://github.com/elmarco/mybd/edit/main/data/series/{}.toml", a.series_slug)
                                                target="_blank"
                                                rel="noopener"
                                                class="flex-shrink-0 p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                                                title="Edit data on GitHub"
                                            >
                                                <span class="material-symbols-outlined text-xl">"edit"</span>
                                            </a>
                                        </div>
                                        {a.tome.map(|t| view! {
                                            <p class="text-lg text-gray-500 dark:text-gray-400 mt-1">"Tome "{t}</p>
                                        })}
                                        <p class="text-lg text-gray-600 dark:text-gray-300 mt-1">
                                            <a href=format!("/series/{}", a.series_slug) class="hover:text-indigo-accent">
                                                {a.series_title}
                                            </a>
                                        </p>

                                        // Authors (from API)
                                        {(!a.authors.is_empty()).then(|| {
                                            let authors = a.authors.clone();
                                            view! {
                                                <div class="flex flex-wrap gap-2 mt-4">
                                                    {authors.into_iter().map(|author| {
                                                        let href = format!(
                                                            "/author/{}",
                                                            urlencoding::encode(&author.slug)
                                                        );
                                                        view! {
                                                            <a href=href class="inline-flex items-center gap-1 px-2.5 py-1 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-full text-xs text-gray-700 dark:text-gray-200 transition-colors">
                                                                {author.display_name}
                                                                {author.role.map(|r| view! {
                                                                    <span class="text-gray-400 dark:text-gray-500">" · "{r}</span>
                                                                })}
                                                            </a>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }
                                        })}

                                        // Metadata row
                                        <div class="flex flex-wrap gap-4 mt-4 text-sm text-gray-500 dark:text-gray-400">
                                            {a.publisher.map(|p| view! {
                                                <span class="flex items-center gap-1">
                                                    <span class="material-symbols-outlined text-gray-400 dark:text-gray-500" style="font-size: 16px;">"business"</span>
                                                    {p}
                                                </span>
                                            })}
                                            {a.number_of_pages.map(|n| view! {
                                                <span class="flex items-center gap-1">
                                                    <span class="material-symbols-outlined text-gray-400 dark:text-gray-500" style="font-size: 16px;">"description"</span>
                                                    {n}" pages"
                                                </span>
                                            })}
                                            {a.publication_date.clone().map(|d| view! {
                                                <span class="flex items-center gap-1">
                                                    <span class="material-symbols-outlined text-gray-400 dark:text-gray-500" style="font-size: 16px;">"calendar_today"</span>
                                                    {format_date(&d)}
                                                </span>
                                            })}
                                            {a.ean.map(|ean| view! {
                                                <span class="flex items-center gap-1">
                                                    <span class="material-symbols-outlined text-gray-400 dark:text-gray-500" style="font-size: 16px;">"barcode"</span>
                                                    {ean}
                                                </span>
                                            })}
                                        </div>

                                        // Summary
                                        {a.summary.map(|desc| view! {
                                            <div class="mt-6 prose prose-gray dark:prose-invert max-w-none">
                                                <p class="text-gray-700 dark:text-gray-300">{desc}</p>
                                            </div>
                                        })}

                                    </div>
                                </div>

                                // Navigation: next arrow
                                <div class="hidden md:flex self-center flex-shrink-0">
                                {move || nav_ids.get().1.map(|next_slug| view! {
                                    <a
                                        href=format!("/album/{next_slug}")
                                        class="flex items-center justify-center w-10 h-10 bg-white dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 rounded-full shadow-lg transition-colors cursor-pointer"
                                        aria-label="Next album"
                                    >
                                        <span class="material-symbols-outlined text-gray-700 dark:text-gray-200">"chevron_right"</span>
                                    </a>
                                })}
                                </div>
                                </div>
                            })
                        }
                        Ok(None) => EitherOf3::B(view! {
                            <div class="text-center py-12">
                                <h2 class="text-2xl font-bold text-gray-300">"Album not found"</h2>
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
