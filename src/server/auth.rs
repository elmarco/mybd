use crate::models::UserPublic;
use leptos::prelude::*;
use server_fn::ServerFnError;

#[server]
pub async fn register(
    username: String,
    email: String,
    password: String,
    #[server(default)] display_name: String,
) -> Result<(), ServerFnError> {
    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };
    let pool = crate::db::pool();

    if username.is_empty() || email.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("All fields are required"));
    }
    if username.len() > 30 {
        return Err(ServerFnError::new("Username must be at most 30 characters"));
    }
    if email.len() > 254 {
        return Err(ServerFnError::new("Email is too long"));
    }
    if password.len() < 8 {
        return Err(ServerFnError::new("Password must be at least 8 characters"));
    }
    if password.len() > 128 {
        return Err(ServerFnError::new(
            "Password must be at most 128 characters",
        ));
    }
    if display_name.len() > 100 {
        return Err(ServerFnError::new(
            "Display name must be at most 100 characters",
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ServerFnError::new(
            "Username must contain only letters, numbers, hyphens, and underscores",
        ));
    }

    let display_name = if display_name.trim().is_empty() {
        username.clone()
    } else {
        display_name
    };

    let password_hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
    })
    .await
    .map_err(|e| ServerFnError::new(format!("Hashing task failed: {e}")))?
    .map_err(|e| ServerFnError::new(format!("Failed to hash password: {e}")))?;

    let avatar_url = crate::gravatar::gravatar_url(&email, 200);

    let user_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, email, display_name, password_hash, avatar_url) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&username)
    .bind(&email)
    .bind(&display_name)
    .bind(&password_hash)
    .bind(&avatar_url)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            ServerFnError::new("Username or email already taken")
        } else {
            ServerFnError::new(format!("Registration failed: {e}"))
        }
    })?;

    create_session(user_id, &pool).await?;
    Ok(())
}

#[server]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let pool = crate::db::pool();

    if email.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Email and password are required"));
    }

    let row =
        sqlx::query_as::<_, (i64, String)>("SELECT id, password_hash FROM users WHERE email = ?")
            .bind(&email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    let (user_id, hash) = row.ok_or_else(|| ServerFnError::new("Invalid email or password"))?;

    if hash.is_empty() {
        return Err(ServerFnError::new(
            "This account uses Google sign-in. Please use the Google button.",
        ));
    }

    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
    })
    .await
    .map_err(|e| ServerFnError::new(format!("Verification task failed: {e}")))?
    .map_err(|_| ServerFnError::new("Invalid email or password"))?;

    create_session(user_id, &pool).await?;
    Ok(())
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let pool = crate::db::pool();

    if let Some(session_id) = get_session_id() {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(&session_id)
            .execute(&pool)
            .await
            .ok();
    }

    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    response_options.insert_header(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&format!(
            "session_id=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
            crate::google_auth::secure_flag()
        ))
        .map_err(|e| ServerFnError::new(format!("Failed to build cookie: {e}")))?,
    );

    leptos_axum::redirect("/");
    Ok(())
}

#[server]
pub async fn get_current_user() -> Result<Option<UserPublic>, ServerFnError> {
    let pool = crate::db::pool();

    let session_id = match get_session_id() {
        Some(id) => id,
        None => return Ok(None),
    };

    let user = sqlx::query_as::<_, UserPublic>(
        "SELECT u.id, u.username, u.display_name, u.avatar_url, u.bio,
                u.location, u.latitude, u.longitude,
                u.is_public, u.wishlist_public, u.created_at
         FROM users u
         JOIN sessions s ON s.user_id = u.id
         WHERE s.id = ? AND s.expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
    )
    .bind(&session_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    match user {
        Some(user) => Ok(Some(user)),
        None => {
            sqlx::query("DELETE FROM sessions WHERE id = ?")
                .bind(&session_id)
                .execute(&pool)
                .await
                .ok();
            Ok(None)
        }
    }
}

#[server]
pub async fn update_profile(
    display_name: String,
    bio: String,
    avatar_url: String,
    is_public: Option<String>,
    wishlist_public: Option<String>,
    location: String,
) -> Result<(), ServerFnError> {
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    if display_name.len() > 100 {
        return Err(ServerFnError::new(
            "Display name must be at most 100 characters",
        ));
    }
    if bio.len() > 1000 {
        return Err(ServerFnError::new("Bio must be at most 1000 characters"));
    }
    if !avatar_url.is_empty() && !avatar_url.starts_with("https://") {
        return Err(ServerFnError::new("Avatar URL must use HTTPS"));
    }
    if avatar_url.len() > 2048 {
        return Err(ServerFnError::new("Avatar URL is too long"));
    }
    if location.len() > 200 {
        return Err(ServerFnError::new(
            "Location must be at most 200 characters",
        ));
    }

    // HTML checkboxes send value="true" when checked, absent when unchecked
    let is_public = is_public.as_deref() == Some("true");
    let wishlist_public = wishlist_public.as_deref() == Some("true");

    let bio = if bio.is_empty() { None } else { Some(bio) };
    let avatar_url = if avatar_url.is_empty() {
        let email = sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = ?")
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .ok();
        email.map(|e| crate::gravatar::gravatar_url(&e, 200))
    } else {
        Some(avatar_url)
    };

    let location = if location.trim().is_empty() {
        None
    } else {
        Some(location.trim().to_string())
    };

    let (latitude, longitude) = if let Some(loc) = &location {
        crate::server::geocoding::geocode_query(loc)
            .await?
            .map(|(lat, lon)| (Some(lat), Some(lon)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    sqlx::query(
        "UPDATE users SET display_name = ?, bio = ?, avatar_url = ?, is_public = ?,
         wishlist_public = ?, location = ?, latitude = ?, longitude = ?,
         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(&display_name)
    .bind(&bio)
    .bind(&avatar_url)
    .bind(is_public)
    .bind(wishlist_public)
    .bind(&location)
    .bind(latitude)
    .bind(longitude)
    .bind(user.id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to update profile: {e}")))?;

    Ok(())
}

#[server]
pub async fn delete_account(confirmation: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    if confirmation != "Bachi-bouzouk" {
        return Err(ServerFnError::new(
            "Please type \"Bachi-bouzouk\" to confirm account deletion",
        ));
    }

    // ON DELETE CASCADE handles sessions, user_albums
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete account: {e}")))?;

    // Clear the session cookie
    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    response_options.insert_header(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&format!(
            "session_id=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
            crate::google_auth::secure_flag()
        ))
        .map_err(|e| ServerFnError::new(format!("Failed to build cookie: {e}")))?,
    );

    Ok(())
}

#[cfg(feature = "ssr")]
fn get_session_id() -> Option<String> {
    use http::header;
    let headers = leptos::context::use_context::<http::request::Parts>()?;
    let cookie_header = headers.headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("session_id="))
        .map(|s| s.trim_start_matches("session_id=").to_string())
}

#[cfg(feature = "ssr")]
async fn create_session(user_id: i64, pool: &sqlx::SqlitePool) -> Result<(), ServerFnError> {
    use uuid::Uuid;

    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(user_id)
        .bind(&expires_at)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create session: {e}")))?;

    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    let cookie = format!(
        "session_id={session_id}; Path=/; Max-Age=604800; HttpOnly; SameSite=Lax{}",
        crate::google_auth::secure_flag()
    );
    response_options.insert_header(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&cookie)
            .map_err(|e| ServerFnError::new(format!("Failed to build cookie: {e}")))?,
    );

    Ok(())
}
