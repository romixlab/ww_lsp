# ww_lsp beside rust-analyzer, with kakoune-lsp. Source it from kakrc after `eval %sh{kak-lsp}`
hook global BufSetOption filetype=rust %{
    set-option -add buffer lsp_servers %{
        [ww_lsp]
        root_globs = ["Cargo.toml", ".git"]
        args = ["serve"]
    }
}
lsp-inlay-hints-enable global
