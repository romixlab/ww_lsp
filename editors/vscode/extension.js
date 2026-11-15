// Starts `ww_lsp serve` for Rust files. VS Code extensions run on JavaScript, so this one small file isn't Rust.
const { workspace } = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function semanticTokens() {
  return workspace.getConfiguration('ww_lsp').get('semanticTokens');
}

function activate() {
  const command = workspace.getConfiguration('ww_lsp').get('path');
  client = new LanguageClient(
    'ww_lsp',
    'ww_lsp',
    { command, args: ['serve'] },
    {
      documentSelector: [{ scheme: 'file', language: 'rust' }],
      // VS Code shows the semantic tokens of one provider only, the first that answers with any: off by default,
      // ww_lsp answers none and rust-analyzer's are shown
      middleware: {
        provideDocumentSemanticTokens: (document, token, next) =>
          semanticTokens() ? next(document, token) : undefined,
        provideDocumentRangeSemanticTokens: (document, range, token, next) =>
          semanticTokens() ? next(document, range, token) : undefined,
      },
    },
  );
  return client.start();
}

function deactivate() {
  return client?.stop();
}

module.exports = { activate, deactivate };
