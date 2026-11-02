# ww_lsp. `just` lists the recipes; every work repo has test, lint, install and deploy.

default:
    @just --list

# The tests. `just test --changed` (a session, before `tpm work ready`): only the crates changed against main and
# their dependents; plain `just test` is the full run of `tpm land`'s gate
test *flags:
    if [ "{{flags}}" = "--changed" ]; then tpm test changed -- cargo nextest run --workspace; else {{just_executable()}} test-all {{flags}}; fi

# Every test through nextest, then the doctests (nextest doesn't run them). `UPDATE_GOLDEN=1 just test` writes the
# golden files of crates/ww_lsp_core/tests/fixtures anew: review the diff
test-all:
    cargo nextest run --workspace
    cargo test --workspace --doc

# Clippy without warnings, formatting, no unused dependency
lint:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check
    cargo machete

# Build with the deploy profile and put ww_lsp on PATH through the kept-releases stack (tpm install)
install:
    cargo build --profile deploy --locked -p ww_lsp
    if command -v tpm >/dev/null; then tpm install target/deploy/ww_lsp; else cargo install --locked --path crates/ww_lsp; fi

# Bring this PC up to main: pull, install, check --version against main
deploy:
    if command -v tpm >/dev/null; then tpm repos pull ww_lsp; else git pull --ff-only; fi
    just install
    ww_lsp --version | grep -q "($(git rev-parse --short HEAD)" || { echo "ww_lsp --version does not show $(git rev-parse --short HEAD)"; ww_lsp --version; exit 1; }
    ww_lsp --version
