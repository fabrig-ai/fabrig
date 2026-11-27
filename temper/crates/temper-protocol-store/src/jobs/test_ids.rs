//! Written id narrowing jobs (temper/docs/design/protocol/store-versions.md, 1).
use super::refusal;
use crate::v25;
use alloc::{boxed::Box, string::String};
use jig_protocol_store::{JobAnswer, JobPage, JobRefusal, JobWrite};
use skein_codec::Record;
use skein_codec::{
    layout::{Misconverted, Mode, Plan, Refusal, Side, Workspace},
    schema::{self, DeclIx, FieldIx, Schema, Shape, Span, Type, TypeIx},
    value::{Node, Value},
};
use skein_lib::{Buffer, List, Map, Reader, Writer};

const TEXT: &str = crate::OWNER_SET.families.first().expect("store family").current;
const SUBSTITUTIONS: [(&str, &str); 8] = [
    ("record FailingTest { test_id: bytes 1024", "record FailingTest { test_id: bytes 4096"),
    ("record TierFlake { test: bytes 1024", "record TierFlake { test: bytes 4096"),
    ("pending: list 1537 TierPending", "pending: list 759 TierPending"),
    ("landings: list 1537 TierLanding", "landings: list 759 TierLanding"),
    ("blamed: list 1537 Blame", "blamed: list 759 Blame"),
    ("waiting: list 1537 TierGroup", "waiting: list 759 TierGroup"),
    // These unplanned layouts predate waiting filings. Their default is empty;
    // the wide id layout plus eight full filings would exceed u32's row bound.
    ("waiting: list 8 VerdictFiling = []", "waiting: list 0 VerdictFiling = []"),
    ("landings: list 1537 TierPostedLanding", "landings: list 759 TierPostedLanding"),
];

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "checked substitutions derive the job's text and name a missing declaration"
)]
pub(crate) fn wide_text(text: &str) -> Result<String, JobRefusal> {
    let mut wide = String::from(text);
    for (words, replacement) in SUBSTITUTIONS {
        if wide.match_indices(words).count() != 1 {
            return Err(refusal(None, &alloc::format!("the binary's text has no single {words}")));
        }
        wide = wide.replacen(words, replacement, 1);
    }
    Ok(wide)
}

pub(super) fn page(job: &str, page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let (variant, width) = match job {
        "test-ids-landing" => (b"landing".as_slice(), 0),
        "test-ids-receipt" => (b"receipt".as_slice(), 1),
        "test-ids-tier" => (b"tier".as_slice(), 3),
        "test-ids-verification" => (b"verification".as_slice(), 1),
        _ => unreachable!("dispatched id job"),
    };
    let mut counts = List::with_capacity(width);
    for _ in 0..width {
        counts.push(0_u64).expect("declared count");
    }
    let Some(layout) = &page.layout else {
        if !page.rows.is_empty() {
            return Err(refusal(None, "test id job rows have no layout"));
        }
        return Ok(JobAnswer { writes: Box::new([]), through: None, counts: counts.into_boxed() });
    };
    if page.rows.is_empty() {
        return Ok(JobAnswer { writes: Box::new([]), through: None, counts: counts.into_boxed() });
    }
    let old = parse(&layout.text)?;
    let new = parse(TEXT.as_bytes())?;
    let mut held = List::uncapped();
    for text in &layout.held {
        held.push(parse(text)?).expect("uncapped held texts");
    }
    let mut held_refs = List::uncapped();
    for text in &held {
        held_refs.push(text).expect("uncapped held texts");
    }
    let old_variant = match &layout.variant {
        Some(name) => Some(name.as_bytes()),
        None => None,
    };
    let old_root = old.root(layout.root.as_bytes(), old_variant).ok_or(refusal(None, "unreadable stored root"))?;
    let from = Side { text: &old, root: old_root, held: held_refs.as_slice() };
    let to = Side { text: &new, root: new.root(b"Row", Some(variant)).expect("binary value type"), held: &[] };
    let mut tally = None;
    let mut out = Buffer::with_capacity(0);
    let bound = u32::try_from(page.room.bytes).unwrap_or(u32::MAX);
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    for row in &page.rows {
        let value = row.at_version(old.version())?;
        let converted = match convert_row(layout.mode, &from, &to, &value, &mut out, bound) {
            Ok(converted) => converted,
            Err(error) => return Err(JobRefusal { key: Some(row.key.clone()), why: error.why }),
        };
        let write = JobWrite::Put { key: row.key.clone(), value: out.to_boxed() };
        let cost = bytes.checked_add(write.bytes());
        let fits = converted
            && match (cost, writes.len().checked_add(1)) {
                (Some(cost), Some(count)) => cost <= page.room.bytes && count <= page.room.writes,
                (None, _) | (_, None) => false,
            };
        if !fits {
            if through.is_none() {
                return Err(refusal(Some(row.key.clone()), "row exceeds the test id job edge"));
            }
            return Ok(JobAnswer { writes: writes.into_boxed(), through, counts: counts.into_boxed() });
        }
        bytes = cost.expect("checked cost");
        writes.push(write).expect("uncapped writes");
        if width != 0 {
            // Counts belong to the job, not the converter's output. Analyze
            // accepted rows in the same wide form, reusing one plan for the page.
            if tally.is_none() {
                tally = Some(match Wide::new(layout.mode, &from, &to, variant) {
                    Ok(wide) => wide,
                    Err(error) => return Err(JobRefusal { key: Some(row.key.clone()), why: error.why }),
                });
            }
            let amounts = match tally.as_mut().expect("counted job has a plan").counts(&value, out.as_slice()) {
                Ok(amounts) => amounts,
                Err(why) => return Err(refusal(Some(row.key.clone()), why)),
            };
            for (total, amount) in counts.iter_mut().zip(amounts) {
                *total = total.saturating_add(amount);
            }
        }
        through = Some(row.key.clone());
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None, counts: counts.into_boxed() })
}

