# Project Guidelines

## Code Style
Follow Rust conventions with Actix-web and SQLx. Use meaningful imports grouped by functionality (e.g., Actix components, SQLx types). Reference [src/main.rs](src/main.rs) for server setup and [src/error/errors.rs](src/error/errors.rs) for error handling patterns. Prefer async/await with #[actix_web::main] macro for main function.

## Architecture
Actix-web HTTP server with SQLx for PostgreSQL database interactions. Custom ApiError enum for HTTP status mapping (NotFound=404, InvalidInput=400, etc.). Planned modular structure: models (data structs), repositories (SQL queries), services (business logic), handlers (endpoint functions), routes (URL mapping). Database pool shared via Actix app data. Uses dotenv for environment variables, requiring DATABASE_URL.

## Build and Test
- Build: `cargo build` or `cargo build --release`
- Run: `cargo run` (starts on 127.0.0.1:8080)
- Test: `cargo test` (uses SQLite for isolated tests)
- Docker: `docker build -t client_management_api .` then `docker run -e DATABASE_URL=<url> client_management_api`
- Init without local Rust: `docker run --rm -v $(pwd):/app -w /app rust:latest cargo init`

## Conventions
- Use `Result<T, ApiError>` for meaningful error responses with correct HTTP codes instead of generic `Box<dyn std::error::Error>`
- Async main with `#[actix_web::main]` macro for Tokio runtime
- Environment variables via dotenv; require DATABASE_URL for Postgres
- Validation with validator crate, errors converted to ApiError::InvalidInput
- SQLx pool with max 5 connections; use Pool<Postgres> type
- See [MAIN.md](MAIN.md) for main.rs structure, [ERRORS.md](ERRORS.md) for error patterns, [SQLx.md](SQLx.md) for database setup, [README.md](README.md) for getting started</content>
<parameter name="filePath">/workspaces/payment_rust_evolution/.github/copilot-instructions.md