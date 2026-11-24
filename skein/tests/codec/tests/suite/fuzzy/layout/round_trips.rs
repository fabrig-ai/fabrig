//! Keep/Read round trips over the `every` texts and one-edit pairs (skein/docs/design/data/codec/layouts.md, 2 and 6).

use super::tests::{accepts_root, roots};
use crate::support::generated;
use skein_codec::layout::{Misconverted, Mode, Plan, Refused, Underived, Why, Workspace};
use skein_codec::schema::{self, DeclIx, FieldIx, Root, Schema, Shape};
use skein_codec::value::Value;
use skein_lib::{Buffer, Rng};

const EVERY: [&str; 3] = [
    include_str!("../../../../schema/every-v1.schema"),
    include_str!("../../../../schema/every-v2.schema"),
    include_str!("../../../../schema/every-v3.schema"),
];
const DRAWS: u32 = 32;

type Refusal = (Refused, &'static str, &'static str);

struct Text {
    schema: Schema,
    draw: skein_codegen::Schema,
}

impl Text {
    fn parse(text: &str) -> Self {
        Self {
            schema: schema::parse(text.as_bytes()).expect("layout schema"),
            draw: skein_codegen::parse(text).expect("draw schema"),
        }
    }
}

#[derive(Clone, Copy)]
struct Site<'a> {
    schema: &'a Schema,
    root: Root,
}

impl Site<'_> {
    fn plan(self, mode: Mode, to: Self) -> Result<Plan, Underived> {
        // The generator draws opaque contents for holds. Neither side resolves
        // them; pieces of an unchanged type also stay opaque to the pass.
        Plan::between(mode, self.schema, self.root, &[], to.schema, to.root, &[])
    }

    fn field(self, name: &str) -> FieldIx {
        let Shape::Record { first, count, .. } = self.schema.declaration(self.root.decl).shape else {
            unreachable!("restored field belongs to a record")
        };
        (first.index()..first.index().checked_add(count).expect("field range fits"))
            .map(FieldIx::new)
            .find(|&at| self.schema.spelled(self.schema.field(at).name) == name.as_bytes())
            .expect("restored field in source")
    }
}

struct Scratch {
    work: Workspace,
    out: Buffer,
    value: Value,
}

impl Scratch {
    fn new() -> Self {
        Self { work: Workspace::new(), out: Buffer::with_capacity(4096), value: Value::with_room(512) }
    }
}

struct Trip<'a> {
    from: Site<'a>,
    to: Site<'a>,
    keep: Plan,
    back: Result<Plan, Underived>,
    label: &'a str,
    expected: Expect,
}

#[derive(Clone, Copy, Debug)]
enum Drawn {
    Equal,
    Restored,
    Refused,
}

#[derive(Default)]
struct Seen {
    equal: bool,
    restored: bool,
    refused: bool,
}

impl Seen {
    fn record(&mut self, outcome: Drawn) {
        match outcome {
            Drawn::Equal => self.equal = true,
            Drawn::Restored => self.restored = true,
            Drawn::Refused => self.refused = true,
        }
    }

    fn check(&self, expected: Expect, label: &str) {
        let seen = (self.equal, self.restored, self.refused);
        let wanted = match expected {
            Exact => (true, false, false),
            Mixed(_) => (true, false, true),
            AllRefused(_) => (false, false, true),
            Restore(_, _) => (true, true, false),
            Unplanned(_) => unreachable!("a root with no plan is not sampled"),
        };
        assert_eq!(seen, wanted, "{label}: saw (equal, restored, refused) for {expected:?}");
    }
}

