//! A world's worker speaks chosen wire versions (temper/docs/design/protocol/wires.md, 11).
use crate::referee::Store;
use jig_protocol_worker_wire::{Spoken, spoken};
use std::cell::RefCell;
use temper_service::worker_settings::Wires;

pub(crate) const MESSAGE: &str = "@incident-edit Continue across the bridged wires";
#[derive(Default)]
struct State {
    versions: Option<Vec<(&'static str, u16)>>,
    opened: usize,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

pub(crate) fn active() -> bool {
    STATE.with(|state| state.borrow().versions.is_some())
}
fn cap(wire: &Spoken, versions: &[(&str, u16)]) -> Spoken {
    let version = versions.iter().find(|(name, _)| *name == wire.name).map_or(wire.lowest, |(_, v)| *v);
    Spoken::current(wire.name, version, wire.layout(version).expect("chosen version is kept"))
}
/// Omitted versions mean each wire's floor. The engine is never capped.
#[must_use]
pub fn wires(versions: &[(&str, u16)]) -> Wires {
    let limits = temper_service::limits::profile(temper_service::limits::Profile::Small);
    let payload = temper_service::worker_settings::payload_limits(&temper_service::limits::worker(&limits))
        .expect("world payload limits");
    let mut application = temper_protocol_worker_wire::application(&payload).expect("world application");
    for wire in application.spoken.iter_mut() {
        *wire = cap(wire, versions);
    }
    let mut jig = spoken();
    for wire in jig.iter_mut() {
        *wire = cap(wire, versions);
    }
    Wires { jig, application }
}
pub(crate) fn worker_wires() -> Option<Wires> {
    STATE.with(|state| state.borrow().versions.as_ref().map(|versions| wires(versions)))
}
fn at_compatible_versions(run: impl FnOnce()) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            STATE.with(|state| *state.borrow_mut() = State::default());
        }
    }
    // wires.md, 11: contract 2 needs smith-charter 5 and jig-worker-wire 13;
    // every other worker wire stays at its floor.
    STATE.with(|state| {
        *state.borrow_mut() = State { versions: Some(vec![("smith-charter", 5), ("jig-worker-wire", 13)]), opened: 0 };
    });
    let _reset = Reset;
    run();
    STATE.with(|state| assert!(state.borrow().opened > 0, "the engine logged the bridged opening"));
}
pub(crate) fn note(note: &temper_service::Note) {
    if !active() {
        return;
    }
    assert!(
        !matches!(
            note,
            temper_service::Note::Engine(
                temper_engine_service::Note::RunRefused(_) | temper_engine_service::Note::WorkerOpeningRefused { .. }
            )
        ),
        "no attempt or opening refused: {note:?}"
    );
    if let temper_service::Note::Engine(temper_engine_service::Note::WorkerOpened { worker, wires }) = note {
        let line = temper_engine_shell::lines::note_line(
            "temper",
            temper_engine_service::Note::WorkerOpened { worker: worker.clone(), wires: wires.clone() },
        );
        assert!(line.contains("opened: jig-worker-wire 13,"), "{line}");
        assert!(line.contains("smith-charter 5,"), "{line}");
        STATE.with(|state| state.borrow_mut().opened += 1);
    }
}
pub(crate) fn tasks(store: &Store) {
    if !active() {
        return;
    }
    let mut count = 0;
    for row in store.rows.values() {
        if let temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
            jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
        )) = row
        {
            assert_eq!(task.tries, jig_domain_tasks::Tries::NONE, "task {} spends no try", task.number);
            for word in &task.inbox {
                assert!(
                    !word.words.windows(b"version mismatch".len()).any(|part| part == b"version mismatch"),
                    "no version mismatch notice on task {}",
                    task.number
                );
            }
            count += 1;
        }
    }
    assert!(count >= 3, "the goal/chat, change and review ran");
}
/// A goal's change lands, with a message and an answered host call in its running run.
pub fn goal() {
    at_compatible_versions(crate::fixtures::core_after_project::bridged_goal);
}
/// Checkout uses the initial identity; delivery uses its mid-run renewal.
pub fn renewal() {
    at_compatible_versions(|| {
        let counts = crate::m15::delivery(true);
        assert_eq!(counts.tokens, 5);
        assert_eq!(counts.checkpoints, 5);
    });
}
