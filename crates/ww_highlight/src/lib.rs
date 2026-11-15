//! WireWeaver syntax highlighting for docs sites: the browser highlighter ([JS]), its colours ([CSS]) and the token
//! lists it reads ([tokens_json]), from the names ww_lsp's semantic tokens use. A docs site vendors copies of the
//! three files and checks them against these.

use serde_json::{Value, json};
use ww_lsp_core::{TokenKind, names};

/// `ww_highlight.js`: adds `ww-tok ww-macro`, `ww-mode`, `ww-attr`, `ww-unit`, `ww-prefix` and `ww-wrapper` classes
/// to WireWeaver words in highlighted Rust blocks, reading `ww_tokens.json` from beside itself.
pub const JS: &str = include_str!("../assets/ww_highlight.js");

/// `ww_highlight.css`: the colours of those classes.
pub const CSS: &str = include_str!("../assets/ww_highlight.css");

/// `ww_tokens.json` as checked in, what [tokens_json] writes.
pub const TOKENS_JSON: &str = include_str!("../assets/ww_tokens.json");

/// The key of each kind in `ww_tokens.json`, as `ww_highlight.js` reads it.
fn key(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::ItemMacro => "macros",
        TokenKind::AccessMode => "accessModes",
        TokenKind::Attribute => "attributes",
        TokenKind::Wrapper => "wrappers",
        TokenKind::Alias => "aliases",
        TokenKind::Unit => "units",
        TokenKind::Prefix => "prefixes",
        TokenKind::Origin => "origins",
    }
}

/// `ww_tokens.json`: every name by kind, sorted, pretty-printed with a final newline.
#[must_use]
pub fn tokens_json() -> String {
    let names = names();
    let mut all = serde_json::Map::new();
    all.insert(
        "_meta".into(),
        json!({
            "purpose": "Token lists for ww_highlight.js, by kind.",
            "generator": "ww_lsp tokens --json: WireWeaver's own names from ww_lsp, exact_units' from its tables (units::ALL, types, Prefix::ALL, origins::ALL)."
        }),
    );
    for kind in TokenKind::ALL {
        all.insert(key(kind).into(), json!(names.of(kind)));
    }
    let mut out = serde_json::to_string_pretty(&Value::Object(all))
        .expect("a map of strings always serializes");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The checked-in token file is what [tokens_json] writes; `UPDATE_GOLDEN=1` writes it anew.
    #[test]
    fn tokens_json_is_current() {
        let got = tokens_json();
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/ww_tokens.json");
            std::fs::write(path, &got).unwrap();
            return;
        }
        assert_eq!(
            got, TOKENS_JSON,
            "assets/ww_tokens.json is stale; UPDATE_GOLDEN=1 writes it anew"
        );
    }

    #[test]
    fn every_kind_the_script_reads() {
        let v: Value = serde_json::from_str(&tokens_json()).unwrap();
        for key in [
            "macros",
            "accessModes",
            "attributes",
            "wrappers",
            "prefixes",
            "units",
            "origins",
            "aliases",
        ] {
            assert!(JS.contains(&format!("data.{key}")), "{key}");
            assert!(!v[key].as_array().unwrap().is_empty(), "{key}");
        }
        assert!(v["macros"].as_array().unwrap().contains(&json!("sink")));
        assert!(
            v["accessModes"]
                .as_array()
                .unwrap()
                .contains(&json!("batched"))
        );
    }
}