impl<'a> Trip<'a> {
    fn plan(from: Site<'a>, to: Site<'a>, scratch: &mut Scratch, label: &'a str, expected: Expect) -> Option<Self> {
        let keep = from.plan(Mode::Keep, to);
        let read = from.plan(Mode::Read, to);
        assert_eq!(keep.as_ref().err(), read.as_ref().err(), "{label}: plan errors in both modes");
        if let Unplanned(why) = expected {
            assert_eq!(keep.err().map(|error| error.why), Some(why), "{label}: expected no plan");
            return None;
        }
        assert!(keep.is_ok(), "{label}: expected a plan, got {:?}", keep.err());
        let keep = keep.unwrap();
        let back = to.plan(Mode::Read, from);
        assert!(scratch.work.fit(&keep), "{label}: Keep workspace");
        if let Ok(back) = &back {
            assert!(scratch.work.fit(back), "{label}: Read workspace");
        }
        Some(Self { from, to, keep, back, label, expected })
    }

    fn check(&self, bytes: &mut [u8], scratch: &mut Scratch, round: u32) -> Drawn {
        let label = self.label;
        let converted = self.keep.convert(bytes, &mut scratch.out, &mut scratch.work);
        if let Err(Misconverted::Refused(index)) = converted {
            let found = self.keep.refusal(index);
            let (Mixed(refusal) | AllRefused(refusal)) = self.expected else {
                unreachable!("{label} draw {round}: unexpected refusal {found:?}")
            };
            assert_eq!(found, Some(refusal), "{label} draw {round}: refusal's source part");
            return Drawn::Refused;
        }
        assert_eq!(converted, Ok(()), "{label} draw {round}: Keep conversion");
        let target = scratch.out.to_boxed();
        assert!(accepts_root(self.to.schema, self.to.root, &target), "{label} draw {round}: target variant");
        let decoded = scratch.value.decode(self.to.schema, self.to.root.decl, target);
        assert!(decoded.is_ok(), "{label} draw {round}: target cannot decode: {decoded:?}");
        let back = self.back.as_ref();
        assert!(back.is_ok(), "{label} draw {round}: Keep converted, but no way back: {:?}", back.err());
        let converted = back.unwrap().convert(scratch.value.bytes(), &mut scratch.out, &mut scratch.work);
        assert_eq!(converted, Ok(()), "{label} draw {round}: Read conversion");

        let mut outcome = Drawn::Equal;
        if let Restore(field, default) = self.expected {
            let root = scratch.value.decode(self.from.schema, self.from.root.decl, (&*bytes).into()).unwrap();
            let field = scratch.value.field(root, self.from.field(field)).unwrap();
            let span = scratch.value.span(field).unwrap();
            assert_eq!(span.end.checked_sub(span.start), Some(1), "restore a single-byte field");
            let byte = bytes.get_mut(usize::try_from(span.start).unwrap()).expect("field in source bytes");
            if *byte != default {
                *byte = default;
                outcome = Drawn::Restored;
            }
        }
        assert_eq!(scratch.out.as_slice(), bytes, "{label} draw {round}: round trip");
        outcome
    }
}

#[test]
fn shipped_every_layouts_round_trip_seeded_draws() {
    let texts = EVERY.map(Text::parse);
    let mut rng = Rng::new(0x1A70_0750_0000_0003);
    let mut scratch = Scratch::new();
    for old in &texts {
        for new in &texts {
            let a = old.schema.version();
            let b = new.schema.version();
            if a == b {
                continue;
            }
            let mut any_round_trips = false;
            for at in 0..old.schema.declarations() {
                let decl = DeclIx::new(at);
                let name = old.schema.spelled(old.schema.declaration(decl).name);
                let root_name = str::from_utf8(name).unwrap();
                for root in roots(&old.schema, decl) {
                    let variant = root.variant.map(|at| old.schema.spelled(old.schema.variant(at).name));
                    let Some(target) = new.schema.root(name, variant) else { continue };
                    let from = Site { schema: &old.schema, root };
                    let to = Site { schema: &new.schema, root: target };
                    let label = format!("every v{a} → v{b} {root_name} {variant:?}");
                    let expected = shipped_outcome(a, b, root_name);
                    let Some(trip) = Trip::plan(from, to, &mut scratch, &label, expected) else { continue };
                    let mut seen = Seen::default();
                    for round in 0..DRAWS {
                        let mut bytes = generated::draw(&old.draw, root_name, &mut rng);
                        if accepts_root(&old.schema, root, &bytes) {
                            seen.record(trip.check(&mut bytes, &mut scratch, round));
                        }
                    }
                    seen.check(expected, &label);
                    any_round_trips |= seen.equal;
                }
            }
            assert!(any_round_trips, "every v{a} → v{b}: no root made a round trip");
        }
    }
}

