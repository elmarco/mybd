-- Squashed migration: full schema.

-- Users & sessions
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    bio TEXT,
    is_public INTEGER NOT NULL DEFAULT 0,
    wishlist_public BOOLEAN NOT NULL DEFAULT 0,
    google_id TEXT,
    country TEXT,
    city TEXT,
    latitude REAL,
    longitude REAL,
    location TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE UNIQUE INDEX idx_users_google_id ON users(google_id);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    expires_at TEXT NOT NULL
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Series / albums / ownership
CREATE TABLE series (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    work_type TEXT NOT NULL DEFAULT 'bd'
        CHECK (work_type IN ('comic', 'manga', 'bd')),
    description TEXT,
    cover_url TEXT,
    year INTEGER,
    number_of_albums INTEGER,
    is_terminated BOOLEAN,
    bubble_id TEXT,
    slug TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE UNIQUE INDEX idx_series_bubble ON series(bubble_id) WHERE bubble_id IS NOT NULL;

CREATE TABLE albums (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    title TEXT,
    tome INTEGER,
    cover_url TEXT,
    ean TEXT,
    bubble_id TEXT,
    summary TEXT,
    publisher TEXT,
    number_of_pages INTEGER,
    publication_date TEXT,
    height_cm REAL,
    width_cm REAL,
    length_cm REAL,
    weight_kg REAL,
    slug TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX idx_albums_series ON albums(series_id);
CREATE UNIQUE INDEX idx_albums_bubble ON albums(bubble_id) WHERE bubble_id IS NOT NULL;
CREATE UNIQUE INDEX idx_albums_ean ON albums(ean) WHERE ean IS NOT NULL;

CREATE TABLE user_albums (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    owned BOOLEAN NOT NULL DEFAULT 1,
    wishlisted BOOLEAN NOT NULL DEFAULT 0,
    for_sale_price REAL NULL,
    owned_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    PRIMARY KEY (user_id, album_id)
);
CREATE INDEX idx_user_albums_user ON user_albums(user_id);

-- Ephemeral OAuth CSRF tokens
CREATE TABLE oauth_states (
    state TEXT PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Social features
CREATE TABLE follows (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    PRIMARY KEY (user_id, following_id)
);
CREATE INDEX idx_follows_following ON follows(following_id);

CREATE TABLE album_loans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lender_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    borrower_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(album_id, lender_id)
);
CREATE INDEX idx_album_loans_lender ON album_loans(lender_id);
CREATE INDEX idx_album_loans_borrower ON album_loans(borrower_id);

CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    payload TEXT NOT NULL DEFAULT '{}',
    read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX idx_notifications_user ON notifications(user_id);

-- Authors
CREATE TABLE authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    display_name TEXT NOT NULL,
    slug TEXT,
    bio TEXT,
    bubble_id TEXT UNIQUE,
    date_birth TEXT,
    date_death TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE UNIQUE INDEX idx_authors_slug ON authors(slug) WHERE slug IS NOT NULL;

CREATE TABLE album_authors (
    album_id INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    role TEXT,
    PRIMARY KEY (album_id, author_id)
);
CREATE INDEX idx_album_authors_author ON album_authors(author_id);
