//! What an exact_units type in a source file stands for: its unit, scale, storage and bounds, or its origin for a
//! point. Every name, symbol and number comes from exact_units itself: [units::by_name] for the units,
//! [Prefix::from_name] for the prefixes, [origins::ALL] for the origins, [types::ALIASES] for the aliases (`Millis`,
//! `Q15`), the [Storage] and [FixedScale] consts of the marker types for the rest. What is ours is only which number
//! types to look for: the storages have no table by name in exact_units, so they are listed here as types and their
//! values read from the types.

use exact_units::prefix::{self, FixedScale};
use exact_units::types::{self, AliasDef, AliasParam};
use exact_units::units;
use exact_units::{Magnitude, OriginDef, Prefix, ScaleDef, Storage, UnitExpr, origins};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use syn::{Expr, GenericArgument, Lit, PathArguments, Type, UnOp};

/// An exact_units type, understood.
#[derive(Debug, Clone)]
pub enum Desc {
    /// `Fixed<U, P, T, B>` or an alias of one.
    Fixed(FixedDesc),
    /// `Point<O, Fixed<..>>`: a position against an origin (°C).
    Point {
        /// The origin, with its symbol and offset.
        origin: Box<OriginDef>,
        /// The number inside.
        fixed: FixedDesc,
    },
}

/// The parts of a `Fixed`.
#[derive(Debug, Clone)]
pub struct FixedDesc {
    /// The unit, named or a `Per` / `Times` of named ones.
    pub unit: &'static UnitExpr,
    /// The scale of one step in the unit.
    pub scale: ScaleDesc,
    /// The number type.
    pub storage: StorageDesc,
    /// The bounds marker and the range it allows, in steps.
    pub bounds: BoundsDesc,
}

/// A scale marker: `Milli`, `Pow2<-15>`, `Rational<1, 65535>`.
#[derive(Debug, Clone)]
pub struct ScaleDesc {
    /// The marker's name: `"Milli"`, `"Pow2"`.
    pub name: &'static str,
    /// The exact factor.
    pub magnitude: Magnitude,
    /// As written before a unit's symbol: `"m"`; empty for `One` and the families.
    pub symbol: &'static str,
}

/// A number type a `Fixed` is stored as.
#[derive(Debug, Clone, Copy)]
pub struct StorageDesc {
    /// `"i16"`.
    pub name: &'static str,
    /// Width in bits.
    pub bits: u32,
    /// Smallest value as an integer (0 for floats).
    pub min: i128,
    /// Largest value as an integer (0 for floats).
    pub max: i128,
    /// `f32` and `f64`.
    pub float: bool,
}

/// The bounds of a `Fixed`.
#[derive(Debug, Clone)]
pub struct BoundsDesc {
    /// The marker's name: `"Unbounded"`, `"Saturating"`, `"Range"`.
    pub name: String,
    /// The lowest stored value allowed, `None` for the storage's.
    pub min: Option<i128>,
    /// The highest stored value allowed, `None` for the storage's.
    pub max: Option<i128>,
}

impl FixedDesc {
    /// The lowest and highest stored value (steps), `None` for a float.
    #[must_use]
    pub fn int_range(&self) -> Option<(i128, i128)> {
        if self.storage.float {
            return None;
        }
        Some((
            self.bounds.min.unwrap_or(self.storage.min),
            self.bounds.max.unwrap_or(self.storage.max),
        ))
    }

    /// Whether the bounds are narrower than the storage (a `Range`).
    #[must_use]
    pub fn narrowed(&self) -> bool {
        self.bounds.min.is_some_and(|m| m != self.storage.min)
            || self.bounds.max.is_some_and(|m| m != self.storage.max)
    }
}

/// Type aliases of the same file (`type Vbus = Fixed<Volt, Milli, u16>;`), by name, that [describe] looks through.
pub type Aliases = HashMap<String, Type>;

