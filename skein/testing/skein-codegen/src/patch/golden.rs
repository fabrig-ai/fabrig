//! Fixed-rule values run through a kept patch into golden files (data/codec/command.md, 3).

use alloc::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use skein_codec::patch::{Build, Capacity, Halt, Old, Patcher, PlanIx, Site, Step};
use skein_codec::schema::{self, DeclIx, FieldIx, Held, Shape, Type, TypeIx};
use skein_codec::value::{Node, Value};

use super::{
    Error, Tree,
    files::{Crate, golden_path},
    module::read_patch,
    text::{Block, PatchText},
};

/// The owner's written dispatch, called once at each written stop.
pub type Written<C> = fn(Site, &Old, &mut Build, Node, &mut C) -> Result<(), Halt>;

/// What a golden test does with a patch's golden files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Fails on a file missing, different or stale, and writes nothing.
    Check,
    /// Writes them for review.
    Write,
}

/// Checks a patch's golden results, or writes them for review, as `mode` says.
pub fn golden<C: Default>(tree: &mut Tree, patch: u32, written: Written<C>, mode: Mode) -> Result<(), Error> {
    let krate = Crate::read(tree)?;
    run_goldens(tree, &krate, patch, written, mode == Mode::Write)
}

/// [`golden`], writing when `SKEIN_PATCH_WRITE=1` and checking otherwise.
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads SKEIN_PATCH_WRITE, the patch golden rewrite switch"
)]
pub fn goldens<C: Default>(owner: &Path, patch: u32, written: Written<C>) -> Result<(), Error> {
    let write = std::env::var_os("SKEIN_PATCH_WRITE").is_some_and(|value| value == "1");
    golden(&mut Tree::disk(owner), patch, written, if write { Mode::Write } else { Mode::Check })
}

pub(crate) fn run_goldens<C: Default>(
    tree: &mut Tree,
    krate: &Crate,
    number: u32,
    written: Written<C>,
    write: bool,
) -> Result<(), Error> {
    let mut failures = Vec::new();
    let Some(source) = krate.patches.get(&number) else {
        return Err(Error::Golden(vec![format!("patch {number}: patch text missing")]));
    };
    let patch = read_patch(source, number).map_err(|error| Error::Golden(vec![format!("patch {number}: {error}")]))?;
    for block in &patch.blocks {
        let label = format!("patch {number}, {}", block.family);
        if let Err(error) = family(tree, krate, number, source, &patch, block, written, write, false, &mut failures) {
            failures.push(format!("{label}: {error}"));
        }
        if super::run::needs_reverse(krate, block).is_some()
            && let Err(error) = family(tree, krate, number, source, &patch, block, written, write, true, &mut failures)
        {
            failures.push(format!("{label}/down: {error}"));
        }
    }
    if failures.is_empty() { Ok(()) } else { Err(Error::Golden(failures)) }
}

fn kept<'a>(krate: &'a Crate, block: &Block, version: u16) -> Result<&'a str, String> {
    krate
        .kept
        .get(&block.family)
        .and_then(|versions| versions.get(&version))
        .map(|text| text.source.as_str())
        .ok_or_else(|| format!("kept version {version} missing"))
}

// Eight times the larger fixed-rule value plus both schema texts and a kilobyte gives
// room for written answers. Nodes also count empty records, whose encodings
// need no bytes. An answer beyond this room reports a capacity failure.
fn capacity(old: &str, new: &str, source: &str, largest: usize, nodes: u32) -> Result<Capacity, String> {
    let text = u32::try_from(old.len().max(new.len()).max(source.len())).map_err(|error| error.to_string())?;
    if text > schema::MAX_TEXT {
        return Err("text exceeds the patcher's capacity".into());
    }
    let room = largest
        .checked_add(old.len())
        .and_then(|size| size.checked_add(new.len()))
        .and_then(|size| size.checked_mul(8))
        .and_then(|size| size.checked_add(1024))
        .and_then(|size| u32::try_from(size).ok())
        .ok_or("fixed-rule values exceed the patcher's capacity")?;
    let nodes = nodes
        .checked_mul(8)
        .and_then(|nodes| nodes.checked_add(1024))
        .ok_or("fixed-rule value nodes exceed the patcher's capacity")?;
    Ok(Capacity { versions: 2, patches: 2, text, bytes: room, nodes: nodes.max(room) })
}

