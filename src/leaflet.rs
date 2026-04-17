/// Shared Leaflet loading utilities (client-side only).

#[cfg(feature = "hydrate")]
pub async fn ensure_leaflet_loaded() {
    use wasm_bindgen::JsValue;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Load Leaflet CSS if not already loaded
    if document
        .query_selector("link[href*='leaflet']")
        .ok()
        .flatten()
        .is_none()
    {
        let link = document.create_element("link").unwrap();
        link.set_attribute("rel", "stylesheet").unwrap();
        link.set_attribute("href", "https://unpkg.com/leaflet@1/dist/leaflet.css")
            .unwrap();
        document.head().unwrap().append_child(&link).unwrap();
    }

    // Load Leaflet JS if not already loaded
    let leaflet_loaded =
        js_sys::Reflect::get(&window, &JsValue::from_str("L")).unwrap_or(JsValue::UNDEFINED);

    if leaflet_loaded.is_undefined() {
        load_script("https://unpkg.com/leaflet@1/dist/leaflet.js").await;
    }
}

#[cfg(feature = "hydrate")]
pub async fn ensure_markercluster_loaded() {
    use wasm_bindgen::JsValue;

    ensure_leaflet_loaded().await;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Load MarkerCluster CSS if not already loaded
    if document
        .query_selector("link[href*='MarkerCluster']")
        .ok()
        .flatten()
        .is_none()
    {
        for href in [
            "https://unpkg.com/leaflet.markercluster@1/dist/MarkerCluster.css",
            "https://unpkg.com/leaflet.markercluster@1/dist/MarkerCluster.Default.css",
        ] {
            let link = document.create_element("link").unwrap();
            link.set_attribute("rel", "stylesheet").unwrap();
            link.set_attribute("href", href).unwrap();
            document.head().unwrap().append_child(&link).unwrap();
        }
    }

    // Load MarkerCluster JS if not already loaded
    let l = js_sys::Reflect::get(&window, &JsValue::from_str("L")).unwrap();
    let has_cluster = js_sys::Reflect::get(&l, &JsValue::from_str("markerClusterGroup"))
        .unwrap_or(JsValue::UNDEFINED);

    if has_cluster.is_undefined() {
        load_script("https://unpkg.com/leaflet.markercluster@1/dist/leaflet.markercluster.js")
            .await;
    }
}

/// Wait for a DOM element to exist (for SPA navigation where the view
/// may not be mounted yet when map init is called).
#[cfg(feature = "hydrate")]
pub async fn wait_for_element(id: &str) {
    let document = web_sys::window().unwrap().document().unwrap();
    let id = id.to_string();
    loop {
        if document.get_element_by_id(&id).is_some() {
            return;
        }
        gloo_timers::future::TimeoutFuture::new(10).await;
    }
}

#[cfg(feature = "hydrate")]
async fn load_script(src: &str) {
    use wasm_bindgen::prelude::*;

    let document = web_sys::window().unwrap().document().unwrap();
    let (tx, rx) = futures::channel::oneshot::channel::<()>();
    let script = document.create_element("script").unwrap();
    script.set_attribute("src", src).unwrap();

    let tx_err = {
        use std::sync::Arc;
        let tx = Arc::new(std::sync::Mutex::new(Some(tx)));
        let tx_clone = tx.clone();

        let on_load = Closure::once(move || {
            if let Some(tx) = tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
        });
        script
            .add_event_listener_with_callback("load", on_load.as_ref().unchecked_ref())
            .unwrap();
        on_load.forget();

        tx_clone
    };

    let on_error = Closure::once(move || {
        if let Some(tx) = tx_err.lock().unwrap().take() {
            let _ = tx.send(());
        }
    });
    script
        .add_event_listener_with_callback("error", on_error.as_ref().unchecked_ref())
        .unwrap();
    on_error.forget();

    document.head().unwrap().append_child(&script).unwrap();
    let _ = rx.await;
}
