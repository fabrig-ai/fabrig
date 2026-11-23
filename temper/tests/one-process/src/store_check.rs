//! Cold starts over the world's copied disk (temper/docs/design/testing/store-check.md, 2–4).
use crate::{upgrade::rehearsal, world::Settings};
use jig_conformance::store_check::{StoreChecks, finding};
use std::{cell::RefCell, rc::Rc};
use temper_playground::{documents::Configuration, programs::Harness};

use skein_fake_machine::Machine;
use temper_playground::machine;

/// The run's selection count and the number of isolated rehearsals it performed.
#[derive(Debug)]
pub struct Report {
    pub commits: u64,
    pub checked: u64,
    pub rehearsals: u64,
    pub words: String,
}

enum Pending {
    Check(u64),
    Keep,
}

struct State {
    checks: StoreChecks,
    pending: Option<Pending>,
    kept: Option<Machine>,
    newest: u64,
    rehearsals: u64,
    settings: Settings,
    disk: Rc<RefCell<Machine>>,
    document: Rc<Configuration>,
    given: Option<temper_service::stock::Given>,
    inputs: Option<rehearsal::Inputs>,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
    static POLICY: RefCell<Option<StoreChecks>> = const { RefCell::new(None) };
}

/// Override the next phase's default sample, restoring the previous policy on unwind.
pub fn with_policy<R>(checks: StoreChecks, run: impl FnOnce() -> R) -> R {
    struct Reset(Option<StoreChecks>);
    impl Drop for Reset {
        fn drop(&mut self) {
            POLICY.with(|at| *at.borrow_mut() = self.0.take());
        }
    }
    let _reset = Reset(POLICY.with(|at| at.replace(Some(checks))));
    run()
}

pub(crate) fn scope<R>(
    settings: Settings,
    disk: Rc<RefCell<Machine>>,
    document: Rc<Configuration>,
    run: impl FnOnce() -> Result<R, String>,
) -> (Result<R, String>, Report) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            STATE.with(|at| {
                if let Some(state) = at.take() {
                    eprintln!("{}; {} rehearsals", state.checks.report(), state.rehearsals);
                }
            });
        }
    }
    let checks = POLICY.with(RefCell::take).unwrap_or_else(|| StoreChecks::one(settings.seed));
    STATE.with(|at| {
        assert!(at.borrow().is_none(), "one live store-check scope");
        *at.borrow_mut() = Some(State {
            checks,
            pending: None,
            kept: None,
            newest: 0,
            rehearsals: 0,
            settings,
            disk,
            document,
            given: None,
            inputs: None,
        });
    });
    let _reset = Reset;
    let result = run();
    STATE.with(|at| {
        let mut state = at.borrow_mut();
        let state = state.as_mut().expect("store-check scope");
        let result = result.and_then(|outcome| {
            if let Some(number) = state.checks.end() {
                let copy = state.kept.take().expect("the referee kept the selected commit's disk");
                state.rehearse(copy, number)?;
            }
            Ok(outcome)
        });
        let report = Report {
            commits: state.checks.commits(),
            checked: state.checks.checked(),
            rehearsals: state.rehearsals,
            words: state.checks.report(),
        };
        (result, report)
    })
}

/// Called only for commits returned by the durable referee, including recovery's commits.
pub(crate) fn observed(number: u64) {
    STATE.with(|at| {
        if let Some(state) = at.borrow_mut().as_mut() {
            state.newest = state.newest.max(number);
            if state.checks.due(number) {
                state.pending.get_or_insert(Pending::Check(number));
            } else if state.checks.keeps() {
                state.pending = Some(Pending::Keep);
            }
        }
    });
}

