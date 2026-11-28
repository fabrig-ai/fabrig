//! A changed overlay rule touches the elements it styles (temper/docs/design/client/journey-writer-requests.md, 1.2).

use std::collections::{BTreeMap, BTreeSet};

use skein_browser::boundary::Node;

use crate::{
    host::Host,
    look_compare::{
        Side,
        css::{Declarations, chunk, opaque, overlay},
        request::Request,
    },
};

pub(super) type Selectors = BTreeMap<String, BTreeSet<String>>;
pub(super) type Names<'a> = BTreeSet<&'a str>;
pub(super) type Selected<'a> = BTreeMap<Node, Names<'a>>;

pub(super) fn selectors(host: &mut Host, request: &Request, side: Side) -> Result<Selectors, String> {
    let path = format!("{}/overlay.css", side.root());
    let from = overlay(host, &request.parent, &path)?;
    let to = overlay(host, &request.commit, &path)?;
    let mut selectors = Selectors::new();
    for (sheet, other) in [(&to, &from), (&from, &to)] {
        for (key, declarations) in &sheet.rules {
            if other.rules.get(key) != Some(declarations) {
                add(&mut selectors, &key.prelude, declarations)?;
            }
        }
    }
    Ok(selectors)
}

fn add(selectors: &mut Selectors, prelude: &str, declarations: &Declarations) -> Result<(), String> {
    let names = selectors.entry(without_states(prelude)?).or_default();
    names.extend(declarations.keys().cloned());
    Ok(())
}

fn without_states(source: &str) -> Result<String, String> {
    let mut selector = String::with_capacity(source.len());
    let mut copy_from = 0;
    let mut at = 0;
    while let Some(c) = source[at..].chars().next() {
        if let Some(end) = opaque(source, at)? {
            at = end;
            continue;
        }
        let start = at;
        at += c.len_utf8();
        if c == '[' {
            let (_, close, tail) = chunk(&source[at..], b"]")?;
            if close.is_none() {
                return Err("unfinished attribute selector".into());
            }
            at = source.len() - tail.len();
        } else if c == ':' {
            let double = source[at..].starts_with(':');
            let from = at + usize::from(double);
            let end = name_end(source, from)?;
            let name = &source[from..end];
            if !name.is_empty() && (double || state(name)) {
                selector.push_str(&source[copy_from..start]);
                at = end;
                if source[end..].starts_with('(') {
                    let (_, close, tail) = chunk(&source[end + 1..], b")")?;
                    if close.is_none() {
                        return Err("unfinished pseudo-element".into());
                    }
                    at = source.len() - tail.len();
                }
                copy_from = at;
            }
        }
    }
    selector.push_str(&source[copy_from..]);
    Ok(selector)
}

fn name_end(source: &str, mut at: usize) -> Result<usize, String> {
    while let Some(c) = source[at..].chars().next() {
        if c == '\\' {
            at = opaque(source, at)?.expect("an escape is opaque");
        } else if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') || !c.is_ascii() {
            at += c.len_utf8();
        } else {
            break;
        }
    }
    Ok(at)
}

fn state(name: &str) -> bool {
    ["hover", "active", "focus", "focus-visible", "focus-within", "before", "after", "first-line", "first-letter"]
        .iter()
        .any(|state| name.eq_ignore_ascii_case(state))
}
