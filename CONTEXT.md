# CLI Demo Studio - Project Context and Handoff

> Last updated: March 22, 2026  
> Branch at handoff: main

## Change Log Since Last Handoff

### 2026-03-22

- **Frontend development workflow** now split for independent builds:
  - Added `.vscode/tasks.json` with parallel frontend/backend task runners.
  - Frontend: `trunk serve` runs on `:8080` with `APP_API_BASE_URL=http://localhost:3001`.
  - Backend: `cargo run -p server` runs on `:3001` with dev env overrides for CORS/auth redirect.
  - Combined task launches both in parallel without manual rebuild loops.
- **Demo editor rewritten to premium workspace architecture**:
  - `DemoEditorPage` now uses `editor-workspace` with `editor-topbar`, `script-pane`, and `stage-pane`.
  - Steps render as seamless Notion-style `step-block` rows (replacing clunky card feel).
  - Native HTML5 drag-and-drop reorder is active in `StepListEditor`/`StepCard` (with Up/Down fallback controls kept).
  - Added a draggable splitter between script and stage panes so users can resize authoring vs preview space.
  - Mobile fallback stacks panes and disables the splitter for usability.
- **Demo creation UX improved dramatically**:
  - Added unified `CommandBlockEditor` component for paired command+output editing.
  - New "Command Block" button (`btn-primary-light` style) creates both steps at once.
  - Command block editor has branded container with clear input fields and placeholders.
  - Reduces demo authoring friction from 3 steps (add command, add output, edit each) to 1 click + inline edit.
- **Engine mode selector** added to demo settings form:
  - Dropdown to choose Sequential vs FreePlay mode.
  - Clear help text: "Sequential = users execute commands in order, FreePlay = users can run any command at any time".
  - Properly bound to DemoSettings signal for save/publish flow.
- **GitHub avatar loading fixed**:
  - Updated CSP headers to allow images from `https://avatars.githubusercontent.com`.
  - Both embed and dashboard routes now include `img-src 'self' https://avatars.githubusercontent.com`.
  - Hard refresh required after backend restart to clear old CSP cache.

### 2026-03-19

- Auth/login flow hardening and UX fixes:
  - `/api/auth/github` now serves a visible OAuth handoff page with auto-redirect and manual fallback link (no blank page behavior).
  - Sidebar auth UX now includes clearer signed-in profile state, explicit logout action, and improved auth error messaging.
  - Added periodic session re-check + manual "Sync Session" control so post-login state reflects without requiring a hard reload.
- Routing and static serving reliability updates:
  - Dashboard SPA fallback now returns `200` with `index.html` (instead of serving HTML with a `404` status).
  - Added API/auth route resilience for local/proxy edge cases (`/auth` alias and explicit API miss handling).
- Frontend visual redesign landed (terminal/CRT theme):
  - `style/global.css` rewritten to phosphor green on near-black with scanline/flicker effects and terminal-style decorators.
  - `style/tailwind.config.js` updated token palette (`bg`, `panel`, `ink`, `ink-dim`, `muted`, `amber`, `danger`, `border`).
  - App and embed terminal surfaces upgraded (titlebar dots, prompt styles, RUN controls, boot screen shell).
  - Live preview updated to terminal chrome with consistent fallback handling.
- Projects wiring improved in dashboard demos flow:
  - Demos page now supports project assignment at create time.
  - Demos page now supports filtering by project.
  - Demo rows now display project labels (`Unassigned` fallback when absent).
- Projects page now supports deletion:
  - Added per-project delete action with confirmation dialog and in-row deleting state.
  - Uses existing backend `DELETE /api/projects/{id}` endpoint (cascades associated demos per schema).
- Reusable warning dialog component added for destructive actions:
  - New app component for confirmation modals used instead of browser alert/confirm dialogs.
  - Integrated into both project deletion and demo deletion flows.
- Demo project reassignment now supported after creation:
  - `UpdateDemoRequest` now accepts `project_id` updates (assign to a project or clear assignment).
  - Server validates ownership when reassigning to a project.
  - Demos page now includes per-demo project selector to update assignment in place.
- API client base URL derivation simplified:
  - Removed hardcoded dev port mapping in app API base derivation.
  - API base now resolves from browser origin or `APP_API_BASE_URL`.
- Build workflow validated end-to-end:
  - `cargo check -p app -p embed -p server` passes.
  - Dashboard build: `cd crates/app && trunk build --release`.
  - Embed build: `cd crates/embed && trunk build --release --dist /home/yash/Desktop/Coding/cli-demo-studio/dist-embed`.
- Context docs improved for frontend handoff:
  - Added a dedicated frontend reboot brief with IA, UX goals, page contracts, API mapping, and acceptance criteria for a from-scratch redesign.

