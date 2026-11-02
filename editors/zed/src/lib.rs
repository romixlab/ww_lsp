//! Registers `ww_lsp serve` with Zed for Rust files. The binary is not downloaded: it comes from `lsp.ww_lsp.binary`
//! in the settings, or else from PATH.

use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, Command, LanguageServerId, Result, Worktree};

struct WwLsp;

impl zed::Extension for WwLsp {
    fn new() -> Self {
        WwLsp
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let binary = LspSettings::for_worktree(id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let (path, arguments) =
            binary.map_or((None, None), |binary| (binary.path, binary.arguments));
        let command = path
            .or_else(|| worktree.which("ww_lsp"))
            .ok_or("ww_lsp is not on PATH: install it (https://github.com/romixlab/ww_lsp) or set lsp.ww_lsp.binary.path")?;
        Ok(Command {
            command,
            args: arguments.unwrap_or_else(|| vec!["serve".into()]),
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(WwLsp);