const OUTPUT_ROOM: &str = "narrowed test id value exceeds output capacity";

fn convert_row(
    mode: Mode,
    from: &Side<'_>,
    to: &Side<'_>,
    value: &[u8],
    out: &mut Buffer,
    bound: u32,
) -> Result<bool, Refusal> {
    let initial = u32::try_from(value.len()).unwrap_or(bound).max(1024).min(bound);
    if out.capacity() < initial {
        *out = Buffer::with_capacity(initial);
    }
    for _ in 0..=u32::BITS {
        match convert(mode, from, to, value, out) {
            Ok(()) => return Ok(true),
            Err(error) if error.why.as_ref() == OUTPUT_ROOM => {
                if out.capacity() == bound {
                    return Ok(false);
                }
                *out = Buffer::with_capacity(out.capacity().saturating_mul(2).min(bound));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(false)
}

/// Narrow old test ids for a landing, receipt, tier or verification at the binary's layout.
pub fn convert(mode: Mode, from: &Side<'_>, to: &Side<'_>, value: &[u8], out: &mut Buffer) -> Result<(), Refusal> {
    out.clear();
    let family = to.text.spelled(to.text.family());
    let decl = to.text.spelled(to.text.declaration(to.root.decl).name);
    let variant = match to.root.variant {
        Some(variant) => to.text.spelled(to.text.variant(variant).name),
        None => b"".as_slice(),
    };
    match (family, decl, variant) {
        (b"temper-store", b"Row", b"landing" | b"receipt" | b"tier" | b"verification") => {}
        _ => return Err(pass_refusal(from, "unsupported target value type")),
    }
    if to.text.text() != TEXT.as_bytes() || from.text.spelled(from.text.family()) != family {
        return Err(pass_refusal(from, "unsupported layout pair"));
    }
    if u32::try_from(value.len()).is_err() {
        return Err(pass_refusal(from, "test id value exceeds u32"));
    }
    if Reader::new(value).u16() != Some(from.text.version()) {
        return Err(pass_refusal(from, "missing or wrong source version"));
    }
    let mut wide = Wide::new(mode, from, to, variant)?;
    let bytes = match wide.planned(value) {
        Ok(bytes) => bytes,
        Err(why) => return Err(pass_refusal(from, why)),
    };
    let (decoded, node) = match wide.read(bytes) {
        Ok(read) => read,
        Err(why) => return Err(pass_refusal(from, why)),
    };
    let edits = match changes(&wide.schema, &decoded, node, wide.root) {
        Ok(changes) => changes.edits,
        Err(failure) => return Err(failure.refusal(from)),
    };
    let bytes = narrowed(&decoded, edits);
    let Ok(row) = v25::Row::decode(&v25::CEILINGS, &mut Reader::new(&bytes)) else {
        return Err(pass_refusal(from, "narrowed value does not decode at the binary's ceilings"));
    };
    let bytes = row.encode(&v25::CEILINGS).expect("measured row");
    match out.put(&bytes) {
        Ok(()) => Ok(()),
        Err(_) => Err(pass_refusal(from, OUTPUT_ROOM)),
    }
}

#[expect(clippy::disallowed_methods, reason = "the parsed schema's declaration names are ASCII")]
fn pass_refusal(from: &Side<'_>, why: &str) -> Refusal {
    let decl = from.text.spelled(from.text.declaration(from.root.decl).name);
    let decl = core::str::from_utf8(decl).expect("schema declaration name");
    Refusal { declaration: Box::from(decl), part: Box::from(""), why: Box::from(why) }
}

struct Wide {
    schema: Schema,
    plan: Plan,
    root: DeclIx,
    work: Workspace,
}
impl Wide {
    fn new(mode: Mode, from: &Side<'_>, to: &Side<'_>, variant: &[u8]) -> Result<Self, Refusal> {
        let wide = match wide_text(TEXT) {
            Ok(wide) => wide,
            Err(error) => return Err(pass_refusal(from, &error.why)),
        };
        let Ok(schema) = schema::parse(wide.as_bytes()) else {
            return Err(pass_refusal(from, "unreadable test id schema text"));
        };
        let root = schema.root(b"Row", Some(variant)).expect("wide value type");
        let destination = Side { text: &schema, root, held: to.held };
        let Ok(plan) = plan(mode, from, &destination) else {
            return Err(pass_refusal(from, "stored layout has no plan to the wide text"));
        };
        let mut work = Workspace::new();
        if !work.fit(&plan) {
            return Err(pass_refusal(from, "test id plan exceeds its workspace"));
        }
        Ok(Self { schema, plan, root: root.decl, work })
    }

    fn planned(&mut self, value: &[u8]) -> Result<Box<[u8]>, &'static str> {
        planned(&self.plan, &mut self.work, value)
    }

    fn read(&self, bytes: Box<[u8]>) -> Result<(Value, Node), &'static str> {
        let mut decoded = Value::with_room(u32::MAX);
        let Ok(node) = decoded.decode(&self.schema, self.root, bytes) else { return Err("unreadable wide value") };
        Ok((decoded, node))
    }

    fn counts(&mut self, value: &[u8], converted: &[u8]) -> Result<[u64; 3], &'static str> {
        let bytes = self.planned(value)?;
        if bytes.as_ref() == converted {
            return Ok([0; 3]);
        }
        let (decoded, node) = self.read(bytes)?;
        match changes(&self.schema, &decoded, node, self.root) {
            Ok(changes) => Ok(changes.counts),
            Err(failure) => Err(failure.why),
        }
    }
}

