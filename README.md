# Aura - Privacy-focused AI Search Engine

Aura is a blazing fast, privacy-focused search engine with an integrated AI assistant. It compiles to 100% native Rust using Axum on the backend and Leptos (WebAssembly) on the frontend. It proxies queries through SearXNG to protect privacy, and processes AI queries using a user-configured self-hosted Ollama server.

---

## 🐳 Container Installation



1. Create a `docker-compose.yml` file:

```yaml
version: '3'
services:
  aura:
    image: ubermetroid/aura:latest
    container_name: aura
    restart: unless-stopped
    ports:
      - 4408:4408
    environment:
      - PORT=4408
      - AURA_PIN=1234
      - SITE_TITLE=Aura
      - OLLAMA_BASE_URL=http://localhost:11434
      - OLLAMA_MODEL=llama3
```

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4408`.

### Building the Image Locally

To build the Docker container locally from the source files:

```bash
docker build -t ubermetroid/aura:latest .
```


---

## 📋 Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4408` |
| `SITE_TITLE` | Custom website title rendered in navigation headers and browser tabs. | `Aura` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). Use `*` to allow all origins. | `*` |
| `AURA_PIN` | Optional 4–10 digit PIN (numerical only) to lock access to the interface. Leave empty for public mode. *(Supports fallback `PIN`)* | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header (true/false). | `true` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before locking out the user client IP address. | `5` |
| `OLLAMA_BASE_URL` | Base URL of the self-hosted Ollama server for text inference. | `http://localhost:11434` |
| `OLLAMA_MODEL` | Model ID to use for chat inference on the Ollama server. | `llama3` |
| `STATIC_DIR` | Directory containing static compiled frontend assets served by Axum. | `./frontend/dist` |

---

*Note: This repository was forked from [MiniSearch](https://github.com/felladrin/MiniSearch).*