### 2026-03-18

- Embed delivery flow switched to script bootstrap:
  - Added `static/embed.js`.
  - Publish/embed snippets now use `data-demo` and load from `/static/embed.js`.
- Axum static serving updated for local testing:
  - `/static` serves static assets.
  - `/embed-runtime` serves embed runtime build with SPA fallback.
  - Root fallback serves dashboard SPA build.
- Dashboard/editor UI refresh landed:
  - Left sidebar shell layout.
  - Black-and-white terminal-inspired CSS overhaul.
  - Sticky live preview in demo editor.
  - Collapsible step cards with summary-first display.
- CI stabilization updates:
  - Added explicit Postgres readiness wait (`pg_isready`).
  - Set tests to single-threaded execution (`--test-threads=1`).
  - Hardened brittle integration test assumptions around validation status codes.
- Local build/run guidance corrected and validated:
  - Build dashboard from `crates/app`.
  - Build embed from `crates/embed` to top-level `dist-embed`.
  - Run server from workspace root.
  - Dashboard Trunk rust asset currently sets `data-wasm-opt="0"` to avoid local wasm-opt bulk-memory validator failures.

### How To Update This Section

- Add a new dated subsection at the top (newest first).
- Keep each entry short and outcome-focused (what changed + why it matters).
- Move long implementation detail into the relevant section below and keep this as a scan-friendly summary.

## What This Project Is

An embeddable fake CLI simulator platform. Users can:

1. Create projects.
2. Create demo scripts inside projects.
3. Edit step-based terminal flows (command, output, prompt, spinner, etc.).
4. Publish demos and generate share links.
5. Embed the runtime using iframe/script snippets.
6. Let visitors run scripted commands and receive simulated output.

Asciinema import is planned later and is out of scope for phase 1. Billing/payments are also out of scope for phase 1.

## Architecture

Full-stack Rust workspace with four active crates.

| Crate | Purpose |
| --- | --- |
| crates/server | Axum HTTP API (auth, demos, projects, analytics, billing stubs). |
| crates/app | Leptos WASM dashboard (authoring UI). |
| crates/embed | Leptos WASM embed widget (runtime simulator). |
| crates/shared | Shared models, DTOs, validation for server and WASM clients. |

Database is PostgreSQL via SQLx migrations in migrations/.

## Frontend Reboot Brief (For Frontend-Focused AI)

Use this section as the source of truth when rebuilding the frontend from scratch.

### Reboot Intent

- Current dashboard visual language is intentionally considered disposable.
- Goal: replace the current "demo-looking" UI with a clean, production-grade product UX.
- Keep backend contracts and auth flow intact while redesigning frontend information architecture and presentation.

### Product Goal

Build a web app that lets an authenticated user:

1. Manage projects.
2. Manage demos inside/outside projects.
3. Edit demo content and settings.
4. Publish demos and copy share/embed snippets.
5. View analytics for published demos.

### Must-Keep Backend Contracts

- Auth:
  - `GET /api/auth/github`
  - `GET /api/auth/github/callback`
  - `POST /api/auth/logout`
  - `GET /api/me`
- Projects:
  - `GET /api/me/projects`
  - `POST /api/projects`
  - `PATCH /api/projects/{id}`
  - `DELETE /api/projects/{id}`
- Demos:
  - `GET /api/me/demos`
  - `POST /api/demos`
  - `GET /api/demos/{id}`
  - `PATCH /api/demos/{id}`
  - `DELETE /api/demos/{id}`
  - `POST /api/demos/{id}/publish`
- Analytics:
  - `GET /api/demos/{id}/analytics`
  - `GET /api/demos/{id}/analytics/referrers`
  - `GET /api/demos/{id}/analytics/funnel`

### Required Frontend Information Architecture

1. Dashboard home (redirect to Projects or Demos list).
2. Projects management page.
3. Demos list page with project filter + assignment controls.
4. Demo editor page.
5. Publish page (share URL + embed script).
6. Analytics page.
7. Login/auth state shell experience (signed out, signed in, error).

### Current Functional Requirements To Preserve

- Project deletion must use confirmation modal (not browser alert/confirm).
- Demo deletion must use the same reusable confirmation modal component.
- Demo project can be assigned during create and reassigned later.
- Demo project can be cleared (unassigned) after assignment.
- Unauthorized states must show clear login CTA.

### UX Quality Bar (For New Frontend)

- Replace placeholder/demo visual patterns with production-grade spacing, hierarchy, and interaction states.
- Every async action must have: idle, loading, success, and error feedback.
- Avoid global-only status messages when row-level feedback is possible.
- Ensure destructive actions are visually distinct and gated by explicit confirmation.
- Keep keyboard and screen-reader basics: visible focus state, semantic controls, clear labels.