fn plan(mode: Mode, from: &Side<'_>, to: &Side<'_>) -> Result<Plan, skein_codec::layout::Underived> {
    match mode {
        Mode::Read => Plan::derive(from.text, from.root, from.held, to.text, to.root, to.held),
        Mode::Keep => Plan::between(mode, from.text, from.root, from.held, to.text, to.root, to.held),
    }
}

fn planned(plan: &Plan, work: &mut Workspace, bytes: &[u8]) -> Result<Box<[u8]>, &'static str> {
    let Ok(capacity) = u32::try_from(bytes.len()) else {
        return Err("test id value exceeds u32");
    };
    let mut capacity = capacity.max(1024);
    for _ in 0..=u32::BITS {
        let mut out = Buffer::with_capacity(capacity);
        match plan.convert(bytes, &mut out, work) {
            Ok(()) => return Ok(Box::from(out.as_slice())),
            Err(Misconverted::Room) => {
                let next = capacity.saturating_mul(2);
                if next == capacity {
                    return Err("test id conversion exceeds u32");
                }
                capacity = next;
            }
            Err(
                Misconverted::Short
                | Misconverted::Tag
                | Misconverted::Bool
                | Misconverted::Bound
                | Misconverted::Trailing
                | Misconverted::Refused(_),
            ) => return Err("unreadable test id value"),
        }
    }
    Err("test id conversion exceeds u32")
}

struct Changes {
    edits: List<Edit>,
    counts: [u64; 3],
}
struct RuleRefusal {
    declaration: &'static str,
    part: &'static str,
    why: &'static str,
}
impl RuleRefusal {
    fn refusal(self, from: &Side<'_>) -> Refusal {
        // Use the source's declaration and part if present, otherwise name its root.
        if from.text.find(self.declaration.as_bytes()).is_some() {
            Refusal { declaration: Box::from(self.declaration), part: Box::from(self.part), why: Box::from(self.why) }
        } else {
            pass_refusal(from, self.why)
        }
    }
}
#[derive(Debug)]
struct Edit {
    span: Span,
    bytes: Box<[u8]>,
}

