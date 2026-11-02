# ww_lsp

A language server for WireWeaver APIs. It runs beside rust-analyzer and shows the exact_units types of API items as readable inlay hints, with the details on hover:

```rust
#[ww_trait]
pub trait Output {
    property!(rw v_set: Fixed<Volt, Milli, i16>);              // mV · i16 · ±32.767 V
    property!(rw i_limit: Fixed<Ampere, Milli, u16, Range<0, 5000>>); // mA · u16 · 0..5 A
    property!(rw ramp: Millis<u32>);                          // ms · u32
    property!(rw trim: Q15);                                  // Q15 · −1..1
    property!(ro+observe heatsink: Point<Celsius, Fixed<Kelvin, Centi, i16>>); // °C
}
```

Hover on a type gives its unit and dimension, the exact scale, the step, the whole range, the bounds and the size on the wire. Every symbol and number comes from [exact_units](https://github.com/romixlab/exact_units) itself; a type ww_lsp doesn't know gets no hint.

A prototype: what works and what is planned is in [FEATURES.md](FEATURES.md).

## Install

```sh
just install        # or: cargo install --locked --path crates/ww_lsp
ww_lsp --version
```

Try it without an editor:

```sh
ww_lsp hints crates/ww_lsp_core/tests/fixtures/psu_api.rs
ww_lsp hover crates/ww_lsp_core/tests/fixtures/psu_api.rs:30:25
source <(COMPLETE=bash ww_lsp)    # shell completion; zsh and fish the same with their name
```

## Editors

The server is `ww_lsp serve` on stdin and stdout. Attach it to Rust files next to rust-analyzer and turn inlay hints on.

The ready-made setup for each editor is in `editors/`:

- **Neovim 0.11+:** copy [`editors/neovim/ww_lsp.lua`](editors/neovim/ww_lsp.lua) to `~/.config/nvim/plugin/`. For Neovim 0.10 with nvim-lspconfig, run [`ww_lsp_lspconfig.lua`](editors/neovim/ww_lsp_lspconfig.lua) from your config. `:checkhealth vim.lsp` shows both servers attached to a Rust buffer.
- **Helix:** add [`languages.toml`](editors/helix/languages.toml) and [`config.toml`](editors/helix/config.toml) from `editors/helix` to the files of the same name in `~/.config/helix/`.
- **Kakoune with kakoune-lsp:** source [`editors/kakoune/ww_lsp.kak`](editors/kakoune/ww_lsp.kak) from `kakrc`, after `eval %sh{kak-lsp}`.
- **Emacs with lsp-mode:** load [`editors/emacs/ww-lsp.el`](editors/emacs/ww-lsp.el) after lsp-mode. Eglot runs one server per buffer, so it can't run ww_lsp beside rust-analyzer.
- **VS Code and Cursor:** run `just package-vscode`, then install `editors/vscode/ww-lsp-0.1.0.vsix` with `Extensions: Install from VSIX...`. The `ww_lsp.path` setting names a binary that is not on PATH.
- **JetBrains IDEs with the LSP4IJ plugin:** add a language server, choose `Import from custom template...` and pick `editors/jetbrains/ww_lsp`. RustRover doesn't use rust-analyzer, so ww_lsp runs beside its own analysis.
- **Zed:** see below.

### Zed

Zed starts only language servers that an extension registers, so ww_lsp comes with a small extension in `editors/zed`. It is not in Zed's extension registry. On Linux:

```sh
just install-zed
```

Elsewhere, run `zed: install dev extension` from the command palette and pick `editors/zed` (it needs Rust through rustup).

The extension runs `ww_lsp serve` from PATH, beside rust-analyzer, without further settings. Inlay hints have to be on, in `settings.json`:

```json
{
  "inlay_hints": { "enabled": true }
}
```

Only for a binary that is not on PATH:

```json
{
  "lsp": {
    "ww_lsp": { "binary": { "path": "/path/to/ww_lsp", "arguments": ["serve"] } }
  }
}
```

## Layout

- `crates/ww_lsp_core`: the analysis, a library without LSP types.
- `crates/ww_lsp`: the binary, the server and the CLI.

[AGENTS.md](AGENTS.md) has the details for contributors.

## License

MIT, see [LICENSE](LICENSE).
