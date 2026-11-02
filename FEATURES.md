# ww_lsp features and roadmap

The single source of truth for what ww_lsp does, what is being worked on and what is planned.

## How to use this file

- **Status**: ✅ done, 🚧 in progress or partial (the note says what is missing), 📋 planned, 💡 idea, 🐛 known bug.
- **IDs** (`HINT-3`, `LSP-2`) are stable: never renumber or reuse one. A new item takes the next free number of its area (`tpm work claim AREA` in a session). Use the bare ID in commits, CHANGELOG entries and code `TODO`s (`// TODO(HINT-7): ...`); when talking to people, name it with a short slug (`HINT-6 all-unit-groups`).
- Update an item in the same commit as the code: mark it ✅ with a pointer to the code or test. Items that turn out obsolete move to *Dropped and superseded* with a one-line reason.

## Crates

- `ww_lsp_core`: the analysis, no LSP types. Finds the API items of a Rust source with `syn`, describes their exact_units types from exact_units' own tables and consts, and writes the hint and hover text. Byte offsets, and `LineIndex` for lines and UTF-16 columns.
- `ww_lsp`: the binary. `serve` is the language server (`lsp-server`, `lsp-types`), the other subcommands print the same analysis for a terminal or a script.

## Finding API items (`PAR`)

- ✅ **PAR-1 API traits**: `#[ww_trait]` and `#[ww_api_root]` traits anywhere in a file, in modules too (`crates/ww_lsp_core/src/parse.rs`).
- ✅ **PAR-2 Items and their types**: `property!(ro+observe+batched name: T)`, `property!(rw name: T, E)`, `stream!`, `sink!`, `method!` and trait methods (arguments and return type); inside a type that isn't an exact_units one, its arguments (`Option<Millis<u32>>`, `Result<Fixed<..>, E>`).
- ✅ **PAR-3 Local aliases**: non-generic `type X = ..;` items of the same file are looked through (`type Heatsink = Point<Celsius, ..>`).
- 🚧 **PAR-4 A file that doesn't parse**: syn needs the whole file to parse; while it doesn't (in the middle of typing), the server keeps the hints of the last text that did, at their old positions. 📋 An error-tolerant parse (ra_ap_syntax) keeps them in place.
- 📋 **PAR-5 Reuse wire_weaver's crate walker**: it already reads ww_trait items and resolves their types across crates and `use` paths; ww_lsp names a type by its last path segment and knows only the file it sees. Types from other files and crates (an `api` crate's own aliases and unit markers) need the walker.

## Inlay hints (`HINT`)

