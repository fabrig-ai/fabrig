//! Derived layout passes against fixture patches (data/codec/layouts.md, 6).
#[cfg(test)]
mod tests {
    use crate::{generic_support, support::generated};
    use skein_codec::layout::{Plan, Workspace};
    use skein_codec::patch::{Capacity, Patcher, PlanIx, Step};
    use skein_codec::schema::{self, DeclIx, Root, Schema, Shape, Type, VariantIx};
    use skein_codec::value::Value;
    use skein_lib::{Buffer, Rng};
    use std::path::{Path, PathBuf};

    const ROOM: Capacity = Capacity { versions: 2, patches: 1, text: 4096, bytes: 4096, nodes: 512 };
    const DRAWS: u64 = 256;

    #[derive(Debug)]
    struct Case {
        old: Root,
        new: Root,
        plan: Plan,
    }
    #[derive(Debug)]
    struct Fixture {
        old: Schema,
        new: Schema,
        draw: skein_codegen::Schema,
        cases: Vec<Case>,
        patcher: Patcher,
        patch: PlanIx,
        goldens: PathBuf,
    }

    fn fixtures() -> Vec<Fixture> {
        let base = Path::new("../codec-patches");
        let mut paths = std::fs::read_dir(base.join("patches"))
            .expect("fixture patches")
            .map(|entry| entry.expect("patch entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "patch"))
            .collect::<Vec<_>>();
        paths.sort();
        let mut fixtures = Vec::new();
        for path in paths {
            let patch_text = std::fs::read_to_string(&path).expect("patch text");
            if patch_text.lines().any(|line| line.starts_with("written ")) {
                continue;
            }
            let header = patch_text.lines().find(|line| line.starts_with("family ")).expect("family header");
            let words = header.split_whitespace().collect::<Vec<_>>();
            let family = words.get(1).expect("family");
            let from = words.get(2).expect("from");
            let to = words.get(4).expect("to");
            let old_text =
                std::fs::read_to_string(base.join(format!("schema/kept/{family}-v{from}.schema"))).expect("old text");
            let mut new_text =
                std::fs::read_to_string(base.join(format!("schema/kept/{family}-v{to}.schema"))).expect("new text");
            // Fixture patches carry the added values. Give the layouts that same
            // default metadata; neither their layouts nor wire bytes change.
            for line in patch_text.lines().filter_map(|line| line.strip_prefix("add ")) {
                let words = line.split_whitespace().collect::<Vec<_>>();
                if words.len() == 3 {
                    let (_, field) = words.first().expect("target").split_once('.').expect("added field");
                    let needle = format!("{field}: {}", words.get(1).expect("type"));
                    assert_eq!(new_text.matches(&needle).count(), 1, "fixture default has one field");
                    new_text = new_text.replace(&needle, &format!("{needle} = {}", words.get(2).expect("value")));
                }
            }
            let old = schema::parse(old_text.as_bytes()).expect("old schema");
            let new = schema::parse(new_text.as_bytes()).expect("new schema");
            let mut cases = Vec::new();
            for index in 0..old.declarations() {
                let decl = DeclIx::new(index);
                let name = old.spelled(old.declaration(decl).name);
                for root in roots(&old, decl) {
                    let variant = root.variant.map(|at| old.spelled(old.variant(at).name));
                    let dest = new.root(name, variant).expect("derived fixture root");
                    let plan = Plan::derive(&old, root, &[], &new, dest, &[]).expect("no written sites");
                    cases.push(Case { old: root, new: dest, plan });
                }
            }
            let mut patcher = Patcher::new(ROOM);
            patcher.keep(old_text.as_bytes()).expect("keep old");
            patcher.keep(new_text.as_bytes()).expect("keep new");
            let patch = patcher.plan(patch_text.as_bytes()).expect("fixture patch").expect("family");
            let level = path.file_stem().expect("patch level");
            fixtures.push(Fixture {
                draw: skein_codegen::parse(&old_text).expect("draw schema"),
                old,
                new,
                cases,
                patcher,
                patch,
                goldens: base.join("patches/golden").join(level).join(family),
            });
        }
        assert!(!fixtures.is_empty(), "derived fixture coverage");
        fixtures
    }

    fn roots(schema: &Schema, decl: DeclIx) -> Vec<Root> {
        if let Some(root) = schema.root(schema.spelled(schema.declaration(decl).name), None) {
            return vec![root];
        }
        let leading = match schema.declaration(decl).shape {
            Shape::Enum { .. } => decl,
            Shape::Record { first, .. } => match schema.ty(schema.field(first).ty) {
                Type::Named(at) => at,
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Bool
                | Type::Duration
                | Type::Fixed(_)
                | Type::Secret { .. }
                | Type::Text(_)
                | Type::Bytes { .. }
                | Type::List { .. }
                | Type::Option(_) => unreachable!("broad record"),
            },
        };
        let Shape::Enum { first, count } = schema.declaration(leading).shape else { unreachable!("leading enum") };
        (0..count)
            .map(|offset| Root {
                decl,
                variant: Some(VariantIx::new(first.index().checked_add(offset).expect("variant index"))),
            })
            .collect()
    }
    fn accepts_root(schema: &Schema, root: Root, bytes: &[u8]) -> bool {
        let Some(variant) = root.variant else { return true };
        let offset = match schema.declaration(root.decl).shape {
            Shape::Record { versioned: true, .. } => 2,
            Shape::Record { versioned: false, .. } | Shape::Enum { .. } => 0,
        };
        let tag = bytes.get(offset).copied();
        let all = roots(schema, root.decl);
        all.iter().position(|root| root.variant == Some(variant)).map(|index| u8::try_from(index).expect("tag")) == tag
    }
    fn checked_output(schema: &Schema, root: Root, bytes: &[u8]) {
        // The pass is not a decoder: UTF-8 and kept malformed ranges belong to this
        // reader. Success and refusal are both permitted, and neither may panic.
        let mut value = Value::with_room(ROOM.nodes);
        if value.decode(schema, root.decl, bytes.into()).is_ok() {
            assert_eq!(value.bytes(), bytes, "generic decoder preserves canonical bytes");
        }
    }
    fn oracle(patcher: &mut Patcher, patch: PlanIx, root: DeclIx, bytes: &[u8]) -> Vec<u8> {
        patcher.start(patch, root, bytes.into()).expect("old draw decodes");
        assert_eq!(patcher.next(), Ok(Step::Done), "derived fixture has no stops");
        patcher.finish().expect("oracle bytes").to_vec()
    }

    #[test]
    fn derived_fixture_layouts_equal_the_patch_machine_on_seeded_draws() {
        let mut rng = Rng::new(0x1A70_0750_0000_0001);
        for mut fixture in fixtures() {
            let mut work = Workspace::new();
            for case in &fixture.cases {
                assert!(work.fit(&case.plan));
            }
            let mut out = Buffer::with_capacity(ROOM.bytes);
            for at in 0..fixture.old.declarations() {
                let decl = DeclIx::new(at);
                let name =
                    core::str::from_utf8(fixture.old.spelled(fixture.old.declaration(decl).name)).expect("root name");
                for round in 0..DRAWS {
                    let bytes = generated::draw(&fixture.draw, name, &mut rng);
                    let case = fixture
                        .cases
                        .iter()
                        .find(|case| case.old.decl == decl && accepts_root(&fixture.old, case.old, &bytes))
                        .expect("draw's value type");
                    case.plan.convert(&bytes, &mut out, &mut work).expect("draw converts");
                    let expected = oracle(&mut fixture.patcher, fixture.patch, decl, &bytes);
                    assert_eq!(out.as_slice(), expected, "{name} draw {round}: {bytes:?}");
                    Value::with_room(ROOM.nodes)
                        .decode(&fixture.new, case.new.decl, out.to_boxed())
                        .expect("converted draw decodes");
                    if case.plan.keeps() {
                        let mut headed = bytes.clone();
                        if let Shape::Record { versioned: true, .. } = fixture.new.declaration(case.new.decl).shape {
                            headed.get_mut(..2).expect("head").copy_from_slice(&fixture.new.version().to_be_bytes());
                        }
                        assert_eq!(out.as_slice(), headed, "keeps moves only the root head");
                    }
                }
            }
        }
    }

    #[test]
    fn arbitrary_bytes_and_changed_cut_grown_fixture_goldens_never_panic() {
        let mut rng = Rng::new(0x1A70_0750_0000_0002);
        for fixture in fixtures() {
            let mut work = Workspace::new();
            for case in &fixture.cases {
                assert!(work.fit(&case.plan));
            }
            let mut out = Buffer::with_capacity(ROOM.bytes);
            for case in &fixture.cases {
                for _ in 0_u32..256 {
                    let mut bytes = vec![0; usize::try_from(rng.below(65)).expect("length")];
                    for byte in &mut bytes {
                        *byte = u8::try_from(rng.below(256)).expect("byte");
                    }
                    if case.plan.convert(&bytes, &mut out, &mut work).is_ok() {
                        checked_output(&fixture.new, case.new, out.as_slice());
                    }
                }
            }
            for entry in std::fs::read_dir(&fixture.goldens).expect("fixture goldens") {
                let path = entry.expect("golden entry").path();
                let name = path.file_name().expect("golden name").to_str().expect("UTF-8 name");
                let decl = generic_support::golden_root(&fixture.old, name);
                let golden = std::fs::read(&path).expect("golden bytes");
                let case = fixture
                    .cases
                    .iter()
                    .find(|case| case.old.decl == decl && accepts_root(&fixture.old, case.old, &golden))
                    .expect("golden's value type");
                for cut in 0..=golden.len() {
                    if case.plan.convert(golden.get(..cut).expect("cut"), &mut out, &mut work).is_ok() {
                        checked_output(&fixture.new, case.new, out.as_slice());
                    }
                }
                for index in 0..golden.len() {
                    let mut changed = golden.clone();
                    *changed.get_mut(index).expect("chosen byte") ^= 0xff;
                    if case.plan.convert(&changed, &mut out, &mut work).is_ok() {
                        checked_output(&fixture.new, case.new, out.as_slice());
                    }
                }
                let mut grown = golden;
                grown.extend_from_slice(&[0; 8]);
                if case.plan.convert(&grown, &mut out, &mut work).is_ok() {
                    checked_output(&fixture.new, case.new, out.as_slice());
                }
            }
        }
    }
}
