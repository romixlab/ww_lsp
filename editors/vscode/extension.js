// Starts `ww_lsp serve` for Rust files. VS Code extensions run on JavaScript, so this one small file isn't Rust.
const { workspace } = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function activate() {
  const command = workspace.getConfiguration('ww_lsp').get('path');
  client = new LanguageClient(
    'ww_lsp',
    'ww_lsp',
    { command, args: ['serve'] },
    { documentSelector: [{ scheme: 'file', language: 'rust' }] },
  );
  return client.start();
}

function deactivate() {
  return client?.stop();
}

module.exports = { activate, deactivate };
