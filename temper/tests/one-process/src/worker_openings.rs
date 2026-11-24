//! A world's goal and renewal over a worker opening (temper/docs/design/protocol/wires.md, 11).
use crate::referee::Store;
use std::cell::RefCell;

pub(crate) const MESSAGE: &str = "@incident-edit Continue over the worker link";
#[derive(Default)]
struct State {
    active: bool,
    opened: usize,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

pub(crate) fn active() -> bool {
    STATE.with(|state| state.borrow().active)
}
fn at_current_layouts(run: impl FnOnce()) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            STATE.with(|state| *state.borrow_mut() = State::default());
        }
    }
    STATE.with(|state| *state.borrow_mut() = State { active: true, opened: 0 });
    let _reset = Reset;
    run();
    STATE.with(|state| assert!(state.borrow().opened > 0, "the engine logged the opening at its layouts"));
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
    if let temper_service::Note::Engine(temper_engine_service::Note::WorkerOpened { worker, converted }) = note {
        let line = temper_engine_shell::lines::note_line(
            "temper",
            temper_engine_service::Note::WorkerOpened { worker: worker.clone(), converted: converted.clone() },
        );
        assert!(line.contains("opened: at the engine's layouts"), "{line}");
        STATE.with(|state| state.borrow_mut().opened += 1);
    }
}
pub(crate) fn tasks(store: &Store) -> Result<(), String> {
    if !active() {
        return Ok(());
    }
    let mut count = 0;
    for row in store.rows.values() {
        if let temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
            jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
        )) = row
        {
            match (&(task.tries), &(jig_domain_tasks::Tries::NONE)) {
                (left, right) if left != right => {
                    let finding = format!("task {} spends no try", task.number);
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            for word in &task.inbox {
                if word.words.windows(b"version mismatch".len()).any(|part| part == b"version mismatch") {
                    return Err(format!("no version mismatch notice on task {}", task.number));
                }
            }
            count += 1;
        }
    }
    if count < 3 {
        return Err(String::from("the goal/chat, change and review ran"));
    }

    Ok(())
}
/// A goal's change lands, with a message and an answered host call in its running run.
pub fn goal() {
    at_current_layouts(crate::fixtures::core_after_project::goal_over_worker_link);
}
/// Checkout uses the initial identity; delivery uses its mid-run renewal.
pub fn renewal() {
    at_current_layouts(|| {
        let counts = crate::m15::delivery(true);
        assert_eq!(counts.tokens, 5);
        assert_eq!(counts.checkpoints, 5);
    });
}
