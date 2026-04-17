use crate::components::{
    AuthGuard, HelpDialog, HelpDialogOpen, LoginDialog, LoginDialogOpen, Sidebar, TopBar,
};
use crate::pages::*;
use crate::server::auth::get_current_user;
use crate::server::series::get_user_album_count;
use crate::server::social::{get_following_count, get_lent_album_count, get_unread_count};
use leptos::config::LeptosOptions;
use leptos::hydration::HydrationScripts;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::use_location;
use leptos_router::path;

#[derive(Clone, Copy)]
pub struct FollowingCountDisplay(pub RwSignal<Option<i64>>);

#[derive(Clone, Copy)]
pub struct LentCountDisplay(pub RwSignal<Option<i64>>);

#[derive(Clone, Copy)]
pub struct FollowingCountResource(pub Resource<Result<i64, ServerFnError>>);

#[derive(Clone, Copy)]
pub struct LentCountResource(pub Resource<Result<i64, ServerFnError>>);

/// Server-side HTML shell — not hydrated.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" href="/favicon.ico" sizes="48x48"/>
                <link rel="icon" href="/mybd.svg" type="image/svg+xml"/>
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png"/>
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png"/>
                <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png"/>
                <link rel="manifest" href="/site.webmanifest"/>
                <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap"/>
                <script>{r#"(function(){if(localStorage.getItem('theme')==='dark')document.documentElement.classList.add('dark')})()"#}</script>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// Context signal for dark mode.
#[derive(Clone, Copy)]
pub struct DarkMode(pub RwSignal<bool>);

/// Client-hydrated app component.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Initialise dark mode from the class set by the inline <head> script
    let initial_dark = {
        #[cfg(feature = "hydrate")]
        {
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.document_element())
                .is_some_and(|el| el.class_list().contains("dark"))
        }
        #[cfg(not(feature = "hydrate"))]
        false
    };
    let dark_mode = DarkMode(RwSignal::new(initial_dark));
    provide_context(dark_mode);

    let user = Resource::new(|| (), |_| get_current_user());
    provide_context(user);

    let album_count = Resource::new(|| (), |_| get_user_album_count());
    provide_context(album_count);

    // Stable count that doesn't flicker during refetch
    let album_count_display = RwSignal::new(None::<i64>);
    Effect::new(move || {
        if let Some(Ok(n)) = album_count.get() {
            album_count_display.set(Some(n));
        }
    });
    provide_context(album_count_display);

    let friend_count = Resource::new(|| (), |_| get_following_count());
    let friend_count_display = FollowingCountDisplay(RwSignal::new(None::<i64>));
    Effect::new(move || {
        if let Some(Ok(n)) = friend_count.get() {
            friend_count_display.0.set(Some(n));
        }
    });
    provide_context(FollowingCountResource(friend_count));
    provide_context(friend_count_display);

    let lent_count = Resource::new(|| (), |_| get_lent_album_count());
    let lent_count_display = LentCountDisplay(RwSignal::new(None::<i64>));
    Effect::new(move || {
        if let Some(Ok(n)) = lent_count.get() {
            lent_count_display.0.set(Some(n));
        }
    });
    provide_context(LentCountResource(lent_count));
    provide_context(lent_count_display);

    let unread_count = Resource::new(|| (), |_| get_unread_count());
    let unread_count_display = RwSignal::new(0i64);
    Effect::new(move || {
        if let Some(Ok(n)) = unread_count.get() {
            unread_count_display.set(n);
        }
    });
    provide_context(unread_count);
    provide_context(unread_count_display);

    let login_dialog_open = RwSignal::new(false);
    provide_context(LoginDialogOpen(login_dialog_open));

    let help_dialog_open = RwSignal::new(false);
    provide_context(HelpDialogOpen(help_dialog_open));

    provide_context(ResultsTabSetter(RwSignal::new(None)));

    // Global keyboard shortcuts (client-side only)
    #[cfg(feature = "hydrate")]
    {
        use std::cell::Cell;
        use std::rc::Rc;

        let last_g_time: Rc<Cell<f64>> = Rc::new(Cell::new(0.0));
        let nav_index: Rc<Cell<i32>> = Rc::new(Cell::new(-1));

        Effect::new(move || {
            use wasm_bindgen::prelude::*;

            let last_g = last_g_time.clone();
            let nav_idx = nav_index.clone();
            let handler = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    // Skip when typing in input fields (except Escape)
                    let key = ev.key();
                    if key != "Escape" {
                        if let Some(target) = ev.target() {
                            if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                                let tag = el.tag_name();
                                if tag == "INPUT" || tag == "TEXTAREA" || tag == "SELECT" {
                                    return;
                                }
                                if el.is_content_editable() {
                                    return;
                                }
                            }
                        }
                    }

                    let now = js_sys::Date::now();

                    // Helper: update nav focus to the given index
                    let set_nav_focus = |doc: &web_sys::Document, idx: i32| {
                        // Remove previous focus
                        if let Ok(items) = doc.query_selector_all("[data-nav-item].nav-focus") {
                            for i in 0..items.length() {
                                if let Some(el) = items.get(i) {
                                    if let Ok(el) = el.dyn_into::<web_sys::Element>() {
                                        let _ = el.class_list().remove_1("nav-focus");
                                    }
                                }
                            }
                        }
                        // Apply new focus
                        if idx >= 0 {
                            if let Ok(items) = doc
                                .query_selector_all("[data-nav-item]:not(.hidden [data-nav-item])")
                            {
                                if let Some(el) = items.get(idx as u32) {
                                    if let Ok(el) = el.dyn_into::<web_sys::Element>() {
                                        let _ = el.class_list().add_1("nav-focus");
                                        el.scroll_into_view_with_bool(false);
                                    }
                                }
                            }
                        }
                    };

                    match key.as_str() {
                        "j" | "k" => {
                            ev.prevent_default();
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Ok(items) = doc.query_selector_all(
                                    "[data-nav-item]:not(.hidden [data-nav-item])",
                                ) {
                                    let count = items.length() as i32;
                                    if count > 0 {
                                        let cur = nav_idx.get();
                                        let next = if key == "j" {
                                            if cur < count - 1 { cur + 1 } else { cur }
                                        } else if cur > 0 {
                                            cur - 1
                                        } else {
                                            0
                                        };
                                        nav_idx.set(next);
                                        set_nav_focus(&doc, next);
                                    }
                                }
                            }
                        }
                        "Enter" => {
                            let idx = nav_idx.get();
                            if idx >= 0 {
                                ev.prevent_default();
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    if let Ok(items) = doc.query_selector_all(
                                        "[data-nav-item]:not(.hidden [data-nav-item])",
                                    ) {
                                        if let Some(el) = items.get(idx as u32) {
                                            if let Ok(html_el) =
                                                el.dyn_into::<web_sys::HtmlElement>()
                                            {
                                                html_el.click();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "o" => {
                            let idx = nav_idx.get();
                            if idx >= 0 {
                                ev.prevent_default();
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    if let Ok(items) = doc.query_selector_all(
                                        "[data-nav-item]:not(.hidden [data-nav-item])",
                                    ) {
                                        if let Some(el) = items.get(idx as u32) {
                                            if let Ok(html_el) =
                                                el.dyn_into::<web_sys::HtmlElement>()
                                            {
                                                html_el.click();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "1" | "2" | "3" => {
                            if let Some(tab_setter) = use_context::<ResultsTabSetter>() {
                                if let Some(setter) = tab_setter.0.get() {
                                    ev.prevent_default();
                                    let tab = match key.as_str() {
                                        "1" => Tab::Series,
                                        "2" => Tab::Authors,
                                        "3" => Tab::Users,
                                        _ => return,
                                    };
                                    setter.set(tab);
                                }
                            }
                        }
                        " " => {
                            let idx = nav_idx.get();
                            if idx >= 0 {
                                ev.prevent_default();
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    if let Ok(items) = doc.query_selector_all(
                                        "[data-nav-item]:not(.hidden [data-nav-item])",
                                    ) {
                                        if let Some(el) = items.get(idx as u32) {
                                            if let Ok(element) = el.dyn_into::<web_sys::Element>() {
                                                // Check if this is a toggleable album item
                                                if element.get_attribute("data-album-id").is_some()
                                                {
                                                    // Find the button inside and click it
                                                    if let Ok(Some(btn)) =
                                                        element.query_selector("button")
                                                    {
                                                        if let Ok(html_btn) =
                                                            btn.dyn_into::<web_sys::HtmlElement>()
                                                        {
                                                            html_btn.click();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "?" => {
                            ev.prevent_default();
                            help_dialog_open.set(true);
                        }
                        "/" => {
                            ev.prevent_default();
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Ok(Some(el)) = doc.query_selector("#search-input") {
                                    if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                                        let _ = input.focus();
                                    }
                                }
                            }
                        }
                        "Escape" => {
                            if help_dialog_open.get() {
                                help_dialog_open.set(false);
                            } else if login_dialog_open.get() {
                                login_dialog_open.set(false);
                            } else if nav_idx.get() >= 0 {
                                // Clear nav selection
                                nav_idx.set(-1);
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    set_nav_focus(&doc, -1);
                                }
                            } else {
                                // Blur any focused input
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    if let Some(el) = doc.active_element() {
                                        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                                            let _ = html_el.blur();
                                        }
                                    }
                                }
                            }
                        }
                        "g" => {
                            last_g.set(now);
                        }
                        "c" if now - last_g.get() < 1000.0 => {
                            last_g.set(0.0);
                            nav_idx.set(-1);
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(loc) = doc.location() {
                                    let _ = loc.set_href("/collection");
                                }
                            }
                        }
                        "h" if now - last_g.get() < 1000.0 => {
                            last_g.set(0.0);
                            nav_idx.set(-1);
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(loc) = doc.location() {
                                    let _ = loc.set_href("/collection");
                                }
                            }
                        }
                        "s" if now - last_g.get() < 1000.0 => {
                            last_g.set(0.0);
                            nav_idx.set(-1);
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(loc) = doc.location() {
                                    let _ = loc.set_href("/settings");
                                }
                            }
                        }
                        "f" if now - last_g.get() < 1000.0 => {
                            last_g.set(0.0);
                            nav_idx.set(-1);
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(loc) = doc.location() {
                                    let _ = loc.set_href("/friends");
                                }
                            }
                        }
                        "w" if now - last_g.get() < 1000.0 => {
                            last_g.set(0.0);
                            nav_idx.set(-1);
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(loc) = doc.location() {
                                    let _ = loc.set_href("/world");
                                }
                            }
                        }
                        _ => {
                            last_g.set(0.0);
                        }
                    }
                },
            );

            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                let _ = doc
                    .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
                handler.forget();
            }
        });
    }

    let main_ref = NodeRef::<leptos::html::Main>::new();

    view! {
        <Stylesheet id="leptos" href="/pkg/mybd.css"/>
        <Title text="mybd"/>
        <Router>
            <div class="flex h-screen dark:bg-gray-900 overflow-x-hidden">
                <Sidebar/>
                <div class="flex-1 flex flex-col min-w-0">
                    <TopBar/>
                    <main class="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900" node_ref=main_ref>
                        <NavigationDebugger/>
                        <ScrollToTop main_ref=main_ref/>
                        <Routes fallback=|| view! { <NotFoundPage/> }>
                            <Route path=path!("/") view=HomePage/>
                            <Route path=path!("/login") view=LoginPage/>
                            <Route path=path!("/register") view=RegisterPage/>
                            <Route path=path!("/collection") view=|| view! { <CollectionPage/> }/>
                            <Route path=path!("/search") view=|| view! { <ResultPage/> }/>
                            <Route path=path!("/series/bubble/:bubble_id") view=|| view! { <ExternalSeriesPage/> }/>
                            <Route path=path!("/series/:slug") view=|| view! { <SeriesDetailPage/> }/>
                            <Route path=path!("/album/:slug") view=|| view! { <AlbumDetailPage/> }/>
                            <Route path=path!("/author/:slug") view=|| view! { <AuthorPage/> }/>
                            <Route path=path!("/friends") view=|| view! { <AuthGuard><FollowingPage/></AuthGuard> }/>
                            <Route path=path!("/world") view=WorldPage/>
                            <Route path=path!("/lent") view=|| view! { <AuthGuard><LentPage/></AuthGuard> }/>
                            <Route path=path!("/profile/:username") view=ProfilePage/>
                            <Route path=path!("/settings") view=|| view! { <AuthGuard><SettingsPage/></AuthGuard> }/>
                        </Routes>
                    </main>
                </div>
            </div>
            <LoginDialog/>
            <HelpDialog/>
        </Router>
    }
}

/// Debug component — logs navigation events to the browser console.
/// Open DevTools → Console → filter `[nav]` to see click / route / popstate events.
#[component]
fn NavigationDebugger() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    nav_debug_setup();
}

#[cfg(feature = "hydrate")]
fn nav_debug_setup() {
    use wasm_bindgen::prelude::*;
    let location = use_location();

    web_sys::console::log_1(
        &format!(
            "[nav] debugger active at {}",
            location.pathname.get_untracked()
        )
        .into(),
    );

    // 1. Log every Leptos route change
    Effect::new(move || {
        let path = location.pathname.get();
        let search = location.search.get();
        let hash = location.hash.get();
        web_sys::console::log_1(&format!("[nav] route → {path}{search}{hash}").into());
    });

    // 2. Click listeners — capture phase shows intent, bubble phase shows outcome
    Effect::new(move || {
        let capture = Closure::<dyn Fn(web_sys::Event)>::new(move |ev: web_sys::Event| {
            if let Some(a) = nav_debug_find_anchor(&ev) {
                let href = a.get_attribute("href").unwrap_or_default();
                let rel = a.get_attribute("rel").unwrap_or_default();
                let target_attr = a.get_attribute("target").unwrap_or_default();
                web_sys::console::log_1(
                    &format!("[nav] click ▶ href={href:?} rel={rel:?} target={target_attr:?}")
                        .into(),
                );
            }
        });

        let bubble = Closure::<dyn Fn(web_sys::Event)>::new(move |ev: web_sys::Event| {
            if let Some(a) = nav_debug_find_anchor(&ev) {
                let href = a.get_attribute("href").unwrap_or_default();
                web_sys::console::log_1(
                    &format!(
                        "[nav] click ◀ href={href:?} defaultPrevented={}",
                        ev.default_prevented()
                    )
                    .into(),
                );
            }
        });

        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let opts = web_sys::AddEventListenerOptions::new();
            opts.set_capture(true);
            let _ = doc.add_event_listener_with_callback_and_add_event_listener_options(
                "click",
                capture.as_ref().unchecked_ref(),
                &opts,
            );
            capture.forget();

            let _ = doc.add_event_listener_with_callback("click", bubble.as_ref().unchecked_ref());
            bubble.forget();
        }
    });

    // 3. Log browser back/forward
    Effect::new(move || {
        let handler = Closure::<dyn Fn(web_sys::Event)>::new(move |_: web_sys::Event| {
            if let Some(w) = web_sys::window() {
                let href = w.location().href().unwrap_or_default();
                web_sys::console::log_1(&format!("[nav] popstate → {href}").into());
            }
        });

        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("popstate", handler.as_ref().unchecked_ref());
            handler.forget();
        }
    });
}

/// Walk up the DOM from an event target to find the enclosing `<a>` element.
#[cfg(feature = "hydrate")]
fn nav_debug_find_anchor(ev: &web_sys::Event) -> Option<web_sys::Element> {
    use wasm_bindgen::JsCast;
    let target = ev.target()?;
    let mut el: web_sys::Element = target.dyn_into().ok()?;
    loop {
        if el.tag_name() == "A" {
            return Some(el);
        }
        el = el.parent_element()?;
    }
}

/// Invisible component that scrolls the main content area to top on route change.
#[component]
fn ScrollToTop(main_ref: NodeRef<leptos::html::Main>) -> impl IntoView {
    let location = use_location();
    Effect::new(move || {
        // Track pathname so the effect re-runs on navigation
        let _ = location.pathname.get();
        if let Some(el) = main_ref.get() {
            el.set_scroll_top(0);
        }
    });
}
