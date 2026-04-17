use sha2::{Digest, Sha256};

/// Generate a Gravatar URL from an email address using SHA-256.
pub fn gravatar_url(email: &str, size: u32) -> String {
    let hash = Sha256::digest(email.trim().to_lowercase().as_bytes());
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("https://gravatar.com/avatar/{hex}?d=identicon&s={size}")
}