// Fixed-rule lists hold at most one item and fixed-rule value options can be present.
// A named declaration refers backwards, so its node count is already known.
fn fixed_value_nodes(schema: &crate::Schema) -> Result<u32, String> {
    let mut counts = BTreeMap::new();
    let mut largest = 1;
    for declaration in &schema.declarations {
        let (name, children) = match declaration {
            crate::Declaration::Record(record) => {
                let mut children = 0_u32;
                for field in &record.fields {
                    children = children
                        .checked_add(type_nodes(&field.ty, &counts)?)
                        .ok_or("fixed-rule value node count exceeds u32")?;
                }
                (&record.name, children)
            }
            crate::Declaration::Enum(enumeration) => {
                let children = enumeration
                    .variants
                    .iter()
                    .filter_map(|variant| variant.record.as_ref())
                    .map(|name| *counts.get(name.as_str()).expect("earlier record"))
                    .max()
                    .unwrap_or(0);
                (&enumeration.name, children)
            }
        };
        let nodes = children.checked_add(1).ok_or("fixed-rule value node count exceeds u32")?;
        counts.insert(name.as_str(), nodes);
        largest = largest.max(nodes);
    }
    Ok(largest)
}

fn type_nodes(ty: &crate::Type, counts: &BTreeMap<&str, u32>) -> Result<u32, String> {
    match ty {
        crate::Type::Named(name) => Ok(*counts.get(name.as_str()).expect("earlier declaration")),
        crate::Type::List(0, _)
        | crate::Type::U8
        | crate::Type::U16
        | crate::Type::U32
        | crate::Type::U64
        | crate::Type::Bool
        | crate::Type::Duration
        | crate::Type::Fixed(_)
        | crate::Type::Bytes(_)
        | crate::Type::Secret(_)
        | crate::Type::Text(_) => Ok(1),
        crate::Type::List(_, item) | crate::Type::Option(item) => {
            type_nodes(item, counts)?.checked_add(1).ok_or_else(|| "fixed-rule value node count exceeds u32".into())
        }
    }
}

// Only the patch's opened pieces depart from the fixed rule (command.md, 3).
// Declarations refer backwards, so each item's prepared full value is already available.
fn open_fixed_values(
    patcher: &Patcher,
    plan: PlanIx,
    fixed_values: &mut [(DeclIx, crate::FixedValue)],
    nodes: u32,
) -> Result<(), String> {
    let schema = patcher.schema(patcher.versions(plan).from).expect("kept old text");
    let mut samples = BTreeMap::new();
    for (root, fixed_value) in fixed_values {
        if !fixed_value.name.ends_with("_full.bin") {
            continue;
        }
        let mut value = Value::with_room(nodes);
        let at = value
            .decode(schema, *root, fixed_value.bytes.clone().into_boxed_slice())
            .map_err(|error| format!("{}: {error:?}", fixed_value.name))?;
        let mut opened =
            Opened { patcher, plan, schema, samples: &samples, value: &value, bytes: Vec::new(), cursor: 0 };
        opened.declaration(*root, at)?;
        opened.bytes.extend_from_slice(value.bytes().get(opened.cursor..).expect("remaining fixed-rule value bytes"));
        fixed_value.bytes = opened.bytes;
        samples.entry(*root).or_insert_with(|| fixed_value.bytes.clone());
    }
    Ok(())
}

struct Opened<'a> {
    patcher: &'a Patcher,
    plan: PlanIx,
    schema: &'a schema::Schema,
    samples: &'a BTreeMap<DeclIx, Vec<u8>>,
    value: &'a Value,
    bytes: Vec<u8>,
    cursor: usize,
}

