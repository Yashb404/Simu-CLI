# CLI Demo Studio

CLI Demo Studio is an open source platform for creating, publishing, and embedding interactive terminal demos.

It provides:

- A web dashboard to author scripted CLI flows
- A public embed runtime that can be dropped into external sites
- Analytics endpoints for usage and funnel visibility
- A Rust-first architecture with shared contracts across server and client

## Status

Project maturity: active development.

Current focus:

- Editor UX and frontend polish
- Embed runtime behavior quality
- Reliability and test coverage
- Production deployment readiness

## Core Features

- Project and demo management
- Step-based demo authoring (command, output, comment, prompt, spinner, pause, CTA)
- Sequential and free-play engine modes
- Publish workflow with stable references
- Script-based embeds via static bootstrap
- Authenticated dashboard with GitHub OAuth
- Server-side analytics ingestion and reporting endpoints

## Architecture

This is a Rust workspace with four crates:

| Crate | Purpose |
| --- | --- |
| crates/server | Axum API, auth, persistence, static serving, middleware |
| crates/app | Leptos dashboard (WASM) for authoring and management |
| crates/embed | Leptos embed runtime (WASM) for public terminal playback |
| crates/shared | Shared models, DTOs, and validation used across crates |

Database:

- PostgreSQL (SQLx migrations from migrations directory)

## Repository Layout

~~~text
crates/
  app/
  embed/
  server/
  shared/
migrations/
static/
style/
.github/workflows/
~~~

## Tech Stack

- Rust workspace with Cargo
- Axum, Tower, SQLx, Tokio
- Leptos (CSR) for app and embed runtimes
- PostgreSQL
- Trunk for WASM build and local frontend dev serving

## Getting Started

### Prerequisites

- Rust toolchain (rustup)
- Trunk
- Docker and Docker Compose (recommended for local Postgres)

Install Trunk:

~~~bash
cargo install trunk
~~~

Start database:

~~~bash
docker compose up -d db
~~~

### Environment Variables

Create and configure .env in repository root.

Required keys:

- DATABASE_URL
- GITHUB_CLIENT_ID
- GITHUB_CLIENT_SECRET
- SESSION_SECRET (64 hex characters)
- API_URL
- FRONTEND_URL

Optional keys:

- PORT (default 3001)
- CORS_ALLOWED_ORIGINS
- RATE_LIMIT_REQUESTS_PER_MINUTE
- SESSION_TIMEOUT_DAYS
- SESSION_COOKIE_SECURE
- RUST_LOG

Example (development):

~~~dotenv
DATABASE_URL=postgres://postgres:password@localhost:5432/cli_demo_studio
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_client_secret
SESSION_SECRET=replace_with_64_hex_chars
API_URL=http://localhost:3001
FRONTEND_URL=http://localhost:8080
SESSION_COOKIE_SECURE=false
~~~

## Development And CI

Local development uses two processes:

1. Backend: `cargo run -p server`
2. Frontend: `cd crates/app && APP_API_BASE_URL=http://localhost:3001 trunk serve --port 8080`

Recommended validation commands:

~~~bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace
cargo test --workspace -- --test-threads=1
~~~

The GitHub Actions workflow mirrors this split with native Rust checks, wasm builds for `app` and `embed`, and integration tests against Postgres.

## Development Workflow

Run backend and frontend in separate terminals.

Terminal 1 (backend):

~~~bash
cd /cli-demo-studio
. ./.env && FRONTEND_URL=http://localhost:8080 CORS_ALLOWED_ORIGINS=http://localhost:8080,http://localhost:3001 cargo run -p server
~~~

Terminal 2 (dashboard frontend):

~~~bash
cd /crates/app
APP_API_BASE_URL=http://localhost:3001 trunk serve --port 8080
~~~

Alternative:

- Use VS Code tasks from .vscode/tasks.json
- Run Dev: split frontend+backend

## Build and Test

Workspace checks:

~~~bash
cargo check -q
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
~~~

Release-style frontend builds:

~~~bash
cd crates/app
trunk build index.html --release

cd ../embed
trunk build index.html --release -d ../../dist-embed

cd ../..
cargo run -p server
~~~

## Deployment

Render is the supported deployment target.

1. Push the branch to GitHub.
2. In Render, create a Blueprint from `render.yaml`.
3. Set the runtime variables in Render:
   - `DATABASE_URL`
   - `GITHUB_CLIENT_ID`
   - `GITHUB_CLIENT_SECRET`
   - `SESSION_SECRET`
   - `API_URL`
   - `FRONTEND_URL`
4. Recommended optional values:
   - `CORS_ALLOWED_ORIGINS`
   - `SESSION_COOKIE_SECURE=true`
   - `RUST_LOG=server=info,tower_sessions=info`

The repository also includes GitHub Actions for CI, hosted smoke checks, and a Render deploy hook workflow. Configure hosted checks with the `HOSTED_BASE_URL` repository variable and Render deploys with `RENDER_DEPLOY_HOOK_URL` as a secret or variable.

## API Overview

Main route groups:

- Auth: /api/auth/*
- Current user: /api/me
- Projects: /api/projects and /api/me/projects
- Demos: /api/demos and /api/me/demos
- Publish: /api/demos/{id}/publish
- Analytics: /api/demos/{id}/analytics and related subroutes

Health endpoint:

- /api/health

## Embed Usage

Published demos are embedded using a script tag:

~~~html
<script src="http://localhost:3001/static/embed.js" data-demo="your-demo-slug"></script>
~~~

If you update demo steps, re-save and re-publish to ensure the embed points to latest published state.

## Frontend Improvement Guide

You asked for specific frontend help. This is the most impactful sequence to improve quality quickly.

### Priority 1: Interaction Clarity

- Add persistent save status near topbar actions (idle, saving, saved, failed)
- Improve drag and drop affordances with clear insertion marker
- Persist split-pane size in localStorage and restore on load
- Add keyboard support for reordering and focus-visible states

### Priority 2: Editor Information Architecture

- Keep script pane focused on authoring only
- Move advanced settings into collapsible sections with clear defaults
- Group command and output editing as one workflow block
- Add quick step templates for common terminal flows

### Priority 3: Visual Consistency

- Establish spacing scale and type scale tokens
- Normalize card, input, and button states across pages
- Add empty states and loading skeletons for demos and projects
- Ensure contrast and readability for long editing sessions

### Priority 4: Runtime Confidence

- Upgrade stage preview from simplified view to full runtime mount
- Add restart and reset controls in preview
- Show explicit not-found behavior preview in editor
- Add publish-time validation warnings for incomplete scripts

### Priority 5: Reliability and Guardrails

- Add integration tests for create/edit/publish/embed loop
- Add regression tests for command matching behavior
- Harden CSP while preserving local dev websocket needs

## Troubleshooting

### Commands in embed are not matching

- Save Draft and Publish again after editing command steps
- Ensure match pattern is not blank unless intended
- Reload embed host page with hard refresh

### CSP blocks Trunk websocket in development

- Ensure server CSP includes connect-src allowing ws and wss in dev
- Restart backend after policy changes

### Local machine lags during development

- Check memory pressure and swap usage
- Stop unnecessary heavy processes
- Keep only one server process and one trunk process active

## Contributing

Contributions are welcome.

Recommended flow:

1. Fork repository
2. Create feature branch
3. Keep changes scoped and documented
4. Run checks locally
5. Open pull request with clear summary and test notes

Please include:

- What changed
- Why it changed
- How to verify
- Any follow-up work needed

## CI and Deployment

Current workflows in .github/workflows:

- ci.yml
- deploy-render.yml
- hosted-smoke.yml
