//! Derived byte passes between two layouts (data/codec/layouts.md, 1–5).

use crate::schema::{Root, Schema, Span};
use alloc::boxed::Box;
use skein_lib::{Buffer, List};

mod derive;
mod run;
#[cfg(test)]
mod tests;

/// The most nested levels admitted by a plan.
pub const MAX_DEPTH: u32 = 64;

/// A refusal naming the new text, or the old text when `old` is true.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Underived {
    pub why: Why,
    pub decl: Span,
    pub part: Option<Span>,
    pub old: bool,
}

/// Why the two layouts need their owner's written conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Root,
    NoDefault,
    Retyped,
    Reshaped,
    VariantDropped,
    Payload,
    Held,
    Pieces,
    Deep,
}

/// Why a byte pass could not finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Misconverted {
    Short,
    Tag,
    Bool,
    Bound,
    Trailing,
    Room,
}

/// Owned tables, with no reference to either text.
#[derive(Clone, Debug)]
pub struct Plan {
    nodes: List<Node>,
    reads: List<Read>,
    root: u32,
    depth: u32,
    fields: u32,
    head: Option<u16>,
    old_head: bool,
    tag: Option<u8>,
}

#[derive(Clone, Debug)]
struct Node {
    read: u32,
    kind: Kind,
    keeps: bool,
    empty: bool,
    depth: u32,
}

#[derive(Clone, Debug)]
enum Kind {
    Pending,
    Copy,
    Integer { old: u32, new: u32 },
    Named(u32),
    Wrap(u32),
    Option(u32),
    List { bound: u32, item: u32 },
    Record { old_head: bool, head: Option<u16>, fields: List<Field>, in_place: bool },
    Enum(List<Option<Variant>>),
    Content { bound: u32, inner: Inner },
}

#[derive(Clone, Copy, Debug)]
enum Inner {
    Held(u32),
    Pieces(u32),
}

#[derive(Clone, Debug)]
struct Field {
    source: Source,
    node: Option<u32>,
}

#[derive(Clone, Debug)]
enum Source {
    Old(u32),
    Default(Box<[u8]>),
}

#[derive(Clone, Copy, Debug)]
struct Variant {
    tag: u8,
    payload: Option<u32>,
    in_place: bool,
}

#[derive(Clone, Debug)]
struct Read {
    kind: ReadKind,
    minimum: u32,
    depth: u32,
}

#[derive(Clone, Debug)]
enum ReadKind {
    Pending,
    Fixed(u32),
    Named(u32),
    Bool,
    Content(u32),
    Option(u32),
    List { bound: u32, item: u32 },
    Record { head: bool, fields: List<u32> },
    Enum { variants: List<Option<u32>>, only: Option<u8> },
}

/// Reusable mutable scratch, fitted before any pass (data/codec/layouts.md, 4–5).
#[derive(Clone, Debug)]
pub struct Workspace {
    frames: List<run::Frame>,
    scans: List<run::Scan>,
    spans: List<Span>,
    depth: u32,
    fields: u32,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    /// Empty scratch; allocates nothing.
    #[must_use]
    pub fn new() -> Workspace {
        Workspace { frames: List::uncapped(), scans: List::uncapped(), spans: List::uncapped(), depth: 0, fields: 0 }
    }

    /// Grows to serve this plan too, keeping all earlier room.
    #[must_use]
    pub fn fit(&mut self, plan: &Plan) -> bool {
        if plan.workspace_bound().is_none() {
            return false;
        }
        let depth = self.depth.max(plan.depth);
        let fields = self.fields.max(plan.fields);
        let Some(spans) = depth.checked_mul(fields) else { return false };
        if self.frames.len() < depth {
            self.frames = List::with_capacity(depth);
            for _ in 0..depth {
                self.frames.push(run::Frame::Empty).expect("workspace depth fits u32");
            }
        }
        if self.scans.len() < depth {
            self.scans = List::with_capacity(depth);
            for _ in 0..depth {
                self.scans.push(run::Scan::Empty).expect("workspace depth fits u32");
            }
        }
        if self.spans.len() < spans {
            self.spans = List::with_capacity(spans);
            for _ in 0..spans {
                self.spans.push(Span { start: 0, end: 0 }).expect("workspace span count checked");
            }
        }
        self.depth = depth;
        self.fields = fields;
        true
    }

    /// Whether a pass can use this workspace without allocating.
    #[must_use]
    pub fn fits(&self, plan: &Plan) -> bool {
        plan.workspace_bound().is_some() && self.depth >= plan.depth && self.fields >= plan.fields
    }
}

impl Plan {
    /// Plans only the two roots' reach, resolving held families on each side.
    pub fn derive(
        old: &Schema,
        old_root: Root,
        old_held: &[&Schema],
        new: &Schema,
        new_root: Root,
        new_held: &[&Schema],
    ) -> Result<Plan, Underived> {
        derive::plan(old, old_root, old_held, new, new_root, new_held)
    }

    /// Every old value already has the new bytes, apart from its root head.
    #[must_use]
    pub fn keeps(&self) -> bool {
        self.node(self.root).keeps
    }

    /// Clears `out`, then converts without allocating. An unfitted workspace is `Room`.
    pub fn convert(&self, old: &[u8], out: &mut Buffer, work: &mut Workspace) -> Result<(), Misconverted> {
        out.clear();
        if !work.fits(self) {
            return Err(Misconverted::Room);
        }
        run::convert(self, old, out, work)
    }

    /// The scratch tables' heap bound, fitted once and shared by passes.
    #[must_use]
    pub fn workspace_bound(&self) -> Option<u64> {
        let spans = self.depth.checked_mul(self.fields)?;
        let frames = List::<run::Frame>::price(self.depth)?;
        let readers = List::<run::Scan>::price(self.depth)?;
        let tables = List::<Span>::price(spans)?;
        frames.checked_add(readers)?.checked_add(tables)
    }

    fn node(&self, at: u32) -> &Node {
        self.nodes.get(at).expect("planned conversion")
    }

    fn read(&self, at: u32) -> &Read {
        self.reads.get(at).expect("planned span reader")
    }
}

fn add(a: u32, b: u32) -> u32 {
    a.checked_add(b).expect("admitted tables and byte offsets fit u32")
}
