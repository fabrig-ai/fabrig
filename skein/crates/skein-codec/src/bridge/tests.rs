//! Small kept texts and hand-written wire bytes (data/codec/bridges.md, 7).
#![expect(clippy::disallowed_macros, reason = "small schema fixtures use formatting")]

use super::{Bridge, BridgeError, layouts};
use crate::patch::{Build, Capacity, Halt, Old, PatchWrong, Patcher, PlanIx, RefusalIx, Site, Step};
use crate::schema::{self, DeclIx, FieldIx};
use crate::value::{Node, Part, Path, Value};

const CAPACITY: Capacity = Capacity { versions: 4, patches: 8, text: 4096, bytes: 256, nodes: 64 };
const D: DeclIx = DeclIx::new(0);
const FIELD: Path = Path { decl: D, part: Part::Field(FieldIx::new(0)) };
const HEADER: &str = "patch 1\nowner o 7\nfamily f 1 to 2\n";

fn kept(old: &str, new: &str) -> Patcher {
    let mut machine = Patcher::new(CAPACITY);
    machine.keep(format!("family f 1\n{old}").as_bytes()).expect("older text");
    machine.keep(format!("family f 2\n{new}").as_bytes()).expect("newer text");
    machine
}

fn reverse(old: &str, new: &str, lines: &str) -> (Patcher, PlanIx) {
    let mut machine = kept(old, new);
    let plan = machine.reverse(format!("{HEADER}{lines}").as_bytes()).expect("reverse plans").expect("family");
    assert_eq!(machine.reverse_from(2), Some(plan));
    assert_eq!(machine.plan_from(2), None);
    (machine, plan)
}

fn run(machine: &mut Patcher, plan: PlanIx, root: &str, bytes: &[u8]) -> Result<Box<[u8]>, Halt> {
    let version = machine.versions(plan).from;
    let decl = machine.schema(version).expect("source schema").find(root.as_bytes()).expect("root");
    machine.start(plan, decl, bytes.into())?;
    assert_eq!(machine.next()?, Step::Done);
    Ok(machine.finish()?.into())
}

fn no_sites(site: Site, _old: &Old, _new: &mut Build, _at: Node) -> Result<(), Halt> {
    Err(Halt::NoSite(site))
}

#[test]
fn reverses_derived_changes_into_hand_written_older_bytes() {
    // Each row holds a conversion rule, not a list of production operations.
    for (old, new, lines, root, input, expected) in [
        (
            "record R { a: u8 }",
            "record R { a: u8 b: option u8 }",
            "add R.b option u8 none\n",
            "R",
            &[7, 0][..],
            &[7][..],
        ),
        (
            "record Gone {} record R { a: u8 }",
            "record R { a: u8 } record Added {}",
            "drop Gone\nadd Added\n",
            "R",
            &[7],
            &[7],
        ),
        (
            "record Before { a: u8 b: bool }",
            "record After { b: bool renamed: u8 }",
            "rename Before After\nrename After.a renamed\norder After b renamed\n",
            "After",
            &[1, 7],
            &[7, 1],
        ),
        (
            "enum E { a, b } record R { e: E }",
            "enum E { b, a, c } record R { e: E }",
            "order E b a\nadd E.c\n",
            "R",
            &[0],
            &[1],
        ),
        (
            "enum E { a, b } record R { e: E }",
            "enum E { b } record R { e: E }",
            "drop E.a\nwritten E\n",
            "R",
            &[0],
            &[1],
        ),
        ("record R { a: u8 b: u16 }", "record R { a: u8 }", "drop R.b\ndown R.b 513\n", "R", &[7], &[7, 2, 1]),
        ("record R { a: u8 }", "record R { a: u32 }", "widen R.a u8 u32\n", "R", &[0, 0, 0, 255], &[255]),
        (
            "record R { a: text 2 }",
            "record R { a: text 3 }",
            "",
            "R",
            &[0, 0, 0, 2, b'h', b'i'],
            &[0, 0, 0, 2, b'h', b'i'],
        ),
        (
            "record R { a: text 2 }",
            "record R { a: bytes 3 }",
            "widen R.a text 2 bytes 3\n",
            "R",
            &[0, 0, 0, 1, b'x'],
            &[0, 0, 0, 1, b'x'],
        ),
        ("record R { a: u8 }", "record R { a: option u8 }", "wrap R.a u8 option u8\n", "R", &[1, 7], &[7]),
        (
            "record R { a: list 1 option u8 }",
            "record R { a: list 2 option option u16 }",
            "wrap R.a list 1 option u8 list 2 option option u16\n",
            "R",
            &[0, 0, 0, 1, 1, 1, 0, 7],
            &[0, 0, 0, 1, 1, 7],
        ),
        ("record R { a: u8 }", "versioned record R { a: u8 }", "versioned R\n", "R", &[0, 2, 7], &[7]),
        ("versioned record R { a: u8 }", "record R { a: u8 }", "unversioned R\n", "R", &[7], &[0, 1, 7]),
        (
            "versioned record Inner { a: u8 } versioned record R { inner: Inner }",
            "versioned record Inner { a: u8 } versioned record R { inner: Inner }",
            "",
            "R",
            &[0, 2, 0, 2, 7],
            &[0, 1, 0, 1, 7],
        ),
    ] {
        let (mut machine, plan) = reverse(old, new, lines);
        assert_eq!(run(&mut machine, plan, root, input), Ok(expected.into()), "{lines} {new}");
        let older = machine.schema(1).expect("older text");
        let old_root = machine
            .counterpart(plan, machine.schema(2).expect("newer text").find(root.as_bytes()).expect("new root"))
            .expect("old root");
        Value::with_room(CAPACITY.nodes)
            .decode(older, old_root, expected.into())
            .expect("hand bytes decode by older text");
    }
}

