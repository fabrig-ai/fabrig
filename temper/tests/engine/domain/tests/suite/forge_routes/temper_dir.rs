//! The root maps forge reviews for the method check independently of landing rules.
use super::approval_identity::{forge_with_only_temper, post_approval_read, settle};
use super::light_rules::{change_row, pushed_path_change_in, pushed_path_change_with_base_edit};
use super::*;
use temper_engine_domain_forge_change::{Repair, State};

fn world() -> World {
    let mut world = World::configured_limits(true, true, true, 1000, Gate::default(), None, StoryLimits::default())
        .with_landing_deployment(Box::new([engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: Box::new([b"forge".as_slice().into()]),
                last: authority::Last::Open(Box::new([])),
            },
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        }]))
        .with_settings_owner_configured(&forge_top::user_key(1, 7), |config| {
            temper_engine_domain_world::oracle::config::landing(config, 1).temper_dir_sign_off = true;
            for stock in &mut config.kinds_stock {
                if stock.kind == 4 && stock.scope == 1 {
                    stock.value.size = temper_engine_smith::kinds::size(&stock.value.value);
                }
            }
        });
    forge_with_only_temper(&mut world);
    fake::grant(&mut world.fake, b"org/repo", 2, raw::Permission::Write);
    super::rules_by_path::seed_signers(&mut world);
    world.forge_reads = Some(Vec::new());
    world
}

