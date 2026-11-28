//! Project charter decisions and their restoration through the root's ports.
#![cfg(test)]

use jig_domain_core::{self as core, ProjectCharters};
use skein_fake_forge_domain as fake;
use skein_lib::{List, bytes};
use step_project_fixture::{World, old_store, start};
use temper_engine_domain::{Fact, ProjectChartersDiagnostic, ProjectChartersFault};
use temper_engine_step_kit::{project_charters::*, start::limits};

temper_engine_step_kit::start_fixture!(temper_engine_domain);
temper_engine_step_kit::project_charter_fixture!(temper_engine_domain, super::step_start_fixture);

#[test]
fn a_moved_home_revision_gives_or_clears_charters_in_its_own_commit() {
    let mut world = World::new(&[(IMPLEMENTER, WORDS)]);
    let charters_only = world.tip();
    let (first, sets) = world.revision_commit();
    assert!(first.method && first.refused.is_none());
    assert_eq!(first.number, 1);
    assert_eq!(sets, None, "charters without ways.toml do not opt in");
    assert_eq!(world.root.core().project_charters(PROJECT), 0);

    world.advance(CAREFUL, CAREFUL_WORDS);
    world.advance(WAY, BUILD);
    world.advance(WAYS, OFFERS);
    let picked = world.tip();
    world.notice();
    let (row, sets) = world.revision_commit();
    assert_eq!(row.number, 2);
    assert_eq!(row.commit, Some(fake::commit_bytes(picked)));
    let hex = temper_engine_domain::commit_text(&row.commit.unwrap());
    let source = bytes::concat(&[REPOSITORY, b" at ", &hex[..12]]);
    assert_eq!(
        sets,
        Some(ProjectCharters {
            project: PROJECT,
            revision: row.number,
            source,
            charters: Box::new([careful(), implementer()]),
        })
    );
    assert_eq!(world.root.core().project_charters(PROJECT), row.number);
    assert!(world.root.pop_fact().is_none());

    // Hooks are read but are not part of the revision. This good read merely
    // advances the observation, not the revision number or its set.
    world.advance(b".temper/hooks/config.toml", b"");
    world.notice();
    let (same, sets) = world.revision_commit();
    assert_eq!(same.observed, row.observed + 1);
    assert_eq!((same.number, same.commit, same.refused), (row.number, row.commit, None));
    assert_eq!(sets, None);
    assert!(world.root.pop_fact().is_none(), "the unchanged number is not even offered");

    fake::reset_branch(&mut world.forge, &world.fake_env, REPOSITORY, MAIN, charters_only, 7).unwrap();
    world.notice();
    let (removed, sets) = world.revision_commit();
    assert_eq!((removed.number, removed.commit, removed.refused), (3, Some(fake::commit_bytes(charters_only)), None));
    let cleared = sets.expect("removing ways.toml clears even retained charter files");
    assert_eq!((cleared.project, cleared.revision), (PROJECT, 3));
    assert!(cleared.charters.is_empty(), "the replacement set is empty");
    assert_eq!(world.root.core().project_charters(PROJECT), removed.number);
}

#[test]
fn start_backfills_an_old_revision_and_does_not_offer_it_again() {
    let limits = limits();
    let mut store = old_store(&limits, true);
    store.keep(revision(&[(IMPLEMENTER, WORDS), (WAYS, OFFERS), (WAY, BUILD)]));
    let expected = ProjectCharters {
        project: PROJECT,
        revision: 4,
        source: b"acme/shop at 1a2b3c4d5e6f".as_slice().into(),
        charters: Box::new([implementer()]),
    };
    let durable = saved_set(expected.clone()).key();
    assert!(!store.rows.contains_key(&durable));
    for _ in 0..2_u32 {
        let (mut root, _) = start(&limits, &mut store, true);
        assert_eq!(root.core().project_charters(PROJECT), 4);
        assert_eq!(root.core().project_charter_set(PROJECT, 4), Some(&expected));
        assert_eq!(store.rows.get(&durable), Some(&saved_set(expected.clone())));
        assert!(root.pop_fact().is_none(), "a matching restored revision is not offered again");
    }
}

#[test]
fn start_reports_a_core_refusal_or_an_unreadable_file_and_keeps_the_previous_set() {
    let limits = limits();
    for (text, expected) in [
        (
            CAREFUL_WORDS,
            ProjectChartersFault::Refused(core::ProjectChartersFault::Count {
                charter: b"implementer".as_slice().into(),
                bound: 1,
            }),
        ),
        (
            BAD_WORDS,
            ProjectChartersFault::Unreadable {
                path: CAREFUL.into(),
                line: 1,
                why: "`## part, on: frist`: `frist` is no reason a part is selected by".into(),
            },
        ),
    ] {
        let mut store = old_store(&limits, true);
        let previous = ProjectCharters {
            project: PROJECT,
            revision: 3,
            source: REPOSITORY.into(),
            charters: Box::new([implementer()]),
        };
        store.keep(saved_set(previous.clone()));
        store.keep(revision(&[(CAREFUL, text), (IMPLEMENTER, WORDS), (WAYS, OFFERS), (WAY, BUILD)]));
        let (mut root, _) = start(&limits, &mut store, true);
        assert_eq!(
            root.pop_fact(),
            Some(Fact::ProjectCharters(ProjectChartersDiagnostic { project: PROJECT, revision: 4, fault: expected }))
        );
        assert!(root.pop_fact().is_none(), "the rejected revision was offered once");
        assert_eq!(root.core().project_charters(PROJECT), 3);
        assert_eq!(root.core().project_charter_set(PROJECT, 3), Some(&previous));
    }
}

#[test]
fn forge_declaration_offers_every_write_and_makes_merges_wait() {
    use jig_domain_contract::{
        effects::{
            EffectForm::{Creation, Set, Transition},
            Recovery::{Conditional, Idempotent, Unrecoverable},
        },
        resources::{ResourceKind, Rule, Taken},
    };
    const FORGE: u16 = 7;
    let declaration = temper_engine_domain::forge_declaration(FORGE);
    assert_eq!(declaration.connector, jig_domain_contract::Connector(FORGE));
    assert_eq!(
        declaration.resources.as_ref(),
        &[ResourceKind { kind: 1, hold: Rule::Exclusive { taken: Taken::Waits } }]
    );
    let mut rows = List::uncapped();
    for effect in &declaration.effects {
        rows.push((effect.kind, effect.form, effect.recovery, effect.waits_for_dependencies)).unwrap();
    }
    assert_eq!(
        rows.as_slice(),
        &[
            (2, Transition, Unrecoverable, false),
            (3, Creation, Conditional, false),
            (4, Transition, Conditional, true),
            (5, Creation, Unrecoverable, false),
            (6, Set, Idempotent, false),
            (7, Creation, Unrecoverable, false),
            (8, Creation, Unrecoverable, false),
            (9, Creation, Unrecoverable, false),
            (10, Set, Idempotent, false),
            (11, Transition, Idempotent, false),
            (12, Set, Idempotent, false),
            (13, Set, Idempotent, false),
            (14, Set, Idempotent, false),
            (15, Set, Idempotent, false),
            (16, Creation, Unrecoverable, false),
            (17, Transition, Unrecoverable, false),
        ]
    );
}
