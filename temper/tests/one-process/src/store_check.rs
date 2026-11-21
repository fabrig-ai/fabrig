//! Cold starts over the world's copied disk (temper/docs/design/testing/store-check.md, 2–4).
use crate::{upgrade::rehearsal, world::Settings};
use jig_conformance::store_check::{StoreChecks, finding};
use std::{cell::RefCell, rc::Rc};
use temper_playground::{documents::Configuration, programs::Harness};

// Focused run phases have a median of 21 commits, rounded up to a power of two.
const ONE_IN: u64 = 32;

/// The run's selection count and the number of isolated rehearsals it performed.
#[derive(Debug)]
pub struct Report {
    pub commits: u64,
    pub checked: u64,
    pub rehearsals: u64,
    pub words: String,
}

struct State {
    checks: StoreChecks,
    pending: Option<u64>,
    newest: u64,
    rehearsals: u64,
    settings: Settings,
    disk: Rc<RefCell<skein_fake_machine::Machine>>,
    document: Configuration,
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
    disk: Rc<RefCell<skein_fake_machine::Machine>>,
    mut document: Configuration,
    run: impl FnOnce() -> R,
) -> (R, Report) {
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
    crate::world::OneProcess::configure(&mut document);
    // Parameters omit world settings for ordinary stories; their defaults use these same settings.
    if document.get("world_settings").is_none() {
        document["world_settings"] = crate::fixtures::world_document()["world_settings"].clone();
    }
    let checks = POLICY.with(RefCell::take).unwrap_or_else(|| StoreChecks::sampled(settings.seed, ONE_IN));
    STATE.with(|at| {
        assert!(at.borrow().is_none(), "one live store-check scope");
        *at.borrow_mut() = Some(State { checks, pending: None, newest: 0, rehearsals: 0, settings, disk, document });
    });
    let _reset = Reset;
    let result = run();
    let report = STATE.with(|at| {
        let state = at.borrow();
        let state = state.as_ref().expect("store-check scope");
        Report {
            commits: state.checks.commits(),
            checked: state.checks.checked(),
            rehearsals: state.rehearsals,
            words: state.checks.report(),
        }
    });
    (result, report)
}

/// Called only for commits returned by the durable referee, including recovery's commits.
pub(crate) fn observed(number: u64) {
    STATE.with(|at| {
        if let Some(state) = at.borrow_mut().as_mut() {
            state.newest = state.newest.max(number);
            if state.checks.due(number) && state.pending.is_none() {
                state.pending = Some(number);
            }
        }
    });
}

/// The referee calls this outside every process turn, with the machine frozen.
pub(crate) fn check() -> Result<(), String> {
    STATE.with(|at| {
        let mut state = at.borrow_mut();
        let Some(state) = state.as_mut() else { return Ok(()) };
        let Some(_sampled) = state.pending.take() else { return Ok(()) };
        #[cfg(test)]
        tests::inject(state);
        let mut copy = state.disk.borrow().fork();
        let number = newest(&mut copy).max(state.newest);
        let given = crate::world::OneProcess::given(&state.document);
        let result = rehearsal::check(copy, state.settings, state.document.clone(), given);
        state.rehearsals += 1;
        match result {
            Ok((_, 0)) => Ok(()),
            Ok((words, _)) | Err(words) => Err(finding(number, "upgrade rehearsal", &words)),
        }
    })
}

// The copy may hold a completed frame whose Sync completion the live service has not reaped yet.
fn newest(disk: &mut skein_fake_machine::Machine) -> u64 {
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

    thread_local! { static CORRUPT: Cell<bool> = const { Cell::new(false) }; }

    pub(super) fn inject(state: &mut State) {
        if !CORRUPT.with(|at| at.replace(false)) {
            return;
        }
        let key =
            jig_protocol_store::keys::encode(&jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment), 64)
                .expect("deployment key");
        let mut disk = state.disk.borrow_mut();
        crate::golden::files::change_record(&mut disk, &key, Some(&[0]));
        state.newest = newest(&mut disk);
        assert!(state.checks.due(state.newest), "corrupt fixture uses every commit");
    }

    fn story() -> (world::Outcome, [u8; 32], u64) {
        let (outcome, disk, store) = world::run_phase(
            fixtures::calm(fixtures::SEED),
            world::Story::Chat { words: b"Hello" },
            world::Phase::Story,
            temper_playground::machine::fresh(),
            referee::Store::new(),
            Rc::new(RefCell::new(referee::Run::default())),
        );
        (outcome, disk.state_digest(), store.applied)
    }

    #[test]
    fn every_commit_rehearses_without_changing_the_live_story() {
        let (checked, digest, last) = with_policy(StoreChecks::every(), story);
        assert_eq!(checked.chats_answered, 1);
        assert_eq!(checked.store_checks.checked, checked.store_checks.commits);
        assert!(checked.store_checks.rehearsals > 0);
        assert_eq!(checked.store_checks.rehearsals, checked.store_checks.checked);
        let (unchecked, without, unchecked_last) = with_policy(StoreChecks::none("comparison of live disks"), story);
        assert_eq!(unchecked.store_checks.checked, 0);
        assert_eq!(checked.store_checks.commits, unchecked.store_checks.commits);
        assert_eq!(last, unchecked_last);
        assert_eq!(digest, without, "checking leaves all live disk state, handles and counters untouched");
    }

    #[test]
    fn the_default_sample_reports_the_storys_commits() {
        let (outcome, _, _) = story();
        assert_eq!(outcome.chats_answered, 1);
        assert_eq!(outcome.store_checks.commits, 22);
        assert_eq!(outcome.store_checks.checked, 1);
        assert_eq!(outcome.store_checks.rehearsals, 1);
        assert_eq!(outcome.store_checks.words, "store check: 1 of 22 commits checked (one in 32, seed 7)");
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