#[test]
fn added_value_and_appended_variant_are_unheld_only_on_the_other_side() {
    let (mut machine, plan) =
        reverse("record R { a: u8 }", "record R { a: u8 b: option u8 }", "add R.b option u8 none\n");
    assert_eq!(run(&mut machine, plan, "R", &[7, 0]), Ok([7].as_slice().into()));
    assert_eq!(
        run(&mut machine, plan, "R", &[7, 1, 3]),
        Err(Halt::Unheld(Path { decl: D, part: Part::Field(FieldIx::new(1)) }))
    );
    // No add line: an appended variant is an implicit change of the forward plan.
    let (mut machine, plan) = reverse("enum E { a } record R { e: E }", "enum E { a, b } record R { e: E }", "");
    assert_eq!(run(&mut machine, plan, "R", &[0]), Ok([0].as_slice().into()));
    assert_eq!(run(&mut machine, plan, "R", &[1]), Err(Halt::Unheld(Path { decl: D, part: Part::Tag })));
}

#[test]
fn narrowed_widths_bounds_utf8_and_options_judge_the_actual_value() {
    for (old, new, lines, good, expected, bad) in [
        ("u8", "u16", "widen R.a u8 u16\n", &[0, 255][..], &[255][..], &[1, 0][..]),
        ("text 2", "text 3", "", &[0, 0, 0, 2, b'o', b'k'], &[0, 0, 0, 2, b'o', b'k'], &[0, 0, 0, 3, b'b', b'a', b'd']),
        (
            "text 2",
            "bytes 2",
            "widen R.a text 2 bytes 2\n",
            &[0, 0, 0, 1, b'x'],
            &[0, 0, 0, 1, b'x'],
            &[0, 0, 0, 1, 255],
        ),
        ("bytes 1", "bytes 2", "", &[0, 0, 0, 1, 4], &[0, 0, 0, 1, 4], &[0, 0, 0, 2, 4, 5]),
        ("secret 1", "secret 2", "", &[0, 0, 0, 1, 4], &[0, 0, 0, 1, 4], &[0, 0, 0, 2, 4, 5]),
        ("list 1 u8", "list 2 u8", "", &[0, 0, 0, 1, 7], &[0, 0, 0, 1, 7], &[0, 0, 0, 2, 7, 8]),
        ("u8", "option u8", "wrap R.a u8 option u8\n", &[1, 7], &[7], &[0]),
    ] {
        let (mut machine, plan) =
            reverse(&format!("record R {{ a: {old} }}"), &format!("record R {{ a: {new} }}"), lines);
        assert_eq!(run(&mut machine, plan, "R", good), Ok(expected.into()), "{old} to {new}");
        assert_eq!(run(&mut machine, plan, "R", bad), Err(Halt::Unheld(FIELD)), "{old} to {new}");
    }
}

