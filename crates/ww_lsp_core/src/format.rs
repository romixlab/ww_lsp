//! The text of a hint and of a hover from a [Desc].
//!
//! A hint is short and reads left to right like the value: the unit with its prefix, the storage, the range for a
//! value narrow enough that the range matters (16 bits or less, or bounded): `mV · i16 · ±32.767 V`. A Q format
//! names itself instead of unit and storage: `Q15 · −1..1`. A point shows its origin's symbol: `°C`. The hover has
//! everything: the exact scale, the step, the range to the last digit, the storage and its size on the wire.

use crate::describe::{Desc, FixedDesc};
use exact_units::{Magnitude, OriginDef, UnitExpr};
use std::fmt::Write as _;

/// The minus sign, as exact_units writes it in symbols (`10⁻³`) and the hints write it in numbers.
const MINUS: char = '−';

/// Joins the parts of a hint.
const SEP: &str = " · ";

/// The inlay hint for a type.
#[must_use]
pub fn hint(desc: &Desc) -> String {
    match desc {
        Desc::Fixed(f) => fixed_hint(f),
        Desc::Point { origin, .. } => origin.symbol.to_string(),
    }
}

fn fixed_hint(f: &FixedDesc) -> String {
    let mut parts = vec![];
    match q_format(f) {
        Some(q) => parts.push(q),
        None => {
            parts.push(scaled_symbol(f));
            parts.push(f.storage.name.to_string());
        }
    }
    if (f.storage.bits <= 16 || f.narrowed())
        && let Some(range) = range_short(f)
    {
        parts.push(range);
    }
    parts.retain(|p| !p.is_empty());
    parts.join(SEP)
}

/// The unit of one step: `mV`, `KiB`; for a scale without a prefix symbol the unit times the factor (`V·10⁻⁴`,
/// `·1/65535` for a ratio, written without the dot).
fn scaled_symbol(f: &FixedDesc) -> String {
    let symbol = f.unit.to_string();
    if !f.scale.symbol.is_empty() || f.scale.magnitude == Magnitude::ONE {
        return format!("{}{symbol}", f.scale.symbol);
    }
    let factor = f.scale.magnitude.to_string();
    if symbol.is_empty() {
        factor.trim_start_matches('·').to_string()
    } else {
        format!("{symbol}{factor}")
    }
}

/// `Q15`, `Q3.12`, `UQ16.16` (with the unit after it unless a ratio) when the scale is `2^-n` of an integer storage.
fn q_format(f: &FixedDesc) -> Option<String> {
    if f.storage.float {
        return None;
    }
    let bits = i32::try_from(f.storage.bits).ok()?;
    let frac = (1..=bits).find(|&n| {
        i8::try_from(-n)
            .is_ok_and(|e| (-63..=0).contains(&e) && f.scale.magnitude == Magnitude::pow2(e))
    })?;
    let signed = f.storage.min < 0;
    let q = match (signed, bits - frac - i32::from(signed)) {
        (true, 0) => format!("Q{frac}"),
        (true, int) => format!("Q{int}.{frac}"),
        (false, int) => format!("UQ{int}.{frac}"),
    };
    let symbol = f.unit.to_string();
    Some(if symbol.is_empty() {
        q
    } else {
        format!("{q} {symbol}")
    })
}

/// The range in the unit itself (no prefix), short: `±32.767 V` for a signed decimal storage that spans its whole
/// range (one step more below than above, as two's complement has), `−1..1` for a Q15, `0..65.535 V`.
fn range_short(f: &FixedDesc) -> Option<String> {
    let (min, max) = f.int_range()?;
    let symbol = base_symbol(f.unit);
    let decimal = f.scale.magnitude.as_pow10().is_some();
    let lo = number(min, &f.scale.magnitude, 4).0;
    let hi = number(max, &f.scale.magnitude, 4).0;
    // ± for a symmetric range, and for the full range of a signed decimal storage (one step more below than
    // above): `±32.767 V` reads better than `−32.768..32.767 V`. A Q format keeps its interval, `−1..1`.
    let twos = decimal && min < 0 && max.checked_add(1).and_then(i128::checked_neg) == Some(min);
    let text = if twos || (min < 0 && min.checked_neg() == Some(max)) {
        format!("±{hi}")
    } else {
        format!("{lo}..{hi}")
    };
    Some(with_symbol(&text, &symbol))
}

fn with_symbol(number: &str, symbol: &str) -> String {
    if symbol.is_empty() {
        number.to_string()
    } else {
        format!("{number} {symbol}")
    }
}

fn base_symbol(unit: &UnitExpr) -> String {
    unit.to_string()
}

/// `steps · scale` as text and whether it is exact: exact when the decimal needs at most 6 digits after the point,
/// else rounded to `sig` significant digits; `−` for the sign.
#[must_use]
pub fn number(steps: i128, scale: &Magnitude, sig: usize) -> (String, bool) {
    let (text, exact) = match exact_decimal(steps, scale) {
        Some(t) => (t, true),
        #[expect(
            clippy::cast_precision_loss,
            reason = "a rounded number is what is shown here"
        )]
        None => (rounded(steps as f64 * scale.to_f64(), sig), false),
    };
    let text = match text.strip_prefix('-') {
        Some(rest) => format!("{MINUS}{rest}"),
        None => text,
    };
    (text, exact)
}

