use axum::{
    extract::{Extension, Query},
    response::{IntoResponse, Redirect, Response},
};
use http::{HeaderValue, header};
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
pub struct CallbackParams {
    code: String,
    state: Option<String>,
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(serde::Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

/// GET /auth/google/start — generate CSRF state, store in DB, redirect to Google.
pub async fn google_start(req: axum::extract::Request) -> Response {
    let client_id = match std::env::var("GOOGLE_CLIENT_ID") {
        Ok(id) if !id.is_empty() => id,
        _ => return Redirect::to("/login").into_response(),
    };

    let host = extract_host(&req);
    let redirect_uri = build_redirect_uri(&host);

    let state = uuid::Uuid::new_v4().to_string();

    // Store state server-side (avoids all browser cookie issues with self-signed TLS)
    let pool = crate::db::pool();
    // Clean up expired states (> 10 min old) while we're here
    sqlx::query("DELETE FROM oauth_states WHERE created_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-10 minutes')")
        .execute(&pool)
        .await
        .ok();
    if let Err(e) = sqlx::query("INSERT INTO oauth_states (state) VALUES (?)")
        .bind(&state)
        .execute(&pool)
        .await
    {
        tracing::error!(error = %e, "failed to store oauth state");
        return Redirect::to("/login").into_response();
    }

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
         client_id={}&\
         redirect_uri={}&\
         response_type=code&\
         scope=openid%20email%20profile&\
         state={}&\
         access_type=offline",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state),
    );

    tracing::debug!(%redirect_uri, "starting google auth");

    Redirect::to(&url).into_response()
}

/// GET /auth/google/callback — exchange code for token, find/create user, set session.
pub async fn google_callback(
    Query(params): Query<CallbackParams>,
    Extension(pool): Extension<SqlitePool>,
    req: axum::extract::Request,
) -> Response {
    // Verify CSRF state against server-side store
    let state = match &params.state {
        Some(s) if !s.is_empty() => s.clone(),
        _ => {
            tracing::warn!("google callback missing state parameter");
            return Redirect::to("/login").into_response();
        }
    };

    let deleted = sqlx::query("DELETE FROM oauth_states WHERE state = ?")
        .bind(&state)
        .execute(&pool)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);

    if deleted == 0 {
        tracing::warn!(%state, "unknown or expired oauth state");
        return Redirect::to("/login").into_response();
    }

    let host = extract_host(&req);
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
    let redirect_uri = build_redirect_uri(&host);
    tracing::debug!(%redirect_uri, "google callback");

    // Exchange authorization code for access token
    let client = reqwest::Client::new();
    let token_res = match client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", params.code.as_str()),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("redirect_uri", &redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "token exchange request failed");
            return Redirect::to("/login").into_response();
        }
    };

    let token: TokenResponse = match token_res.json().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "token response parse failed");
            return Redirect::to("/login").into_response();
        }
    };

    // Fetch user info from Google
    let userinfo: GoogleUserInfo = match client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(&token.access_token)
        .send()
        .await
    {
        Ok(r) => match r.json().await {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(error = %e, "userinfo parse failed");
                return Redirect::to("/login").into_response();
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "userinfo request failed");
            return Redirect::to("/login").into_response();
        }
    };

    // Find or create user
    let user_id = match find_or_create_user(&pool, &userinfo).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "find_or_create_user failed");
            return Redirect::to("/login").into_response();
        }
    };

    // Create session
    let session_id = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    if let Err(e) = sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(user_id)
        .bind(&expires_at)
        .execute(&pool)
        .await
    {
        tracing::error!(error = %e, "session insert failed");
        return Redirect::to("/login").into_response();
    }

    let session_cookie = format!(
        "session_id={session_id}; Path=/; Max-Age=604800; HttpOnly; SameSite=Lax{sf}",
        sf = secure_flag()
    );

    let mut response = Redirect::to("/collection").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&session_cookie).unwrap(),
    );

    response
}