#[test]
fn reverse_opened_pieces_and_held_bytes_obey_the_older_bound() {
    let (mut machine, plan) = reverse(
        "record Item { a: u8 } record R { a: bytes 5 pieces Item }",
        "record Item { a: u16 } record R { a: bytes 12 pieces Item }",
        "widen Item.a u8 u16\n",
    );
    assert_eq!(
        run(&mut machine, plan, "R", &[0, 0, 0, 6, 0, 0, 0, 2, 0, 7]),
        Ok([0, 0, 0, 5, 0, 0, 0, 1, 7].as_slice().into())
    );
    assert_eq!(
        run(&mut machine, plan, "R", &[0, 0, 0, 12, 0, 0, 0, 2, 0, 7, 0, 0, 0, 2, 0, 8]),
        Err(Halt::Unheld(Path { decl: DeclIx::new(1), part: Part::Field(FieldIx::new(1)) }))
    );
    let texts = [
        "family f 1\nrecord R { a: bytes 1 holds other Thing }",
        "family f 2\nrecord R { a: bytes 2 holds other Thing }",
    ];
    let patches = ["patch 1\nowner o 1\nfamily f 1 to 2\n"];
    let mut bridge = Bridge::new(CAPACITY, &texts, &patches, 1, no_sites).expect("held bridge");
    let good = [0, 0, 0, 1, 7].as_slice();
    assert_eq!(bridge.translate(2, 1, b"R", good.into()), Ok(good.into()));
    assert_eq!(bridge.translate(2, 1, b"R", [0, 0, 0, 2, 7, 8].as_slice().into()), Err(Halt::Unheld(FIELD)));
}

#[test]
fn reversing_piece_clauses_follows_what_the_forward_conversion_carried() {
    let input = [0, 0, 0, 6, 0, 0, 0, 2, 0, 7].as_slice();
    for (old_clause, new_clause, expected) in
        [(" pieces Item", "", [0, 0, 0, 5, 0, 0, 0, 1, 7].as_slice()), ("", " pieces Item", input)]
    {
        let texts = [
            format!("family f 1\nrecord Item {{ a: u8 }} record R {{ a: bytes 12{old_clause} }}"),
            format!("family f 2\nrecord Item {{ a: u16 }} record R {{ a: bytes 12{new_clause} }}"),
        ];
        let patches = ["patch 1\nowner o 1\nfamily f 1 to 2\nwiden Item.a u8 u16\n"];
        let mut bridge = Bridge::new(
            CAPACITY,
            &[texts.first().expect("older"), texts.get(1).expect("newer")],
            &patches,
            1,
            no_sites,
        )
        .expect("piece clause bridge");
        assert_eq!(bridge.translate(2, 1, b"R", input.into()), Ok(expected.into()), "{old_clause} to {new_clause}");
    }
}

#[test]
fn missing_reverse_names_its_forward_line_and_breaking_has_no_reverse() {
    for (old, new, lines, wrong, line) in [
        ("record R { a: u8 }", "record R {}", "drop R.a\n", PatchWrong::Irreversible, 4),
        (
            "record R { a: u8 }",
            "record R { a: bool }",
            "retype R.a u8 bool\nwritten R.a\n",
            PatchWrong::Irreversible,
            4,
        ),
        ("record R { a: u8 }", "record R {}", "drop R.a\nbreaking \"a no longer exists\"\n", PatchWrong::Breaking, 5),
    ] {
        let mut machine = kept(old, new);
        machine.plan(format!("{HEADER}{lines}").as_bytes()).expect("forward still plans");
        let error = machine.reverse(format!("{HEADER}{lines}").as_bytes()).expect_err("no reverse");
        assert_eq!((error.wrong, error.line), (wrong, line));
    }
}