/// `ty` as an exact_units type, `None` when it isn't one this knows.
#[must_use]
pub fn describe(ty: &Type, aliases: &Aliases) -> Option<Desc> {
    describe_depth(ty, aliases, 0)
}

fn describe_depth(ty: &Type, aliases: &Aliases, depth: u8) -> Option<Desc> {
    // an alias of an alias of ...: a loop in the file's aliases ends here
    if depth > 8 {
        return None;
    }
    let (name, args) = last_segment(ty)?;
    match name.as_str() {
        "Fixed" => fixed(&args, aliases).map(Desc::Fixed),
        "Point" => {
            let origin = origin(args.first()?)?;
            let Desc::Fixed(fixed) = describe_depth(args.get(1)?, aliases, depth + 1)? else {
                return None;
            };
            Some(Desc::Point {
                origin: Box::new(origin),
                fixed,
            })
        }
        _ => {
            if let Some(alias) = types::ALIASES.iter().find(|a| a.name == name) {
                return alias_desc(alias, &args, aliases).map(Desc::Fixed);
            }
            let local = aliases.get(&name)?;
            describe_depth(local, aliases, depth + 1)
        }
    }
}

/// `Fixed<U, P, T>` or `Fixed<U, P, T, B>`.
fn fixed(args: &[Type], aliases: &Aliases) -> Option<FixedDesc> {
    let [u, p, t, rest @ ..] = args else {
        return None;
    };
    let storage = storage(t, aliases)?;
    let bounds = match rest.first() {
        None => BoundsDesc {
            name: "Unbounded".into(),
            min: None,
            max: None,
        },
        Some(b) => bounds(b, &storage)?,
    };
    Some(FixedDesc {
        unit: unit(u)?,
        scale: scale(p)?,
        storage,
        bounds,
    })
}

/// The unit a type names: a marker of exact_units by name, or `Per` / `Times` / `Sqrt` of such.
fn unit(ty: &Type) -> Option<&'static UnitExpr> {
    let (name, args) = last_segment(ty)?;
    match (name.as_str(), args.as_slice()) {
        ("Per", [a, b]) => Some(compound(UnitExpr::Div(unit(a)?, unit(b)?))),
        ("Times", [a, b]) => Some(compound(UnitExpr::Mul(unit(a)?, unit(b)?))),
        ("Sqrt", [a]) => Some(compound(UnitExpr::Sqrt(unit(a)?))),
        (_, []) => units::by_name(&name),
        _ => None,
    }
}

/// A compound unit kept for the life of the process, so that exact_units' [UnitExpr] (which holds its parts as
/// `&'static`) can name it and print its symbol. Each distinct one is made once: the set is bounded by the units the
/// files spell out.
fn compound(expr: UnitExpr) -> &'static UnitExpr {
    static MADE: OnceLock<Mutex<HashMap<String, &'static UnitExpr>>> = OnceLock::new();
    let key = expr.name().to_string();
    let mut made = MADE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    made.entry(key).or_insert_with(|| Box::leak(Box::new(expr)))
}