async fn find_or_create_user(pool: &SqlitePool, info: &GoogleUserInfo) -> Result<i64, sqlx::Error> {
    // 1. Check if user already linked via google_id
    if let Some(id) = sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE google_id = ?")
        .bind(&info.sub)
        .fetch_optional(pool)
        .await?
    {
        // Update avatar on each login
        if let Some(pic) = &info.picture {
            sqlx::query("UPDATE users SET avatar_url = ? WHERE id = ?")
                .bind(pic)
                .bind(id)
                .execute(pool)
                .await
                .ok();
        }
        return Ok(id);
    }

    // 2. Check if email already exists (link Google to existing account)
    if let Some(id) = sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE email = ?")
        .bind(&info.email)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query(
            "UPDATE users SET google_id = ?, avatar_url = COALESCE(avatar_url, ?) WHERE id = ?",
        )
        .bind(&info.sub)
        .bind(
            info.picture
                .as_deref()
                .unwrap_or(&crate::gravatar::gravatar_url(&info.email, 200)),
        )
        .bind(id)
        .execute(pool)
        .await?;
        return Ok(id);
    }

    // 3. Create new user
    let display_name = info.name.clone().unwrap_or_else(|| info.email.clone());
    let username = generate_username(&info.email, pool).await;
    let avatar = info
        .picture
        .clone()
        .unwrap_or_else(|| crate::gravatar::gravatar_url(&info.email, 200));

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, email, display_name, password_hash, avatar_url, google_id) \
         VALUES (?, ?, ?, '', ?, ?) RETURNING id",
    )
    .bind(&username)
    .bind(&info.email)
    .bind(&display_name)
    .bind(&avatar)
    .bind(&info.sub)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// GET /auth/logout — delete session, clear cookie, redirect to /.
pub async fn logout_handler(headers: http::HeaderMap) -> Response {
    let session_id = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with("session_id="))
                .map(|s| s.trim_start_matches("session_id=").to_string())
        });

    if let Some(sid) = session_id {
        let pool = crate::db::pool();
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(&sid)
            .execute(&pool)
            .await
            .ok();
    }

    let clear_cookie = format!(
        "session_id=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{sf}",
        sf = secure_flag()
    );
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&clear_cookie).unwrap(),
    );
    response
}

/// Extract host from request: URI authority (HTTP/2), then Host header (HTTP/1.1).
fn extract_host(req: &axum::extract::Request) -> String {
    // HTTP/2: authority is in the URI
    if let Some(authority) = req.uri().authority() {
        return authority.to_string();
    }
    // HTTP/1.1: Host header
    req.headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3000")
        .to_string()
}

fn site_url() -> Option<String> {
    std::env::var("SITE_URL").ok().filter(|s| !s.is_empty())
}

/// Returns "; Secure" when behind a TLS-terminating reverse proxy (SITE_URL starts with https).
pub fn secure_flag() -> &'static str {
    if site_url().is_some_and(|u| u.starts_with("https")) {
        "; Secure"
    } else {
        ""
    }
}

fn build_redirect_uri(host: &str) -> String {
    if let Some(base) = site_url() {
        let base = base.trim_end_matches('/');
        return format!("{base}/auth/google/callback");
    }
    format!("http://{host}/auth/google/callback")
}

/// Generate a unique username from the email prefix.
async fn generate_username(email: &str, pool: &SqlitePool) -> String {
    let base: String = email
        .split('@')
        .next()
        .unwrap_or("user")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    let base = if base.is_empty() {
        "user".to_string()
    } else {
        base
    };

    let mut username = base.clone();
    let mut counter = 1u32;
    loop {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE username = ?")
            .bind(&username)
            .fetch_one(pool)
            .await
            .unwrap_or(1);

        if exists == 0 {
            break;
        }
        username = format!("{base}{counter}");
        counter += 1;
    }

    username
}
