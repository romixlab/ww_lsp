# Changelog

All notable changes to ww_lsp. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/). Feature IDs refer to [FEATURES.md](FEATURES.md).

## [Unreleased]

### Added

- The prototype: a language server beside rust-analyzer for WireWeaver APIs (P2401#2nov-0609). `ww_lsp serve` speaks LSP over stdio (`lsp-server`, full document sync).
- Inlay hints after each exact_units type of a `#[ww_trait]` / `#[ww_api_root]` trait's `property!`, `stream!`, `sink!` and `method!` items and methods: `Fixed<Volt, Milli, i16>` as `mV · i16 · ±32.767 V`, `Q15` as `Q15 · −1..1`, `Millis<u32>` as `ms · u32`, `Point<Celsius, ..>` as `°C` (HINT-1 .. HINT-5).
- Hover on such a type: unit and dimension, the exact scale, step, range, bounds and size on the wire (HOV-1).
- `ww_lsp hints FILE...` and `ww_lsp hover FILE:LINE:COL`: the same analysis for a terminal or a script (CLI-1); `--version` with the git SHA and commit time, shell completion with live `.rs` file values (CLI-2), `ww_lsp usage` from a local per-subcommand record (CLI-3).
- Editor setup for Neovim, Helix and Zed in the README.

