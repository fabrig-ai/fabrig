//! Commit-pinned reads of the change's method (domain/forge/temper-dir.md, 4).
use super::*;
use fixtures::{BASE, CHANGE, HEAD};
use skein_forge::api::{Answer, Commit, CommitMessage, Entry, EntryKind, Error, Op, Read};
use temper_engine_domain_forge_change::{self as policy, Decision, Delegate, Effect, Repair, Status, Way};

const MOVED: Commit = [14; 32];
const WAYS: &[u8] = b".temper/ways.toml";

fn fixture(state: policy::State, way: Way, base: &[u8]) -> Domain {
    let mut d = fixtures::adopted_domain();
    let mut row = fixtures::opening_row();
    row.base = Box::from(base);
    row.pull = Some(1);
    row.pending = None;
    row.effect = policy::EffectResult::None;
    row.change.state = state;
    row.change.ready_since = Some(env().wall);
    row.change.way = way;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    standing::keep(&mut d, 5, 1, standing::rule());
    d
}

fn first() -> policy::State {
    policy::State::First { head: HEAD, ready_since: env().wall }
}

fn gating() -> policy::State {
    policy::State::Gating { head: HEAD, asked: Box::new([]) }
}

fn gather(d: &mut Domain, base: Commit, contains_base: Status) -> Box<[Request]> {
    let row = d.change(CHANGE).expect("change");
    let branch_name = row.branch.clone();
    let pull = skein_forge::api::Pull { base: row.base.clone(), base_commit: Some(base), ..fixtures::open_pull() };
    fixtures::step_change(d);
    for _ in 0_u32..16 {
        let (call, op) = fixtures::next_call(d);
        if let Op::Read(Read::Branch { branch }) = op {
            fixtures::answer(d, call, Ok(Answer::Commit(if branch == branch_name { HEAD } else { base })));
        } else if let Op::Read(Read::Pull { .. }) = op {
            fixtures::answer(d, call, Ok(Answer::Pull(pull.clone())));
        } else if let Op::Read(Read::Checks { .. }) = op {
            fixtures::answer(d, call, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
        } else if let Op::Read(Read::Commits { before, .. }) = op {
            let commits: Box<[CommitMessage]> = if before == HEAD && contains_base == Status::Passed {
                Box::new([])
            } else {
                Box::new([CommitMessage {
                    id: if before == HEAD { base } else { HEAD },
                    parents: Box::new([]),
                    message: Box::new([]),
                }])
            };
            fixtures::answer(d, call, Ok(Answer::Commits { commits, more: false }));
        } else if let Op::Read(Read::Compare { before, after }) = op {
            assert_eq!((before, after), (BASE, MOVED), "an independent landing observation");
            fixtures::answer(
                d,
                call,
                Ok(Answer::Compare { before, after, ahead: 1, behind: 0, changed: 0, files: Box::new([]) }),
            );
        } else {
            assert_eq!(op, Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }));
            return fixtures::answer(d, call, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        }
    }
    panic!("gather reaches reviews");
}

fn compare(d: &mut Domain, base: Commit, path: &[u8], whole: bool) -> Box<[Request]> {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Compare { before: base, after: HEAD }));
    fixtures::answer(
        d,
        call,
        Ok(Answer::Compare {
            before: base,
            after: HEAD,
            ahead: 1,
            behind: 0,
            changed: if whole { 1 } else { 2 },
            files: Box::new([Box::from(path)]),
        }),
    )
}

fn file_read(d: &mut Domain, head: Commit, path: &[u8], result: Result<Answer, Error>) -> Box<[Request]> {
    let (call, op) = fixtures::next_call(d);
    let Op::Read(Read::File { head: read_head, path: read_path, .. }) = op else { panic!("file read, got {op:?}") };
    assert_eq!((read_head, read_path.as_ref()), (head, path));
    fixtures::answer(d, call, result)
}

fn directory(d: &mut Domain, head: Commit, path: &[u8], name: &[u8], kind: EntryKind) -> Box<[Request]> {
    file_read(
        d,
        head,
        path,
        Ok(Answer::Directory {
            head,
            path: Box::from(path),
            entries: Box::new([Entry { name: name.into(), kind }]),
            truncated: false,
        }),
    )
}

