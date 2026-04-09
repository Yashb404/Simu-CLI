# Hosting and Continuous Delivery

This repository is prepared for hosted development with a robust CI pipeline and recurring smoke checks.

## Fastest Deploy Path (Render)

1. Push this branch to GitHub.
2. In Render, create a Blueprint from this repo using `render.yaml`.
3. Set required env vars in Render dashboard:
	- `DATABASE_URL`
	- `GITHUB_CLIENT_ID`
	- `GITHUB_CLIENT_SECRET`
	- `SESSION_SECRET` (must be 64 hex chars)
	- `API_URL` (your public Render URL)
	- `FRONTEND_URL` (same public Render URL)
4. Deploy and verify `GET /api/health` returns `OK`.

## What is included

- `Dockerfile`: builds app, embed runtime, and server in one image.
- `render.yaml`: Render blueprint for one-click service provisioning.
- `.dockerignore`: trims Docker build context.
- `.env.example`: required and optional runtime configuration values.
- `.github/workflows/ci.yml`: quality gates for formatting, linting, tests, native/wasm checks, and release builds.
- `.github/workflows/hosted-smoke.yml`: scheduled checks against your hosted app.
- `.github/hosted-base-url.txt`: base URL used by scheduled smoke checks.

## CI behavior

The CI workflow verifies:

1. `cargo fmt --check`
2. `cargo clippy -D warnings` for server/shared
3. `cargo check` for native crates and wasm targets
4. `trunk build` for dashboard and embed assets
5. `cargo build --release -p server`
6. `cargo test -p server -p shared -- --test-threads=1`

## Smoke checks

The smoke workflow runs every 30 minutes and validates:

- `GET /api/health`
- `GET /`
- `GET /static/embed.js`

Configure it by replacing the first line in `.github/hosted-base-url.txt` with your deployed base URL.

You can also run it manually with `workflow_dispatch` and pass a temporary `base_url` input.

## Runtime env requirements

Set these in your hosting provider:

- `DATABASE_URL`
- `GITHUB_CLIENT_ID`
- `GITHUB_CLIENT_SECRET`
- `SESSION_SECRET` (64 hex chars)
- `API_URL` (or rely on `RENDER_EXTERNAL_URL` fallback)
- `FRONTEND_URL` (or rely on `RENDER_EXTERNAL_URL` fallback)

Recommended:

- `CORS_ALLOWED_ORIGINS`
- `SESSION_COOKIE_SECURE=true`
- `RUST_LOG=server=info,tower_sessions=info`
