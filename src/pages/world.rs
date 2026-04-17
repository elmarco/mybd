use crate::server::social::get_public_user_locations;
use leptos::prelude::*;

#[component]
pub fn WorldPage() -> impl IntoView {
    let locations = Resource::new(|| (), |_| get_public_user_locations());

    view! {
        <div class="flex flex-col h-full px-4 py-8 max-w-7xl mx-auto">
            <div class="flex items-center gap-2 mb-4">
                <span class="material-symbols-outlined text-2xl text-gray-700 dark:text-gray-300">"public"</span>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">"World Map"</h1>
                <Suspense fallback=|| ()>
                    {move || Suspend::new(async move {
                        let count = locations.await.map(|l| l.len()).unwrap_or(0);
                        view! {
                            <span class="text-sm text-gray-500 dark:text-gray-400 ml-auto">
                                {count} " collector" {if count != 1 { "s" } else { "" }}
                            </span>
                        }
                    })}
                </Suspense>
            </div>

            <div
                id="world-map"
                class="flex-1 min-h-[400px] rounded-xl overflow-hidden shadow-sm bg-gray-200 isolate"
            />

            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    let users = locations.await.unwrap_or_default();
                    init_world_map(users);
                })}
            </Suspense>
        </div>
    }
}

fn init_world_map(_users: Vec<crate::models::UserLocation>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsValue;

        let users = _users;

        leptos::task::spawn_local(async move {
            crate::leaflet::ensure_markercluster_loaded().await;
            crate::leaflet::wait_for_element("world-map").await;

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let l = js_sys::Reflect::get(&window, &JsValue::from_str("L")).unwrap();

            // Remove existing map instance if re-navigating via SPA
            if let Some(container) = document.get_element_by_id("world-map") {
                let existing = js_sys::Reflect::get(&container, &JsValue::from_str("_leaflet_id"));
                if let Ok(id) = existing {
                    if !id.is_undefined() {
                        // Container already has a map — remove it first
                        let map_fn = js_sys::Function::from(
                            js_sys::Reflect::get(&l, &JsValue::from_str("map")).unwrap(),
                        );
                        let old_map = map_fn.call1(&l, &JsValue::from_str("world-map")).unwrap();
                        let remove_fn = js_sys::Function::from(
                            js_sys::Reflect::get(&old_map, &JsValue::from_str("remove")).unwrap(),
                        );
                        let _ = remove_fn.call0(&old_map);
                    }
                }
            }

            // L.map("world-map")
            let map_fn = js_sys::Function::from(
                js_sys::Reflect::get(&l, &JsValue::from_str("map")).unwrap(),
            );
            let map = map_fn.call1(&l, &JsValue::from_str("world-map")).unwrap();

            // map.setView([30, 10], 3)
            let set_view = js_sys::Function::from(
                js_sys::Reflect::get(&map, &JsValue::from_str("setView")).unwrap(),
            );
            let coords = js_sys::Array::new();
            coords.push(&JsValue::from_f64(30.0));
            coords.push(&JsValue::from_f64(10.0));
            set_view
                .call2(&map, &coords, &JsValue::from_f64(3.0))
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

            // L.markerClusterGroup()
            let cluster_fn = js_sys::Function::from(
                js_sys::Reflect::get(&l, &JsValue::from_str("markerClusterGroup")).unwrap(),
            );
            let cluster_group = cluster_fn.call0(&l).unwrap();

            // Add markers
            let marker_fn = js_sys::Function::from(
                js_sys::Reflect::get(&l, &JsValue::from_str("marker")).unwrap(),
            );

            for user in &users {
                let marker_coords = js_sys::Array::new();
                marker_coords.push(&JsValue::from_f64(user.latitude));
                marker_coords.push(&JsValue::from_f64(user.longitude));
                let marker = marker_fn.call1(&l, &marker_coords).unwrap();

                // Build popup HTML
                let initial = user
                    .display_name
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                let album_label = if user.album_count == 1 {
                    "album"
                } else {
                    "albums"
                };
                let popup_html = format!(
                    r#"<div style="display:flex;align-items:center;gap:8px;min-width:140px">
                        <div style="width:32px;height:32px;background:#7c8aff;border-radius:50%;display:flex;align-items:center;justify-content:center;color:#fff;font-weight:bold;font-size:14px;flex-shrink:0">{initial}</div>
                        <div>
                            <a href="/profile/{username}" style="font-weight:600;color:#7c8aff;text-decoration:none;font-size:13px">{display_name}</a>
                            <div style="color:#888;font-size:11px">{album_count} {album_label}</div>
                        </div>
                    </div>"#,
                    initial = initial,
                    username = urlencoding::encode(&user.username),
                    display_name = html_escape(&user.display_name),
                    album_count = user.album_count,
                    album_label = album_label,
                );

                let bind_popup = js_sys::Function::from(
                    js_sys::Reflect::get(&marker, &JsValue::from_str("bindPopup")).unwrap(),
                );
                let marker = bind_popup
                    .call1(&marker, &JsValue::from_str(&popup_html))
                    .unwrap();

                // Add marker to cluster group
                let add_layer = js_sys::Function::from(
                    js_sys::Reflect::get(&cluster_group, &JsValue::from_str("addLayer")).unwrap(),
                );
                add_layer.call1(&cluster_group, &marker).unwrap();
            }

            // Add cluster group to map
            let add_to_map = js_sys::Function::from(
                js_sys::Reflect::get(&cluster_group, &JsValue::from_str("addTo")).unwrap(),
            );
            add_to_map.call1(&cluster_group, &map).unwrap();
        });
    }
}

#[cfg(feature = "hydrate")]
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
