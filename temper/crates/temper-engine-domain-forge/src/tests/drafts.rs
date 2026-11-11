//! Landed draft rules with scripted branch and file reads.
use super::*;
use jig_domain_people::ApplicationScope;
use skein_forge::api::{Answer, Error, Op, Read};
const REQUEST: Token = Token::new(90);
const TOKEN: Token = Token::new(91);
const PROJECT: u32 = 5;
const PATH: &[u8] = b"docs/design/draft/catch-up.md";
const HEAD: [u8; 32] = [3; 32];
fn setup() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}
fn row(path: &[u8]) -> LandedDraft {
    LandedDraft {
        repository: REPO,
        path: path.into(),
        project: PROJECT,
        branch: b"main".as_slice().into(),
        commit: HEAD,
        title: b"Catch-up barrier".as_slice().into(),
        writer: 7,
        at: env().wall,
        chat: Some(7),
        goal: None,
        ready: true,
    }
}
fn set(d: &mut Domain, path: &[u8], change: DraftChange) -> Box<[Request]> {
    outputs(
        d,
        Event::SetDraft {
            request: REQUEST,
            token: TOKEN,
            party: 7,
            call: SetDraft { project: PROJECT, repository: None, path: path.into(), change },
        },
    )
}
fn begin(d: &mut Domain, path: &[u8]) {
    d.connect.answers.remove(&TOKEN);
    assert!(set(d, path, DraftChange::Ready { chat: None }).is_empty());
    assert_eq!(d.application_answer(TOKEN), None);
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"main".as_slice().into() }));
    assert!(fixtures::answer(d, call, Ok(Answer::Commit(HEAD))).is_empty());
}
fn file(d: &mut Domain, path: &[u8], result: Result<Answer, Error>) -> Box<[Request]> {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::File { head: HEAD, path: path.into(), max_bytes: 4096 }));
    fixtures::answer(d, call, result)
}
fn ready_row(d: &mut Domain, path: &[u8]) -> Box<[Request]> {
    begin(d, path);
    file(
        d,
        path,
        Ok(Answer::File {
            head: HEAD,
            path: path.into(),
            bytes: b"# Catch-up barrier\n".as_slice().into(),
            truncated: false,
        }),
    )
}
fn refused(d: &Domain, why: DraftRefusal) {
    assert_eq!(d.application_answer(TOKEN), Some(Answered::SetDraft(SetDraftAnswer::Refused(why))));
}
fn drafts(d: &Domain) -> DraftsView {
    let Some(Document::Drafts(view)) =
        d.document(drafts_subject(PROJECT), ApplicationScope::Project(PROJECT), 7, env().wall, &LIMITS)
    else {
        panic!("drafts document")
    };
    view
}
#[test]
fn reads_distinguish_files_directories_missing_and_unreachable() {
    for (result, why) in [
        (
            Ok(Answer::Directory { head: HEAD, path: PATH.into(), entries: Box::new([]), truncated: false }),
            DraftRefusal::NotAFile,
        ),
        (Err(Error::Missing), DraftRefusal::Missing),
        (Err(Error::Forbidden), DraftRefusal::Unreachable),
    ] {
        let mut d = setup();
        begin(&mut d, PATH);
        let out = file(&mut d, PATH, result);
        refused(&d, why);
        assert!(d.drafts.is_empty());
        assert_eq!(out.as_ref(), [Request::ApplicationDone { request: REQUEST, token: TOKEN }]);
    }
    let mut d = setup();
    set(&mut d, PATH, DraftChange::Ready { chat: None });
    fixtures::answer_next(&mut d, Err(Error::Missing));
    refused(&d, DraftRefusal::Missing);
    let mut d = setup();
    d.writers.remove(&1);
    outputs(&mut d, Event::Connection { forge: 1, state: Connection::NotConnected });
    set(&mut d, PATH, DraftChange::Ready { chat: None });
    refused(&d, DraftRefusal::Unreachable);
    assert!(ready(&mut d).is_empty());
}
#[test]
fn titles_use_the_first_heading_and_cut_at_a_character_boundary() {
    let ascii = format!("# {}", "b".repeat(300));
    let long = format!("# {}{}b\n# ignored", "a".repeat(199), "é".repeat(50));
    for (bytes, expected) in [
        (b"No heading\n## not level one".as_slice(), b"".as_slice()),
        (b"\xff\n# Not UTF-8".as_slice(), b"".as_slice()),
        (ascii.as_bytes(), "b".repeat(200).as_bytes()),
        (long.as_bytes(), "a".repeat(199).as_bytes()),
        (b"intro\n#  Catch-up barrier  \n# ignored".as_slice(), b"Catch-up barrier".as_slice()),
    ] {
        let mut d = setup();
        begin(&mut d, PATH);
        file(&mut d, PATH, Ok(Answer::File { head: HEAD, path: PATH.into(), bytes: bytes.into(), truncated: true }));
        assert_eq!(drafts(&d).rows[0].row.title.as_ref(), expected);
    }
}
#[test]
fn full_projects_evict_only_the_oldest_unlisted_row_and_replace_existing_paths() {
    let mut d = setup();
    for n in 0..DRAFTS {
        let mut row = row(format!("{n:02}.md").as_bytes());
        row.at = Wall::from_nanos(u64::from(n));
        outputs(&mut d, Event::Restore { record: Stored::LandedDraft(row) });
    }
    d.drafts.get_mut(&(REPO, b"62.md".as_slice().into())).unwrap().at = Wall::from_nanos(63);
    assert_eq!(drafts(&d).rows.len(), 64);
    assert_eq!(drafts(&d).rows[0].row.path.as_ref(), b"62.md");
    ready_row(&mut d, PATH);
    refused(&d, DraftRefusal::Full);
    assert_eq!(d.drafts.len(), 64);
    let out = ready_row(&mut d, b"63.md");
    assert!(matches!(d.application_answer(TOKEN), Some(Answered::SetDraft(SetDraftAnswer::Kept(_)))));
    for request in out {
        if let Request::Erase { .. } = request {
            panic!("existing path erased a row");
        }
    }
    d.drafts.get_mut(&(REPO, b"00.md".as_slice().into())).unwrap().ready = false;
    let out = ready_row(&mut d, PATH);
    assert!(
        out.contains(&Request::Erase { key: Key::LandedDraft { repository: REPO, path: b"00.md".as_slice().into() } })
    );
    let mut expected = row(PATH);
    expected.chat = None;
    assert!(out.contains(&Request::Save { record: Stored::LandedDraft(expected), release: None, read_afresh: None }));
    assert_eq!(d.drafts.len(), 64);
    assert!(!d.drafts.contains_key(&(REPO, b"00.md".as_slice().into())));
    d.drafts.get_mut(&(REPO, b"01.md".as_slice().into())).unwrap().goal = Some(40);
    d.drafts.get_mut(&(REPO, b"01.md".as_slice().into())).unwrap().at = Wall::from_nanos(0);
    d.drafts.get_mut(&(REPO, b"02.md".as_slice().into())).unwrap().ready = false;
    d.drafts.get_mut(&(REPO, b"02.md".as_slice().into())).unwrap().at = Wall::from_nanos(0);
    let out = ready_row(&mut d, b"another.md");
    assert!(
        out.contains(&Request::Erase { key: Key::LandedDraft { repository: REPO, path: b"01.md".as_slice().into() } })
    );
    assert_eq!(d.drafts.len(), 64);
    let out = ready_row(&mut d, b"03.md");
    for request in out {
        if let Request::Erase { .. } = request {
            panic!("existing path erased a row");
        }
    }
    assert!(matches!(d.application_answer(TOKEN), Some(Answered::SetDraft(SetDraftAnswer::Kept(_)))));
}
#[test]
fn unchanged_rows_answer_without_writes_and_keep_the_chat() {
    let mut d = setup();
    outputs(&mut d, Event::Restore { record: Stored::LandedDraft(row(PATH)) });
    assert_eq!(ready_row(&mut d, PATH).as_ref(), [Request::ApplicationDone { request: REQUEST, token: TOKEN }]);
    assert_eq!(drafts(&d).rows[0].row, row(PATH));
    assert_eq!(set(&mut d, PATH, DraftChange::Started { goal: 40 }).len(), 3);
    assert_eq!(set(&mut d, PATH, DraftChange::Started { goal: 40 }).len(), 1);
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test combines captured output from removal and its terminal")]
fn draft_rows_are_erased_with_their_project_and_forge() {
    for project in [true, false] {
        let mut d = setup();
        outputs(&mut d, Event::Restore { record: Stored::LandedDraft(row(PATH)) });
        let out = if project {
            outputs(&mut d, Event::ReleaseProject { project: PROJECT, request: REQUEST })
        } else {
            outputs(&mut d, Event::Restore { record: Stored::Forge(removal::record(1, Mode::Connected)) });
            let mut out = outputs(
                &mut d,
                Event::Application {
                    request: REQUEST,
                    token: TOKEN,
                    retention: Duration::from_secs(60),
                    ask: Ask::RemoveForge { forge: 1 },
                    role: Some(jig_domain_people::Role::Owner),
                    carried: None,
                },
            )
            .into_vec();
            out.extend(outputs(&mut d, Event::SecretsErased { forge: 1 }).into_vec());
            out.into_boxed_slice()
        };
        assert!(out.contains(&Request::Erase { key: Key::LandedDraft { repository: REPO, path: PATH.into() } }));
        assert!(out.contains(&Request::Marked { tasks: Box::new([drafts_subject(PROJECT)]) }));
        assert!(d.drafts.is_empty());
    }
}
#[test]
fn restart_refuses_the_sixty_fifth_draft_in_one_project() {
    let mut d = setup();
    for n in 0..DRAFTS {
        outputs(&mut d, Event::Restore { record: Stored::LandedDraft(row(format!("{n}.md").as_bytes())) });
    }
    assert!(d.restore_fit_failure().is_none());
    outputs(&mut d, Event::Restore { record: Stored::LandedDraft(row(b"0.md")) });
    assert!(d.restore_fit_failure().is_none());
    outputs(&mut d, Event::Restore { record: Stored::LandedDraft(row(PATH)) });
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::LandedDraft),
            set: RestoreSet::Rows(RecordKind::LandedDraft),
            held: 64,
            additional: 1,
            limit: 64
        })
    );
    assert_eq!(d.drafts.len(), 64);
}
