;;; ww-lsp.el --- ww_lsp beside rust-analyzer, with lsp-mode -*- lexical-binding: t; -*-

;; Load it after lsp-mode. Eglot runs one server per buffer, so it can't run ww_lsp beside rust-analyzer.

;;; Code:

(require 'lsp-mode)

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("ww_lsp" "serve"))
  :activation-fn (lsp-activate-on "rust")
  :add-on? t
  :server-id 'ww_lsp))

(setq lsp-inlay-hint-enable t)

;; Semantic tokens stay off: lsp-mode colours a buffer with the tokens of one server only (the first workspace), so
;; ww_lsp's would compete with rust-analyzer's. `(setq lsp-semantic-tokens-enable t)' turns them on for both.

(provide 'ww-lsp)
;;; ww-lsp.el ends here
