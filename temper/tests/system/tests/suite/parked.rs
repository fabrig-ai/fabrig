//! Saved producer work survives answers across landing merges
//! (temper/docs/design/domain/worker/checkouts.md).
use skein_fake_llm_domain::api::{Script, Turn};
use temper_system_world::composed::{World, call};

fn question() -> Turn {
    call(b"message", br#"{"target":2,"form":"question","words":"Keep going? I recommend yes."}"#)
}

fn script(continuation: Vec<Turn>) -> Script {
    let mut turns = vec![
        call(b"write", br#"{"path":"first.txt","content":"first delivery\n"}"#),
        call(b"deliver", br#"{"title":"First edit","body":"Keep this work."}"#),
        call(b"write", br#"{"path":"second.txt","content":"second saved\n"}"#),
        call(b"write", br#"{"path":"kept.txt","content":"saved work\n"}"#),
        question(),
        call(b"wait", b"{}"),
    ];
    turns.extend(continuation);
    Script { cue: b"Read only what the change needs".as_slice().into(), turns: turns.into_boxed_slice() }
}

#[test]
fn a_parked_producers_edits_survive_a_clean_merge_and_a_contained_resume() {
    let mut world = World::parked_edits(
        925,
        b"goal.txt",
        script(vec![
            question(),
            call(b"wait", b"{}"),
            question(),
            call(b"wait", b"{}"),
            call(b"write", br#"{"path":"third.txt","content":"third delivery\n"}"#),
            call(b"finish", br#"{"title":"All edits","body":"Saved work is ready."}"#),
        ]),
    );
    world.continue_parked_edits(&[
        (b"kept.txt", b"saved work\n"),
        (b"first.txt", b"first delivery\n"),
        (b"second.txt", b"second saved\n"),
        (b"third.txt", b"third delivery\n"),
        (b"goal.txt", b"goal moved\n"),
    ]);
}

#[test]
fn a_parked_producers_saved_work_survives_a_save_refused_for_its_markers() {
    let mut world = World::parked_edits(
        926,
        b"second.txt",
        script(vec![
            question(),
            call(b"wait", b"{}"),
            call(b"read", br#"{"path":"second.txt"}"#),
            call(b"write", br#"{"path":"second.txt","content":"settled\n"}"#),
            call(b"finish", br#"{"title":"Settled edits","body":"Saved work is ready."}"#),
        ]),
    );
    world.continue_parked_edits(&[
        (b"kept.txt", b"saved work\n"),
        (b"first.txt", b"first delivery\n"),
        (b"second.txt", b"settled\n"),
    ]);
}
