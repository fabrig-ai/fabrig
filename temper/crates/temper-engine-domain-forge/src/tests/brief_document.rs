//! Document sections at a branch head (temper/docs/design/protocol/configuration/settings.md, 6).
use super::*;
use skein_forge::api::{Answer, Error, Read};

const SECTION: Token = Token::new(72);
const HEAD: [u8; 32] = [0x11; 32];
const PREFIX: &[u8] = b"At main's head 1111111111111111111111111111111111111111111111111111111111111111\n";
// The step fixture's brief allotment, within its client answer bound.
const BUDGET: u32 = 512;
// Header (80 bytes), one text line (6), and the exact cut notice (15).
const LINE_BUDGET: u32 = 101;

fn document() -> BriefDocument {
    BriefDocument {
        host: b"forge.example".as_slice().into(),
        owner: b"org".as_slice().into(),
        name: b"repo".as_slice().into(),
        branch: b"main".as_slice().into(),
        path: b"rules.md".as_slice().into(),
        heading: None,
    }
}

fn with_head(tail: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(PREFIX.len().checked_add(tail.len()).expect("bounded fixture words"));
    writer.put(PREFIX).expect("fixture prefix");
    writer.put(tail).expect("fixture text");
    writer.finish()
}

fn setup() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    d
}

fn plan(d: &mut Domain, document: BriefDocument, budget: u32) -> Queue<Request> {
    outputs(d, Event::PlanDocument { section: SECTION, document: Box::new(document) });
    let mut out = Queue::uncapped();
    let env = env();
    step(
        d,
        &env,
        Event::GatherPlanned {
            task: 9,
            section: SECTION,
            parts: 1,
            bytes: budget,
            ci_budget: 0,
            deadline: env.now.saturating_add(Duration::from_secs(30)),
        },
        &mut out,
    );
    out
}

fn read(out: &mut Queue<Request>) -> (Token, Read) {
    let Some(Request::BriefClient { event: skein_forge_client::Event::Read { owner, repository, read } }) = out.pop()
    else {
        panic!("one document read: {out:?}")
    };
    assert_eq!(repository, REPO);
    assert!(out.is_empty());
    (owner, read)
}

fn answer(d: &mut Domain, owner: Token, result: Result<Answer, Error>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    let missing = result == Err(Error::Missing);
    assert!(brief::brief_answer(d, owner, result, LIMITS.client.rows, env().now, &mut out));
    if missing {
        let Some(Request::BriefReadRefused { error: Some(Error::Missing), .. }) = out.pop() else {
            panic!("missing read diagnostic");
        };
    }
    out
}

fn file_read(d: &mut Domain, doc: BriefDocument, budget: u32) -> Token {
    let (owner, branch) = read(&mut plan(d, doc, budget));
    assert_eq!(branch, Read::Branch { branch: b"main".as_slice().into() });
    let (file_owner, file) = read(&mut answer(d, owner, Ok(Answer::Commit(HEAD))));
    assert_eq!(file_owner, owner);
    assert_eq!(file, Read::File { head: HEAD, path: b"rules.md".as_slice().into(), max_bytes: CLIENT.answer_bytes });
    owner
}

fn finish(d: &mut Domain, owner: Token, bytes: &[u8], truncated: bool) {
    let mut out = answer(
        d,
        owner,
        Ok(Answer::File { head: HEAD, path: b"rules.md".as_slice().into(), bytes: bytes.into(), truncated }),
    );
    let Some(Request::BriefSized { section: SECTION, size: Ok(_) }) = out.pop() else {
        panic!("sized document: {out:?}")
    };
    assert!(out.is_empty());
}

fn take(d: &mut Domain) -> Box<[u8]> {
    d.take_brief(SECTION).expect("held document")
}

fn gathered(doc: BriefDocument, bytes: &[u8], budget: u32, truncated: bool) -> Box<[u8]> {
    let mut d = setup();
    let owner = file_read(&mut d, doc, budget);
    finish(&mut d, owner, bytes, truncated);
    take(&mut d)
}

#[test]
fn a_document_section_is_the_file_at_the_branch_head() {
    let expected = with_head(b"Read these rules.\nKeep them.\n");
    assert_eq!(gathered(document(), b"Read these rules.\nKeep them.\n", BUDGET, false).as_ref(), expected.as_ref());
}

