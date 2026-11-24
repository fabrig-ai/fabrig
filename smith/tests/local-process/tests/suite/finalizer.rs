//! `smith exec`'s output and private chat through the shipped Local (smith/docs/design/shell/commands.md, 2–3).
use skein_fake_machine::Is;
use skein_io::kernel::Exit;
use smith_events::{Event, Status};
use smith_local_process_world::{
    World,
    process::Finalizer,
    world::{Files, HeadlessReply},
};

const ANSWER: &[u8] = b"First answer";
const RESULT: &str = "answer.txt";
const RUNTIME: &[u8] = b"runtime";
const PRIVATE: &str = "runtime/.smith-exec-test";

fn result_files(json: bool, chat: &str) -> Finalizer {
    Finalizer { json, chat: chat.into(), result: Some(RESULT.into()), private: None }
}

fn private_files(json: bool) -> Finalizer {
    Finalizer { json, chat: "main".into(), result: None, private: Some(PRIVATE.into()) }
}

fn assert_records(world: &World, status: Status, exit: i64) {
    let records = world.output_records();
    assert!(matches!(&records.last().expect("last stdout record").event, Event::SessionEnded(end) if end.exit == exit));
    let [completed]: [&smith_events::RunCompleted; 1] =
        super::run_completed(&records).try_into().expect("one run.completed");
    assert_eq!(completed.status, status);
}

fn last_lines(world: &World) -> impl Iterator<Item = &str> {
    std::str::from_utf8(world.errors()).expect("UTF-8 diagnostic").lines().rev()
}

#[test]
fn smith_exec_replaces_the_result_file_with_only_its_text() {
    let files = Files::new();
    let mut world = World::headless(31, &files, result_files(false, "main"), HeadlessReply::Accept, 0);
    world.write_file(RESULT.as_bytes(), b"an older and longer result\n");
    world.settle();

    assert_eq!(world.exit(), Some(Exit::Code(0)));
    assert_eq!(world.file_bytes(RESULT.as_bytes()), ANSWER);
    assert_eq!(world.shown(), ANSWER);
    assert!(last_lines(&world).next().expect("acceptance line").starts_with("smith: accepted in "));
}

#[test]
fn smith_exec_result_file_failure_exits_1_after_an_accepted_run() {
    let files = Files::new();
    let mut world = World::headless(32, &files, result_files(true, "main"), HeadlessReply::Accept, 1);
    world.make_directory(RESULT.as_bytes());
    world.settle();

    assert_eq!(world.exit(), Some(Exit::Code(1)));
    assert!(
        world
            .errors()
            .ends_with(b"smith: run accepted, but its result could not be written to answer.txt: IsADirectory\n"),
        "the result error is last: {}",
        String::from_utf8_lossy(world.errors()),
    );
    assert_records(&world, Status::Accepted, 1);
    assert!(world.directory_entries(RESULT.as_bytes()).is_empty(), "the destination is still a directory");
}

#[test]
fn smith_exec_result_file_failure_warns_before_a_parked_answers_last_line() {
    let files = Files::new();
    let mut world = World::headless(33, &files, result_files(true, "plan"), HeadlessReply::Wait, 5);
    world.make_directory(RESULT.as_bytes());
    world.settle();

    assert_eq!(world.exit(), Some(Exit::Code(5)));
    assert!(
        world.errors().ends_with(
            concat!(
                "smith: warning: its result could not be written to answer.txt: IsADirectory\n",
                "smith: needs input: answer with `smith exec --chat plan`\n",
            )
            .as_bytes()
        ),
        "result warning just before the parked answer: {}",
        String::from_utf8_lossy(world.errors())
    );
    assert_records(&world, Status::Parked, 5);
}

#[test]
fn smith_exec_ephemeral_removes_its_private_files_after_a_failed_answer() {
    let files = Files::new();
    let mut world = World::headless(34, &files, private_files(true), HeadlessReply::Filter, 1);
    world.settle();

    assert_eq!(world.exit(), Some(Exit::Code(1)));
    assert_records(&world, Status::Failed, 1);
    assert!(world.directory_entries(RUNTIME).is_empty(), "the runtime base is empty after exit");
    assert!(world.directory_entries(b"chat").is_empty(), "nothing was stored under the state directory");
}

#[test]
fn smith_exec_ephemeral_keeps_a_foreign_file_and_warns_before_the_last_line() {
    const FOREIGN: &str = "foreign.txt";
    const CONTENT: &[u8] = b"kept foreign content";
    let files = Files::new();
    let mut world = World::headless(35, &files, private_files(false), HeadlessReply::Accept, 0);
    world.foreign_on_turn(FOREIGN, CONTENT);
    world.settle();

    assert_eq!(world.exit(), Some(Exit::Code(0)));
    let name = std::path::Path::new(PRIVATE).file_name().expect("private leaf").as_encoded_bytes();
    assert_eq!(world.directory_entries(RUNTIME), [(Is::Directory, name.into())]);
    assert_eq!(world.directory_entries(PRIVATE.as_bytes()), [(Is::File, FOREIGN.as_bytes().into())]);
    assert_eq!(world.file_bytes(format!("{PRIVATE}/{FOREIGN}").as_bytes()), CONTENT);
    assert!(world.directory_entries(b"chat").is_empty(), "nothing was stored under the state directory");
    let mut lines = last_lines(&world);
    assert!(lines.next().expect("acceptance line").starts_with("smith: accepted in "));
    let warning = format!("smith: warning: ephemeral directory {PRIVATE}: NotEmpty");
    assert_eq!(lines.next(), Some(warning.as_str()));
}
