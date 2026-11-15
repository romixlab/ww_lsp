//! The names WireWeaver syntax highlighting looks for, by kind. WireWeaver's own (its item macros, access modes,
//! attributes and `ww_si` wrappers) are listed here; every exact_units name comes from exact_units' tables:
//! [units::ALL] and [types::UNIT_FORMS], [Prefix::ALL], [origins::ALL], [types::WRAPPERS] and [types::ALIASES].

use exact_units::prefix::{FixedScale, One};
use exact_units::{Prefix, UnitExpr, origins, types, units};

/// The macros of an API item: `property!(ro v_out: ..)`.
pub const ITEM_MACROS: &[&str] = &["property", "method", "stream", "sink"];

/// The access words before a `property!`'s name: `ro+observe`.
pub const ACCESS_MODES: &[&str] = &["ro", "rw", "wo", "observe", "batched"];

/// WireWeaver's attributes: `#[ww_trait]`, `#[derive_shrink_wrap]`, `#[flag]`.
pub const ATTRIBUTES: &[&str] = &[
    "ww_trait",
    "ww_api_root",
    "derive_shrink_wrap",
    "errors",
    "optional",
    "no_unit_hint",
    "default",
    "flag",
    "slot",
    "future_version",
    "wire",
];

/// WireWeaver's own number types around exact_units' (`ww_si`): `Timed<V, Tm>`, `Quantity`, `AnyQuantity`.
pub const WW_WRAPPERS: &[&str] = &["Timed", "Quantity", "AnyQuantity"];

/// What a name is, for highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// `property`, `method`, `stream`, `sink` before their `!`.
    ItemMacro,
    /// `ro`, `rw`, `wo`, `observe`, `batched` in a `property!`.
    AccessMode,
    /// A WireWeaver attribute's name: `ww_trait`.
    Attribute,
    /// A number type that holds the others: `Fixed`, `Point`, `Timed`.
    Wrapper,
    /// An alias of a `Fixed` or `Complex`: `Millis`, `Q15`.
    Alias,
    /// A unit marker, or a unit made of others: `Volt`, `Per`.
    Unit,
    /// A prefix marker: `Milli`, `Kibi`, `One`.
    Prefix,
    /// An origin of a `Point`: `Celsius`.
    Origin,
}

impl TokenKind {
    /// Every kind, in the order of the enum.
    pub const ALL: [Self; 8] = [
        Self::ItemMacro,
        Self::AccessMode,
        Self::Attribute,
        Self::Wrapper,
        Self::Alias,
        Self::Unit,
        Self::Prefix,
        Self::Origin,
    ];

    /// Its place in [ALL](Self::ALL).
    #[must_use]
    pub fn index(self) -> u32 {
        self as u32
    }

    /// Whether the name comes from exact_units (or `ww_si`): a library type, not syntax.
    #[must_use]
    pub fn is_library(self) -> bool {
        matches!(
            self,
            Self::Wrapper | Self::Alias | Self::Unit | Self::Prefix | Self::Origin
        )
    }
}

/// Every name of each kind, sorted, without repeats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names {
    /// [TokenKind::ItemMacro].
    pub item_macros: Vec<&'static str>,
    /// [TokenKind::AccessMode].
    pub access_modes: Vec<&'static str>,
    /// [TokenKind::Attribute].
    pub attributes: Vec<&'static str>,
    /// [TokenKind::Wrapper].
    pub wrappers: Vec<&'static str>,
    /// [TokenKind::Alias].
    pub aliases: Vec<&'static str>,
    /// [TokenKind::Unit].
    pub units: Vec<&'static str>,
    /// [TokenKind::Prefix].
    pub prefixes: Vec<&'static str>,
    /// [TokenKind::Origin].
    pub origins: Vec<&'static str>,
}

impl Names {
    /// The names of `kind`.
    #[must_use]
    pub fn of(&self, kind: TokenKind) -> &[&'static str] {
        match kind {
            TokenKind::ItemMacro => &self.item_macros,
            TokenKind::AccessMode => &self.access_modes,
            TokenKind::Attribute => &self.attributes,
            TokenKind::Wrapper => &self.wrappers,
            TokenKind::Alias => &self.aliases,
            TokenKind::Unit => &self.units,
            TokenKind::Prefix => &self.prefixes,
            TokenKind::Origin => &self.origins,
        }
    }

    /// The kind of a type's name, `None` for a name that is none of exact_units' or WireWeaver's.
    #[must_use]
    pub fn type_kind(&self, name: &str) -> Option<TokenKind> {
        [
            TokenKind::Wrapper,
            TokenKind::Alias,
            TokenKind::Origin,
            TokenKind::Unit,
            TokenKind::Prefix,
        ]
        .into_iter()
        .find(|k| self.of(*k).binary_search(&name).is_ok())
    }
}

/// The names, built once from exact_units' tables.
#[must_use]
pub fn names() -> &'static Names {
    static NAMES: std::sync::OnceLock<Names> = std::sync::OnceLock::new();
    NAMES.get_or_init(build)
}

fn build() -> Names {
    let sorted = |names: Vec<&'static str>| {
        let mut names = names;
        names.sort_unstable();
        names.dedup();
        names
    };
    let units = units::ALL
        .iter()
        .filter_map(|u| match u {
            UnitExpr::Named(def) => Some(def.id.name),
            _ => None,
        })
        .chain(types::UNIT_FORMS.iter().copied());
    // the run-time prefix of no scale is `Unit`; its marker is `One`
    let prefixes = Prefix::ALL
        .iter()
        .filter(|p| **p != Prefix::Unit)
        .map(Prefix::name)
        .chain([One::NAME]);
    Names {
        item_macros: sorted(ITEM_MACROS.to_vec()),
        access_modes: sorted(ACCESS_MODES.to_vec()),
        attributes: sorted(ATTRIBUTES.to_vec()),
        wrappers: sorted(types::WRAPPERS.iter().chain(WW_WRAPPERS).copied().collect()),
        aliases: sorted(types::ALIASES.iter().map(|a| a.name).collect()),
        units: sorted(units.collect()),
        prefixes: sorted(prefixes.collect()),
        origins: sorted(origins::ALL.iter().map(|o| o.name).collect()),
    }
}
