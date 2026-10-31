//! A chat's delegation card through the shipped terminal, against client/README.md, 5.7: a QA pass run by `--profile qa`.
use crate::support;

use skein_io::kernel::Exit;
use skein_world::end_to_end::{Binary, Command, Mode};
use std::path::Path;
use support::{Scratch, WITHIN, configuration, terminal};

// The playground forge creates the seeded widget commit (1), then the CSV change
// commit (2), before merging task 2's change (3).
const PLAYGROUND_MERGE_COUNT: u64 = 3;

fn playground_merge_short_id() -> String {
    let id = temper_fake_git::id::id(PLAYGROUND_MERGE_COUNT);
    std::str::from_utf8(&id[..7]).expect("fake commit ids are ASCII hex").to_owned()
}

// The shipped terminal opens at Inbox (temper/docs/design/client/README.md, 6.10).
// Start the playground with the welcome's act (draft/playground-reset.md, 6.1),
// then follow support/person.rs's Chats and Scripted model with the same CSV request.
fn playground_terminal(directory: &Path, link: &[u8]) -> Binary {
    let private = directory.join("client");
    let link_file = directory.join("sign-in-link");
    support::write_private(&link_file, link);
    let signed = support::ctl::alone(
        directory,
        support::ctl::Call::bare(&[
            private.to_str().expect("client path"),
            "sign-in",
            link_file.to_str().expect("link path"),
        ]),
    );
    assert_eq!(signed.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&signed.stderr));
    std::fs::remove_file(link_file).expect("remove spent link");
    let started = support::ctl::alone(
        directory,
        support::ctl::Call::bare(&[private.to_str().expect("client path"), "playground", "start"]),
    );
    assert_eq!(started.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&started.stderr));
    let document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("temper.json")).expect("init document"))
            .expect("document JSON");
    let socket = document["application"]["clients"]["listener"]["socket"].as_str().expect("client socket");
    terminal::open(directory, &private, format!("http://{socket}").as_bytes())
}

fn start_csv_change(person: &mut terminal::Typist) {
    person.wait_for(b"Inbox", WITHIN);
    person.type_on(b"Chats", b"");
    person.wait_for(b"No chats yet.", WITHIN);
    person.type_on(b"Playground", b"");
    person.wait_for(b"Ready", WITHIN);
    person.type_on(b"Scripted model", b"");
    person.wait_for(b"Scripted model (pressed)", WITHIN);
    person.type_on(b"Start a chat: (empty) (enter sends)", b"Add CSV export to widget");
}

#[test]
fn running_card_names_the_change_and_exposes_its_steps() {
    let scratch = Scratch::startup("qa-delegation-running");
    let mut arguments = vec!["box".into()];
    arguments.extend(configuration::flags(scratch.path()));
    let mut boxed = Binary::start(
        Command {
            program: skein_scratch::paths::binary("temper"),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("the first run starts");
    let link = terminal::link(&mut boxed, WITHIN);
    let opened = playground_terminal(scratch.path(), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    start_csv_change(&mut person);
    person.wait_for("T2 · CHANGE".as_bytes(), WITHIN);
    person.wait_for(b"Running", WITHIN);
    person.wait_for(b"Export widget's table as CSV", WITHIN);
    person.wait_for(b"Steps", WITHIN);
    person.wait_for(b"Branch pushed", WITHIN);
    person.wait_for(b"Pull request opened", WITHIN);
    person.wait_for(b"CI passed", WITHIN);
    person.wait_for(b"Landed", WITHIN);
    person.type_on(b"Task 2", b"");
    person.wait_for(b"== Export widget's table as CSV ==", WITHIN);
    person.type_line(b"q");
    assert_eq!(person.exit(WITHIN), 0, "{}", person.stderr());
    scratch.remove();
}

#[test]
fn result_anchor_preserves_the_landed_words_and_task_link() {
    let scratch = Scratch::startup("qa-delegation-landed-words");
    let mut arguments = vec!["box".into()];
    arguments.extend(configuration::flags(scratch.path()));
    let mut boxed = Binary::start(
        Command {
            program: skein_scratch::paths::binary("temper"),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("the first run starts");
    let link = terminal::link(&mut boxed, WITHIN);
    let opened = playground_terminal(scratch.path(), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    start_csv_change(&mut person);
    person.wait_for("CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.".as_bytes(), WITHIN);
    // This contiguous block checks the title and the first seven characters
    // of the result's merge commit, and keeps the four stand-in steps in
    // their design order.
    person.wait_for(
        format!("Task 2:\n    T2 · CHANGE\n    * Landed\n    ### Export widget's table as CSV\n    CI passed. Landed · merge commit {merge}\n    Steps:\n      - Branch pushed (completed)\n      - Pull request opened (completed)\n      - CI passed (completed)\n      - Landed (completed)\n", merge = playground_merge_short_id()).as_bytes(),
        WITHIN,
    );
    person.type_on(b"Task 2", b"");
    person.wait_for(b"== Export widget's table as CSV ==", WITHIN);
    person.type_line(b"q");
    assert_eq!(person.exit(WITHIN), 0, "{}", person.stderr());
    scratch.remove();
}

#[test]
fn original_delegate_call_uses_the_ended_task_document() {
    let scratch = Scratch::startup("qa-delegation-original-call");
    let mut arguments = vec!["box".into()];
    arguments.extend(configuration::flags(scratch.path()));
    let mut boxed = Binary::start(
        Command {
            program: skein_scratch::paths::binary("temper"),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("the first run starts");
    let link = terminal::link(&mut boxed, WITHIN);
    let opened = playground_terminal(scratch.path(), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    start_csv_change(&mut person);
    person.wait_for("CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.".as_bytes(), WITHIN);
    // The small conversation is whole (client/README.md, 5.4), its first turn
    // within the committed turn's own text budget: the original delegate call
    // carries the card, from the ended task's own document (5.7).
    person.wait_for(
        format!("I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.\n  Task 2:\n    T2 · CHANGE\n    * Landed\n    ### Export widget's table as CSV\n    CI passed. Landed · merge commit {merge}\n", merge = playground_merge_short_id()).as_bytes(),
        WITHIN,
    );
    person.type_on(b"Task 2", b"");
    person.wait_for(b"== Export widget's table as CSV ==", WITHIN);
    person.type_line(b"q");
    assert_eq!(person.exit(WITHIN), 0, "{}", person.stderr());
    scratch.remove();
}
