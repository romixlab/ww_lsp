//! The WireWeaver API items of a source and the types in them.
//!
//! A `#[ww_trait]` (or `#[ww_api_root]`) trait holds macro items, `property!(ro+observe i_a: Ampere)`,
//! `property!(rw mode: Mode, ModeError)`, `stream!(rx: Chunk)`, `sink!(tx: Chunk)`, and methods,
//! `fn set_limit(limit: Fixed<Ampere, Milli, u16>) -> Result<(), Error>`. Each type in them is looked at, and inside
//! a type that isn't an exact_units one its arguments (`Option<Millis<u32>>`).

use crate::describe::{Aliases, describe};
use crate::{Found, ParseError, format};
use proc_macro2::{Span, TokenTree};
use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{GenericArgument, Ident, ItemTrait, PathArguments, ReturnType, Token, TraitItem, Type};

pub(crate) fn find(src: &str) -> Result<Vec<Found>, ParseError> {
    let file = syn::parse_file(src).map_err(|e| ParseError {
        offset: e.span().byte_range().start,
        message: e.to_string(),
    })?;
    let mut aliases = AliasCollector::default();
    aliases.visit_file(&file);
    let mut traits = TraitCollector::default();
    traits.visit_file(&file);
    let mut found = vec![];
    for t in traits.traits {
        for item in &t.items {
            for (what, ty) in item_types(item) {
                walk(src, &aliases.0, &what, &ty, &mut found);
            }
        }
    }
    found.sort_by_key(|f| f.range.start);
    Ok(found)
}

/// The non-generic `type X = ..;` items anywhere in the file.
#[derive(Default)]
struct AliasCollector(Aliases);

impl Visit<'_> for AliasCollector {
    fn visit_item_type(&mut self, i: &syn::ItemType) {
        if i.generics.params.is_empty() {
            self.0.insert(i.ident.to_string(), (*i.ty).clone());
        }
    }
}

/// The API traits anywhere in the file, in modules too.
#[derive(Default)]
struct TraitCollector {
    traits: Vec<ItemTrait>,
}

impl Visit<'_> for TraitCollector {
    fn visit_item_trait(&mut self, i: &ItemTrait) {
        let api = i.attrs.iter().any(|a| {
            a.path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "ww_trait" || s.ident == "ww_api_root")
        });
        if api {
            self.traits.push(i.clone());
        }
    }
}

/// The types an item names, each with the item's description.
fn item_types(item: &TraitItem) -> Vec<(String, Type)> {
    match item {
        TraitItem::Macro(m) => {
            let Some(kind) = m.mac.path.segments.last().map(|s| s.ident.to_string()) else {
                return vec![];
            };
            if !matches!(kind.as_str(), "property" | "stream" | "sink" | "method") {
                return vec![];
            }
            let Ok((name, types)) = macro_item.parse2(m.mac.tokens.clone()) else {
                return vec![];
            };
            let what = format!("{kind} {name}");
            types.into_iter().map(|t| (what.clone(), t)).collect()
        }
        TraitItem::Fn(f) => {
            let what = format!("fn {}", f.sig.ident);
            let args = f.sig.inputs.iter().filter_map(|a| match a {
                syn::FnArg::Typed(p) => Some((*p.ty).clone()),
                syn::FnArg::Receiver(_) => None,
            });
            let ret = match &f.sig.output {
                ReturnType::Type(_, t) => Some((**t).clone()),
                ReturnType::Default => None,
            };
            args.chain(ret).map(|t| (what.clone(), t)).collect()
        }
        _ => vec![],
    }
}

/// `ro+observe+batched name: Type`, `rw name: Type, ErrorType`, `name: Type`: the name and the types. Access words
/// and flags before the name are skipped up to the first `ident:` (not `::`).
fn macro_item(input: ParseStream<'_>) -> syn::Result<(Ident, Vec<Type>)> {
    while !(input.peek(Ident) && input.peek2(Token![:]) && !input.peek2(Token![::])) {
        if input.is_empty() {
            return Err(input.error("no `name: Type` in the item"));
        }
        input.parse::<TokenTree>()?;
    }
    let name: Ident = input.parse()?;
    input.parse::<Token![:]>()?;
    let mut types = vec![input.parse::<Type>()?];
    while input.parse::<Option<Token![,]>>()?.is_some() && !input.is_empty() {
        types.push(input.parse()?);
    }
    Ok((name, types))
}

/// `ty` if it is an exact_units type, else the types inside it.
fn walk(src: &str, aliases: &Aliases, what: &str, ty: &Type, found: &mut Vec<Found>) {
    if let Some(desc) = describe(ty, aliases) {
        let range = byte_range(ty.span());
        let Some(written) = src.get(range.clone()) else {
            return;
        };
        found.push(Found {
            range,
            item: what.to_string(),
            hint: format::hint(&desc),
            hover: format::hover(&desc, written),
            desc,
        });
        return;
    }
    match ty {
        Type::Path(p) => {
            for seg in &p.path.segments {
                if let PathArguments::AngleBracketed(a) = &seg.arguments {
                    for arg in &a.args {
                        if let GenericArgument::Type(t) = arg {
                            walk(src, aliases, what, t, found);
                        }
                    }
                }
            }
        }
        Type::Tuple(t) => {
            for t in &t.elems {
                walk(src, aliases, what, t, found);
            }
        }
        Type::Array(a) => walk(src, aliases, what, &a.elem, found),
        Type::Slice(s) => walk(src, aliases, what, &s.elem, found),
        Type::Reference(r) => walk(src, aliases, what, &r.elem, found),
        Type::Paren(p) => walk(src, aliases, what, &p.elem, found),
        Type::Group(g) => walk(src, aliases, what, &g.elem, found),
        _ => {}
    }
}

fn byte_range(span: Span) -> std::ops::Range<usize> {
    span.byte_range()
}