/// A scale marker: a named prefix (`Milli`, `Kibi`, `One`) or a family with its numbers (`Pow10<-4>`, `Pow2<-15>`,
/// `Rational<1, 65535>`, `FullScale<V, 12>`, `FullScaleMax<V, 16>`).
fn scale(ty: &Type) -> Option<ScaleDesc> {
    let (name, args) = last_segment_args(ty)?;
    let int = |i: usize| args.get(i).and_then(const_int);
    let family = |name: &'static str, magnitude: Magnitude| ScaleDesc {
        name,
        magnitude,
        symbol: "",
    };
    match name.as_str() {
        "One" => Some(of_scale::<prefix::One>()),
        "Pow10" => Some(family(
            "Pow10",
            Magnitude::checked_pow10(i8::try_from(int(0)?).ok()?)?,
        )),
        "Pow2" => {
            let e = i8::try_from(int(0)?).ok()?;
            (-63..=63)
                .contains(&e)
                .then(|| family("Pow2", Magnitude::pow2(e)))
        }
        "Rational" => {
            let (num, den) = (u64::try_from(int(0)?).ok()?, u64::try_from(int(1)?).ok()?);
            Some(family("Rational", Magnitude::checked_ratio(num, den).ok()?))
        }
        "FullScale" | "FullScaleMax" => {
            let GenericArgument::Type(v) = args.first()? else {
                return None;
            };
            let v = scale(v)?.magnitude;
            let bits = u8::try_from(int(1)?)
                .ok()
                .filter(|b| (1..=63).contains(b))?;
            let steps = if name == "FullScale" {
                Magnitude::pow2(i8::try_from(bits).ok()?)
            } else {
                Magnitude::checked_ratio((1u64 << bits) - 1, 1).ok()?
            };
            let name = if name == "FullScale" {
                "FullScale"
            } else {
                "FullScaleMax"
            };
            Some(family(name, v.checked_div(steps).ok()?))
        }
        _ => {
            let p = Prefix::from_name(&name)?;
            Some(ScaleDesc {
                name: p.name(),
                magnitude: p.magnitude()?,
                symbol: p.symbol(),
            })
        }
    }
}

fn of_scale<P: FixedScale>() -> ScaleDesc {
    of_scale_def(&P::DEF)
}

fn of_scale_def(def: &ScaleDef) -> ScaleDesc {
    ScaleDesc {
        name: def.name,
        magnitude: def.scale,
        symbol: def.symbol,
    }
}

fn of_storage<T: Storage>() -> StorageDesc {
    StorageDesc {
        name: T::NAME,
        bits: T::BITS,
        min: T::MIN_INT,
        max: T::MAX_INT,
        float: T::IS_FLOAT,
    }
}

/// The number types of exact_units a WireWeaver API stores a `Fixed` in.
fn storages() -> [StorageDesc; 10] {
    [
        of_storage::<u8>(),
        of_storage::<u16>(),
        of_storage::<u32>(),
        of_storage::<u64>(),
        of_storage::<i8>(),
        of_storage::<i16>(),
        of_storage::<i32>(),
        of_storage::<i64>(),
        of_storage::<f32>(),
        of_storage::<f64>(),
    ]
}

/// A number type by its [Storage::NAME]: `"i16"`.
fn storage_by_name(name: &str) -> Option<StorageDesc> {
    storages().into_iter().find(|s| s.name == name)
}

/// The number type a type names, by its [Storage::NAME].
fn storage(ty: &Type, aliases: &Aliases) -> Option<StorageDesc> {
    let all = storages();
    let (name, _) = last_segment(ty)?;
    all.into_iter().find(|s| s.name == name).or_else(|| {
        // a file's own alias of a number type, one level
        let (local, _) = last_segment(aliases.get(&name)?)?;
        all.into_iter().find(|s| s.name == local)
    })
}

/// A bounds marker: `Unbounded`, `Wrapping`, `Saturating` (the storage's range) or `Range<MIN, MAX>` /
/// `RangeU64<MIN, MAX>`.
fn bounds(ty: &Type, storage: &StorageDesc) -> Option<BoundsDesc> {
    let (name, args) = last_segment_args(ty)?;
    match name.as_str() {
        "Unbounded" | "Wrapping" | "Saturating" => Some(BoundsDesc {
            name,
            min: None,
            max: None,
        }),
        "Range" | "RangeU64" => {
            let (min, max) = (const_int(args.first()?)?, const_int(args.get(1)?)?);
            (storage.min <= min && min <= max && max <= storage.max).then_some(BoundsDesc {
                name,
                min: Some(min),
                max: Some(max),
            })
        }
        _ => None,
    }
}

