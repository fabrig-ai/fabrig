use std::{collections::BTreeMap, path::Path};

use super::{
    Host, Report, Side, code, read,
    request::{Request, blob},
    text,
};

const SHIPPED: [&str; 2] =
    ["temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-shell-web/task-pages.css"];

pub(super) fn compare(
    host: &mut Host,
    request: &Request,
    root: &Path,
    side: Side,
    report: &mut Report,
) -> Result<(), String> {
    let path = format!("{}/overlay.css", side.root());
    let parent = overlay(host, &request.parent, &path)?;
    let requested = overlay(host, &request.commit, &path)?;
    let mut landed = Sheet::default();
    if side == Side::Web {
        for path in SHIPPED {
            let files = read(host, &root.join(path))?.ok_or_else(|| format!("missing {path}"))?;
            landed.read(text(&files, path)?).map_err(|why| format!("{path}: {why}"))?;
        }
    }
    rule_rows(report, side, &parent, &requested, &landed);
    if side == Side::Web {
        report.row("the order the rules cascade in", "to read");
    }
    Ok(())
}

fn overlay(host: &mut Host, commit: &str, path: &str) -> Result<Sheet, String> {
    let bytes = blob(host, commit, path)?.unwrap_or_default();
    let path = format!("{commit}:{path}");
    let mut sheet = Sheet::default();
    sheet.read(text(&bytes, &path)?).map_err(|why| format!("{path}: {why}"))?;
    Ok(sheet)
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Key {
    prelude: String,
    conditions: Vec<String>,
}

type Declarations = BTreeMap<String, String>;

#[derive(Default)]
struct Sheet {
    rules: BTreeMap<Key, Declarations>,
    at_rules: Vec<(Key, String)>,
    order: Vec<Entry>,
}

enum Entry {
    Rule(Key),
    AtRule(usize),
}

impl Sheet {
    fn read(&mut self, mut source: &str) -> Result<(), String> {
        source = source.strip_prefix('\u{feff}').unwrap_or(source);
        let mut conditions = Vec::new();
        loop {
            let (before, delimiter, after) = chunk(source, b"{};")?;
            let prelude = normalized(before)?;
            source = after;
            if delimiter == Some(b'}') {
                if !prelude.is_empty() || conditions.pop().is_none() {
                    return Err("unexpected }".into());
                }
                continue;
            }
            let Some(delimiter) = delimiter else {
                return if prelude.is_empty() && conditions.is_empty() {
                    Ok(())
                } else {
                    Err("unfinished CSS rule".into())
                };
            };
            if delimiter == b';' && prelude.is_empty() {
                continue;
            }
            if delimiter == b'{' && conditional(&prelude) {
                conditions.push(prelude);
                continue;
            }
            let key = Key { prelude, conditions: conditions.clone() };
            let body = if delimiter == b'{' {
                let (block, end, rest) = chunk(source, b"}")?;
                if end.is_none() {
                    return Err("unfinished CSS block".into());
                }
                source = rest;
                Some(block)
            } else {
                None
            };
            self.add(key, body)?;
        }
    }

    fn add(&mut self, key: Key, body: Option<&str>) -> Result<(), String> {
        if key.prelude.starts_with('@') {
            let body = if let Some(body) = body { format!("{{{}}}", normalized(body)?) } else { ";".into() };
            self.order.push(Entry::AtRule(self.at_rules.len()));
            self.at_rules.push((key, body));
            return Ok(());
        }
        let body = body.filter(|_| !key.prelude.is_empty()).ok_or("expected a selector and declaration block")?;
        if !self.rules.contains_key(&key) {
            self.order.push(Entry::Rule(key.clone()));
        }
        declarations(body, self.rules.entry(key).or_default())
    }
}

fn conditional(prelude: &str) -> bool {
    let end =
        prelude.find(|c: char| !(c == '@' || c == '-' || c == '_' || c.is_alphanumeric())).unwrap_or(prelude.len());
    let name = &prelude[..end];
    name.eq_ignore_ascii_case("@media") || name.eq_ignore_ascii_case("@supports")
}

fn declarations(mut body: &str, declarations: &mut Declarations) -> Result<(), String> {
    while !body.is_empty() {
        let (declaration, _, rest) = chunk(body, b";")?;
        body = rest;
        let (name, separator, value) = chunk(declaration, b":")?;
        let mut name = normalized(name)?;
        if separator.is_none() && name.is_empty() {
            continue;
        }
        if separator.is_none() || name.is_empty() {
            return Err(format!("expected a CSS declaration, found {declaration:?}"));
        }
        if !name.starts_with("--") {
            name.make_ascii_lowercase();
        }
        declarations.insert(name, normalized(value)?);
    }
    Ok(())
}

// A delimiter is visible only outside strings, escapes, comments and nested groups.
fn chunk<'a>(source: &'a str, delimiters: &[u8]) -> Result<(&'a str, Option<u8>, &'a str), String> {
    let mut closing = Vec::new();
    let mut at = 0;
    while let Some(&byte) = source.as_bytes().get(at) {
        if let Some(end) = opaque(source, at)? {
            at = end;
            continue;
        }
        if closing.is_empty() && delimiters.contains(&byte) {
            return Ok((&source[..at], Some(byte), &source[at + 1..]));
        }
        match byte {
            b'(' => closing.push(b')'),
            b'[' => closing.push(b']'),
            b'{' => closing.push(b'}'),
            b')' | b']' | b'}' => {
                if closing.pop() != Some(byte) {
                    return Err(format!("unexpected {} in CSS", char::from(byte)));
                }
            }
            _ => (),
        }
        at += 1;
    }
    if !closing.is_empty() {
        return Err("unfinished CSS group".into());
    }
    Ok((source, None, ""))
}