fn good_tree(d: &mut Domain) -> Box<[Request]> {
    file_read(
        d,
        HEAD,
        b".temper",
        Ok(Answer::Directory {
            head: HEAD,
            path: b".temper".as_slice().into(),
            entries: Box::new([
                Entry { name: b"ways".as_slice().into(), kind: EntryKind::Directory },
                Entry { name: b"ways.toml".as_slice().into(), kind: EntryKind::File },
            ]),
            truncated: false,
        }),
    );
    directory(d, HEAD, b".temper/ways", b"pair.toml", EntryKind::File);
    file_read(
        d,
        HEAD,
        WAYS,
        Ok(Answer::File {
            head: HEAD,
            path: WAYS.into(),
            bytes: b"offer = [\"pair\"]\n".as_slice().into(),
            truncated: false,
        }),
    );
    let path = b".temper/ways/pair.toml";
    file_read(
        d,
        HEAD,
        path,
        Ok(Answer::File {
            head: HEAD,
            path: path.as_slice().into(),
            bytes: b"format = 1\n".as_slice().into(),
            truncated: false,
        }),
    )
}

fn offers(d: &mut Domain, bytes: &[u8]) -> Box<[Request]> {
    directory(d, HEAD, b".temper", b"ways.toml", EntryKind::File);
    file_read(d, HEAD, WAYS, Ok(Answer::File { head: HEAD, path: WAYS.into(), bytes: bytes.into(), truncated: false }))
}

fn decision(requests: &[Request]) -> (Decision, Option<&ChangeEvidence>) {
    for request in requests {
        if let Request::ChangeDecision { task: CHANGE, decision, evidence, .. } = request {
            return (*decision, evidence.as_ref());
        }
    }
    panic!("no decision: {requests:?}");
}

fn merged(requests: &[Request], base: Commit, expected: TemperDirEvidence) {
    let (effect, evidence) = decision(requests);
    assert_eq!(effect, Decision::Effect(Effect::Merge { head: HEAD, base }));
    let evidence = evidence.expect("merge evidence");
    assert_eq!((evidence.head, evidence.base_tip, evidence.temper_dir), (Some(HEAD), base, expected));
}

#[test]
fn temper_dir_coverage_controls_the_reads_and_merge_evidence() {
    for (path, whole, expected) in [
        (b"src/main.rs".as_slice(), true, TemperDirEvidence::Uncovered),
        (b".temper/hooks/config.toml".as_slice(), true, TemperDirEvidence::Uncovered),
        (b"src/main.rs".as_slice(), false, TemperDirEvidence::Passed { tree: HEAD }),
        (WAYS, true, TemperDirEvidence::Passed { tree: HEAD }),
    ] {
        let mut d = fixture(first(), Way::Queue, b"main");
        gather(&mut d, BASE, Status::Passed);
        let compared = compare(&mut d, BASE, path, whole);
        let done = if !whole {
            file_read(&mut d, HEAD, b".temper", Err(Error::Missing));
            directory(&mut d, BASE, b".temper", b"ways.toml", EntryKind::File)
        } else if path == WAYS {
            good_tree(&mut d)
        } else if path.starts_with(b".temper/") {
            directory(&mut d, HEAD, b".temper", b"hooks", EntryKind::Directory)
        } else {
            compared
        };
        merged(&done, BASE, expected);
        assert!(d.steps.is_empty());
    }
    let mut d = fixture(first(), Way::Queue, b"goal");
    let done = gather(&mut d, BASE, Status::Passed);
    merged(&done, BASE, TemperDirEvidence::Uncovered);
    assert!(d.steps.is_empty(), "another branch doesn't need a comparison");
}

