# Stage 1: Build frontend using Trunk
FROM rust:1.96-alpine AS frontend-builder
RUN apk add --no-cache musl-dev curl tar
RUN rustup target add wasm32-unknown-unknown
RUN curl -L https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-musl.tar.gz | tar -xzf- -C /usr/local/bin
WORKDIR /app
COPY . .
RUN cd frontend && trunk build --release

# Stage 2: Build backend Rust server
FROM rust:1.96-alpine AS backend-builder
RUN apk add --no-cache musl-dev git
WORKDIR /app
COPY . .
RUN cargo build --release --bin backend

# Stage 3: Runner stage
FROM alpine:latest
ENV PORT=4408
EXPOSE $PORT

# Install SearXNG run dependencies and runtime shared libraries
RUN apk add --no-cache \
  python3 \
  py3-pip \
  libstdc++ \
  openssl \
  libffi \
  libxml2 \
  libxslt \
  git \
  bash \
  curl \
  ca-certificates

# Install compilation headers (will be deleted after pip build)
RUN apk add --no-cache --virtual .build-deps \
  python3-dev \
  build-base \
  libffi-dev \
  openssl-dev \
  libxml2-dev \
  libxslt-dev

# Create searxng folders
RUN mkdir -p /usr/local/searxng /etc/searxng

# Set up SearXNG Python environment
WORKDIR /usr/local/searxng
RUN python3 -m venv searxng-venv && \
  /usr/local/searxng/searxng-venv/bin/pip install --upgrade pip && \
  /usr/local/searxng/searxng-venv/bin/pip install wheel setuptools pyyaml lxml

# Copy SearXNG settings override
COPY searxng-settings.yml /etc/searxng/settings.yml

# Clone and install SearXNG
RUN git clone https://github.com/searxng/searxng.git /usr/local/searxng/searxng-src && \
  cd /usr/local/searxng/searxng-src && \
  sed -i 's/ultrasecretkey/'$(openssl rand -hex 32)'/g' /etc/searxng/settings.yml && \
  /usr/local/searxng/searxng-venv/bin/pip install -r requirements.txt && \
  /usr/local/searxng/searxng-venv/bin/pip install --no-build-isolation -e .

# Clean up dev headers to keep image slim
RUN apk del .build-deps

# Copy compiled backend from Stage 2
COPY --from=backend-builder /app/target/release/backend /usr/local/bin/backend

# Copy compiled frontend from Stage 1 to a permanent location outside of possible dev volumes
COPY --from=frontend-builder /app/frontend/dist /var/www/aura

# Set static dir env var pointing to the permanent location
ENV STATIC_DIR=/var/www/aura
ENV PORT=4408
ENV NODE_ENV=production
ENV LOG_DIR=/app/log

# Set execute permissions
RUN chmod +x /usr/local/bin/backend

# Healthcheck on the Rust server status endpoint
HEALTHCHECK --interval=5m CMD curl -f http://localhost:4408/status || exit 1

# Start SearXNG in background and Aura server in foreground
CMD ["/bin/sh", "-c", "(cd /usr/local/searxng/searxng-src && /usr/local/searxng/searxng-venv/bin/python -m searx.webapp > /dev/null 2>&1) & backend"]
