# ww_lsp. `just` lists the recipes; every work repo has test, lint, install and deploy.

default:
    @just --list

# The tests. `just test --changed`: only the crates changed against main and their dependents;
# plain `just test` is the full run CI does
test *flags:
    if [ "{{flags}}" = "--changed" ]; then \
        if command -v tpm >/dev/null; then tpm test changed -- cargo nextest run --workspace; else {{just_executable()}} test-all; fi; \
    else {{just_executable()}} test-all {{flags}}; fi

# Every test through nextest, then the doctests (nextest doesn't run them). `UPDATE_GOLDEN=1 just test` writes the
# golden files of crates/ww_lsp_core/tests/fixtures anew: review the diff
test-all:
    cargo nextest run --workspace
    cargo test --workspace --doc

# Clippy without warnings, formatting, no unused dependency
lint:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check
    cargo fmt --manifest-path editors/zed/Cargo.toml --check
    cargo machete

# Build with the deploy profile and put ww_lsp on PATH through the kept-releases stack (tpm install)
install:
    cargo build --profile deploy --locked -p ww_lsp
    if command -v tpm >/dev/null; then tpm install target/deploy/ww_lsp; else cargo install --locked --path crates/ww_lsp; fi

# Build the Zed extension and put it among Zed's installed ones (Linux); a running Zed picks it up by itself
install-zed:
    rustup target add wasm32-wasip2
    cargo build --manifest-path editors/zed/Cargo.toml --release --locked --target wasm32-wasip2
    mkdir -p {{data_directory()}}/zed/extensions/installed/ww-lsp
    cp editors/zed/extension.toml {{data_directory()}}/zed/extensions/installed/ww-lsp/
    cp editors/zed/target/wasm32-wasip2/release/ww_lsp_zed.wasm {{data_directory()}}/zed/extensions/installed/ww-lsp/extension.wasm

# Package the VS Code extension as a .vsix in editors/vscode
package-vscode:
    cd editors/vscode && npm ci && npx --yes @vscode/vsce package

# Bring this PC up to main: pull, install, check --version against main
deploy:
    if command -v tpm >/dev/null; then tpm repos pull ww_lsp; else git pull --ff-only; fi
    just install
    ww_lsp --version | grep -q "($(git rev-parse --short HEAD)" || { echo "ww_lsp --version does not show $(git rev-parse --short HEAD)"; ww_lsp --version; exit 1; }
    ww_lsp --version