/// The referee calls this outside every process turn, with the machine frozen.
pub(crate) fn check() -> Result<(), String> {
    STATE.with(|at| {
        let mut state = at.borrow_mut();
        let Some(state) = state.as_mut() else { return Ok(()) };
        match state.pending.take() {
            Some(Pending::Check(number)) => {
                #[cfg(test)]
                tests::inject(state);
                let copy = state.disk.borrow().fork();
                state.rehearse(copy, number.max(state.newest))
            }
            Some(Pending::Keep) => {
                let copy = state.disk.borrow().fork();
                #[cfg(test)]
                let copy = tests::inject_kept(copy, state.newest);
                state.kept = Some(copy);
                Ok(())
            }
            None => Ok(()),
        }
    })
}

// The live start already derives these defaults from the phase's configuration.
pub(crate) fn given(build: impl FnOnce() -> temper_service::stock::Given) -> temper_service::stock::Given {
    STATE.with(|at| {
        let mut state = at.borrow_mut();
        let Some(state) = state.as_mut() else { return build() };
        if let Some(given) = &state.given {
            return given.copied();
        }
        let given = build();
        state.given = Some(given.copied());
        given
    })
}

impl State {
    fn rehearse(&mut self, copy: Machine, number: u64) -> Result<(), String> {
        let copy = machine::Machine::new(copy);
        let inputs = self.inputs.get_or_insert_with(|| {
            let mut document = (*self.document).clone();
            crate::world::OneProcess::configure(&mut document);
            // Parameters omit world settings for ordinary stories.
            if document.get("world_settings").is_none() {
                document["world_settings"] = crate::fixtures::world_document()["world_settings"].clone();
            }
            rehearsal::Inputs::new(document)
        });
        let given = self.given.as_ref().expect("live start's defaults");
        self.rehearsals += 1;
        rehearsal::check(&copy, self.settings, inputs, given).map_err(|words| {
            // The source is unchanged by the tool. Read it only to name a refusal:
            // its newest frame can be ahead of the observation that kept it.
            let number = newest(&mut copy.0.borrow_mut()).max(number);
            finding(number, "upgrade rehearsal", &words)
        })
    }
}

