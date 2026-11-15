# ww_highlight

WireWeaver syntax highlighting for a docs site built with Zensical or MkDocs Material: a small script that adds classes to WireWeaver words inside the already-highlighted ```` ```rust ```` blocks, its CSS, and the token lists it reads.

- `assets/ww_highlight.js`: the highlighter, run in the browser on every page.
- `assets/ww_highlight.css`: its colours, the theme's own `--md-code-hl-*-color` variables.
- `assets/ww_tokens.json`: the names by kind, written by `ww_lsp tokens --json` from the names ww_lsp's semantic tokens use (exact_units' from its tables).

A docs site vendors copies of the three files. The crate gives them as `JS`, `CSS` and `tokens_json()`, so a site's test can check its copies against the ww_lsp it pins.