### Visual Design Constraints

- Full visual reset is allowed (including colors, typography, spacing, components).
- Do not break route structure or backend API contracts.
- Avoid styling choices that reduce legibility/contrast for long-form editing pages.
- Desktop-first authoring UX, but fully functional on mobile breakpoints.

### Suggested Frontend Rebuild Plan

1. Rebuild app shell/navigation and auth status area.
2. Rebuild Projects and Demos list pages (highest traffic surfaces).
3. Rebuild editor workflow (step management + preview + save/publish states).
4. Rebuild analytics and publish pages.
5. Unify design tokens and component primitives last.

### Frontend Definition of Done

- All required routes render with production-ready UX.
- Auth state updates reliably after login/logout.
- CRUD + publish + reassignment flows are fully functional without manual refresh.
- No browser-native alert/confirm dialogs remain for core destructive actions.
- `cargo check -p app -p server -p shared` passes and `trunk build --release` succeeds.

## Current State

### Done and compiling

#### Server

- GitHub OAuth2 login with session storage.
- CRUD for projects and demos.
- Demo publishing flow with slug/version behavior.
- Analytics ingestion and aggregate endpoints.
- Billing status/subscribe stubs only.
- Middleware stack: logging, metrics, security headers, rate limit, CORS, sessions.
- Serves built static assets for local testing and embed distribution:
  - `/static` -> `static/` (includes `static/embed.js` bootstrapper)
  - `/embed-runtime` -> embed build output with SPA fallback to `index.html`
  - fallback SPA route -> dashboard build output

#### Shared contracts

- Demo/project/user models and DTOs are centralized.
- Analytics/common-error DTOs are centralized and used by server handlers.
- Validation utilities are present for slug/color/limits.

#### App dashboard

- Core pages are present: projects, demos, demo editor, publish, analytics, demo view.
- Demo editor page now handles orchestration (load/save/status/preview wiring).
- Step editing logic is modularized into components/step_editors.rs.
- Shell/navigation refactor: left sidebar layout replacing top header.
- Global styling now includes a premium black workspace mode for demo editing:
  - topbar + dual-pane layout (`script-pane` and `stage-pane`)
  - script blocks with badge + drag handle (Notion-style rows)
  - floating stage/terminal container with subtle dot-grid background
  - draggable split-pane control to resize script/preview proportions
- Demo settings include engine mode selector (Sequential vs FreePlay) with helper copy.

#### Embed runtime

- Working command-matching playback engine in components/terminal.rs.
- Enter key support and Run button both trigger command execution.
- Fallback sample demo is still used when fetch fails (explicit TODO/FIXME exists).
- Embed bootstrap flow is now script-based:
  - publisher snippet: `<script src="{origin}/static/embed.js" data-demo="<slug>"></script>`
  - `static/embed.js` injects iframe for `/embed-runtime/index.html?demo_id=...&api_base=...`

### In progress and gaps

| Area | Status | Notes |
| --- | --- | --- |
| Step editors for Prompt/Spinner/CTA/Pause | In progress | Command/Output flow polished; other editors need final UX tuning. |
| Live preview | Partial | Stage UI is premium/floating; runtime is still simplified panel, not full embed mount. |
| Theme/settings editing UX | Partial | Core settings present; advanced theming controls still basic. |
| Analytics charts | Stub | Fetch exists, charts remain basic/stubbed. |
| Integration tests | Minimal | Only simple router health-style tests exist. |
| Security hardening | Partial | CSP still includes unsafe-inline. |
| Embed local workflow docs | Updated | Verified trunk build + Axum serve flow added below. |

## Scope Decisions (Phase 1)

### In scope

- Project/demo CRUD with usable authoring UX.
- Full fake terminal playback in sequential/free-play modes.
- Publish/share/embed flow.
- Keep build green and structure maintainable.

### Out of scope

- Billing/payment implementation.
- Team collaboration/permissions.
- Asciinema import pipeline.
- Distributed cache and full observability overhaul.

## Structural Cleanup Applied

1. Removed unused top-level hello-world entrypoint at src/main.rs.
2. Removed dead placeholder components:
   - crates/app/src/components/step_card.rs
   - crates/app/src/components/theme_editor.rs
3. Removed dead module exports from crates/app/src/components/mod.rs.
4. Simplified cfg structure in crates/embed/src/lib.rs for readability.
5. Modularized editor logic into crates/app/src/components/step_editors.rs and simplified page-level orchestration in crates/app/src/pages/demo_editor.rs.

