set shell := ["bash", "-euo", "pipefail", "-c"]

# Build the single-binary release artifact (frontend embedded into the server binary).
build:
    cd frontend && npm ci && npm run build
    cargo build --release -p folder-sync-server
    @echo "Binary: target/release/folder-sync"

# Run the dev loop: axum backend + Vite dev server (with HMR and /api proxy) together.
# Stops both when interrupted (Ctrl-C).
dev *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo run -p folder-sync-server -- {{ARGS}} &
    backend_pid=$!
    trap 'kill "$backend_pid" 2>/dev/null || true' EXIT
    (cd frontend && npm install --no-audit --no-fund >/dev/null && npm run dev)

# Generate test fixture drive folders for local development.
fixtures OUT="fixtures":
    cargo run -p xtask -- gen-fixtures --out {{OUT}}

# Run backend unit tests.
test:
    cargo test --workspace

# Format and lint the Rust workspace.
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
