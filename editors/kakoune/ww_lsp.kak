# ww_lsp beside rust-analyzer, with kakoune-lsp. Source it from kakrc after `eval %sh{kak-lsp}`
hook global BufSetOption filetype=rust %{
    set-option -add buffer lsp_servers %{
        [ww_lsp]
        root_globs = ["Cargo.toml", ".git"]
        args = ["serve"]
    }
}
lsp-inlay-hints-enable global

# Semantic tokens are opt-in: kakoune-lsp keeps the tokens of only one server per buffer, the first to answer, so
# ww_lsp's can replace rust-analyzer's. To try them, uncomment both parts; the faces are kakoune-lsp's to define.
# hook global WinSetOption filetype=rust %{
#     hook window -group semantic-tokens BufReload .* lsp-semantic-tokens
#     hook window -group semantic-tokens NormalIdle .* lsp-semantic-tokens
#     hook window -group semantic-tokens InsertIdle .* lsp-semantic-tokens
# }
# set-option global lsp_semantic_tokens %{
#     [[faces]]
#     face = "meta"
#     token = "macro"
#     [[faces]]
#     face = "keyword"
#     token = "keyword"
#     [[faces]]
#     face = "attribute"
#     token = "decorator"
#     [[faces]]
#     face = "type"
#     token = "struct"
#     [[faces]]
#     face = "type"
#     token = "type"
#     [[faces]]
#     face = "value"
#     token = "enumMember"
#     [[faces]]
#     face = "value"
#     token = "number"
#     [[faces]]
#     face = "type"
#     token = "enum"
# }
