# Frontend (crates/app) — local development

Development
-----------
1. Install trunk and wasm32 target if you haven't:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

2. Serve the frontend with Trunk (proxied to the backend):

```bash
# from repo root
cd crates/app
APP_API_BASE_URL=http://localhost:3001 trunk serve --port 8080
```

Notes
-----
- Frontend API calls use the compile-time `API_BASE_URL` / runtime `APP_API_BASE_URL`.
- The Trunk proxy is configured to forward `/api/v1` to the backend. Use `/api/v1` paths in API calls.
- To build a production frontend image, use the repo-level `Dockerfile.frontend` which accepts an `API_BASE_URL` build-arg.
