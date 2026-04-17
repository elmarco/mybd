pub mod app;
pub mod components;
pub mod leaflet;
pub mod models;
pub mod pages;
pub mod server;
pub mod utils;

#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod google_auth;
#[cfg(feature = "ssr")]
pub mod gravatar;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
    // Signal to e2e tests that WASM hydration is complete
    if let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    {
        let _ = body.set_attribute("data-hydrated", "");
    }
}
