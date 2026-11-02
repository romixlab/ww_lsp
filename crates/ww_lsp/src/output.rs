//! `ww_lsp hints` and `ww_lsp hover`: the analysis printed for a terminal or a script.

use crate::Error;
use anstream::{eprintln, println};
use anstyle::{AnsiColor, Style};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use ww_lsp_core::{Found, LineIndex};

const PATH: Style = AnsiColor::Cyan.on_default();
const HINT: Style = AnsiColor::Green.on_default().bold();
const DIM: Style = Style::new().dimmed();
const ERROR: Style = AnsiColor::Red.on_default().bold();

/// `error: <message>` on stderr.
pub(crate) fn error(message: &str) {
    eprintln!("{ERROR}error:{ERROR:#} {message}");
}

fn read(path: &Path) -> Result<(String, Vec<Found>), Error> {
    let src =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let found = ww_lsp_core::analyze(&src).map_err(|e| {
        let (line, col) = LineIndex::new(&src).position(e.offset);
        format!(
            "{}:{}:{}: not Rust: {}",
            path.display(),
            line + 1,
            col + 1,
            e.message
        )
    })?;
    Ok((src, found))
}

/// One line per hint, `FILE:LINE:COL  HINT  (ITEM)`, at the end of the type as the editor shows it. A file that
/// doesn't parse is an error on stderr and exit 1, after the other files.
pub(crate) fn hints(files: &[PathBuf]) -> Result<ExitCode, Error> {
    let mut failed = false;
    for path in files {
        let (src, found) = match read(path) {
            Ok(r) => r,
            Err(e) => {
                error(&e.to_string());
                failed = true;
                continue;
            }
        };
        let index = LineIndex::new(&src);
        for f in found {
            let (line, col) = index.position(f.range.end);
            println!(
                "{PATH}{}:{}:{}{PATH:#}  {HINT}{}{HINT:#}  {DIM}({}){DIM:#}",
                path.display(),
                line + 1,
                col + 1,
                f.hint,
                f.item
            );
        }
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// The hover at `FILE:LINE:COL` (1-based); exit 1 with a note when there is no exact_units type there.
pub(crate) fn hover(at: &str) -> Result<ExitCode, Error> {
    let bad = || format!("`{at}`: expected FILE:LINE:COL, 1-based (src/lib.rs:24:30)");
    let mut parts = at.rsplitn(3, ':');
    let (Some(col), Some(line), Some(file)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(bad().into());
    };
    let (Ok(line), Ok(col)) = (line.parse::<u32>(), col.parse::<u32>()) else {
        return Err(bad().into());
    };
    if line == 0 || col == 0 {
        return Err(bad().into());
    }
    let (src, found) = read(Path::new(file))?;
    let offset = LineIndex::new(&src).offset(line - 1, col - 1);
    match ww_lsp_core::at(&found, offset) {
        Some(f) => {
            println!("{}", f.hover.trim_end());
            Ok(ExitCode::SUCCESS)
        }
        None => {
            eprintln!("{DIM}no exact_units type of a #[ww_trait] item at {at}{DIM:#}");
            Ok(ExitCode::FAILURE)
        }
    }
}
