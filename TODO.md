# TODO

# the x in the search bar should only be shown when there is text to clear

# when populating an author page, we should query all results from bubble (large request or by pagess)

# Language switch somewhere (bottom left?)

# public profile

Use tabs: Collection / Lent (future: / To Sell / Wish), listing albums


# search page

- if there are more than 20 results, we should be able to page next/prev / show all results

# tests

 Entire pages with no tests:
  - Author detail page (/author/:slug)
  - Series detail page (/series/:slug)
  - World map page (/world)

  Untested interactive components:
  - Barcode scanner modal
  - Help dialog (?)
  - Dark mode toggle
  - Sidebar collapse/expand
  - Mobile drawer
  - Location autocomplete (settings)
  - Search results pagination (Previous/Next/Show all)

  Partially tested flows:
  - Lending to a friend (selecting borrower from dropdown) — only the return flow is tested
  - Settings: only display name, bio, and public toggle are tested — avatar URL, wishlist visibility, and location are not
  - For-sale: basic flow tested, but Enter-to-save / Escape-to-cancel are not

  Well covered:
  - Auth (register, login, logout, delete account)
  - Album ownership & wishlist toggling
  - Follow/unfollow + friend management
  - Notifications
  - Search (query, tabs, history with arrow keys)
  - Album detail keyboard navigation (arrows)
  - Sidebar count reactivity

# design document

## Security

- [ ] Add rate limiting on auth endpoints (login, register) — e.g. `tower-governor`
- [x] Add input validation in `update_profile` — enforce length limits on `display_name` (100), `bio` (1000), validate `avatar_url` is https
- [x] Add session cleanup — background task or periodic purge of expired sessions (currently only cleaned at login)
- [ ] Add notification retention policy — purge notifications older than 90 days
- [ ] Sanitize server error messages — return generic messages to clients, log details server-side only
- [ ] Ensure session cookies have `Secure` flag unconditionally in production mode
- [ ] Add ARIA labels to interactive elements — hamburger menu, scanner, notification bell, dialog close buttons
- [ ] Add `role="dialog"` and `aria-modal="true"` to all modal components

## Performance

- [ ] Add pagination to `get_user_collection` and `search_user_collection` — currently returns all rows
- [ ] Add database indexes: `sessions(expires_at)`, `album_loans(borrower_id)`
- [x] Wrap Argon2 hashing in `spawn_blocking()` to avoid blocking the Tokio runtime

## Observability

- [x] Integrate `tracing` crate for structured logging (replace `eprintln!`)

## Infrastructure

- [x] Add GitHub Actions CI pipeline — fmt, clippy (ssr + wasm), tests
- [x] Enable SQLite WAL journal mode for concurrent read/write access
- [ ] Add SQLite backup strategy — cron job or scheduled task using `.backup` command

## Code quality

- [ ] Break up large page components — `album_detail.rs` and `series_detail.rs` into sub-components
- [ ] Extract shared error banner component (duplicated in `login_dialog.rs` and `settings.rs`)
- [ ] Use `COUNT(DISTINCT ua.album_id)` consistently across all collection queries (`series.rs:495` missing DISTINCT)
- [ ] Remove or populate unused `updated_at` field on users table
- [ ] Complete PWA manifest — add `start_url`, `scope`, `description`

## Email verification on account creation

- [ ] Add email verification token schema — new migration with `email_verification_tokens` table + `email_verified` column on `users`
- [ ] Generate and store verification token on registration — create token in `register()` and mark user as unverified
- [ ] Send verification email — add email sending (e.g. `lettre`) with a `/auth/verify?token=...` link
- [ ] Add /auth/verify endpoint — validate token, mark user verified, redirect
- [ ] Gate authenticated features on verified email — show "verify your email" banner, restrict unverified users
- [ ] Add resend verification email flow — server function + UI for re-sending the email