#[expect(
    clippy::too_many_lines,
    reason = "the written rule judges id spans, in-flight work and posted indices together"
)]
fn changes(schema: &Schema, value: &Value, node: Node, root: DeclIx) -> Result<Changes, RuleRefusal> {
    let nodes = named_nodes(schema, value, node, root);
    let mut long = Map::uncapped();
    for &(node, decl) in &nodes {
        let name = schema.spelled(schema.declaration(decl).name);
        let field = match name {
            b"FailingTest" => Some(b"test_id".as_slice()),
            b"TierFlake" => Some(b"test".as_slice()),
            _ => None,
        };
        if let Some(field) = field {
            let id = field_node(schema, value, node, decl, field);
            if value.content(id).expect("id content").len() > skein_lib::usize_of(crate::test_id_ceiling()) {
                assert!(
                    long.insert(value.span(node).expect("named span").start, ()).expect("uncapped long ids").is_none(),
                    "one visit per long id"
                );
            }
        }
    }
    let mut edits = List::uncapped();
    let mut counts = [0_u64; 3];
    let mut record_many = false;
    let mut flake_list = None;
    for &(node, decl) in &nodes {
        let name = schema.spelled(schema.declaration(decl).name);
        if name == b"TierRecord" && !long.is_empty() {
            let run = field_node(schema, value, node, decl, b"run");
            if value.some(run).expect("run option").is_some() {
                return Err(RuleRefusal {
                    declaration: "TierRecord",
                    part: "run",
                    why: "a tier run in flight holds a failing test's id past 1,024 bytes",
                });
            }
            let failing = field_node(schema, value, node, decl, b"failing");
            record_many = contains_long(value, failing, &long);
            flake_list = Some(field_node(schema, value, node, decl, b"flakes"));
        }
        if name == b"LandingSelection" && contains_long(value, node, &long) {
            return Err(RuleRefusal {
                declaration: "LandingSelection",
                part: "",
                why: "a landing's search by test holds a failing test's id past 1,024 bytes",
            });
        }
        if name == b"Failures" && contains_long(value, node, &long) {
            edits
                .push(Edit { span: value.span(node).expect("failures span"), bytes: Box::new([1]) })
                .expect("uncapped edits");
            let count = counts.first_mut().expect("many count");
            *count = count.saturating_add(1);
        }
        if name == b"TierFlake" && contains_long(value, node, &long) {
            edits
                .push(Edit { span: value.span(node).expect("flake span"), bytes: Box::new([]) })
                .expect("uncapped edits");
            let count = counts.get_mut(1).expect("flakes count");
            *count = count.saturating_add(1);
        }
    }
    if let Some(list) = flake_list {
        let removed = u32::try_from(*counts.get(1).expect("flakes count")).expect("bounded flakes");
        if removed != 0 {
            let span = value.span(list).expect("flake list span");
            let count = value.count(list).expect("flake count").checked_sub(removed).expect("removed flakes");
            edits
                .push(Edit {
                    span: Span { start: span.start, end: span.start.checked_add(4).expect("count end") },
                    bytes: Box::new(count.to_be_bytes()),
                })
                .expect("uncapped edits");
        }
    }
    if record_many {
        for &(node, decl) in &nodes {
            if schema.spelled(schema.declaration(decl).name) == b"TierPostedBlame" {
                let tag = schema.spelled(schema.variant(value.tag(node).expect("blame tag")).name);
                let new = match tag {
                    b"owns" => Some(3),
                    b"suspect" => Some(4),
                    _ => None,
                };
                if let Some(tag) = new {
                    edits
                        .push(Edit { span: value.span(node).expect("blame span"), bytes: Box::new([tag]) })
                        .expect("uncapped edits");
                    let count = counts.get_mut(2).expect("posted count");
                    *count = count.saturating_add(1);
                }
            }
        }
        if counts.get(2) != Some(&0) {
            for &(node, decl) in &nodes {
                if schema.spelled(schema.declaration(decl).name) == b"TierPosting" {
                    let entry = field_node(schema, value, node, decl, b"entry");
                    if value.some(entry).expect("entry option").is_some() {
                        return Err(RuleRefusal {
                            declaration: "TierPosting",
                            part: "entry",
                            why: "a tier posting in flight would change for a failing test's id past 1,024 bytes",
                        });
                    }
                }
            }
        }
    }
    Ok(Changes { edits, counts })
}

