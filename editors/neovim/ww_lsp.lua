-- ww_lsp beside rust-analyzer, for Neovim 0.11+. Copy to ~/.config/nvim/plugin/ww_lsp.lua
vim.lsp.config('ww_lsp', {
  cmd = { 'ww_lsp', 'serve' },
  filetypes = { 'rust' },
  root_markers = { 'Cargo.toml', '.git' },
})
vim.lsp.enable('ww_lsp')
vim.lsp.inlay_hint.enable(true)
