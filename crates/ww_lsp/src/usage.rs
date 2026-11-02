//! Every run records its subcommand, locally, so commands nobody uses can be found and removed: one line
//! `UNIX_SECONDS<TAB>SUBCOMMAND` appended to `$XDG_STATE_HOME/ww_lsp/usage.tsv` (`~/.local/state/...`). Best effort:
//! a record that can't be written is skipped without a word. `WW_LSP_USAGE=off` turns it off (the tests do).

use crate::Error;
use anstream::println;
use anstyle::{AnsiColor, Style};
use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

fn file() -> Option<PathBuf> {
    let state = std::env::var_os("XDG_STATE_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))?;
    Some(state.join("ww_lsp").join("usage.tsv"))
}

pub(crate) fn record(command: &str) {
    if std::env::var_os("WW_LSP_USAGE").is_some_and(|v| v == "off") {
        return;
    }
    let Some(path) = file() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{secs}\t{command}");
    }
}

/// `ww_lsp usage`: runs per subcommand, most first, every subcommand listed (0 for those never run).
pub(crate) fn show() -> Result<ExitCode, Error> {
    let path = file().ok_or("no HOME or XDG_STATE_HOME: no usage record")?;
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut counts: BTreeMap<&str, usize> = ["serve", "hints", "hover", "usage"]
        .into_iter()
        .map(|c| (c, 0))
        .collect();
    for line in text.lines() {
        if let Some((_, cmd)) = line.split_once('\t') {
            *counts.entry(cmd).or_default() += 1;
        }
    }
    let mut rows: Vec<_> = counts.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let bold = Style::new().bold();
    let zero = AnsiColor::Yellow.on_default();
    for (cmd, n) in rows {
        let style = if n == 0 { zero } else { bold };
        println!("{style}{n:>6}{style:#}  {cmd}");
    }
    let dim = Style::new().dimmed();
    println!("{dim}from {}{dim:#}", path.display());
    Ok(ExitCode::SUCCESS)
}