## Immediate Next Tasks

1. Persist split-pane width in localStorage and restore on editor load.
2. Add visual drop-target affordance during drag-and-drop (insertion line/zone feedback).
3. Implement Prompt/Spinner/CTA/Pause editor UX polish to match new block editor quality.
4. Replace simplified live preview with full runtime mount in stage pane.
5. Add terminal reset/restart control in embed runtime.
6. Add integration tests for auth + project/demo ownership/publish critical path.
7. Decide if wasm-opt should remain disabled for dashboard Trunk build (`data-wasm-opt="0"`) or be re-enabled after toolchain upgrade.

## Key File Map

```text
crates/
  server/src/
    main.rs
    router.rs
    auth.rs
    state.rs
    config.rs
    error.rs
    handlers/
      demos.rs
      projects.rs
      analytics.rs
      common_errors.rs
      billing.rs
      auth.rs
    middleware/
      rate_limit.rs
      security_headers.rs
      metrics.rs
      logging.rs
    services/
      og_image.rs

  shared/src/
    models/
      demo.rs
      analytics.rs
      common_error.rs
      project.rs
      user.rs
    dto/
      demo_dto.rs
      analytics_dto.rs
      common_error_dto.rs
      project_dto.rs
    validation.rs
    error.rs
    services/
      embed_generator.rs

  app/src/
    app.rs
    api.rs
    pages/
      projects.rs
      demos.rs
      demo_editor.rs
      demo_view.rs
      publish.rs
      analytics.rs
      settings.rs
    components/
      live_preview.rs
      step_editors.rs
      embed_code_generator.rs
      charts/

  embed/src/
    lib.rs
    components/
      terminal.rs
    matching.rs
    input_handler.rs
    animation.rs
    api.rs
    messaging.rs
```

## Build Commands

```bash
cargo check -q
cargo build
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Current state: cargo check -q passes.

## Development Workflow (Recommended for Active Development)

Use VS Code tasks for split frontend/backend development with live reload:

```bash
# Terminal 1: Backend runs on :3001
cd /home/yash/Desktop/Coding/cli-demo-studio
. ./.env && FRONTEND_URL=http://localhost:8080 CORS_ALLOWED_ORIGINS=http://localhost:8080,http://localhost:3001 cargo run -p server

# Terminal 2: Frontend runs on :8080 with hot reload
cd /home/yash/Desktop/Coding/cli-demo-studio/crates/app
APP_API_BASE_URL=http://localhost:3001 trunk serve --port 8080
```

**Or use VS Code tasks** (`.vscode/tasks.json`):
- Press `Cmd/Ctrl + Shift + D` → Run and Debug
- Select "Dev: split frontend+backend" to launch both in parallel
- Each runs in its own terminal and rebuilds on file changes

Benefits:
- Frontend rebuilds instantly on save (no manual build)
- Backend runs independently on port 3001
- API calls from 8080 automatically proxy to 3001
- Clean separation means fewer merge conflicts in dev

## Build and Release Workflow

For building production assets:

```bash
# 1) Build dashboard assets
cd crates/app
trunk build index.html --release

# 2) Build embed runtime into top-level dist-embed
cd ../embed
trunk build index.html --release -d ../../dist-embed

# 3) Run backend server (serves /api + /static + /embed-runtime)
cd ../..
cargo run -p server
```

Then open `http://localhost:3001` and use the publish snippet:

```html
<script src="http://localhost:3001/static/embed.js" data-demo="your-demo-slug"></script>
```

Notes:
- Running embed build from `crates/app` with `-d dist-embed` is incorrect for this workspace layout.
- Dashboard `index.html` currently uses `data-wasm-opt="0"` on the Trunk rust asset to avoid wasm-opt bulk-memory validator failures in this environment.

## Previous Workflow (Local Embed Test - Validated)

See "Build and Release Workflow" above. This workspace layout requires:
- Dashboard build from `crates/app`: `trunk build index.html --release`
- Embed build to top-level `dist-embed`: `cd ../embed && trunk build index.html --release -d ../../dist-embed`
- Server runs from workspace root: `cargo run -p server`

The development workflow above is now the recommended approach for active frontend development.

## CI Notes (March 18)

- Workflow now includes explicit Postgres readiness wait (`pg_isready`) before checks/tests.
- Test step uses `--test-threads=1` for stability in integration tests.
- Integration test contract was hardened to avoid brittle assumptions when framework validation can return either `400` or `422`.

## Post-Phase-1 Backlog

1. Asciinema import.
2. Billing/payments.
3. Team collaboration/permissions.
4. Distributed rate limiting/cache.
5. Full observability rollout.
6. Rich analytics charting.