fn narrowed(value: &Value, edits: List<Edit>) -> Box<[u8]> {
    let mut ordered = Map::uncapped();
    for edit in edits.into_boxed() {
        assert!(ordered.insert(edit.span.start, edit).expect("uncapped edits").is_none(), "distinct rule spans");
    }
    let mut length = value.bytes().len();
    for (_, edit) in &ordered {
        let removed = edit.span.end.checked_sub(edit.span.start).expect("span length");
        length = length.checked_sub(skein_lib::usize_of(removed)).expect("removed span");
        length = length.checked_add(edit.bytes.len()).expect("narrowed length");
    }
    let mut writer = Writer::new(length);
    let mut start = 0;
    for (_, edit) in &ordered {
        let before = value.bytes().get(start..skein_lib::usize_of(edit.span.start)).expect("nonoverlapping edits");
        writer.put(before).expect("narrowing value");
        writer.put(&edit.bytes).expect("narrowing value");
        start = skein_lib::usize_of(edit.span.end);
    }
    writer.put(value.bytes().get(start..).expect("remaining bytes")).expect("narrowing value");
    writer.finish()
}

fn parse(text: &[u8]) -> Result<Schema, JobRefusal> {
    match schema::parse(text) {
        Ok(schema) => Ok(schema),
        Err(_) => Err(refusal(None, "unreadable test id schema text")),
    }
}
fn contains_long(value: &Value, node: Node, long: &Map<u32, ()>) -> bool {
    let span = value.span(node).expect("named span");
    long.range(span.start..span.end).next().is_some()
}
fn field_node(schema: &Schema, value: &Value, node: Node, decl: DeclIx, name: &[u8]) -> Node {
    let (first, count) = match schema.declaration(decl).shape {
        Shape::Record { first, count, .. } => (first, count),
        Shape::Enum { .. } => unreachable!("named record"),
    };
    for offset in 0..count {
        let field = FieldIx::new(first.index().checked_add(offset).expect("field index"));
        if schema.spelled(schema.field(field).name) == name {
            return value.field(node, field).expect("record field");
        }
    }
    unreachable!("binary field")
}

#[derive(Debug)]
enum Walk {
    Decl(Node, DeclIx),
    Type(Node, TypeIx),
}
fn named_nodes(schema: &Schema, value: &Value, node: Node, root: DeclIx) -> List<(Node, DeclIx)> {
    let mut pending = List::uncapped();
    let mut nodes = List::uncapped();
    pending.push(Walk::Decl(node, root)).expect("uncapped walk");
    // Each indexed node takes at most a type visit and a declaration visit.
    for _ in 0..u64::from(u32::MAX).saturating_mul(2) {
        let Some(last) = pending.len().checked_sub(1) else {
            return nodes;
        };
        let next = pending.swap_remove(last).expect("last walk item");
        match next {
            Walk::Decl(node, decl) => {
                nodes.push((node, decl)).expect("uncapped named nodes");
                match schema.declaration(decl).shape {
                    Shape::Record { first, count, .. } => {
                        for offset in 0..count {
                            let field = FieldIx::new(first.index().checked_add(offset).expect("field index"));
                            pending
                                .push(Walk::Type(
                                    value.field(node, field).expect("record field"),
                                    schema.field(field).ty,
                                ))
                                .expect("uncapped walk");
                        }
                    }
                    Shape::Enum { .. } => {
                        if let Some(child) = value.payload(node).expect("enum payload") {
                            let decl =
                                schema.variant(value.tag(node).expect("enum tag")).record.expect("payload declaration");
                            pending.push(Walk::Decl(child, decl)).expect("uncapped walk");
                        }
                    }
                }
            }
            Walk::Type(node, ty) => match schema.ty(ty) {
                Type::Named(decl) => {
                    pending.push(Walk::Decl(node, decl)).expect("uncapped walk");
                }
                Type::Option(ty) => {
                    if let Some(node) = value.some(node).expect("option") {
                        pending.push(Walk::Type(node, ty)).expect("uncapped walk");
                    }
                }
                Type::List { item, .. } => {
                    let mut items = value.items(node).expect("list");
                    for _ in 0..value.count(node).expect("list count") {
                        let node = value.next_item(&mut items).expect("counted list item");
                        pending.push(Walk::Type(node, item)).expect("uncapped walk");
                    }
                }
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Bool
                | Type::Duration
                | Type::Fixed(_)
                | Type::Secret { .. }
                | Type::Text(_)
                | Type::Bytes { .. } => {}
            },
        }
    }
    unreachable!("the generic node allowance bounds the walk")
}
