//! A chat's delegation card through the shipped terminal, against client/README.md, 5.7: a QA pass run by `--profile qa`.
pub mod support;

use skein_world::end_to_end::{Binary, Command, Mode};
use support::{Scratch, WITHIN, configuration, terminal};

// The demo forge creates the seeded widget commit (1), then the CSV change
// commit (2), before merging task 2's change (3).
const DEMO_MERGE_COUNT: u64 = 3;

fn demo_merge_short_id() -> String {
    let id = temper_fake_git::id::id(DEMO_MERGE_COUNT);
    std::str::from_utf8(&id[..7]).expect("fake commit ids are ASCII hex").to_owned()
}

#[test]
fn running_card_names_the_change_and_exposes_its_steps() {
    // Choose the demo explicitly so this check is independent of the default
    // project's first-run banner and suggested-request setup.
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
    let opened = terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    // The first run offers the playground (client/README.md); its scripted
    // model is then the chat's default.
    person.wait_for(b"Try the playground", WITHIN);
    person.type_on(b"Try the playground", b"");
    person.wait_for("Using Scripted model · Default".as_bytes(), WITHIN);
    person.type_on(b"Chats", b"");
    person.wait_for(b"Start a chat: (empty) (enter sends)", WITHIN);
    // The chat box's Enter starts the chat (client/README.md).
    person.type_on(b"Start a chat: (empty) (enter sends)", b"Add CSV export to widget");
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
    let opened = terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    // The first run offers the playground (client/README.md); its scripted
    // model is then the chat's default.
    person.wait_for(b"Try the playground", WITHIN);
    person.type_on(b"Try the playground", b"");
    person.wait_for("Using Scripted model · Default".as_bytes(), WITHIN);
    person.type_on(b"Chats", b"");
    person.wait_for(b"Start a chat: (empty) (enter sends)", WITHIN);
    // The chat box's Enter starts the chat (client/README.md).
    person.type_on(b"Start a chat: (empty) (enter sends)", b"Add CSV export to widget");
    person.wait_for("CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.".as_bytes(), WITHIN);
    // This contiguous block checks the title and the first seven characters
    // of the result's merge commit, and keeps the four stand-in steps in
    // their design order.
    person.wait_for(
        format!("Task 2:\n    T2 · CHANGE\n    * Landed\n    ### Export widget's table as CSV\n    CI passed. Landed · merge commit {merge}\n    Steps:\n      - Branch pushed (completed)\n      - Pull request opened (completed)\n      - CI passed (completed)\n      - Landed (completed)\n", merge = demo_merge_short_id()).as_bytes(),
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
    let opened = terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    // The first run offers the playground (client/README.md); its scripted
    // model is then the chat's default.
    person.wait_for(b"Try the playground", WITHIN);
    person.type_on(b"Try the playground", b"");
    person.wait_for("Using Scripted model · Default".as_bytes(), WITHIN);
    person.type_on(b"Chats", b"");
    person.wait_for(b"Start a chat: (empty) (enter sends)", WITHIN);
    // The chat box's Enter starts the chat (client/README.md).
    person.type_on(b"Start a chat: (empty) (enter sends)", b"Add CSV export to widget");
    person.wait_for("CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.".as_bytes(), WITHIN);
    // The small conversation is whole (client/README.md, 5.4), its first turn
    // within the committed turn's own text budget: the original delegate call
    // carries the card, from the ended task's own document (5.7).
    person.wait_for(
        format!("I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.\n  Task 2:\n    T2 · CHANGE\n    * Landed\n    ### Export widget's table as CSV\n    CI passed. Landed · merge commit {merge}\n", merge = demo_merge_short_id()).as_bytes(),
        WITHIN,
    );
    person.type_on(b"Task 2", b"");
    person.wait_for(b"== Export widget's table as CSV ==", WITHIN);
    person.type_line(b"q");
    assert_eq!(person.exit(WITHIN), 0, "{}", person.stderr());
    scratch.remove();
}