/// [number] for a hover: `≈` before a rounded one.
fn number_marked(steps: i128, scale: &Magnitude) -> String {
    match number(steps, scale, 6) {
        (t, true) => t,
        (t, false) => format!("≈{t}"),
    }
}

/// `steps · num/den` as a decimal, when it ends within 6 digits after the point.
fn exact_decimal(steps: i128, scale: &Magnitude) -> Option<String> {
    let (num, den) = scale.to_ratio()?;
    let (num, den) = (i128::try_from(num).ok()?, i128::try_from(den).ok()?);
    let value = steps.checked_mul(num)?;
    for digits in 0..=6u32 {
        let pow = 10i128.pow(digits);
        let scaled = value.checked_mul(pow)?;
        if scaled % den == 0 {
            let q = scaled / den;
            if digits == 0 {
                return Some(q.to_string());
            }
            let sign = if q < 0 { "-" } else { "" };
            let q = q.unsigned_abs();
            let pow = pow.unsigned_abs();
            let frac = format!("{:0width$}", q % pow, width = digits as usize);
            let frac = frac.trim_end_matches('0');
            return Some(if frac.is_empty() {
                format!("{sign}{}", q / pow)
            } else {
                format!("{sign}{}.{frac}", q / pow)
            });
        }
    }
    None
}

/// `v` to `sig` significant digits, trailing zeros dropped; in e-notation outside 1e-4..1e9.
fn rounded(v: f64, sig: usize) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let mag = v.abs().log10().floor();
    if !(-4.0..9.0).contains(&mag) {
        return format!("{v:.prec$e}", prec = sig.saturating_sub(1));
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "mag is in -4..9"
    )]
    let decimals = (sig as i64 - 1 - mag as i64).max(0) as usize;
    let text = format!("{v:.decimals$}");
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    }
}

/// The hover for a type, Markdown: the type as written, the hint, then one line per fact.
#[must_use]
pub fn hover(desc: &Desc, written: &str) -> String {
    let mut out = format!("```rust\n{written}\n```\n\n**{}**\n\n", hint(desc));
    match desc {
        Desc::Fixed(f) => fixed_facts(&mut out, f, None),
        Desc::Point { origin, fixed } => {
            let reference = if origin.reference.is_empty() {
                String::new()
            } else {
                format!(" from {}", origin.reference)
            };
            let _ = writeln!(
                out,
                "- origin: `{}` ({}), at {} {}{reference}",
                origin.name, origin.symbol, origin.offset, origin.unit
            );
            fixed_facts(&mut out, fixed, Some(origin));
        }
    }
    out
}

/// The facts of a `Fixed`; for a point's number the step and range are the shown numbers, in the origin's symbol
/// (`°C`, each step of the origin's own being `step` of the unit).
fn fixed_facts(out: &mut String, f: &FixedDesc, origin: Option<&OriginDef>) {
    let symbol = f.unit.to_string();
    let unit_scale = f.unit.magnitude().filter(|m| *m != Magnitude::ONE);
    let dim = f.unit.dim().to_string();
    let _ = write!(
        out,
        "- unit: `{}` {}",
        f.unit.name(),
        quoted_or_ratio(&symbol)
    );
    if !dim.is_empty() && dim != "1" && dim != symbol {
        let _ = write!(out, ", {dim}");
    }
    if let Some(m) = unit_scale {
        let _ = write!(
            out,
            ", 1 {symbol} = {} SI",
            m.to_string().trim_start_matches('·')
        );
    }
    out.push('\n');
    let scale = f.scale.magnitude.to_string();
    let scale = if scale.is_empty() {
        "1".into()
    } else {
        scale.trim_start_matches('·').to_string()
    };
    let _ = writeln!(out, "- scale: {scale} (`{}`), exact", f.scale.name);
    let (shown, shown_symbol) = match origin {
        Some(o) => (
            f.scale
                .magnitude
                .checked_div(o.step)
                .unwrap_or(f.scale.magnitude),
            o.symbol.to_string(),
        ),
        None => (f.scale.magnitude, symbol.clone()),
    };
    let _ = writeln!(
        out,
        "- step: {}",
        with_symbol(&number_marked(1, &shown), &shown_symbol)
    );
    if let Some((min, max)) = f.int_range() {
        let _ = writeln!(
            out,
            "- range: {} ..= {} (`{}`, {min} ..= {max} steps)",
            number_marked(min, &shown),
            with_symbol(&number_marked(max, &shown), &shown_symbol),
            f.bounds.name
        );
    } else {
        let _ = writeln!(out, "- range: the float's (`{}`)", f.bounds.name);
    }
    let bytes = f.storage.bits / 8;
    let _ = writeln!(
        out,
        "- storage: `{}`, {} on the wire",
        f.storage.name,
        if bytes == 1 {
            "1 byte".into()
        } else {
            format!("{bytes} bytes")
        }
    );
}

fn quoted_or_ratio(symbol: &str) -> String {
    if symbol.is_empty() {
        "(a ratio, no symbol)".into()
    } else {
        format!("({symbol})")
    }
}