/// An origin of exact_units: its marker by name (`Celsius`), or `Angle<U>` by its unit.
fn origin(ty: &Type) -> Option<OriginDef> {
    let (name, args) = last_segment(ty)?;
    let unit = match args.as_slice() {
        [] => None,
        [u] => Some(unit(u)?),
        _ => return None,
    };
    origins::ALL
        .iter()
        .find(|o| o.name == name && unit.is_none_or(|u| u == o.unit))
        .copied()
}

/// An alias of a `Fixed` that exact_units declares, with the parameters a source gives: each required one, then
/// those with a default, in order. An alias of a `Complex` is no `Fixed`: `None`.
fn alias_desc(alias: &AliasDef, args: &[Type], aliases: &Aliases) -> Option<FixedDesc> {
    if alias.wrapper != "Fixed" || args.len() > alias.params.len() {
        return None;
    }
    let (mut unit, mut scale, mut storage, mut bounds) = (
        alias.unit,
        alias.scale.as_ref().map(of_scale_def),
        alias.storage.and_then(storage_by_name),
        alias.bounds.map(|name| BoundsDesc {
            name: name.into(),
            min: None,
            max: None,
        }),
    );
    // a parameter left out takes its default, if there is one
    let mut bounds_arg = None;
    for (param, arg) in alias.params.iter().zip(args) {
        match param {
            AliasParam::U => unit = Some(self::unit(arg)?),
            AliasParam::P => scale = Some(self::scale(arg)?),
            AliasParam::T => storage = Some(self::storage(arg, aliases)?),
            AliasParam::B => bounds_arg = Some(arg),
        }
    }
    let storage = storage?;
    if let Some(b) = bounds_arg {
        bounds = Some(self::bounds(b, &storage)?);
    }
    Some(FixedDesc {
        unit: unit?,
        scale: scale?,
        storage,
        bounds: bounds?,
    })
}

/// The last path segment's name and its type arguments: `Fixed` and `[Volt, Milli, i16]`.
fn last_segment(ty: &Type) -> Option<(String, Vec<Type>)> {
    let (name, args) = last_segment_args(ty)?;
    let types = args
        .into_iter()
        .filter_map(|a| match a {
            GenericArgument::Type(t) => Some(t),
            _ => None,
        })
        .collect();
    Some((name, types))
}

/// The last path segment's name and all its generic arguments, consts too: `Pow2` and `[-15]`. A const argument
/// written as a bare name (`Pow2<-15>` parses `-15` as an expression, `Range<0, N>` `N` as a type) comes as either.
fn last_segment_args(ty: &Type) -> Option<(String, Vec<GenericArgument>)> {
    let ty = match ty {
        Type::Group(g) => &*g.elem,
        Type::Paren(p) => &*p.elem,
        t => t,
    };
    let Type::Path(p) = ty else {
        return None;
    };
    if p.qself.is_some() {
        return None;
    }
    let seg = p.path.segments.last()?;
    let args = match &seg.arguments {
        PathArguments::None => vec![],
        PathArguments::AngleBracketed(a) => a.args.iter().cloned().collect(),
        PathArguments::Parenthesized(_) => return None,
    };
    Some((seg.ident.to_string(), args))
}

/// An integer const argument: `3`, `-15`, `{ -15 }`.
fn const_int(arg: &GenericArgument) -> Option<i128> {
    match arg {
        GenericArgument::Const(e) => expr_int(e),
        _ => None,
    }
}

fn expr_int(e: &Expr) -> Option<i128> {
    match e {
        Expr::Lit(l) => match &l.lit {
            Lit::Int(i) => i.base10_parse().ok(),
            _ => None,
        },
        Expr::Unary(u) if matches!(u.op, UnOp::Neg(_)) => expr_int(&u.expr)?.checked_neg(),
        Expr::Block(b) => match b.block.stmts.as_slice() {
            [syn::Stmt::Expr(e, None)] => expr_int(e),
            _ => None,
        },
        Expr::Paren(p) => expr_int(&p.expr),
        Expr::Group(g) => expr_int(&g.expr),
        _ => None,
    }
}
