# Aura — Documentation

**Status:** Aura is currently in active development. The previous
`docs/` directory (16 files, ~2,400 lines) described a React + Mantine
+ Wllama + Vite + TypeScript architecture that **does not exist** in
this repository. It was deleted in 2026-06-26 because every claim in
those docs was inconsistent with the actual code.

This file replaces it with a short, honest description of the real
stack. The GitHub profile currently lists aura under the broader
"server apps" portfolio note that "service layer being rewritten and
aligned with shared-assets" — that is the authoritative status.

---

## What Aura actually is

A self-hosted AI search assistant. Privacy-preserving because it
proxies search queries through **SearXNG** rather than contacting
Google/Bing directly, and AI queries through a self-hosted
**Ollama** server rather than a hosted LLM API.

Two main surfaces:

- **Search results page** — proxies to SearXNG, returns the aggregated
  results.
- **AI chat sidebar** — sends the user's query + search context to a
  local Ollama instance and streams the response.

## Actual stack

| Layer | Tech | Where |
|-------|------|-------|
| Backend HTTP | **Axum 0.7** (not 0.8 like the other apps) | `backend/src/main.rs` |
| Async runtime | **Tokio** (full features) | `backend/Cargo.toml` |
| HTTP client | **Reqwest 0.12** (rustls, no native-tls) | calls SearXNG and Ollama |
| In-memory state | **DashMap 5.5** | rate limits, session cache |
| Auth | **argon2** password hashing + cookies | `backend/src/auth.rs` |
| Middleware | tower-http (`cors`, `compression-full`, `set-header`) | `backend/src/main.rs` |
| Frontend | **Leptos 0.6** (CSR) — **not Yew** | `frontend/src/app.rs` |
| WASM glue | wasm-bindgen 0.2.121 | `frontend/Cargo.toml` |
| Build | **Trunk** (per the profile) | `frontend/Trunk.toml` |
| Container base | Nix-built via `flake.nix` (no Alpine) | `flake.nix` |

## Source of truth

Because the rewritten docs/ hasn't landed yet, read these files directly:

- `backend/src/main.rs` — router, middleware order, port 4408
- `backend/src/inference.rs` — Ollama client + streaming
- `backend/src/search.rs` — SearXNG proxy
- `backend/src/auth.rs` — argon2 password hashing, session cookies
- `frontend/src/app.rs` — Leptos root component
- `frontend/src/components/` — UI components
- `flake.nix` — Nix build, port mapping, Docker image tag
- `docker-compose.yml` — companion SearXNG config, volumes, env vars
- `README.md` — install + run instructions (still accurate)

## What aura is **not**

To be explicit, so future contributors don't get misled:

- **Not React.** Leptos with `csr` feature is the only frontend framework.
- **Not Mantine.** No CSS-in-JS framework; styling is hand-rolled CSS.
- **Not Wllama.** AI runs on a self-hosted Ollama server, not in-browser.
- **Not Vite.** Trunk is the WASM bundler.
- **Not TypeScript.** All code is Rust.
- **Not AI Horde.** No multi-provider AI routing.
- **Not OpenAI.** No external LLM API calls.

## Roadmap for proper documentation

When aura's rewrite stabilizes, these docs will be rewritten in this order:

1. `architecture.md` — request flow diagrams, middleware order
2. `ollama-setup.md` — how to point aura at a self-hosted Ollama
3. `searxng-setup.md` — privacy guarantees + searxng-settings.yml explained
4. `deployment.md` — Docker Compose, Nix flake, Unraid template
5. `configuration.md` — environment variables (PORT, SITE_TITLE, ALLOWED_ORIGINS, etc.)

Until then, **the code is the documentation.** Inline `///` doc comments
on every public function in `backend/src/` are the most reliable source.