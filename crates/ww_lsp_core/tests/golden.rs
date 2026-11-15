//! The hints and hovers of the fixtures against their golden files: `fixtures/NAME.rs` beside `NAME.hints` (one
//! line per hint, `LINE:COL  HINT  (ITEM)`, 1-based, at the end of the type) and `NAME.hover` (every hover, in
//! source order). `UPDATE_GOLDEN=1 cargo nextest run` writes them anew; review the diff before committing.

use std::fmt::Write as _;
use std::path::Path;
use ww_lsp_core::LineIndex;

fn check(name: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let src = std::fs::read_to_string(dir.join(format!("{name}.rs"))).unwrap();
    let found = ww_lsp_core::analyze(&src).unwrap();
    let index = LineIndex::new(&src);
    let (mut hints, mut hovers) = (String::new(), String::new());
    for f in &found {
        let (line, col) = index.position(f.range.end);
        writeln!(hints, "{}:{}  {}  ({})", line + 1, col + 1, f.hint, f.item).unwrap();
        writeln!(hovers, "<!-- {} -->\n{}", f.item, f.hover).unwrap();
    }
    for (ext, got) in [("hints", hints), ("hover", hovers)] {
        let path = dir.join(format!("{name}.{ext}"));
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&path, &got).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(
            got,
            want,
            "{} differs; UPDATE_GOLDEN=1 writes it anew",
            path.display()
        );
    }
}

#[test]
fn psu_api() {
    check("psu_api");
}

#[test]
fn not_rust_is_an_error_with_its_place() {
    let err = ww_lsp_core::analyze("#[ww_trait]\ntrait Psu {").unwrap_err();
    // syn points at the brace left open
    assert_eq!(err.offset, 22);
}

#[test]
fn unknown_types_and_plain_traits_have_no_hint() {
    let src = "#[ww_trait]\ntrait A { property!(rw x: Fixed<Furlong, Milli, i16>); property!(rw y: u8); }\n\
               trait B { fn v() -> Fixed<Volt, Milli, i16>; }";
    assert!(ww_lsp_core::analyze(src).unwrap().is_empty());
}

#[test]
fn compound_units_and_families() {
    let src = "#[ww_trait]\ntrait A {\n\
               property!(rw a: Fixed<Per<Volt, Second>, Kilo, i32>);\n\
               property!(rw b: Fixed<Volt, FullScale<Rational<33, 10>, 12>, u16>);\n\
               property!(rw c: Fixed<Second, Pow10<-4>, u8>);\n\
               property!(rw d: Q15<Volt>);\n}";
    let hints: Vec<_> = ww_lsp_core::analyze(src)
        .unwrap()
        .into_iter()
        .map(|f| f.hint)
        .collect();
    assert_eq!(
        hints,
        [
            "kV/s · i32",
            "V·33/40960 · u16 · 0..52.8 V",
            "s·10⁻⁴ · u8 · 0..0.0255 s",
            "Q15 V · −1..1 V"
        ]
    );
}

/// The aliases and origins come from exact_units' tables (HINT-9): those the old hand lists missed too.
#[test]
fn aliases_and_origins_from_exact_units() {
    let src = "#[ww_trait]\ntrait A {\n\
               property!(rw a: Ppm<i16>);\n\
               property!(rw b: PhaseBam16);\n\
               property!(rw c: Bytes<u8, Range<1, 64>>);\n\
               property!(rw d: Point<Angle<Degree>, Fixed<Degree, One, i16>>);\n\
               property!(rw e: Point<GpsEpoch, Millis<u64>>);\n\
               property!(rw f: CQ15);\n\
               property!(rw g: Millis<u8, u8>);\n}";
    let hints: Vec<_> = ww_lsp_core::analyze(src)
        .unwrap()
        .into_iter()
        .map(|f| (f.item, f.hint))
        .collect();
    // a Complex is no Fixed and a parameter too many is no alias: no hint, never a guess
    assert_eq!(
        hints,
        [
            ("property a", "µ · i16 · ±0.032767"),
            ("property b", "UQ0.16 rev · 0..1 rev"),
            ("property c", "B · u8 · 1..64 B"),
            ("property d", "°"),
            ("property e", "s GPS"),
        ]
        .map(|(i, h)| (i.to_string(), h.to_string()))
    );
}
