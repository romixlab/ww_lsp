//! The WireWeaver words of a source, for semantic highlighting: item macros and their access modes, WireWeaver
//! attributes, and exact_units' names in types.
//!
//! A wrapper or an alias (`Fixed`, `Millis`) is marked wherever a type names it; a unit, prefix or origin only among
//! the arguments of one (`Volt` in `Fixed<Volt, Milli, i16>`, `Celsius` in `Point<Celsius, ..>`). A
//! name the file declares a type of its own for (`struct Row`, `enum Item`) is that type, never exact_units'.

use crate::names::{Names, TokenKind, names};
use crate::parse::macro_item;
use crate::{ParseError, parse};
use proc_macro2::{TokenStream, TokenTree};
use std::collections::HashSet;
use std::ops::Range;
use syn::parse::Parser;
use syn::visit::Visit;
use syn::{GenericArgument, Ident, PathArguments, Type};

/// A WireWeaver word in a source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Where it is written, bytes; never more than one line.
    pub range: Range<usize>,
    /// What it is.
    pub kind: TokenKind,
}

/// The WireWeaver words of `src`, in source order. A source that doesn't parse is an error; the caller keeps what it
/// had.
pub fn tokens(src: &str) -> Result<Vec<Token>, ParseError> {
    let found = find(src);
    // as in `analyze`: the spans' thread-local map would grow with every parse
    proc_macro2::extra::invalidate_current_thread_spans();
    found
}

fn find(src: &str) -> Result<Vec<Token>, ParseError> {
    let file = parse::file(src)?;
    let mut own = OwnTypes::default();
    own.visit_file(&file);
    let mut v = Collector {
        names: names(),
        own: own.0,
        tokens: vec![],
    };
    v.visit_file(&file);
    let mut tokens = v.tokens;
    tokens.sort_by_key(|t| (t.range.start, t.range.end));
    tokens.dedup();
    Ok(tokens)
}

/// The names of the types the file declares itself.
#[derive(Default)]
struct OwnTypes(HashSet<String>);

impl Visit<'_> for OwnTypes {
    fn visit_item(&mut self, i: &syn::Item) {
        let ident = match i {
            syn::Item::Struct(s) => Some(&s.ident),
            syn::Item::Enum(e) => Some(&e.ident),
            syn::Item::Union(u) => Some(&u.ident),
            syn::Item::Type(t) => Some(&t.ident),
            syn::Item::Trait(t) => Some(&t.ident),
            _ => None,
        };
        if let Some(ident) = ident {
            self.0.insert(ident.to_string());
        }
        syn::visit::visit_item(self, i);
    }
}

struct Collector {
    names: &'static Names,
    /// The file's own type names: never an exact_units one.
    own: HashSet<String>,
    tokens: Vec<Token>,
}

impl Collector {
    fn push(&mut self, ident: &Ident, kind: TokenKind) {
        self.tokens.push(Token {
            range: ident.span().byte_range(),
            kind,
        });
    }

    /// A type, `inside` the arguments of an exact_units one or not.
    fn ty(&mut self, ty: &Type, inside: bool) {
        match ty {
            Type::Path(p) if p.qself.is_none() => {
                let Some(last) = p.path.segments.last() else {
                    return;
                };
                let name = last.ident.to_string();
                let kind = if self.own.contains(&name) {
                    None
                } else {
                    self.names.type_kind(&name)
                };
                let marked = match kind {
                    Some(TokenKind::Wrapper | TokenKind::Alias) => true,
                    Some(TokenKind::Unit | TokenKind::Prefix | TokenKind::Origin) => inside,
                    _ => false,
                };
                if let (true, Some(kind)) = (marked, kind) {
                    self.push(&last.ident, kind);
                }
                for seg in &p.path.segments {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        for arg in &a.args {
                            if let GenericArgument::Type(t) = arg {
                                self.ty(t, marked || inside && kind.is_some());
                            }
                        }
                    }
                }
            }
            Type::Tuple(t) => t.elems.iter().for_each(|t| self.ty(t, inside)),
            Type::Array(a) => self.ty(&a.elem, inside),
            Type::Slice(s) => self.ty(&s.elem, inside),
            Type::Reference(r) => self.ty(&r.elem, inside),
            Type::Paren(p) => self.ty(&p.elem, inside),
            Type::Group(g) => self.ty(&g.elem, inside),
            other => syn::visit::visit_type(self, other),
        }
    }

    /// The access words of an item macro, up to its `name:`.
    fn access_modes(&mut self, tokens: &TokenStream) {
        let tokens: Vec<TokenTree> = tokens.clone().into_iter().collect();
        for (i, tt) in tokens.iter().enumerate() {
            let TokenTree::Ident(ident) = tt else {
                continue;
            };
            let named =
                matches!(tokens.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':');
            if named {
                return;
            }
            if self
                .names
                .access_modes
                .binary_search(&ident.to_string().as_str())
                .is_ok()
            {
                self.push(ident, TokenKind::AccessMode);
            }
        }
    }
}

impl Visit<'_> for Collector {
    fn visit_attribute(&mut self, a: &syn::Attribute) {
        if let Some(last) = a.path().segments.last()
            && self
                .names
                .attributes
                .binary_search(&last.ident.to_string().as_str())
                .is_ok()
        {
            self.push(&last.ident, TokenKind::Attribute);
        }
    }

    fn visit_trait_item_macro(&mut self, m: &syn::TraitItemMacro) {
        m.attrs.iter().for_each(|a| self.visit_attribute(a));
        let Some(last) = m.mac.path.segments.last() else {
            return;
        };
        let name = last.ident.to_string();
        if self
            .names
            .item_macros
            .binary_search(&name.as_str())
            .is_err()
        {
            return;
        }
        self.push(&last.ident, TokenKind::ItemMacro);
        self.access_modes(&m.mac.tokens);
        if let Ok((_, types)) = macro_item.parse2(m.mac.tokens.clone()) {
            types.iter().for_each(|t| self.ty(t, false));
        }
    }

    fn visit_type(&mut self, t: &Type) {
        self.ty(t, false);
    }
}