fn newest(disk: &mut Machine) -> u64 {
    use skein_fake_machine::How;
    let active = crate::golden::files::active(disk);
    let active = std::str::from_utf8(&active).expect("active text").trim();
    let root = disk.reopen_root(0);
    let store = disk.open(root, format!("generations/{active}/store").as_bytes(), How::Directory).expect("store");
    let mut newest = 0;
    loop {
        let entries = disk.list(store, 32, 4096).expect("store entries");
        if entries.is_empty() {
            break;
        }
        for (_, name) in entries {
            if !name.starts_with(b"log-") {
                continue;
            }
            let file = disk.open(store, &name, How::ReadNoFollow).expect("log");
            let size = u32::try_from(disk.stat(file).size).expect("world log size");
            let bytes = disk.read(file, 0, size).expect("log bytes");
            if let Some((number, _)) = crate::referee::durable::frames(&bytes, 0).0.last() {
                newest = newest.max(*number);
            }
            disk.close(file);
        }
    }
    disk.close(store);
    disk.close(root);
    newest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fixtures, referee, world};
    use std::cell::Cell;

    const CHAT_SEED: u64 = 45;
    const CHAT_KEPT: u64 = 17;

    thread_local! {
        static CORRUPT: Cell<bool> = const { Cell::new(false) };
        static CORRUPT_KEPT: Cell<Option<u64>> = const { Cell::new(None) };
    }

    pub(super) fn inject(state: &mut State) {
        if !CORRUPT.with(|at| at.replace(false)) {
            return;
        }
        let mut disk = state.disk.borrow_mut();
        corrupt(&mut disk);
        state.newest = newest(&mut disk);
        assert!(state.checks.due(state.newest), "corrupt fixture uses every commit");
    }

    pub(super) fn inject_kept(mut copy: Machine, number: u64) -> Machine {
        CORRUPT_KEPT.with(|at| {
            if at.get() == Some(number) {
                corrupt(&mut copy);
                at.set(None);
            }
        });
        copy
    }

    fn corrupt(disk: &mut Machine) {
        let key =
            jig_protocol_store::keys::encode(&jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment), 64)
                .expect("deployment key");
        crate::golden::files::change_record(disk, &key, Some(&[0]));
    }

    fn story(seed: u64, phase: world::Phase) -> (world::Outcome, [u8; 32], u64) {
        let (outcome, disk, store) = world::run_phase(
            fixtures::calm(seed),
            world::Story::Chat { words: b"Hello" },
            phase,
            temper_playground::machine::fresh(),
            referee::Store::new(),
            Rc::new(RefCell::new(referee::Run::default())),
        );
        (outcome, disk.state_digest(), store.applied)
    }

    #[test]
    fn every_commit_rehearses_without_changing_the_live_story() {
        let run = || story(fixtures::SEED, world::Phase::Committed);
        let (checked, digest, last) = with_policy(StoreChecks::every(), run);
        assert_eq!(checked.store_checks.commits, 1);
        assert_eq!(checked.store_checks.checked, checked.store_checks.commits);
        assert!(checked.store_checks.rehearsals > 0);
        assert_eq!(checked.store_checks.rehearsals, checked.store_checks.checked);
        let (unchecked, without, unchecked_last) = with_policy(StoreChecks::none("comparison of live disks"), run);
        assert_eq!(unchecked.store_checks.checked, 0);
        assert_eq!(checked.store_checks.commits, unchecked.store_checks.commits);
        assert_eq!(last, unchecked_last);
        assert_eq!(digest, without, "checking leaves all live disk state, handles and counters untouched");
    }

    #[test]
    fn the_default_sample_reports_the_storys_commits() {
        // Seed 45 keeps commit 17 out of this phase's 22 commits.
        let (outcome, _, _) = story(CHAT_SEED, world::Phase::Story);
        assert_eq!(outcome.chats_answered, 1);
        assert_eq!(outcome.store_checks.commits, 22);
        assert_eq!(outcome.store_checks.checked, 1);
        assert_eq!(outcome.store_checks.rehearsals, 1);
        assert_eq!(
            outcome.store_checks.words,
            "store check: 1 of 22 commits checked (one per run, seed 45, commit 17)"
        );
    }

    #[test]
    fn a_kept_refusal_fails_the_completed_phase() {
        CORRUPT_KEPT.with(|at| at.set(Some(CHAT_KEPT)));
        let run = Rc::new(RefCell::new(referee::Run::default()));
        let words = world::try_run_phase(
            fixtures::calm(CHAT_SEED),
            world::Story::Chat { words: b"Hello" },
            world::Phase::Story,
            temper_playground::machine::fresh(),
            referee::Store::new(),
            run.clone(),
        )
        .err()
        .expect("the phase must return the kept record's refusal");
        assert!(run.borrow().applied > CHAT_KEPT, "the live phase continued past the kept commit");
        // Frame 18 is already on disk when the referee observes commit 17;
        // the corrupt record appends commit 19 to that copy, not the live disk.
        assert!(words.contains("store check: commit 19: start refused at upgrade rehearsal:"), "{words}");
        assert!(words.contains("record not decodable: jig-store-core 0, store key 010000"), "{words}");
    }

    #[test]
    fn a_corrupt_live_commit_fails_with_its_number_and_the_tools_refusal() {
        CORRUPT.with(|at| at.set(true));
        let words = with_policy(StoreChecks::every(), || {
            world::try_run_phase(
                fixtures::calm(fixtures::SEED),
                world::Story::Chat { words: b"Hello" },
                world::Phase::Story,
                temper_playground::machine::fresh(),
                referee::Store::new(),
                Rc::new(RefCell::new(referee::Run::default())),
            )
        })
        .err()
        .expect("the copied live record must refuse a start");
        assert!(words.contains("store check: commit 2: start refused at upgrade rehearsal:"), "{words}");
        assert!(words.contains("record not decodable: jig-store-core 0, store key 010000"), "{words}");
    }
}
