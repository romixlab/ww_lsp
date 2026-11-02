//! Live completion values: `source <(COMPLETE=bash ww_lsp)` registers it (zsh and fish alike), clap_complete's
//! dynamic engine calls back into the binary.

use clap_complete::CompletionCandidate;
use std::ffi::OsStr;
use std::path::Path;

/// Files over this size are offered without a look inside.
const LOOK_LIMIT: u64 = 512 * 1024;

/// Directories and `.rs` files that continue `current`, each `.rs` with how many exact_units types of API items it
/// has (`3 typed API items`), so the files worth asking about stand out.
pub(crate) fn rust_files(current: &OsStr) -> Vec<CompletionCandidate> {
    let current = current.to_string_lossy();
    let (dir, prefix) = match current.rfind('/') {
        Some(i) => (&current[..=i], &current[i + 1..]),
        None => ("", &*current),
    };
    let Ok(entries) = std::fs::read_dir(if dir.is_empty() { "." } else { dir }) else {
        return vec![];
    };
    let mut out: Vec<CompletionCandidate> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.starts_with(prefix) || (name.starts_with('.') && !prefix.starts_with('.')) {
                return None;
            }
            let path = format!("{dir}{name}");
            let meta = e.metadata().ok()?;
            if meta.is_dir() {
                return Some(
                    CompletionCandidate::new(format!("{path}/")).help(Some("directory".into())),
                );
            }
            if Path::new(&name).extension() != Some(OsStr::new("rs")) {
                return None;
            }
            Some(CompletionCandidate::new(&path).help(Some(look(&path, meta.len()).into())))
        })
        .collect();
    out.sort_by(|a, b| a.get_value().cmp(b.get_value()));
    out
}

fn look(path: &str, len: u64) -> String {
    if len > LOOK_LIMIT {
        return format!("{} KiB, not looked into", len / 1024);
    }
    match std::fs::read_to_string(path).map(|s| ww_lsp_core::analyze(&s)) {
        Ok(Ok(found)) if found.is_empty() => "no typed API items".into(),
        Ok(Ok(found)) if found.len() == 1 => "1 typed API item".into(),
        Ok(Ok(found)) => format!("{} typed API items", found.len()),
        Ok(Err(_)) => "doesn't parse".into(),
        Err(_) => "unreadable".into(),
    }
}
