use crate::app::{FollowingCountDisplay, FollowingCountResource};
use crate::components::{Avatar, SeriesCard};
use crate::models::UserPublic;
use crate::server::profile::{
    get_public_collection, get_public_profile, get_public_profile_stats, get_public_wishlist,
};
use crate::server::social::{follow_user, is_following, unfollow_user};
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

fn share_button(display_name: String, username: String) -> impl IntoView {
    let copied = RwSignal::new(false);

    view! {
        <button
            class="ml-auto flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors cursor-pointer"
            on:click=move |_| {
                let display_name = display_name.clone();
                let username = username.clone();
                #[cfg(feature = "hydrate")]
                {
                    leptos::task::spawn_local(async move {
                        use wasm_bindgen::JsCast;

                        let window = web_sys::window().unwrap();
                        let url = format!("{}/profile/{}", window.location().origin().unwrap_or_default(), urlencoding::encode(&username));
                        let title = format!("{}'s collection on mybd", display_name);

                        // Try Web Share API first
                        let navigator = window.navigator();
                        let shared = {
                            let share_data = js_sys::Object::new();
                            let _ = js_sys::Reflect::set(&share_data, &"title".into(), &title.clone().into());
                            let _ = js_sys::Reflect::set(&share_data, &"url".into(), &url.clone().into());
                            if let Ok(promise) = js_sys::Reflect::get(&navigator, &"share".into()) {
                                if promise.is_function() {
                                    let share_fn: js_sys::Function = promise.unchecked_into();
                                    if let Ok(p) = share_fn.call1(&navigator, &share_data) {
                                        let promise: js_sys::Promise = p.unchecked_into();
                                        wasm_bindgen_futures::JsFuture::from(promise).await.is_ok()
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        if !shared {
                            // Fallback: copy to clipboard
                            if let Ok(clipboard) = js_sys::Reflect::get(&navigator, &"clipboard".into())
                                && !clipboard.is_undefined()
                            {
                                let clipboard: web_sys::Clipboard = clipboard.unchecked_into();
                                let promise = clipboard.write_text(&url);
                                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                                copied.set(true);
                                let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                                    copied.set(false);
                                });
                                let _ = web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                                    cb.as_ref().unchecked_ref(),
                                    2000,
                                );
                                cb.forget();
                            }
                        }
                    });
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    let _ = (display_name, username);
                }
            }
        >
            <span class="material-symbols-outlined" style="font-size: 18px;">
                {move || if copied.get() { "check" } else { "share" }}
            </span>
            <span>{move || if copied.get() { "Link copied!" } else { "Share" }}</span>
        </button>
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();
    let username = Signal::derive(move || params.read().get("username").unwrap_or_default());

    let profile = Resource::new(move || username.get(), get_public_profile);
    let collection = Resource::new(move || username.get(), get_public_collection);
    let stats = Resource::new(move || username.get(), get_public_profile_stats);
    let wishlist = Resource::new(move || username.get(), get_public_wishlist);

    view! {
        <div class="max-w-6xl mx-auto px-4 py-8">
            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading profile..."</p> }>
                {move || Suspend::new(async move {
                    match profile.await {
                        Ok(Some(user)) => {
                            EitherOf3::A(view! {
                                <div>
                                    // Profile header
                                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm p-6 mb-8">
                                        <div class="flex items-center gap-4">
                                            <Avatar url=user.avatar_url.clone() name=user.display_name.clone() size="w-16 h-16" text_size="text-2xl"/>
                                            <div>
                                                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{user.display_name.clone()}</h1>
                                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                                    "@"{user.username.clone()}
                                                    {user.location.as_ref().map(|loc| format!(" · {loc}")).unwrap_or_default()}
                                                </p>
                                            </div>
                                            {share_button(user.display_name.clone(), user.username.clone())}
                                        </div>
                                        {user.bio.clone().map(|bio| view! {
                                            <p class="mt-4 text-gray-600 dark:text-gray-300">{bio}</p>
                                        })}

                                        // Location map
                                        {
                                            let lat = user.latitude;
                                            let lng = user.longitude;
                                            let location_label = user.location.clone().unwrap_or_default();
                                            (lat.is_some() && lng.is_some()).then(move || {
                                                let map_id = "profile-map";
                                                init_leaflet_map(
                                                    map_id,
                                                    lat.unwrap(),
                                                    lng.unwrap(),
                                                    location_label,
                                                );
                                                view! {
                                                    <div id=map_id class="mt-4 h-48 rounded-xl overflow-hidden z-0"></div>
                                                }
                                            })
                                        }

                                        // Stats
                                        <Suspense fallback=|| ()>
                                            {move || Suspend::new(async move {
                                                stats.await.ok().map(|s| {
                                                    let last_active = s.last_active.map(|d| crate::utils::format_date(&d));
                                                    view! {
                                                    <div class="flex gap-6 mt-4 text-sm text-gray-600 dark:text-gray-400">
                                                        <div>
                                                            <span class="font-bold text-gray-900 dark:text-gray-100">{s.album_count}</span>
                                                            " albums"
                                                        </div>
                                                        <div>
                                                            <span class="font-bold text-gray-900 dark:text-gray-100">{s.lent_count}</span>
                                                            " lent"
                                                        </div>
                                                        {(s.wishlist_count > 0).then(|| view! {
                                                            <div>
                                                                <span class="font-bold text-gray-900 dark:text-gray-100">{s.wishlist_count}</span>
                                                                " wishlist"
                                                            </div>
                                                        })}
                                                        {last_active.map(|d| view! {
                                                            <div>
                                                                "Active "
                                                                <span class="font-bold text-gray-900 dark:text-gray-100">{d}</span>
                                                            </div>
                                                        })}
                                                    </div>
                                                }})
                                            })}
                                        </Suspense>

                                        // Follow toggle
                                        {
                                            let profile_user_id = user.id;
                                            let profile_username = user.username.clone();
                                            let current_user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();
                                            let is_own_profile = move || {
                                                current_user.get()
                                                    .and_then(|r| r.ok())
                                                    .flatten()
                                                    .map(|u| u.username == profile_username)
                                                    .unwrap_or(false)
                                            };
                                            let is_logged_in = move || {
                                                current_user.get()
                                                    .and_then(|r| r.ok())
                                                    .flatten()
                                                    .is_some()
                                            };

                                            let follow_status = Resource::new(
                                                || (),
                                                move |_| async move { is_following(profile_user_id).await },
                                            );
                                            let is_following_signal = RwSignal::new(false);
                                            Effect::new(move || {
                                                if let Some(Ok(val)) = follow_status.get() {
                                                    is_following_signal.set(val);
                                                }
                                            });
                                            let toggling = RwSignal::new(false);

                                            view! {
                                                <Show when=move || is_logged_in() && !is_own_profile()>
                                                    <button
                                                        class="mt-4 flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors cursor-pointer"
                                                        class=("bg-indigo-accent", move || !is_following_signal.get())
                                                        class=("text-white", move || !is_following_signal.get())
                                                        class=("hover:bg-indigo-accent-dark", move || !is_following_signal.get())
                                                        class=("bg-gray-100", move || is_following_signal.get())
                                                        class=("text-gray-700", move || is_following_signal.get())
                                                        class=("hover:bg-red-50", move || is_following_signal.get())
                                                        class=("hover:text-red-600", move || is_following_signal.get())
                                                        prop:disabled=move || toggling.get()
                                                        on:click=move |_| {
                                                            toggling.set(true);
                                                            let currently_following = is_following_signal.get();
                                                            leptos::task::spawn_local(async move {
                                                                let result = if currently_following {
                                                                    unfollow_user(profile_user_id).await
                                                                } else {
                                                                    follow_user(profile_user_id).await
                                                                };
                                                                if result.is_ok() {
                                                                    is_following_signal.set(!currently_following);
                                                                    if let Some(d) = use_context::<FollowingCountDisplay>() {
                                                                        d.0.update(|n| *n = n.map(|c| c + if currently_following { -1 } else { 1 }));
                                                                    }
                                                                    if let Some(res) = use_context::<FollowingCountResource>() {
                                                                        res.0.refetch();
                                                                    }
                                                                }
                                                                toggling.set(false);
                                                            });
                                                        }
                                                    >
                                                        <span class="material-symbols-outlined" style="font-size: 18px;">
                                                            {move || if is_following_signal.get() { "person_remove" } else { "person_add" }}
                                                        </span>
                                                        {move || if is_following_signal.get() { "Remove friend" } else { "Add friend" }}
                                                    </button>
                                                </Show>
                                            }
                                        }
                                    </div>

                                    // Collection
                                    <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4">"Collection"</h2>
                                    <Suspense fallback=|| view! { <p class="text-gray-500">"Loading collection..."</p> }>
                                        {move || Suspend::new(async move {
                                            match collection.await {
                                                Ok(items) if items.is_empty() => {
                                                    EitherOf3::A(view! { <p class="text-gray-500">"No items in this collection yet."</p> })
                                                }
                                                Ok(items) => {
                                                    let owner = username.get();
                                                    EitherOf3::B(view! {
                                                        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                                                            {items.into_iter().map(|swo| {
                                                                let ownership = Some((swo.owned_count, swo.total_albums));
                                                                let for_sale_count = swo.for_sale_count;
                                                                let is_terminated = swo.is_terminated.unwrap_or(false);
                                                                let series: crate::models::Series = swo.into();
                                                                let owner = owner.clone();
                                                                view! { <SeriesCard series=series ownership=ownership owner=owner for_sale_count=for_sale_count is_terminated=is_terminated/> }
                                                            }).collect_view()}
                                                        </div>
                                                    })
                                                }
                                                Err(e) => EitherOf3::C(view! {
                                                    <p class="text-red-500">{format!("Error: {e}")}</p>
                                                }),
                                            }
                                        })}
                                    </Suspense>

                                    // Wishlist (if public)
                                    <Suspense fallback=|| ()>
                                        {move || Suspend::new(async move {
                                            match wishlist.await {
                                                Ok(items) if !items.is_empty() => {
                                                    Some(view! {
                                                        <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4 mt-8">"Wishlist"</h2>
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
                                                _ => None,
                                            }
                                        })}
                                    </Suspense>
                                </div>
                            })
                        }
                        Ok(None) => {
                            EitherOf3::B(view! {
                                <div class="text-center py-12">
                                    <h2 class="text-2xl font-bold text-gray-300">"Profile not found"</h2>
                                    <p class="text-gray-500 mt-2">"This profile is private or doesn't exist."</p>
                                </div>
                            })
                        }
                        Err(e) => EitherOf3::C(view! {
                            <p class="text-red-500">{format!("Error: {e}")}</p>
                        }),
                    }
                })}
            </Suspense>
        </div>
    }
}

fn init_leaflet_map(_map_id: &str, _lat: f64, _lng: f64, _label: String) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsValue;

        let map_id = _map_id.to_string();
        let lat = _lat;
        let lng = _lng;
        let label = _label;

        leptos::task::spawn_local(async move {
            crate::leaflet::ensure_leaflet_loaded().await;
            crate::leaflet::wait_for_element(&map_id).await;

            // Initialize the map
            let window = web_sys::window().unwrap();
            let l = js_sys::Reflect::get(&window, &JsValue::from_str("L")).unwrap();

            // L.map(id, options)
            let map_fn = js_sys::Function::from(
                js_sys::Reflect::get(&l, &JsValue::from_str("map")).unwrap(),
            );
            let opts = js_sys::Object::new();
            js_sys::Reflect::set(&opts, &"dragging".into(), &JsValue::FALSE).unwrap();
            js_sys::Reflect::set(&opts, &"keyboard".into(), &JsValue::FALSE).unwrap();
            let map = map_fn
                .call2(&l, &JsValue::from_str(&map_id), &opts)
                .unwrap();

            // map.setView([lat, lng], zoom)
            let set_view = js_sys::Function::from(
                js_sys::Reflect::get(&map, &JsValue::from_str("setView")).unwrap(),
            );
            let coords = js_sys::Array::new();
            coords.push(&JsValue::from_f64(lat));
            coords.push(&JsValue::from_f64(lng));
            set_view
                .call2(&map, &coords, &JsValue::from_f64(10.0))
                .unwrap();

            // L.tileLayer(url).addTo(map)
            let tile_layer_fn = js_sys::Function::from(
                js_sys::Reflect::get(&l, &JsValue::from_str("tileLayer")).unwrap(),
            );
            let tile = tile_layer_fn
                .call1(
                    &l,
                    &JsValue::from_str("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"),
                )
                .unwrap();
            let add_to = js_sys::Function::from(
                js_sys::Reflect::get(&tile, &JsValue::from_str("addTo")).unwrap(),
            );
            add_to.call1(&tile, &map).unwrap();

            // L.marker([lat, lng]).addTo(map).bindPopup(label)
            let marker_fn = js_sys::Function::from(
                js_sys::Reflect::get(&l, &JsValue::from_str("marker")).unwrap(),
            );
            let marker_coords = js_sys::Array::new();
            marker_coords.push(&JsValue::from_f64(lat));
            marker_coords.push(&JsValue::from_f64(lng));
            let marker = marker_fn.call1(&l, &marker_coords).unwrap();
            let add_to_fn = js_sys::Function::from(
                js_sys::Reflect::get(&marker, &JsValue::from_str("addTo")).unwrap(),
            );
            let marker = add_to_fn.call1(&marker, &map).unwrap();
            if !label.is_empty() {
                let bind_popup = js_sys::Function::from(
                    js_sys::Reflect::get(&marker, &JsValue::from_str("bindPopup")).unwrap(),
                );
                bind_popup
                    .call1(&marker, &JsValue::from_str(&label))
                    .unwrap();
            }
        });
    }
}