fn opaque(source: &str, at: usize) -> Result<Option<usize>, String> {
    let bytes = source.as_bytes();
    match bytes[at] {
        b'/' if bytes.get(at + 1) == Some(&b'*') => {
            let content = at + 2;
            source[content..]
                .find("*/")
                .map(|end| Some(content + end + 2))
                .ok_or_else(|| "unfinished CSS comment".into())
        }
        b'\\' => Ok(Some(escaped(source, at)?)),
        quote @ (b'\'' | b'"') => {
            let mut end = at + 1;
            while let Some(&byte) = bytes.get(end) {
                if byte == quote {
                    return Ok(Some(end + 1));
                }
                end = if byte == b'\\' { escaped(source, end)? } else { end + 1 };
            }
            Err("unfinished CSS string".into())
        }
        _ => Ok(None),
    }
}

fn escaped(source: &str, at: usize) -> Result<usize, String> {
    let next = &source[at + 1..];
    // A CRLF is one escaped newline in a CSS string.
    if next.starts_with("\r\n") {
        return Ok(at + 3);
    }
    next.chars().next().map(|c| at + 1 + c.len_utf8()).ok_or_else(|| "unfinished CSS escape".into())
}

fn normalized(source: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut at = 0;
    let mut space = false;
    while let Some(c) = source[at..].chars().next() {
        let special = opaque(source, at)?;
        if let ('/', Some(end)) = (c, special) {
            at = end;
        } else if c.is_ascii_whitespace() {
            space = !out.is_empty();
            at += 1;
        } else {
            if space {
                out.push(' ');
                space = false;
            }
            let end = special.unwrap_or(at + c.len_utf8());
            out.push_str(&source[at..end]);
            at = end;
        }
    }
    Ok(out)
}

fn rule_rows(report: &mut Report, side: Side, parent: &Sheet, requested: &Sheet, landed: &Sheet) {
    let current_at = unmatched(requested, parent);
    let removed_at = unmatched(parent, requested);
    let mut changed_at = BTreeMap::<&Key, usize>::new();
    for entry in &requested.order {
        match entry {
            Entry::Rule(key) => {
                let declarations = &requested.rules[key];
                if parent.rules.get(key) != Some(declarations) {
                    rule_row(report, side, key, declarations, landed.rules.get(key), false);
                }
            }
            Entry::AtRule(index) if current_at[*index] => {
                let (key, _) = &requested.at_rules[*index];
                *changed_at.entry(key).or_default() += 1;
                at_row(report, key);
            }
            Entry::AtRule(_) => (),
        }
    }
    for entry in &parent.order {
        match entry {
            Entry::Rule(key) if !requested.rules.contains_key(key) => {
                rule_row(report, side, key, &parent.rules[key], landed.rules.get(key), true);
            }
            Entry::AtRule(index) if removed_at[*index] => {
                let (key, _) = &parent.at_rules[*index];
                if let Some(count) = changed_at.get_mut(key).filter(|count| **count > 0) {
                    *count -= 1;
                } else {
                    at_row(report, key);
                }
            }
            Entry::Rule(_) | Entry::AtRule(_) => (),
        }
    }
}

fn unmatched(ours: &Sheet, theirs: &Sheet) -> Vec<bool> {
    // Identical opaque at-rules cancel, even if another rule with the same prelude moved past them.
    let mut seen = BTreeMap::<_, usize>::new();
    for entry in &theirs.at_rules {
        *seen.entry(entry).or_default() += 1;
    }
    ours.at_rules
        .iter()
        .map(|entry| {
            if let Some(count) = seen.get_mut(entry).filter(|count| **count > 0) {
                *count -= 1;
                false
            } else {
                true
            }
        })
        .collect()
}

fn at_row(report: &mut Report, key: &Key) {
    report.row(&format!("at-rule {}", code(&key.prelude)), "to read");
}

fn rule_row(
    report: &mut Report,
    side: Side,
    key: &Key,
    declarations: &Declarations,
    landed: Option<&Declarations>,
    removed: bool,
) {
    let selector = code(&key.prelude);
    if side == Side::Tui {
        report.row(&format!("TUI rule {selector}"), "to read");
        return;
    }
    let mut item = format!("{}rule {selector}", if removed { "removed " } else { "" });
    if !key.conditions.is_empty() {
        item.push_str(" in ");
        item.push_str(&code(&key.conditions.join(" > ")));
    }
    if removed {
        if landed.is_some_and(|landed| declarations.iter().any(|(property, value)| landed.get(property) == Some(value)))
        {
            report.problem(&item, "differs: kept");
        } else {
            report.row(&item, "landed");
        }
    } else if let Some(landed) = landed {
        let differing = declarations
            .iter()
            .filter(|(property, value)| landed.get(*property) != Some(*value))
            .map(|(property, _)| property.as_str())
            .collect::<Vec<_>>();
        if differing.is_empty() {
            report.row(&item, "landed");
        } else {
            report.problem(&item, &format!("differs: {}", differing.join(", ")));
        }
    } else {
        report.problem(&item, "missing");
    }
}
