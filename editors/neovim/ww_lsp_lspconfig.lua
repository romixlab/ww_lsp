-- ww_lsp beside rust-analyzer, for Neovim 0.10 with nvim-lspconfig. Run it from your config after nvim-lspconfig loads
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
vim.lsp.inlay_hint.enable(true)