#[test]
fn down_lines_are_checked_in_forward_plans_and_cannot_accompany_breaking() {
    for lines in [
        "drop R.a\nbreaking \"gone\"\ndown R.a 0\n",
        "drop R.a\ndown R.a 0\nbreaking \"gone\"\n",
        "drop R.a\nbreaking \"gone\"\ndown written R\n",
    ] {
        let mut machine = kept("record R { a: u8 }", "record R {}");
        assert_eq!(
            machine.plan(format!("{HEADER}{lines}").as_bytes()).expect_err("down beside breaking").wrong,
            PatchWrong::Twice
        );
    }
    for lines in ["drop R.a\ndown R.a 0\ndown written R\n", "drop R.a\ndown written R\ndown R.a 0\n"] {
        let mut machine = kept("record R { a: u8 }", "record R {}");
        assert_eq!(
            machine.plan(format!("{HEADER}{lines}").as_bytes()).expect_err("overlapping down targets").wrong,
            PatchWrong::Site
        );
    }
}

#[test]
fn down_sites_share_text_order_and_own_their_declared_refusals() {
    // Sites 0 and 1 belong to another family; this block's forward is 2 and down is 3.
    for target in ["R", "R.a"] {
        let patch = format!(
            "patch 1\nowner o 7\nfamily other 1 to 2\nwritten R\ndown written R\nfamily f 1 to 2\nretype R.a u8 bool\nwritten R.a\ndown written {target}\nrefuses no \"no old value\"\n"
        );
        let mut machine = kept("record R { a: u8 }", "record R { a: bool }");
        let plan = machine.reverse(patch.as_bytes()).expect("reverse").expect("family");
        machine.start(plan, D, Box::from([1])).expect("start");
        let at = match machine.next().expect("stop") {
            Step::Written { site, at } => {
                assert_eq!(site, Site::new(7, 3));
                at
            }
            other @ (Step::Held { .. } | Step::Done) => unreachable!("written, got {other:?}"),
        };
        let (old, new) = machine.parts();
        let flag = old.boolean(old.field(at, FieldIx::new(0)).expect("field")).expect("boolean");
        assert!(flag);
        assert_eq!(new.refuse(RefusalIx::new(0)), Halt::Refused(RefusalIx::new(0)));
        assert_eq!(machine.refusal(RefusalIx::new(0)), Some(b"no old value".as_slice()));
    }
    for target in ["R", "R.a"] {
        let (mut machine, plan) = reverse(
            "record R { a: u8 }",
            "record R { a: bool }",
            &format!("retype R.a u8 bool\nwritten R.a\ndown written {target}\n"),
        );
        assert!(machine.total(plan, D));
        machine.start(plan, D, Box::from([1])).expect("start");
        match machine.next().expect("written stop") {
            Step::Written { site, .. } => assert_eq!(site.index(), 1),
            Step::Held { .. } | Step::Done => unreachable!("written site"),
        }
        let (_, new) = machine.parts();
        if target == "R" {
            new.open().expect("record");
        }
        new.put_uint(7).expect("old value");
        if target == "R" {
            new.close().expect("record end");
        }
        assert_eq!(machine.next(), Ok(Step::Done));
        assert_eq!(machine.finish(), Ok([7].as_slice()));
    }
}

#[test]
fn authored_reverses_build_reshaped_declarations_and_changed_payloads() {
    for (old, new, lines, root, input, variant, expected) in [
        (
            "record R { a: u8 }",
            "enum R { yes }",
            "reshape R\nwritten R\ndown written R\n",
            "R",
            &[0][..],
            false,
            &[7][..],
        ),
        (
            "record P { a: u8 } enum E { keep: P }",
            "record P { a: u8 } record Q { b: bool } enum E { keep: Q }",
            "add Q\npayload E.keep P Q\nwritten E\ndown written E\n",
            "E",
            &[0, 1],
            true,
            &[0, 7],
        ),
    ] {
        let (mut machine, plan) = reverse(old, new, lines);
        let decl = machine.schema(2).expect("newer").find(root.as_bytes()).expect("root");
        machine.start(plan, decl, input.into()).expect("newer record");
        match machine.next().expect("down stop") {
            Step::Written { site, .. } => assert_eq!(site.index(), 1),
            Step::Held { .. } | Step::Done => unreachable!("authored reverse"),
        }
        let (_, builder) = machine.parts();
        if variant {
            builder.open_variant(schema::VariantIx::new(0)).expect("older variant");
        } else {
            builder.open().expect("older record");
        }
        builder.put_uint(7).expect("older field");
        builder.close().expect("older shape complete");
        assert_eq!(machine.next(), Ok(Step::Done));
        assert_eq!(machine.finish(), Ok(expected));
        let schema = machine.schema(1).expect("older");
        Value::with_room(CAPACITY.nodes)
            .decode(schema, schema.find(root.as_bytes()).expect("older root"), expected.into())
            .expect("older decode");
    }
}

