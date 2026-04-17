# mybd

A web application for tracking your comics, manga, and graphic novel collection.

Built with [Leptos](https://leptos.dev/) (Rust full-stack framework), Axum, and SQLite.

## Features

- Search and browse comics catalog
- Track your personal collection
- Lend and sell albums to friends
- Explore authors and series
- Google OAuth and local authentication

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos)
- [Node.js](https://nodejs.org/) (for e2e tests)

### Setup

```sh
cp .env.example .env  # configure your environment
cargo leptos watch
```

### Tests

```sh
cargo test
cargo clippy
npm test  # Playwright e2e tests
```

## License

This project is licensed under the GNU General Public License v3.0 — see [LICENSE](LICENSE) for details.