#[test]
fn a_heading_takes_its_part_to_the_next_heading_of_its_level() {
    for (bytes, expected) in [
        (
            b"# A\n## B\nfirst\n### Deeper\nkeep\n```md\n# fenced\n```\nlast\n## C\nexclude\n".as_slice(),
            b"## B\nfirst\n### Deeper\nkeep\n```md\n# fenced\n```\nlast\n".as_slice(),
        ),
        (
            b"~~~\n## B\n~~~\n# A\n## B\nkeep\n~~~md\n# fenced\n~~~\n# C\nexclude\n",
            b"## B\nkeep\n~~~md\n# fenced\n~~~\n",
        ),
        (b"## Bee\nexclude\n## B \nexclude\n## B\nfirst\n## B\nsecond\n", b"## B\nfirst\n"),
    ] {
        let mut doc = document();
        doc.heading = Some(b"B".as_slice().into());

        assert_eq!(gathered(doc, bytes, BUDGET, false).strip_prefix(PREFIX), Some(expected));
    }
}

#[test]
fn a_long_document_is_cut_at_a_line_end_and_counts_what_it_left_out() {
    for (truncated, budget, tail) in [
        (false, LINE_BUDGET, b"first\n[34 bytes cut]\n".as_slice()),
        (false, LINE_BUDGET - 1, b"[40 bytes cut]\n"),
        (true, LINE_BUDGET + 10, b"first\n[more than 34 bytes cut]\n"),
    ] {
        let expected = with_head(tail);
        assert_eq!(
            gathered(document(), b"first\nsecond second second second\nthird\n", budget, truncated).as_ref(),
            expected.as_ref()
        );
    }
}

#[test]
fn a_document_cut_again_keeps_its_commit() {
    let mut d = setup();
    let owner = file_read(&mut d, document(), BUDGET);
    finish(&mut d, owner, b"first\nsecond second second second\nthird\n", false);
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::BranchHead(BranchHead {
                name: Name {
                    forge: REPO.forge,
                    repository: REPO.repository,
                    what: What::Branch(Box::new([b"main".as_slice().into()])),
                },
                commit: [0x22; 32],
                owned: false,
            }),
        },
    );
    let cut = outputs(&mut d, Event::CutBrief { section: SECTION, bytes: LINE_BUDGET });
    assert_eq!(cut.as_ref(), &[Request::BriefSized { section: SECTION, size: Ok(LINE_BUDGET) }]);
    let cut = outputs(&mut d, Event::CutBrief { section: SECTION, bytes: LINE_BUDGET - 6 });
    assert_eq!(cut.as_ref(), &[Request::BriefSized { section: SECTION, size: Ok(LINE_BUDGET - 6) }]);
    let expected = with_head(b"[40 bytes cut]\n");
    assert_eq!(take(&mut d).as_ref(), expected.as_ref());
    assert!(d.briefs_idle());
}

#[test]
fn a_document_of_a_repository_not_adopted_says_so() {
    let mut d = domain();
    plan(&mut d, document(), BUDGET);
    assert_eq!(take(&mut d).as_ref(), b"Repository org/repo on forge.example is not adopted.");
    assert!(d.briefs_idle());
}

#[test]
fn a_document_of_a_missing_branch_says_so() {
    let mut d = setup();
    let (owner, _) = read(&mut plan(&mut d, document(), BUDGET));
    answer(&mut d, owner, Err(Error::Missing));
    assert_eq!(take(&mut d).as_ref(), b"No branch main in org/repo.");
}

#[test]
fn a_document_of_a_missing_file_says_so() {
    for result in [
        Err(Error::Missing),
        Ok(Answer::Directory {
            head: HEAD,
            path: b"rules.md".as_slice().into(),
            entries: Box::new([]),
            truncated: false,
        }),
    ] {
        let mut d = setup();
        let owner = file_read(&mut d, document(), BUDGET);
        answer(&mut d, owner, result);
        assert_eq!(
            take(&mut d).as_ref(),
            b"No file rules.md in org/repo at main's head 1111111111111111111111111111111111111111111111111111111111111111."
        );
    }
}

#[test]
fn a_document_of_a_missing_heading_says_so() {
    let mut doc = document();
    doc.heading = Some(b"Missing".as_slice().into());
    assert_eq!(
        gathered(doc, b"# Other\n", BUDGET, false).as_ref(),
        b"No heading \"Missing\" in rules.md at main's head 1111111111111111111111111111111111111111111111111111111111111111."
    );
}

#[test]
fn a_failed_document_read_is_missing() {
    for result in [Error::Unavailable, Error::Forbidden, Error::Refused] {
        for file in [false, true] {
            let mut d = setup();
            let owner = if file {
                file_read(&mut d, document(), BUDGET)
            } else {
                read(&mut plan(&mut d, document(), BUDGET)).0
            };
            let mut out = answer(&mut d, owner, Err(result));
            assert_eq!(
                out.pop(),
                Some(Request::BriefReadRefused {
                    section: SECTION,
                    stage: if file { "file" } else { "branch" },
                    reason: brief_words::reason_of(&Err(result)),
                    heads: None,
                    error: Some(result)
                })
            );
            assert_eq!(
                out.pop(),
                Some(Request::BriefSized { section: SECTION, size: Err(brief_words::unread_of(&Err(result))) })
            );
            assert!(out.is_empty());
            assert!(d.take_brief(SECTION).is_none());
            assert!(d.briefs_idle());
        }
    }
}

