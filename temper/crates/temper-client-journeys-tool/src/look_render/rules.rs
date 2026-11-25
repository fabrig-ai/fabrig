//! A changed overlay rule touches the elements it styles (temper/docs/design/client/journey-writer.md, 6.2).

use std::collections::BTreeSet;

use crate::{
    host::Host,
    look_compare::{
        Side,
        css::{chunk, opaque, overlay},
        request::Request,
    },
};

pub(super) fn selectors(host: &mut Host, request: &Request, side: Side) -> Result<BTreeSet<String>, String> {
    let path = format!("{}/overlay.css", side.root());
    let from = overlay(host, &request.parent, &path)?;
    let to = overlay(host, &request.commit, &path)?;
    let mut selectors = BTreeSet::new();
    for (key, declarations) in &to.rules {
        if from.rules.get(key) != Some(declarations) {
            selectors.insert(without_states(&key.prelude)?);
        }
    }
    for key in from.rules.keys() {
        if !to.rules.contains_key(key) {
            selectors.insert(without_states(&key.prelude)?);
        }
    }
    Ok(selectors)
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