fn shipped_outcome(source: u16, target: u16, root: &str) -> Expect {
    match (source, target, root) {
        (1, 2 | 3, "Numbers" | "Stamp" | "Nest" | "Every") | (2 | 3, 1, "Numbers" | "Nest") => {
            Unplanned(Why::NoDefault)
        }
        (1, 2 | 3, "Shape") | (2 | 3, 1, "Shape") => Unplanned(Why::Payload),
        (2 | 3, 1, "Every") => Unplanned(Why::Retyped),
        // Neither `off` nor `zone` exists in v1's matching declaration.
        (2 | 3, 1, "Light") => Mixed((Refused::Variant, "Light", "off")),
        (2 | 3, 1, "Stamp") => AllRefused((Refused::Field, "Stamp", "zone")),
        _ => Exact,
    }
}

#[derive(Clone, Copy, Debug)]
enum Expect {
    Exact,
    Mixed(Refusal),
    AllRefused(Refusal),
    Restore(&'static str, u8),
    Unplanned(Why),
}

struct Edit {
    name: &'static str,
    root: &'static str,
    source: Option<(&'static str, &'static str)>,
    target: Change,
    there: Expect,
    back: Expect,
}

enum Change {
    Replace(&'static str, &'static str),
    Line(&'static str, &'static str),
}

use Change::{Line, Replace};
use Expect::{AllRefused, Exact, Mixed, Restore, Unplanned};

const FLAG: &str = "    flag: bool\n";
const FLAG_REFUSAL: Refusal = (Refused::Field, "Numbers", "flag");
const EDITS: [Edit; 16] = [
    Edit {
        name: "drop with default",
        root: "Numbers",
        source: Some((FLAG, "    flag: bool = true\n")),
        target: Replace(FLAG, ""),
        there: Mixed(FLAG_REFUSAL),
        back: Exact,
    },
    Edit {
        name: "drop without default",
        root: "Numbers",
        source: None,
        target: Replace(FLAG, ""),
        there: AllRefused(FLAG_REFUSAL),
        back: Unplanned(Why::NoDefault),
    },
    Edit {
        name: "drop as tombstone",
        root: "Numbers",
        source: None,
        target: Replace(FLAG, "    dropped flag: bool = true\n"),
        there: Mixed(FLAG_REFUSAL),
        back: Exact,
    },
    Edit {
        name: "drop droppable field",
        root: "Numbers",
        source: Some((FLAG, "    droppable flag: bool = true\n")),
        target: Replace(FLAG, ""),
        there: Restore("flag", 1),
        back: Exact,
    },
    Edit {
        name: "add with default",
        root: "Numbers",
        source: None,
        target: Replace(FLAG, "    flag: bool\n    extra: bool = false\n"),
        there: Exact,
        back: Mixed((Refused::Field, "Numbers", "extra")),
    },
    Edit {
        name: "narrow integer",
        root: "Numbers",
        source: None,
        target: Replace("    short: u16\n", "    short: u8\n"),
        there: Mixed((Refused::Narrowed, "Numbers", "short")),
        back: Exact,
    },
    Edit {
        name: "widen integer",
        root: "Numbers",
        source: None,
        target: Replace("    small: u8\n", "    small: u16\n"),
        there: Exact,
        back: Mixed((Refused::Narrowed, "Numbers", "small")),
    },
    Edit {
        name: "narrow bound",
        root: "Stamp",
        source: None,
        target: Replace("    note: option text 4\n", "    note: option text 2\n"),
        there: Mixed((Refused::Narrowed, "Stamp", "note")),
        back: Exact,
    },
    Edit {
        name: "widen bound",
        root: "Stamp",
        source: None,
        target: Replace("    note: option text 4\n", "    note: option text 8\n"),
        there: Exact,
        back: Mixed((Refused::Narrowed, "Stamp", "note")),
    },
    Edit {
        name: "append variant",
        root: "Light",
        source: None,
        target: Replace("red, amber, green }", "red, amber, green, blue }"),
        there: Exact,
        back: Mixed((Refused::Variant, "Light", "blue")),
    },
    Edit {
        name: "drop variant",
        root: "Light",
        source: None,
        target: Replace("red, amber, green }", "red, green }"),
        there: Mixed((Refused::Variant, "Light", "amber")),
        back: Exact,
    },
    Edit {
        name: "wrap option",
        root: "Numbers",
        source: None,
        target: Replace(FLAG, "    flag: option bool\n"),
        there: Exact,
        back: Mixed((Refused::Absent, "Numbers", "flag")),
    },
    Edit {
        name: "take away option",
        root: "Stamp",
        source: None,
        target: Replace("    note: option text 4\n", "    note: text 4 = \"\"\n"),
        there: Mixed((Refused::Absent, "Stamp", "note")),
        back: Exact,
    },
    Edit {
        name: "rename with was",
        root: "Numbers",
        source: None,
        target: Replace(FLAG, "    ready was flag: bool\n"),
        there: Exact,
        back: Exact,
    },
    Edit {
        name: "retype",
        root: "Numbers",
        source: None,
        target: Replace(FLAG, "    flag: text 4\n"),
        there: Unplanned(Why::Retyped),
        back: Unplanned(Why::Retyped),
    },
    Edit {
        name: "reshape",
        root: "Light",
        source: None,
        target: Line("enum Light {", "record Light {}"),
        there: Unplanned(Why::Reshaped),
        back: Unplanned(Why::Reshaped),
    },
];

fn edited(text: &str, (before, after): (&str, &str)) -> String {
    assert_eq!(text.matches(before).count(), 1, "edit must name exactly one site: {before:?}");
    text.replace(before, after)
}

#[test]
fn edited_every_layouts_round_trip_seeded_draws() {
    let mut rng = Rng::new(0x1A70_0750_0000_0004);
    let mut scratch = Scratch::new();
    for text in EVERY {
        let base = Text::parse(text);
        for edit in &EDITS {
            let adjusted = edit.source.map(|change| Text::parse(&edited(text, change)));
            let replacement = match edit.target {
                Replace(before, after) => (before, after),
                Line(prefix, after) => {
                    (text.lines().find(|line| line.starts_with(prefix)).expect("declaration line"), after)
                }
            };
            let changed = Text::parse(&edited(text, replacement));
            let old = adjusted.as_ref().unwrap_or(&base);
            for (direction, source, target, expected) in
                [("base → edit", old, &changed, edit.there), ("edit → base", &changed, old, edit.back)]
            {
                let label = format!("every v{} {} {direction} {}", base.schema.version(), edit.name, edit.root);
                let name = edit.root.as_bytes();
                let from = Site { schema: &source.schema, root: source.schema.root(name, None).expect("source root") };
                let to = Site { schema: &target.schema, root: target.schema.root(name, None).expect("target root") };
                let Some(trip) = Trip::plan(from, to, &mut scratch, &label, expected) else { continue };
                let mut seen = Seen::default();
                for round in 0..DRAWS {
                    let mut bytes = generated::draw(&source.draw, edit.root, &mut rng);
                    seen.record(trip.check(&mut bytes, &mut scratch, round));
                }
                seen.check(expected, &label);
            }
        }
    }
}
