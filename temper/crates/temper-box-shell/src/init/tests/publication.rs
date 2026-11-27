use super::{Done, Error, Fd, KernelScript, Op, OpenHow, Path, write_file};

const DESTINATION: &str = "/fixture/config/temper.toml";
const NAME: &[u8] = b"temper.toml";
const PARTIAL: &[u8] = b"temper.toml.partial";
const MODE: u32 = 0o644;
const ROOT: Fd = Fd::new(11);
const FILE: Fd = Fd::new(12);
const DOCUMENT: &[u8] = b"listen = \"127.0.0.1:8367\"\n";
const SHORT: u32 = 1;

type Step = (Op, Result<Done, Error>);

fn create(result: Result<Done, Error>) -> Step {
    (Op::Open { root: ROOT, path: PARTIAL.into(), how: OpenHow::Create { mode: Some(MODE) } }, result)
}

fn write(from: u32, result: Result<Done, Error>) -> Step {
    (Op::Write { fd: FILE, bytes: DOCUMENT.into(), from, at: u64::from(from) }, result)
}

fn remove() -> Step {
    (Op::Remove { dir: ROOT, name: PARTIAL.into(), directory: false }, Ok(Done::Nothing))
}

fn close_file() -> Step {
    (Op::Close { fd: FILE }, Ok(Done::Nothing))
}

fn publication(rename: Result<Done, Error>) -> [Step; 5] {
    let remaining = u32::try_from(DOCUMENT.len()).expect("document length") - SHORT;
    [
        create(Ok(Done::Fd(FILE))),
        write(0, Ok(Done::Count(SHORT))),
        write(SHORT, Ok(Done::Count(remaining))),
        close_file(),
        (Op::Rename { from_dir: ROOT, from: PARTIAL.into(), to_dir: ROOT, to: NAME.into(), no_replace: true }, rename),
    ]
}

fn run(steps: impl IntoIterator<Item = Step>) -> Result<(), String> {
    let mut io = KernelScript::operations(steps.into_iter().chain([(Op::Close { fd: ROOT }, Ok(Done::Nothing))]));
    let result = write_file(&mut io, ROOT, Path::new(DESTINATION), DOCUMENT.into(), MODE, |partial| {
        assert_eq!(partial, Path::new("/fixture/config/temper.toml.partial"));
        Ok(())
    });
    io.finish();
    result
}

#[test]
fn publishes_the_complete_file_only_by_rename() {
    run(publication(Ok(Done::Nothing))).expect("file published");
}

#[test]
fn failed_write_removes_partial_and_names_destination() {
    let steps = [create(Ok(Done::Fd(FILE))), write(0, Err(Error::NoSpace)), close_file(), remove()];
    assert_eq!(run(steps).expect_err("write must fail"), "/fixture/config/temper.toml: errno 28");
}

#[test]
fn racing_destination_is_not_replaced() {
    let steps = publication(Err(Error::Exists)).into_iter().chain([remove()]);
    assert_eq!(run(steps).expect_err("rename must fail"), "/fixture/config/temper.toml: errno 17");
}

#[test]
fn stale_partial_is_removed_before_publication() {
    let steps = [create(Err(Error::Exists)), remove()].into_iter().chain(publication(Ok(Done::Nothing)));
    run(steps).expect("file published after removing stale partial");
}
