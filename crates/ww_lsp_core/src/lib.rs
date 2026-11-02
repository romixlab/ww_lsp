//! The analysis behind `ww_lsp`: finds the WireWeaver API items of a Rust source (`#[ww_trait]` and
//! `#[ww_api_root]` traits, their `property!`, `stream!`, `sink!` and `method!` items and their methods), and for each
//! exact_units type in them the text of an inlay hint and of a hover. No LSP types here: positions are byte offsets,
//! [LineIndex] turns them into lines and UTF-16 columns.
//!
//! ```
//! let src = "#[ww_trait]\ntrait Psu {\n    property!(ro v_out: Fixed<Volt, Milli, i16>);\n}\n";
//! let found = ww_lsp_core::analyze(src).unwrap();
//! assert_eq!(found[0].hint, "mV · i16 · ±32.767 V");
//! assert_eq!(&src[found[0].range.clone()], "Fixed<Volt, Milli, i16>");
//! ```

pub mod describe;
pub mod format;
mod line_index;
mod parse;

pub use describe::Desc;
pub use line_index::LineIndex;
use std::ops::Range;

/// An exact_units type in an API item.
#[derive(Debug, Clone)]
pub struct Found {
    /// Where the type is written, bytes.
    pub range: Range<usize>,
    /// The API item it belongs to: `property v_out`, `fn set_limit`.
    pub item: String,
    /// What it stands for.
    pub desc: Desc,
    /// The inlay hint: `mV · i16 · ±32.767 V`.
    pub hint: String,
    /// The hover, Markdown.
    pub hover: String,
}

/// A source that doesn't parse as Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Where, bytes.
    pub offset: usize,
    /// What syn says.
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (byte {})", self.message, self.offset)
    }
}

impl std::error::Error for ParseError {}

/// The exact_units types of the API items in `src`, in source order. A source that doesn't parse (in the middle of
/// typing) is an error; the caller keeps what it had.
pub fn analyze(src: &str) -> Result<Vec<Found>, ParseError> {
    let found = parse::find(src);
    // proc-macro2 keeps every parsed source in a thread-local map for its spans: a server reparsing on every key
    // would grow it without end. The byte ranges are taken, the spans are no longer needed.
    proc_macro2::extra::invalidate_current_thread_spans();
    found
}

/// The innermost type at byte `offset`, for a hover.
#[must_use]
pub fn at(found: &[Found], offset: usize) -> Option<&Found> {
    found
        .iter()
        .filter(|f| f.range.start <= offset && offset <= f.range.end)
        .min_by_key(|f| f.range.len())
}