#[test]
fn a_busy_document_waits_for_reclaim_and_retries_the_same_read() {
    for file in [false, true] {
        let mut d = setup();
        let (owner, expected) = if file {
            let owner = file_read(&mut d, document(), BUDGET);
            (owner, Read::File { head: HEAD, path: b"rules.md".as_slice().into(), max_bytes: CLIENT.answer_bytes })
        } else {
            read(&mut plan(&mut d, document(), BUDGET))
        };
        let mut out = answer(&mut d, owner, Err(Error::Busy));
        brief::retry(&mut d, &mut out);
        assert!(out.is_empty());
        assert!(!brief::waiting(&d));
        d.reclaim();
        assert!(brief::waiting(&d));
        brief::retry(&mut d, &mut out);
        assert_eq!(read(&mut out), (owner, expected));
        if !file {
            read(&mut answer(&mut d, owner, Ok(Answer::Commit(HEAD))));
        }
        finish(&mut d, owner, b"rules\n", false);
        assert_eq!(take(&mut d).as_ref(), with_head(b"rules\n").as_ref());
    }
}

#[test]
fn a_busy_document_fails_at_its_gather_deadline() {
    let mut d = setup();
    let owner = file_read(&mut d, document(), BUDGET);
    let mut out = answer(&mut d, owner, Err(Error::Busy));
    let deadline = env().now.saturating_add(Duration::from_secs(30));
    brief::expire(&mut d, Time::from_nanos(deadline.as_nanos() - 1), &mut out);
    assert!(out.is_empty());
    assert_eq!(brief::next_deadline(&d), Some(deadline));
    brief::expire(&mut d, deadline, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::BriefReadRefused {
            section: SECTION,
            stage: "file",
            reason: "the forge client is busy",
            heads: None,
            error: Some(Error::Busy)
        })
    );
    assert_eq!(out.pop(), Some(Request::BriefSized { section: SECTION, size: Err(brief_words::busy()) }));
    d.reclaim();
    brief::retry(&mut d, &mut out);
    assert!(out.is_empty());
    assert!(d.briefs_idle());
}

#[test]
fn an_inflight_document_stays_late_until_dropped() {
    let mut d = setup();
    file_read(&mut d, document(), BUDGET);
    let mut out = Queue::uncapped();
    brief::expire(&mut d, env().now.saturating_add(Duration::from_secs(30)), &mut out);
    assert_eq!(brief::next_deadline(&d), None);
    assert!(out.is_empty());
    assert!(!d.briefs_idle());
    outputs(&mut d, Event::DropBrief { section: SECTION });
    assert!(d.briefs_idle());
}

#[test]
fn dropping_a_planned_or_busy_document_ends_its_life() {
    for busy in [false, true] {
        let mut d = setup();
        if busy {
            let owner = file_read(&mut d, document(), BUDGET);
            answer(&mut d, owner, Err(Error::Busy));
        } else {
            outputs(&mut d, Event::PlanDocument { section: SECTION, document: Box::new(document()) });
        }
        outputs(&mut d, Event::DropBrief { section: SECTION });
        d.reclaim();
        let mut out = Queue::uncapped();
        brief::retry(&mut d, &mut out);
        assert!(out.is_empty());
        assert!(d.briefs_idle());
    }
}

#[test]
fn a_document_can_use_a_repository_adopted_in_another_project() {
    let mut d = domain();
    let mut repository = adopted();
    repository.project = 9;
    repository.home = false;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    let (owner, branch) = read(&mut plan(&mut d, document(), BUDGET));
    assert_eq!(branch, Read::Branch { branch: b"main".as_slice().into() });
    answer(&mut d, owner, Err(Error::Missing));
    assert_eq!(take(&mut d).as_ref(), b"No branch main in org/repo.");
}

#[test]
fn an_ended_forges_repository_is_not_adopted_for_documents() {
    let mut d = setup();
    d.configured.remove(&REPO.forge);
    let mut out = plan(&mut d, document(), BUDGET);
    let Some(Request::BriefSized { section: SECTION, size: Ok(_) }) = out.pop() else { panic!("reason section") };
    assert!(out.is_empty());
    assert_eq!(take(&mut d).as_ref(), b"Repository org/repo on forge.example is not adopted.");
}

#[test]
fn a_complete_heading_part_needs_no_truncation_marker() {
    let mut doc = document();
    doc.heading = Some(b"B".as_slice().into());
    assert_eq!(
        gathered(doc, b"## B\nwhole part\n## C\npartial rest", BUDGET, true).as_ref(),
        with_head(b"## B\nwhole part\n").as_ref()
    );
}