impl Opened<'_> {
    fn declaration(&mut self, root: DeclIx, at: Node) -> Result<(), String> {
        match self.schema.declaration(root).shape {
            Shape::Record { first, count, .. } => {
                for offset in 0..count {
                    let field = FieldIx::new(first.index().checked_add(offset).expect("bounded field index"));
                    self.ty(self.schema.field(field).ty, self.value.field(at, field).expect("fixed-rule value field"))?;
                }
            }
            Shape::Enum { .. } => {
                let tag = self.value.tag(at).expect("fixed-rule value variant");
                if let Some(record) = self.schema.variant(tag).record {
                    self.declaration(
                        record,
                        self.value.payload(at).expect("fixed-rule value payload").expect("record payload"),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn ty(&mut self, ty: TypeIx, at: Node) -> Result<(), String> {
        match self.schema.ty(ty) {
            Type::Named(root) => self.declaration(root, at)?,
            Type::List { item, .. } => {
                let mut items = self.value.items(at).expect("fixed-rule value list");
                while let Some(at) = self.value.next_item(&mut items) {
                    self.ty(item, at)?;
                }
            }
            Type::Option(item) => {
                if let Some(at) = self.value.some(at).expect("fixed-rule value option") {
                    self.ty(item, at)?;
                }
            }
            Type::Bytes { held: Held::Pieces(item), bound } | Type::Secret { held: Held::Pieces(item), bound }
                if !self.patcher.copies(self.plan, item) =>
            {
                let sample = self.samples.get(&item).expect("earlier item's full fixed-rule value");
                let length = u32::try_from(sample.len()).map_err(|error| error.to_string())?;
                let piece = length.checked_add(4).ok_or("full fixed-rule value piece length exceeds u32")?;
                if piece > bound {
                    return Err(format!("full fixed-rule value piece needs {piece} bytes, bound {bound}"));
                }
                let span = self.value.span(at).expect("fixed-rule value pieces span");
                let start = usize::try_from(span.start).expect("fixed-rule value span fits usize");
                self.bytes.extend_from_slice(
                    self.value.bytes().get(self.cursor..start).expect("ordered fixed-rule value spans"),
                );
                // The bytes field's length, then one skein-codec piece's length and item.
                self.bytes.extend_from_slice(&piece.to_be_bytes());
                self.bytes.extend_from_slice(&length.to_be_bytes());
                self.bytes.extend_from_slice(sample);
                self.cursor = usize::try_from(span.end).expect("fixed-rule value span fits usize");
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::Bytes { .. }
            | Type::Secret { .. } => {}
        }
        Ok(())
    }
}

#[expect(clippy::too_many_arguments, reason = "one family's run shares the patch text, dispatch and failure report")]
fn family<C: Default>(
    tree: &mut Tree,
    krate: &Crate,
    number: u32,
    source: &str,
    patch: &PatchText,
    block: &Block,
    written: Written<C>,
    write: bool,
    down: bool,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let old = kept(krate, block, block.from)?;
    let new = kept(krate, block, block.to)?;
    let old_schema = crate::parse(old).map_err(|error| error.to_string())?;
    let new_schema = crate::parse(new).map_err(|error| error.to_string())?;
    let nodes = fixed_value_nodes(&old_schema)?.max(fixed_value_nodes(&new_schema)?);
    let mut fixed_values = crate::fixed_values::fixed_values(if down { &new_schema } else { &old_schema });
    let largest = fixed_values
        .iter()
        .map(|(_, fixed_value)| fixed_value.bytes.len())
        .chain(
            crate::fixed_values::fixed_values(if down { &old_schema } else { &new_schema })
                .iter()
                .map(|(_, fixed_value)| fixed_value.bytes.len()),
        )
        .max()
        .unwrap_or(0);
    let capacity = capacity(old, new, source, largest, nodes)?;
    let mut patcher = Patcher::new(capacity);
    patcher.keep(old.as_bytes()).map_err(|error| format!("keep {}: {error:?}", block.from))?;
    patcher.keep(new.as_bytes()).map_err(|error| format!("keep {}: {error:?}", block.to))?;
    let forward = patcher
        .plan(source.as_bytes())
        .map_err(|error| format!("line {}: {:?}", error.line, error.wrong))?
        .ok_or("no block planned")?;
    let plan = if down {
        patcher
            .reverse(source.as_bytes())
            .map_err(|error| format!("line {}: {:?}", error.line, error.wrong))?
            .ok_or("no reverse planned")?
    } else {
        forward
    };
    let target_version = patcher.versions(plan).to;
    let directory = golden_path(number).join(&block.family);
    let directory = if down { directory.join("down") } else { directory };
    open_fixed_values(&patcher, plan, &mut fixed_values, capacity.nodes)?;
    let mut decoded = Value::with_room(capacity.nodes);
    let mut results = BTreeMap::new();
    let mut expected = BTreeSet::new();
    for (root, fixed_value) in fixed_values {
        let Some(new_root) = patcher.counterpart(plan, root) else {
            continue;
        };
        let label = if down {
            format!("{}/{}", tree.named(&directory).display(), fixed_value.name)
        } else {
            format!("patch {number}, {}/{}", block.family, fixed_value.name)
        };
        let original = if down { Some(fixed_value.bytes.clone()) } else { None };
        let result = run(&mut patcher, plan, root, fixed_value.bytes.into_boxed_slice(), written).and_then(|bytes| {
            if let Some(original) = original {
                let round_trip = run(&mut patcher, forward, new_root, bytes.clone().into_boxed_slice(), written)?;
                if round_trip != original {
                    return Err(Halt::Altered);
                }
            }
            Ok(bytes)
        });
        match result {
            Ok(bytes) => {
                expected.insert(fixed_value.name.clone());
                match decoded.decode(
                    patcher.schema(target_version).expect("kept target text"),
                    new_root,
                    bytes.clone().into_boxed_slice(),
                ) {
                    Ok(_) => {
                        results.insert(fixed_value.name, bytes);
                    }
                    Err(error) => {
                        failures.push(format!("{label}: new value: {error:?}"));
                    }
                }
            }
            Err(Halt::Unheld(location)) if down => {
                let name = format!("{}.refused", fixed_value.name);
                expected.insert(name.clone());
                results.insert(name, format!("unheld {location:?}\n").into_bytes());
            }
            Err(Halt::Refused(refusal)) => {
                let name = format!("{}.refused", fixed_value.name);
                expected.insert(name.clone());
                if let Some(reason) = patch.refusals().nth(usize::try_from(refusal.index()).expect("refusal index")) {
                    results.insert(name, format!("refuses {reason}\n").into_bytes());
                } else {
                    failures.push(format!("{label}: unknown refusal {}", refusal.index()));
                }
            }
            Err(error) => {
                // A failed run keeps its old result for review in write mode.
                expected.insert(fixed_value.name.clone());
                expected.insert(format!("{}.refused", fixed_value.name));
                failures.push(format!("{label}: {}", halt(error, patch, capacity)));
            }
        }
    }
    compare(tree, &directory, &results, &expected, write, failures);
    Ok(())
}

fn run<C: Default>(
    patcher: &mut Patcher,
    plan: PlanIx,
    root: DeclIx,
    bytes: Box<[u8]>,
    written: Written<C>,
) -> Result<Vec<u8>, Halt> {
    patcher.start(plan, root, bytes)?;
    loop {
        match patcher.next()? {
            Step::Written { site, at } => {
                let (old, new) = patcher.parts();
                written(site, old, new, at, &mut C::default())?;
            }
            Step::Held { at, .. } => {
                let (old, new) = patcher.parts();
                new.carry(old, at)?;
            }
            Step::Done => break,
        }
    }
    patcher.finish().map(<[u8]>::to_vec)
}

fn halt(error: Halt, patch: &PatchText, capacity: Capacity) -> String {
    match error {
        Halt::NoSite(site) => {
            let target = patch
                .sites()
                .nth(usize::try_from(site.index()).expect("site index"))
                .map_or("unknown", |(_, target, _, _)| target);
            format!("patch {} site {} (written {target})", site.patch(), site.index())
        }
        Halt::Room | Halt::Decode(skein_codec::value::Refusal::Room) => {
            format!("Room: fixed-rule value exceeds capacity ({} bytes, {} nodes)", capacity.bytes, capacity.nodes)
        }
        Halt::Decode(_)
        | Halt::Refused(_)
        | Halt::Undeclared(_)
        | Halt::Misread(_)
        | Halt::Misbuilt(_)
        | Halt::Built(_)
        | Halt::Unheld(_)
        | Halt::Altered
        | Halt::Order => format!("{error:?}"),
    }
}

fn compare(
    tree: &mut Tree,
    directory: &Path,
    results: &BTreeMap<String, Vec<u8>>,
    expected: &BTreeSet<String>,
    write: bool,
    failures: &mut Vec<String>,
) {
    if write && let Err(error) = tree.make_directory(directory) {
        failures.push(format!("{}: {error}", tree.named(directory).display()));
        return;
    }
    for (name, bytes) in results {
        let path = directory.join(name);
        if write {
            if let Err(error) = tree.write_file(&path, bytes) {
                failures.push(format!("{}: {error}", tree.named(&path).display()));
            }
        } else {
            match tree.read_file(&path) {
                Ok(held) if held == *bytes => {}
                Ok(_) => failures.push(format!("{}: differs", tree.named(&path).display())),
                Err(error) => failures.push(format!("{}: {error}", tree.named(&path).display())),
            }
        }
    }
    match tree.entries(directory) {
        Ok(mut entries) => {
            let mut paths = Vec::new();
            while let Some(entry) = entries.next(directory) {
                match entry {
                    Ok(path) => paths.push(path),
                    Err(error) => failures.push(format!("{}: {error}", tree.named(directory).display())),
                }
            }
            paths.sort();
            for path in paths {
                if tree.is_file(&path)
                    && !path.file_name().and_then(|name| name.to_str()).is_some_and(|name| expected.contains(name))
                {
                    if write {
                        if let Err(error) = tree.remove_file(&path) {
                            failures.push(format!("{}: {error}", tree.named(&path).display()));
                        }
                    } else {
                        failures.push(format!("{}: stale", tree.named(&path).display()));
                    }
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => failures.push(format!("{}: {error}", tree.named(directory).display())),
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, Written, golden};
    use crate::patch::{Error, Tree};
    use skein_codec::patch::{Build, Halt, Old, RefusalIx, Site};
    use skein_codec::schema::VariantIx;
    use skein_codec::value::Node;
    use std::path::{Path, PathBuf};

    const OLD: &str =
        "family small 1\nrecord Data { held: bytes 3 holds other Other }\nenum Shape { dot, line }\nrecord Drop {}\n";
    const NEW: &str =
        "family small 2\nrecord Data { held: bytes 3 holds other Other score: u8 }\nenum Shape { line }\n";
    const PATCH: &str = "patch 1\nowner sample 1\nfamily small 1 to 2\nadd Data.score u8 7\ndrop Shape.dot\nwritten Shape\nrefuses no_dot \"no dot\"\ndrop Drop\n";
    // Two variants, each with a smallest and a full golden, reach the site.
    const SHAPE_GOLDENS: usize = 4;
    // Two Data goldens and four Shape goldens; the dropped root has no file.
    const NAMES: [&str; 6] = [
        "enum_shape_dot_full.bin",
        "enum_shape_dot_smallest.bin",
        "enum_shape_line_full.bin",
        "enum_shape_line_smallest.bin",
        "record_data_full.bin",
        "record_data_smallest.bin",
    ];

    fn fixture(old: &str, new: &str, patch: &str) -> Tree {
        Tree::new(
            [("schema/kept/small-v1.schema", old), ("schema/kept/small-v2.schema", new), ("patches/0001.patch", patch)]
                .into_iter()
                .map(|(path, text)| (PathBuf::from(path), text.as_bytes().to_vec()))
                .collect(),
        )
    }

    fn answer(_site: Site, old: &Old, new: &mut Build, at: Node, _cx: &mut ()) -> Result<(), Halt> {
        if old.tag(at)? == VariantIx::new(0) { new.put_variant(VariantIx::new(0)) } else { new.carry(old, at) }
    }

    fn no_site(site: Site, _old: &Old, _new: &mut Build, _at: Node, _cx: &mut ()) -> Result<(), Halt> {
        Err(Halt::NoSite(site))
    }

    fn refuse(_site: Site, old: &Old, new: &mut Build, at: Node, _cx: &mut ()) -> Result<(), Halt> {
        if old.tag(at)? == VariantIx::new(0) { Err(new.refuse(RefusalIx::new(0))) } else { new.carry(old, at) }
    }

    const DIRECTORY: &str = "patches/golden/0001/small";

    fn path(name: &str) -> PathBuf {
        Path::new(DIRECTORY).join(name)
    }

    fn read(tree: &Tree, name: &str) -> Vec<u8> {
        tree.read(&path(name)).expect("read golden").expect("golden file")
    }

    fn written(dispatch: Written<()>) -> Tree {
        let mut tree = fixture(OLD, NEW, PATCH);
        golden(&mut tree, 1, dispatch, Mode::Write).expect("write goldens");
        tree
    }

    fn names(tree: &Tree) -> Vec<String> {
        tree.files()
            .expect("files")
            .into_keys()
            .filter(|path| path.starts_with(DIRECTORY))
            .map(|path| path.file_name().expect("file").to_str().expect("UTF-8").to_owned())
            .collect()
    }

    #[test]
    fn opened_piece_goldens_convert_one_full_item_and_leave_copied_pieces_unchanged() {
        const OLD_PIECES: &str = "family small 1\nrecord Item { value: u8 }\nrecord Kept { value: u8 }\nrecord Open { data: bytes 32 pieces Item }\nrecord Copy { data: bytes 3 pieces Kept }\n";
        const NEW_PIECES: &str = "family small 2\nrecord Item { value: u8 added: u8 }\nrecord Kept { value: u8 }\nrecord Open { data: bytes 32 pieces Item }\nrecord Copy { data: bytes 3 pieces Kept }\n";
        let mut tree =
            fixture(OLD_PIECES, NEW_PIECES, "patch 1\nowner sample 1\nfamily small 1 to 2\nadd Item.added u8 7\n");
        golden(&mut tree, 1, no_site, Mode::Write).expect("opened pieces convert their full item");
        let full = read(&tree, "record_open_full.bin");
        let schema = skein_codec::schema::parse(NEW_PIECES.as_bytes()).expect("new schema");
        let root = schema.find(b"Open").expect("opened record");
        let skein_codec::schema::Shape::Record { first, .. } = schema.declaration(root).shape else {
            unreachable!("record")
        };
        let mut value = skein_codec::value::Value::with_room(16);
        let at = value.decode(&schema, root, full.into_boxed_slice()).expect("new full value");
        let bytes = value.content(value.field(at, first).expect("pieces field")).expect("pieces bytes");
        assert_eq!(skein_codec::piece::count(bytes), Ok(1));
        assert_eq!(bytes, [0, 0, 0, 2, u8::MAX, 7], "new length and added value");
        let mut item = skein_codec::value::Value::with_room(16);
        let at = item
            .decode(
                &schema,
                schema.find(b"Item").expect("item record"),
                bytes.get(4..).expect("one piece length").into(),
            )
            .expect("piece decodes as the new item");
        assert_eq!(item.uint(item.field(at, skein_codec::schema::FieldIx::new(1)).unwrap()), Ok(7));
        assert_eq!(read(&tree, "record_open_smallest.bin"), [0; 4]);
        assert_eq!(
            read(&tree, "record_copy_full.bin"),
            [0, 0, 0, 3, 0, 0x7f, 0xff],
            "copied pieces keep the fixed golden pattern"
        );
    }

    #[test]
    fn empty_record_trees_get_nodes_beyond_their_bytes() {
        use core::fmt::Write;
        // Fifteen binary levels above Empty have 65,535 nodes and no bytes.
        const LEVELS: u32 = 15;
        const TREE_NODES: u32 = 65_535;
        let mut text = String::from("family empty 1\nrecord D0 {}\n");
        for level in 1..=LEVELS {
            let previous = level.checked_sub(1).expect("positive level");
            writeln!(text, "record D{level} {{ a: D{previous} b: D{previous} }}").expect("schema text");
        }
        let schema = crate::parse(&text).expect("empty tree schema");
        let nodes = super::fixed_value_nodes(&schema).expect("node bound");
        assert_eq!(nodes, TREE_NODES);
        let capacity = super::capacity(&text, &text, "patch 1\nowner sample 1\n", 0, nodes).expect("capacity");
        assert!(capacity.nodes > TREE_NODES, "empty trees need room despite their zero bytes");
    }

    #[test]
    fn writes_goldens_carries_held_fields_and_checks_new_values() {
        let mut tree = written(answer);
        assert_eq!(names(&tree), NAMES);
        assert_eq!(read(&tree, "record_data_smallest.bin"), b"\0\0\0\0\x07");
        assert_eq!(read(&tree, "record_data_full.bin"), b"\0\0\0\x03\0\x7f\xff\x07");
        assert_eq!(golden(&mut tree, 1, answer, Mode::Check), Ok(()));
    }

    #[test]
    fn missing_changed_and_stale_goldens_are_all_named() {
        // One changed, one missing and one stale file.
        const DRIFTED_FILES: usize = 3;
        let mut tree = written(answer);
        tree.write(&path("record_data_full.bin"), b"changed").expect("changed");
        tree.remove(&path("record_data_smallest.bin")).expect("missing");
        tree.write(&path("extra.bin"), b"stale").expect("stale");
        let Error::Golden(failures) = golden(&mut tree, 1, answer, Mode::Check).expect_err("all drift fails") else {
            unreachable!("golden failures")
        };
        assert_eq!(failures.len(), DRIFTED_FILES);
        for name in ["record_data_full.bin", "record_data_smallest.bin", "extra.bin"] {
            assert!(failures.iter().any(|failure| failure.contains(name)), "{name}: {failures:?}");
        }
    }

    #[test]
    fn write_removes_results_without_a_surviving_golden() {
        let mut tree = written(answer);
        for name in ["extra.bin", "record_drop_full.bin", "enum_shape_dot_full.bin.refused"] {
            tree.write(&path(name), b"stale").expect("stale");
        }
        golden(&mut tree, 1, answer, Mode::Write).expect("rewrite removes stale files");
        assert_eq!(names(&tree).len(), NAMES.len());
    }

    #[test]
    fn missing_written_functions_name_patch_site_and_target() {
        let mut tree = fixture(OLD, NEW, PATCH);
        let Error::Golden(failures) = golden(&mut tree, 1, no_site, Mode::Check).expect_err("unanswered sites") else {
            unreachable!("golden failures")
        };
        let sites =
            failures.iter().filter(|failure| failure.contains("patch 1 site 0 (written Shape)")).collect::<Vec<_>>();
        assert_eq!(sites.len(), SHAPE_GOLDENS);
    }

    #[test]
    fn declared_refusals_are_written_and_checked_as_refused_files() {
        let mut tree = written(refuse);
        for name in ["enum_shape_dot_full.bin.refused", "enum_shape_dot_smallest.bin.refused"] {
            assert_eq!(read(&tree, name), b"refuses no_dot\n");
        }
        assert_eq!(golden(&mut tree, 1, refuse, Mode::Check), Ok(()));
        golden(&mut tree, 1, answer, Mode::Write).expect("change refusal to success");
        assert_eq!(names(&tree).len(), NAMES.len());
    }

    #[test]
    fn planning_refusals_name_patch_family_line_and_reason() {
        let mut tree = fixture(OLD, NEW, "patch 1\nowner sample 1\nfamily small 1 to 2\nadd Data.score u8 ?\n");
        let Error::Golden(failures) = golden(&mut tree, 1, answer, Mode::Check).expect_err("asked value") else {
            unreachable!("golden failures")
        };
        assert_eq!(failures, ["patch 1, small: line 4: Asked"]);
    }
}
