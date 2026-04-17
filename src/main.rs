#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Extension, Router, routing::get};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use mybd::app::App;
    use tower_http::trace::TraceLayer;
    use tracing_subscriber::EnvFilter;

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("mybd=info".parse().unwrap()))
        .init();

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let pool = mybd::db::create_pool().await;

    // Purge expired sessions every hour
    tokio::spawn({
        let pool = pool.clone();
        async move {
            loop {
                let result = sqlx::query(
                    "DELETE FROM sessions WHERE expires_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
                )
                .execute(&pool)
                .await;
                match result {
                    Ok(r) if r.rows_affected() > 0 => {
                        tracing::info!(purged = r.rows_affected(), "expired sessions cleaned up");
                    }
                    Err(e) => tracing::warn!("session cleanup failed: {e}"),
                    _ => {}
                }
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        }
    });

    let app = Router::new()
        .route("/auth/google/start", get(mybd::google_auth::google_start))
        .route(
            "/auth/google/callback",
            get(mybd::google_auth::google_callback),
        )
        .route("/auth/logout", get(mybd::google_auth::logout_handler))
        .route(
            "/debug/metrics",
            get(mybd::server::metrics::metrics_handler),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || mybd::app::shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<LeptosOptions, _>(
            mybd::app::shell,
        ))
        .layer(axum::middleware::from_fn(server_fn_timing))
        .layer(TraceLayer::new_for_http().make_span_with(|req: &axum::extract::Request| {
            tracing::info_span!("request", method = %req.method(), uri = %req.uri())
        }))
        .layer(Extension(pool.clone()))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!(%addr, "listening on HTTP");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
async fn server_fn_timing(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();
    mybd::server::metrics::record(&path, duration);

    if response.status().is_server_error() {
        // Buffer body so we can log it and still return it to the client.
        let (parts, body) = response.into_parts();
        let bytes = axum::body::to_bytes(body, 64 * 1024)
            .await
            .unwrap_or_default();
        let body_str = String::from_utf8_lossy(&bytes);
        tracing::error!(%method, %path, status = %parts.status, ?duration, body = %body_str, "server error");
        axum::response::Response::from_parts(parts, axum::body::Body::from(bytes))
    } else {
        response
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