- ✅ **HINT-1 Fixed**: `Fixed<U, P, T, B>` as unit with prefix, storage and, for 16 bits or less or a `Range`, the range in the unit: `mV · i16 · ±32.767 V`, `mA · u16 · 0..5 A`. ± for a symmetric range and for the full range of a signed storage at a decimal scale. A scale without a prefix symbol is written as a factor: `s·10⁻⁴`, `V·33/40960` (`crates/ww_lsp_core/src/format.rs`).
- ✅ **HINT-2 Q formats**: a `2⁻ⁿ` scale of an integer storage names itself, `Q15 · −1..1`, `Q3.12 A · −8..8 A`, `UQ24.8 W`.
- ✅ **HINT-3 exact_units aliases**: `Millis<T>`, `Micros<T>`, `Hz<T>`, `Bytes<T>`, `Percent<T>`, `Duty16`, `Bam16`, `Q15<U>`, `Q31<U>`, `UQ16_16<U>` and the others of exact_units, read from the alias types themselves (`describe.rs` `ALIASES`).
- ✅ **HINT-4 Points**: `Point<Celsius, Fixed<..>>` as the origin's symbol, `°C`.
- ✅ **HINT-5 Units**: every named unit of the enabled groups by its marker name (exact_units `units::by_name`), and `Per`, `Times`, `Sqrt` of them (`Per<Volt, Second>` as `kV/s` at `Kilo`); scales `One`, the SI and binary prefixes, `Pow10<E>`, `Pow2<E>`, `Rational<N, D>`, `FullScale<V, BITS>`, `FullScaleMax<V, BITS>`; bounds `Unbounded`, `Wrapping`, `Saturating`, `Range`, `RangeU64`. Unknown types get no hint.
- 📋 **HINT-6 All unit groups**: the build enables time, electrical, mechanical, angle, thermal, dimensionless and information; chemical, photometric, radiation, counting and customary units get no hint yet.
- 📋 **HINT-7 Ratios with their sign**: `Percent<u8>` shows `10⁻²` where exact_units writes `%` (its ratio suffixes `%`, `‰`, `ppm`, `ppb` are public only under `defmt`).
- 💡 **HINT-8 Niches**: `0xFE error, 0xFF n/a` for a value with niche codes.
- 💡 **HINT-9 Tables by name in exact_units**: the storages, origins and aliases are listed in `describe.rs` as types (their values come from exact_units' consts); `origins::ALL` and an alias table in exact_units would drop the lists here.
- 📋 **HINT-10 Number types of the wire**: `UNib32`, `UVlq32`, `UVlq64`, `Nibble` and the bit fields (`U12`, `I5`) need exact_units' `shrink_wrap` feature.

## Hover (`HOV`)

- ✅ **HOV-1 Facts of a type**: the type as written, the hint, unit with name and dimension, the unit's factor to SI, the exact scale, step, range to the last digit (`≈` where rounded), bounds, storage and its size on the wire; a point's step and range in the origin's symbol.
- 💡 **HOV-2 The item's docs and access**: `ro+observe+batched`, the error type, the doc comment.

## Language server (`LSP`)

- ✅ **LSP-1 stdio server**: `ww_lsp serve` (and `--stdio`), `lsp-server` + `lsp-types`, synchronous, full document sync, UTF-16 positions (`crates/ww_lsp/src/server.rs`, the in-memory session test there).
- ✅ **LSP-2 Inlay hints and hover**: hints after each type with the hover as tooltip; hover with the type's range.
- 📋 **LSP-3 Diagnostics**: unit mistakes (a `Fixed` whose unit or scale exact_units doesn't know, a range that doesn't fit the storage, a `Product` scale in an API).
- 📋 **LSP-4 Completion**: unit, prefix and origin names from exact_units' tables inside `Fixed<..>` and `Point<..>`.
- 📋 **LSP-5 Shared plumbing with napier_lang**: one LSP base (document store, line index, request loop) for ww_lsp and napier_lang's server (P2525#31oct-2254b, P2525#31oct-2254d).
- 💡 **LSP-6 Incremental sync**: full sync is fine for API files of a few hundred lines.
- ✅ **LSP-7 Zed extension**: Zed starts only servers an extension registers; `editors/zed` runs `ww_lsp serve` for Rust from PATH or `lsp.ww_lsp.binary`, `just install-zed` installs it. Not in Zed's registry. No automated test yet.
- ✅ **LSP-8 Editor configs**: ready-made setup in `editors/` for Neovim, Helix, Kakoune, Emacs (lsp-mode), VS Code and JetBrains (LSP4IJ). Neovim 0.11+ and VS Code were run by hand and returned hints. The rest are not run yet: Neovim 0.10, Helix, Kakoune, Emacs and JetBrains. No automated test yet.

## CLI (`CLI`)

- ✅ **CLI-1 hints and hover**: `ww_lsp hints FILE...` (`FILE:LINE:COL  HINT  (ITEM)`, 1-based), `ww_lsp hover FILE:LINE:COL`; exit 1 for a file that isn't Rust or no type at the position (`crates/ww_lsp/tests/cli.rs`).
- ✅ **CLI-2 Version and completion**: `--version` with git SHA and commit time (`build.rs`); `source <(COMPLETE=bash ww_lsp)` for bash, zsh and fish, `.rs` files offered with how many typed API items they have. 📋 wire_weaver's bash adapter for paths with spaces and quotes.
- ✅ **CLI-3 Usage record**: each run appends its subcommand to `$XDG_STATE_HOME/ww_lsp/usage.tsv`, `ww_lsp usage` counts them; `WW_LSP_USAGE=off` turns it off.

## Dropped and superseded

None yet.
