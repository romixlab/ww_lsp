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

### Neovim 0.11+

Neovim attaches every enabled server whose filetypes match, so rust-analyzer stays as it is:

```lua
vim.lsp.config('ww_lsp', {
  cmd = { 'ww_lsp', 'serve' },
  filetypes = { 'rust' },
  root_markers = { 'Cargo.toml', '.git' },
})
vim.lsp.enable('ww_lsp')
vim.lsp.inlay_hint.enable(true)
```

With nvim-lspconfig on an older Neovim:

```lua
local configs = require('lspconfig.configs')
if not configs.ww_lsp then
  configs.ww_lsp = {
    default_config = {
      cmd = { 'ww_lsp', 'serve' },
      filetypes = { 'rust' },
      root_dir = require('lspconfig.util').root_pattern('Cargo.toml', '.git'),
    },
  }
end
require('lspconfig').ww_lsp.setup({})
vim.lsp.inlay_hint.enable(true) -- Neovim 0.10
```

`:checkhealth vim.lsp` (or `:LspInfo`) shows both servers attached to a Rust buffer.

### Helix

`~/.config/helix/languages.toml`:

```toml
[language-server.ww_lsp]
command = "ww_lsp"
args = ["serve"]

[[language]]
name = "rust"
language-servers = ["rust-analyzer", "ww_lsp"]
```

and in `~/.config/helix/config.toml`:

```toml
[editor.lsp]
display-inlay-hints = true
```

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