#[test]
fn an_added_written_field_uses_the_round_trip_not_its_forward_site_down() {
    fn sites(site: Site, _old: &Old, new: &mut Build, _at: Node) -> Result<(), Halt> {
        assert_eq!(site, Site::new(1, 0));
        new.put_uint(7)
    }
    let texts = ["family f 1\nrecord R {}", "family f 2\nrecord R { a: u8 }"];
    let patches = ["patch 1\nowner o 1\nfamily f 1 to 2\nadd R.a u8 written\nwritten R.a\n"];
    let mut bridge = Bridge::new(CAPACITY, &texts, &patches, 1, sites).expect("derived reverse of forward site");
    assert_eq!(bridge.total(1, b"R"), Some(false));
    assert_eq!(bridge.translate(2, 1, b"R", [7].as_slice().into()), Ok(Box::default()));
    assert_eq!(bridge.translate(2, 1, b"R", [8].as_slice().into()), Err(Halt::Altered));
}

#[test]
fn total_follows_the_roots_reach_not_other_declarations() {
    for (old, new, lines, expected) in [
        ("enum E { a } record R { e: E }", "enum E { a, b } record R { e: E }", "", false),
        (
            "record E { a: u8 } record R { e: E }",
            "record E { a: u8 b: option u8 } record R { e: E }",
            "add E.b option u8 none\n",
            false,
        ),
        (
            "record E { a: u8 } record R { e: E }",
            "record E { a: bool } record R { e: E }",
            "retype E.a u8 bool\nwritten E.a\ndown written E\nrefuses bad \"bad\"\n",
            false,
        ),
        (
            "record E { a: u8 b: bool } record R { e: E }",
            "record E { b: bool renamed: u8 } record R { e: E }",
            "rename E.a renamed\norder E b renamed\n",
            true,
        ),
        (
            "record E { a: u8 } record R { e: E }",
            "record E { a: bool } record R { e: E }",
            "retype E.a u8 bool\nwritten E.a\ndown written E\n",
            true,
        ),
        (
            "record E { a: u8 } record R { e: E }",
            "record E { a: bool } record R { e: E }",
            "retype E.a u8 bool\nwritten E.a\ndown written E.a\n",
            true,
        ),
    ] {
        let old = format!("{old} record Separate {{ a: u8 }}");
        let new = format!("{new} record Separate {{ a: u8 }}");
        let (machine, plan) = reverse(&old, &new, lines);
        let schema = machine.schema(2).expect("newer");
        assert_eq!(machine.total(plan, schema.find(b"R").expect("root")), expected, "{lines}");
        assert!(machine.total(plan, schema.find(b"Separate").expect("separate")));
    }
}

const TEXTS: &[&str] = &[
    "family f 1\nversioned record R { a: u8 }",
    "family f 2\nversioned record R { a: u16 }",
    "family f 3\nversioned record Renamed { a: u16 b: option u8 }",
];
const PATCHES: &[&str] = &[
    "patch 1\nowner o 1\nfamily f 1 to 2\nwiden R.a u8 u16\n",
    "patch 1\nowner o 2\nfamily f 2 to 3\nrename R Renamed\nadd Renamed.b option u8 none\n",
];

