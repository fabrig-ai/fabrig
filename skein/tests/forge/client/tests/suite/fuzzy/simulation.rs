//! The fault and branch sweeps take cases (skein/docs/design/testing/sweeps.md, 4.2 to 4.5).
//! The update keeps its grid of every cut (skein/docs/design/testing/README.md, 7.1).

use std::convert::Infallible;

use skein_forge::api;
use skein_forge_client::{Condition, Effect, Entry};
use skein_forge_client_world::{REPO, Settings, World};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};

#[test]
fn seeded_faults_restarts_and_keyed_effects() {
    let test = this_test!("seeded_faults_restarts_and_keyed_effects");
    let at_gate = gate();
    let mut unavailable = 0;
    let mut timeouts = 0;
    let mut landings = 0;
    let mut limited = 0;
    cases(test, 0..128, |seed| -> Result<(), Infallible> {
        let mut world = World::new(Settings::random(seed));
        for entry in 1..=4 {
            world.make(Entry {
                number: entry,
                task: 7,
                repository: REPO,
                effect: Effect {
                    write: api::Write::CreateIssue {
                        key: Box::new([1, 0, 3, b'k', b'i', b't', u8::try_from(entry).expect("world entry bound")]),
                        title: Box::from(&b"goal"[..]),
                        body: Box::from(&b"plan"[..]),
                    },
                    condition: Condition::None,
                },
                start: None,
                attempt: None,
                failures: 0,
                queued: false,
            });
        }
        world.run_for(2);
        world.restart();
        world.run_for(40);
        world.finish();
        let stats = world.stats();
        assert_eq!(stats.calls, stats.terminals);
        if at_gate {
            unavailable += stats.unavailable;
            timeouts += stats.timeouts;
            landings += stats.late_landings;
            limited += stats.limited;
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        assert!(unavailable > 0 && timeouts > 0 && landings > 0 && limited > 0, "every configured fault actually fell");
    }
}

#[test]
fn unrecoverable_branch_creation_survives_cuts_before_and_after_its_late_copy() {
    let test = this_test!("unrecoverable_branch_creation_survives_cuts_before_and_after_its_late_copy");
    cases(test, 0..64, |seed| -> Result<(), Infallible> {
        let mut world = World::new(Settings::calm(seed));
        let head = world.branch(b"main").expect("fixture main");
        world.land_writes_late(skein_lib::Duration::from_secs(5));
        world.make(Entry {
            number: 1,
            task: 7,
            repository: REPO,
            effect: Effect {
                write: api::Write::CreateBranch {
                    branch: Box::from(&b"kit/7"[..]),
                    commit: skein_forge_client_world::translate::commit(head),
                },
                condition: Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        });
        world.run_for(1);
        world.calm();
        world.run_for(seed % 23);
        world.restart();
        world.run_for(30);
        assert_eq!(world.branch(b"kit/7"), Some(head), "seed {seed}");
        assert_eq!(world.stats().late_landings, 1, "seed {seed}");
        assert_eq!(world.stats().writes, 1, "seed {seed}");
        world.finish();
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn an_update_survives_cuts_before_and_after_its_late_copy() {
    for style in [api::UpdateStyle::Merge, api::UpdateStyle::Rebase] {
        for cut in 0..64 {
            let mut world = World::new(Settings::calm(cut));
            let pull = world.historical_pull();
            let head = world.pull_head(pull);
            let base = world.outside_push(b"main", b"base", b"ahead");
            assert!(!world.is_ancestor(base, head));
            world.land_writes_past_lifetime(skein_lib::Duration::from_secs(25));
            world.make(crate::fixtures::update_entry(pull, head, base, style));
            world.run_for(1);
            world.calm();
            world.run_for(cut);
            world.restart();
            world.run_for(60);
            assert_eq!(world.stats().restarts, 1, "cut {cut}, style {style:?}");
            assert_eq!(world.stats().late_landings, 1, "cut {cut}, style {style:?}");
            assert!(
                !world.outcomes().contains(&(1, skein_forge_client::Outcome::Held)),
                "update held at cut {cut}, style {style:?}"
            );
            assert!(
                world.is_ancestor(base, world.pull_head(pull))
                    || world.outcomes().last() == Some(&(1, skein_forge_client::Outcome::Failed(api::Error::Timeout))),
                "cut {cut}, style {style:?}"
            );
            assert!(world.entry(1).is_none(), "cut {cut}, style {style:?}");
            world.finish();
        }
    }
}