#[test]
fn temper_dir_waits_to_read_until_the_queue_update_and_invalidates_a_moved_base() {
    let mut d = fixture(first(), Way::Queue, b"main");
    gather(&mut d, BASE, Status::Failed);
    let compared = compare(&mut d, BASE, WAYS, true);
    let (effect, evidence) = decision(&compared);
    assert_eq!(effect, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
    assert_eq!(evidence.expect("update evidence").temper_dir, TemperDirEvidence::Pending);
    assert!(d.steps.is_empty(), "no file before updating");

    let mut d = fixture(gating(), Way::Queue, b"main");
    gather(&mut d, BASE, Status::Passed);
    compare(&mut d, BASE, WAYS, true);
    assert_eq!(decision(&good_tree(&mut d)).0, Decision::Ready);
    d.changes.get_mut(&CHANGE).expect("ready change").change.state = first();
    gather(&mut d, MOVED, Status::Passed);
    compare(&mut d, MOVED, WAYS, true);
    // The head stayed the same; a pass at the old base cannot authorize this merge.
    let approved = good_tree(&mut d);
    merged(&approved, MOVED, TemperDirEvidence::Passed { tree: HEAD });
}

#[test]
fn temper_dir_failures_wait_for_a_read_and_repair_only_faults_at_the_own_head() {
    let mut d = fixture(gating(), Way::Turn, b"main");
    gather(&mut d, BASE, Status::Failed);
    compare(&mut d, BASE, WAYS, true);
    let pending = file_read(&mut d, HEAD, b".temper", Err(Error::Forbidden));
    assert!(matches!(decision(&pending).0, Decision::Wait { .. }));
    assert_eq!(d.change(CHANGE).expect("waiting change").change.repairs, 0);
    gather(&mut d, BASE, Status::Failed);
    compare(&mut d, BASE, WAYS, true);
    assert_eq!(decision(&good_tree(&mut d)).0, Decision::Ready);

    for (way, state) in [(Way::Turn, gating()), (Way::Queue, first())] {
        let mut d = fixture(state, way, b"main");
        gather(&mut d, BASE, Status::Passed);
        compare(&mut d, BASE, WAYS, true);
        let fault = offers(&mut d, b"offer = [\"pair\"]\n");
        let why = Repair::TemperDir { head: HEAD };
        assert_eq!(decision(&fault).0, Decision::Delegate(Delegate::Repair(why)));
        let row = d.change(CHANGE).expect("repairing change");
        assert_eq!(row.change.repairs, 1);
        let words = change_words::repair_words(row, why, None, &[], 4096).expect("repair spec words");
        let expected = skein_lib::bytes::concat(&[
            b"temper's check of .temper/ at ",
            &commit_text(&HEAD),
            b":\n.temper/ways.toml:1: `offer` names `pair`, and `.temper/ways/pair.toml` is missing\n",
        ]);
        assert!(words.starts_with(&expected), "{words:?}");
    }

    let mut bounded = fixture(first(), Way::Queue, b"main");
    gather(&mut bounded, BASE, Status::Passed);
    compare(&mut bounded, BASE, WAYS, true);
    offers(&mut bounded, b"offer = [\"pair\"]\na=0\nb=0\nc=0\nd=0\ne=0\nf=0\ng=0\nh=0\ni=0\nj=0\nk=0\nl=0\nm=0\nn=0\no=0\np=0\nq=0\nr=0\ns=0\nt=0\n");
    let reason = bounded.change(CHANGE).expect("repairing change").temper_dir_fault.as_deref().expect("fault text");
    assert!(reason.ends_with(b"\nand 5 more"));
    let mut breaks = 0_u32;
    for byte in reason {
        if *byte == b'\n' {
            breaks = breaks.saturating_add(1);
        }
    }
    assert_eq!(breaks, 17);

    let mut d = fixture(first(), Way::Queue, b"main");
    d.repositories.get_mut(&REPO).expect("adopted").home = false;
    let mut home = adopted();
    home.provider.repository = 9;
    home.name = b"method".as_slice().into();
    d.repositories.insert(home.provider, Box::new(home)).expect("home beside changed repository");
    gather(&mut d, BASE, Status::Passed);
    compare(&mut d, BASE, b".temper/charters/intro.md", true);
    directory(&mut d, HEAD, b".temper", b"charters", EntryKind::Directory);
    let done = file_read(
        &mut d,
        HEAD,
        b".temper/charters",
        Ok(Answer::Directory {
            head: HEAD,
            path: b".temper/charters".as_slice().into(),
            entries: Box::new([]),
            truncated: false,
        }),
    );
    assert_eq!(decision(&done).0, Decision::Delegate(Delegate::Repair(Repair::TemperDir { head: HEAD })));
    assert!(
        d.change(CHANGE)
            .expect("repairing change")
            .temper_dir_fault
            .as_ref()
            .expect("fault words")
            .ends_with(b".temper/charters: a project's ways live in its home repository, org/method")
    );
}