#[test]
fn bridge_speaks_every_pair_above_floor_and_reads_older_versions() {
    let mut bridge = Bridge::new(CAPACITY, TEXTS, PATCHES, 2, no_sites).expect("bridge");
    assert_eq!((bridge.family(), bridge.first(), bridge.floor(), bridge.current()), (b"f".as_slice(), 1, 2, 3));
    let records = [&[0, 1, 7][..], &[0, 2, 0, 7][..], &[0, 3, 0, 7, 0][..]];
    for from in 1_u16..=3 {
        for to in 2_u16..=3 {
            let input = *records.get(usize::from(from.checked_sub(1).expect("version"))).expect("version");
            assert_eq!(
                bridge.translate(from, to, b"Renamed", input.into()),
                Ok((*records.get(usize::from(to.checked_sub(1).expect("version"))).expect("version")).into())
            );
        }
    }
    assert_eq!(bridge.translate(3, 1, b"Renamed", (*records.get(2).expect("third")).into()), Err(Halt::Order));
    assert_eq!(bridge.translate(1, 1, b"Renamed", [0, 1, 7].as_slice().into()), Err(Halt::Order));
    assert_eq!(bridge.translate(0, 2, b"Renamed", Box::default()), Err(Halt::Order));
    assert_eq!(bridge.total(2, b"Renamed"), Some(false));
    assert_eq!(bridge.total(3, b"Renamed"), Some(true));
    assert_eq!(bridge.total(1, b"Renamed"), None);
    assert_eq!(bridge.total(2, b"missing"), None);
    for (index, text) in TEXTS.iter().enumerate() {
        assert_eq!(
            bridge.layout(u16::try_from(index.checked_add(1).expect("index")).expect("version")),
            Some(schema::parse(text.as_bytes()).expect("text").fingerprints().family().digest())
        );
    }
    let layouts = layouts(TEXTS, 2).expect("layouts");
    assert_eq!(layouts.len(), 2);
    assert_eq!(layouts.get(0).copied(), bridge.layout(2));
    assert_eq!(layouts.get(1).copied(), bridge.layout(3));
    assert_eq!(bridge.layout(4), None);
}

#[test]
fn floor_needs_only_the_reverses_above_it() {
    let texts = ["family f 1\nrecord R { a: u8 }", "family f 2\nrecord R {}"];
    let patches = ["patch 1\nowner o 1\nfamily f 1 to 2\ndrop R.a\n"];
    Bridge::new(CAPACITY, &texts, &patches, 2, no_sites).expect("no reverse below floor needed");
    let error = Bridge::new(CAPACITY, &texts, &patches, 1, no_sites).expect_err("needed reverse");
    match error {
        BridgeError::Reverse { version, error } => {
            assert_eq!(version, 2);
            assert_eq!(error.wrong, PatchWrong::Irreversible);
        }
        BridgeError::Keep(_) | BridgeError::Plan { .. } | BridgeError::Floor | BridgeError::Room => {
            unreachable!("reverse error")
        }
    }
}

#[test]
fn round_trip_guards_written_loss_and_two_reverse_passes() {
    fn sites(_site: Site, _old: &Old, new: &mut Build, _at: Node) -> Result<(), Halt> {
        new.open()?;
        new.put_uint(0)?;
        new.close()
    }
    let texts = ["family f 1\nrecord R { a: u8 }", "family f 2\nrecord R { a: u8 b: option u8 }"];
    let patches = ["patch 1\nowner o 1\nfamily f 1 to 2\nadd R.b option u8 none\ndown written R\n"];
    let mut bridge = Bridge::new(CAPACITY, &texts, &patches, 1, sites).expect("bridge with down site");
    assert_eq!(bridge.total(1, b"R"), Some(true), "a site without refusals is trusted");
    assert_eq!(bridge.translate(2, 1, b"R", Box::from([7, 0])), Err(Halt::Altered));
    assert_eq!(bridge.translate(2, 1, b"R", Box::from([0, 0])), Ok([0].as_slice().into()));
    let mut bridge = Bridge::new(CAPACITY, TEXTS, PATCHES, 1, no_sites).expect("two reverses");
    assert_eq!(bridge.translate(3, 1, b"Renamed", Box::from([0, 3, 0, 7, 0])), Ok([0, 1, 7].as_slice().into()));
}
