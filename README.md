# RustSearch - Privacy-focused AI Search Engine

RustSearch is a blazing fast, privacy-focused search engine with an integrated AI assistant. It compiles to 100% native Rust using Axum on the backend and Leptos (WebAssembly) on the frontend. It proxies queries through SearXNG to protect privacy, and processes AI queries using a user-configured self-hosted Ollama server.

---

## 🐳 Container Installation

### Option 1: Docker Compose (Recommended)

1. Create a `docker-compose.yml` file:

```yaml
version: '3'
services:
  rustsearch:
    image: ubermetroid/rustsearch:latest
    container_name: rustsearch
    restart: unless-stopped
    ports:
      - 4408:4408
    environment:
      - PORT=4408
      - RUSTSEARCH_PIN=1234
      - SITE_TITLE=RustSearch
      - OLLAMA_BASE_URL=http://localhost:11434
      - OLLAMA_MODEL=llama3
```

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4408`.

### Option 2: Docker CLI

Run the following command to start the container:

```bash
docker run -d \
  --name rustsearch \
  --restart unless-stopped \
  -p 4408:4408 \
  -e RUSTSEARCH_PIN=1234 \
  -e OLLAMA_BASE_URL=http://192.168.1.50:11434 \
  -e OLLAMA_MODEL=llama3 \
  ubermetroid/rustsearch:latest
```

---

## 📋 Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4408` |
| `SITE_TITLE` | Custom website title rendered in navigation headers and browser tabs. | `RustSearch` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). Use `*` to allow all origins. | `*` |
| `RUSTSEARCH_PIN` | Optional 4–10 digit PIN (numerical only) to lock access to the interface. Leave empty for public mode. *(Supports fallback `PIN`)* | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header (true/false). | `true` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before locking out the user client IP address. | `5` |
| `OLLAMA_BASE_URL` | Base URL of the self-hosted Ollama server for text inference. | `http://localhost:11434` |
| `OLLAMA_MODEL` | Model ID to use for chat inference on the Ollama server. | `llama3` |
| `STATIC_DIR` | Directory containing static compiled frontend assets served by Axum. | `./frontend/dist` |

---

## 📂 Repository Structure

```
.
├── backend/
│   ├── Cargo.toml
│   └── src
│       ├── auth.rs
│       ├── circuit_breaker.rs
│       ├── config.rs
│       ├── handlers.rs
│       ├── inference.rs
│       ├── main.rs
│       ├── search.rs
│       ├── status.rs
│       └── utils.rs
└── frontend/
    ├── Cargo.toml
    ├── favicon.svg
    ├── index.html
    ├── style.css
    └── src
        ├── api.rs
        ├── header.rs
        ├── i18n
        │   ├── de.rs
        │   ├── en.rs
        │   ├── es.rs
        │   ├── fr.rs
        │   ├── ja.rs
        │   ├── pt.rs
        │   ├── ru.rs
        │   └── zh.rs
        ├── i18n.rs
        ├── lib.rs
        ├── login.rs
        ├── main.rs
        ├── theme.rs
        ├── search_panel.rs
        └── types.rs
```
