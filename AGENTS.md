# Working on ww_lsp

Guidance for AI agents and contributors. Read this before changing code.

ww_lsp is a language server for WireWeaver APIs that runs beside rust-analyzer. WireWeaver APIs are Rust traits (`#[ww_trait]`, `property!(ro+observe i_a: Fixed<Ampere, Milli, i32>)`), and rust-analyzer can't be extended, so this is a second server attached to the same files: it shows the exact_units types of API items as inlay hints (`mV · i16 · ±32.767 V`) and explains them on hover. It is a prototype (WireWeaver project P2401, task P2401#2nov-0609).

## FEATURES.md is the source of truth

[FEATURES.md](FEATURES.md) lists every feature with its status, known bugs and what is planned, with stable IDs per area (`PAR`, `HINT`, `HOV`, `LSP`, `CLI`).

- Read the relevant area before starting; a "new" idea is often already there.
- Update it in the same commit as the code. Never renumber or reuse an ID; a new one is the next free number of its area (`tpm work claim AREA` in a session).
- Name IDs with a short slug when talking to people (`HINT-6 all-unit-groups`); commits, CHANGELOG and code `TODO`s keep the bare ID.
- No status anywhere else (README checklists, TODO files).

## CHANGELOG.md records every change

Every change a user would notice gets an entry under `## [Unreleased]` in the same commit (`### Added`, `### Changed`, `### Fixed`, `### Removed`), short, with the feature ID in parentheses. Pure refactors don't need one.

## Versions and landing

- Branches never bump versions. A session's work lands through `tpm land`, which rebases it onto main, moves the `[Unreleased]` entries under the new version and bumps it. A branch only adds to `[Unreleased]`.
- No `git push` from a session; `tpm land` takes the branch to main.

## Layout

Cargo workspace, edition 2024, one shared version (`[workspace.package]`), dependencies in the root `[workspace.dependencies]`.

- `crates/ww_lsp_core/` — the analysis, a library without LSP types.
    - `src/parse.rs` — finds the API traits and their items with `syn` (`full`, `visit`), and the types in them.
    - `src/describe.rs` — what an exact_units type stands for: `Fixed`, `Point`, the aliases, units, scales, storages, bounds, origins. Every symbol and number comes from exact_units (`units::by_name`, `Prefix::from_name`, the `Storage` / `Bounds` / `FixedScale` / `Origin` consts); this file only lists the names to look for.
    - `src/format.rs` — the hint and hover text.
    - `src/line_index.rs` — byte offsets to lines and UTF-16 columns and back.
    - `tests/golden.rs`, `tests/fixtures/` — fixture APIs with their golden hints (`.hints`) and hovers (`.hover`).
- `crates/ww_lsp/` — the binary: `serve` (`src/server.rs`, `lsp-server` + `lsp-types`), `hints` and `hover` (`src/output.rs`), completion (`src/complete.rs`), the usage record (`src/usage.rs`), `build.rs` for `--version`.
- `editors/zed/` — the Zed extension (LSP-7), a workspace of its own outside the root one: it builds only for `wasm32-wasip2`, with its own version and `Cargo.lock`. `zed_extension_api` brings its own duplicate build dependencies (`hashbrown`), left as they are.

## Commands

```sh
just test --changed        # a session, before tpm work ready: the crates changed against main
just test                  # everything (the land gate runs it): nextest, then the doctests
just lint                  # clippy -D warnings, fmt --check, cargo machete
just install               # deploy profile, onto PATH through tpm install
just install-zed           # the Zed extension, into Zed's extensions on Linux
cargo run -p ww_lsp -- hints crates/ww_lsp_core/tests/fixtures/psu_api.rs
UPDATE_GOLDEN=1 cargo nextest run -p ww_lsp_core   # rewrite the golden files, then review the diff
```

cargo-nextest and cargo-machete: `cargo install --locked cargo-nextest cargo-machete`.

## Conventions

- `ww_lsp_core` stays free of LSP types: positions are byte offsets; the binary converts them.
- No hand-written tables of symbols, scales or ranges: take them from exact_units. Where exact_units has no table by name (storages, origins, aliases), list the types and read their consts (FEATURES.md HINT-9).
- Hints are short and read like the value; everything else goes into the hover. A type ww_lsp doesn't understand gets no hint, never a guess.
- proc-macro2 keeps every parsed source in a thread-local map for spans; `analyze` clears it after each parse (`invalidate_current_thread_spans`), so take byte ranges inside it and keep no `Span` past it.
- CLI output: colour through `anstream` / `anstyle`, off when not a terminal, with `NO_COLOR` or `--color never`; errors as `error: ...` on stderr and a non-zero exit.

## Dependencies

All with `default-features = false` and only the features in use; no C code in the tree (`cargo tree -e normal,build | rg -- '-sys|cc v|bindgen'` is empty). Touching a manifest includes `cargo tree -d` (no duplicates) and `cargo machete`.

- exact_units comes from git until it is on crates.io, without `std` and `shrink_wrap`, with the unit groups listed in the root `Cargo.toml`.
- `lsp-server` / `lsp-types` are rust-analyzer's own: synchronous over crossbeam channels, no async runtime.

## Tests

- The analysis: fixtures with golden files (`crates/ww_lsp_core/tests/golden.rs`) and small cases inline there.
- The server: a whole session in memory (`Connection::memory`) in `crates/ww_lsp/src/server.rs`.
- The CLI: the built binary in `crates/ww_lsp/tests/cli.rs`, with the `CARGO_*` variables of the test run removed, `NO_COLOR` and `WW_LSP_USAGE=off`.
- Fixtures are our own code, written for the tests; nothing copied from vendors or other projects.

## Commits

Conventional style (`feat(hint): ...`, `fix(lsp): ...`), the feature ID in the message where there is one.