fn request_changes(world: &mut World, task: u64) {
    fake::grant(&mut world.fake, b"org/repo", 7, raw::Permission::Write);
    let pull = change_row(world, task).pull.expect("pull opened at original head");
    assert!(matches!(
        world.external(
            7,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::RequestChanges),
                body: b"Please rename the way.".as_slice().into(),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
}

fn assert_probe_and_updated_method(
    world: &World,
    original: skein_forge::api::Commit,
    updated: skein_forge::api::Commit,
) {
    let reads = world.forge_reads.as_ref().expect("recorded forge reads");
    let probes: Vec<_> = reads
        .iter()
        .filter_map(|(_, read)| match read {
            skein_forge::api::Read::File { path, head, .. }
                if path.as_ref() == b".temper" && [original, updated].contains(head) =>
            {
                Some(*head)
            }
            _ => None,
        })
        .collect();
    assert_eq!(probes, [original, updated], "probe each pair once");
    let files: Vec<_> = reads
        .iter()
        .filter_map(|(_, read)| match read {
            skein_forge::api::Read::File { path, head, .. }
                if path.as_ref() == b".temper/ways.toml" && [original, updated].contains(head) =>
            {
                Some(*head)
            }
            _ => None,
        })
        .collect();
    assert_eq!(files, [updated], "read the method only at the updated head");
}

#[test]
fn temper_dir_sign_off_counts_only_mapped_owners_and_keeps_clean_predecessors() {
    for old_review in [None, Some(raw::Verdict::RequestChanges), Some(raw::Verdict::Approve)] {
        let (mut world, task, original, base) = pushed_path_change_in(
            world(),
            false,
            b"main",
            &[(b".temper/ways.toml", b"offer = [\"pair\"]\n"), (b".temper/ways/pair.toml", b"format = 1\n")],
        );
        if let Some(verdict) = old_review {
            for _ in 0..200 {
                if change_row(&world, task).pull.is_some() {
                    break;
                }
                world.tick();
            }
            assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
            if verdict == raw::Verdict::RequestChanges {
                request_changes(&mut world, task);
            } else {
                post_approval_read(&mut world, task, 7);
            }
        }
        settle(&mut world);
        let row = change_row(&world, task);
        let updated = row.change.last_head.expect("change head");
        assert_ne!(updated, translate::commit(original));
        assert!(world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
        if old_review == Some(raw::Verdict::Approve) {
            assert_eq!(world.merges, 1, "the owner's approval survives the update");
            assert!(
                world
                    .writes
                    .iter()
                    .any(|write| matches!(write, skein_forge::api::Write::Merge { head, .. } if *head == updated))
            );
        } else {
            assert_eq!(world.merges, 0);
            let head = world.fake.branch(b"org/repo", &row.branch).expect("updated branch");
            assert!(matches!(
                world.external(1, raw::Op::Read(raw::Read::Compare { base, head, page: 1, limit: 10 })),
                raw::Answer::Comparison { contains_base: true, .. }
            ));
        }
        let original = translate::commit(original);
        assert_probe_and_updated_method(&world, original, updated);
        if old_review == Some(raw::Verdict::RequestChanges) {
            assert!(
                matches!(row.change.state, State::Repairing { why: Repair::Review { head }, .. } if head == original),
                "{:?}",
                row.change
            );
            assert_eq!(row.change.repairs, 1);
        } else if old_review.is_none() {
            assert!(matches!(row.change.state, State::Queued { .. }), "{:?}", row.change);
            assert!(!row.change.owns_turn);
            // Account 3 is a mapped member; account 6 has no person.
            post_approval_read(&mut world, task, 3);
            settle(&mut world);
            assert_eq!(world.merges, 0, "a mapped member is not an owner");
            post_approval_read(&mut world, task, 6);
            settle(&mut world);
            assert_eq!(world.merges, 0, "an unmapped account is not an owner");
            assert!(!change_row(&world, task).change.owns_turn);
            post_approval_read(&mut world, task, 7);
            settle(&mut world);
            assert_eq!(world.merges, 1, "{:?}", change_row(&world, task).change);
            assert!(
                world
                    .writes
                    .iter()
                    .any(|write| matches!(write, skein_forge::api::Write::Merge { head, .. } if *head == updated))
            );
        }
        world.close();
    }
    let (mut world, _, _, _) = pushed_path_change_in(world(), false, b"main", &[(b"src/main.rs", b"new")]);
    settle(&mut world);
    assert_eq!(world.merges, 1, "uncovered changes don't need an owner");
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
    world.close();
}

#[test]
fn a_first_faulty_method_is_repaired_before_landing() {
    let (mut world, task, _, _) =
        pushed_path_change_in(world(), false, b"main", &[(b".temper/ways.toml", b"format = 99\noffer = [\"pair\"]\n")]);
    settle(&mut world);
    let row = change_row(&world, task);
    assert!(matches!(row.change.state, State::Repairing { why: Repair::TemperDir { .. }, .. }));
    let fault = row.temper_dir_fault.as_deref().expect("the check keeps its words");
    let words = std::str::from_utf8(fault).expect("fault is text");
    assert!(words.contains(".temper/ways.toml:1: format 99"), "{words}");
    assert_eq!(world.merges, 0);
    world.close();
}

#[test]
fn a_hooks_only_change_lands_without_a_method_comparison() {
    const MERGE_STEPS: u32 = 200;
    let (mut world, task, head, base) =
        pushed_path_change_with_base_edit(world(), b"main", &[(b".temper/hooks/close", b"#!/bin/sh\n")], None);
    for _ in 0..MERGE_STEPS {
        world.tick();
        if world.merges != 0 {
            break;
        }
    }
    assert_eq!(world.merges, 1);
    let (head, base) = (translate::commit(head), translate::commit(base));
    let reads = world.forge_reads.as_ref().expect("recorded forge reads");
    for commit in [head, base] {
        assert!(
            reads.iter().any(|(_, read)| matches!(read,
                skein_forge::api::Read::File { path, head, .. } if path.as_ref() == b".temper" && *head == commit
            )),
            "probe each exact tip"
        );
    }
    assert!(
        !reads.iter().any(|(_, read)| matches!(read, skein_forge::api::Read::Compare { .. })),
        "methodless change must not compare its head to its base: {reads:?}"
    );
    settle(&mut world);
    assert!(matches!(change_row(&world, task).change.state, State::Landed { .. }));
    world.close();
}
