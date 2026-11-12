//! Landed drafts' wire forms and bounds (temper/docs/design/client/forge/records.md).
use super::{fixtures, v1};
use alloc::boxed::Box;
use skein_lib::{List, Reader};

fn landed() -> v1::LandedDraft {
    v1::LandedDraft::new(
        &v1::CEILINGS,
        v1::LandedDraftParts {
            forge: u16::MAX,
            owner: Box::new([b'o'; 1024]),
            name: Box::new([b'n'; 1024]),
            path: Box::new([b'p'; 1024]),
            branch: Box::new([b'b'; 1024]),
            commit: Box::new([b'c'; 64]),
            title: Box::new([b't'; 200]),
            writer: u64::MAX,
            at: u64::MAX,
            chat: Some(7),
            goal: Some(9),
            ready: true,
        },
    )
    .expect("each byte field at its bound")
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn draft_requests_round_trip_each_change_and_repository() {
    let repository = v1::DraftRepository::new(
        &v1::CEILINGS,
        v1::DraftRepositoryParts { owner: b"acme".as_slice().into(), name: b"shop".as_slice().into() },
    )
    .unwrap();
    for (change, home, named) in [
        (
            ready(Some(7)),
            include_bytes!("../../golden/draft_request_ready_chat_home.bin").as_slice(),
            include_bytes!("../../golden/draft_request_ready_chat_repository.bin").as_slice(),
        ),
        (
            ready(None),
            include_bytes!("../../golden/draft_request_ready_home.bin").as_slice(),
            include_bytes!("../../golden/draft_request_ready_repository.bin").as_slice(),
        ),
        (
            v1::DraftChange::NotReady,
            include_bytes!("../../golden/draft_request_not_ready_home.bin").as_slice(),
            include_bytes!("../../golden/draft_request_not_ready_repository.bin").as_slice(),
        ),
        (
            v1::DraftChange::Started(v1::StartedDraft::new(&v1::CEILINGS, v1::StartedDraftParts { goal: 9 }).unwrap()),
            include_bytes!("../../golden/draft_request_started_home.bin").as_slice(),
            include_bytes!("../../golden/draft_request_started_repository.bin").as_slice(),
        ),
    ] {
        for (repository, golden) in [(None, home), (Some(repository.clone()), named)] {
            let value = v1::SetDraft::new(
                &v1::CEILINGS,
                v1::SetDraftParts {
                    project: 3,
                    repository,
                    path: b"docs/design/draft/catch-up.md".as_slice().into(),
                    change: change.clone(),
                },
            )
            .unwrap();
            let record =
                v1::Requesting::new(&v1::CEILINGS, v1::RequestingParts { request: v1::Request::SetDraft(value) })
                    .unwrap();
            assert_eq!(crate::requesting(&record).as_ref(), golden);
            assert_eq!(crate::decode_requesting(golden, &v1::CEILINGS), Some(record));
            assert_eq!(crate::decode_requesting(&fixtures::trailing(golden), &v1::CEILINGS), None);
        }
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn draft_answers_round_trip_the_full_row_and_each_refusal() {
    for (outcome, golden) in [
        (v1::Outcome::Drafted(landed()), include_bytes!("../../golden/draft_answer_drafted.bin").as_slice()),
        (
            refusal(v1::DraftRefusal::NoRepository),
            include_bytes!("../../golden/draft_answer_no_repository.bin").as_slice(),
        ),
        (refusal(v1::DraftRefusal::Path), include_bytes!("../../golden/draft_answer_path.bin").as_slice()),
        (refusal(v1::DraftRefusal::Missing), include_bytes!("../../golden/draft_answer_missing.bin").as_slice()),
        (refusal(v1::DraftRefusal::NotAFile), include_bytes!("../../golden/draft_answer_not_a_file.bin").as_slice()),
        (
            refusal(v1::DraftRefusal::Unreachable),
            include_bytes!("../../golden/draft_answer_unreachable.bin").as_slice(),
        ),
        (refusal(v1::DraftRefusal::Unknown), include_bytes!("../../golden/draft_answer_unknown.bin").as_slice()),
        (refusal(v1::DraftRefusal::Full), include_bytes!("../../golden/draft_answer_full.bin").as_slice()),
    ] {
        let record = v1::Answer::new(&v1::CEILINGS, v1::AnswerParts { outcome }).unwrap();
        assert!(crate::answer(&record).as_ref() == golden, "draft outcome's golden bytes");
        let mut reader = Reader::new(golden);
        assert_eq!(v1::Answer::decode(&v1::CEILINGS, &mut reader), Ok(record));
        assert!(reader.is_empty());
    }
}
fn ready(chat: Option<u64>) -> v1::DraftChange {
    v1::DraftChange::Ready(v1::ReadyDraft::new(&v1::CEILINGS, v1::ReadyDraftParts { chat }).unwrap())
}
fn refusal(reason: v1::DraftRefusal) -> v1::Outcome {
    v1::Outcome::DraftRefused(v1::DraftRefused::new(&v1::CEILINGS, v1::DraftRefusedParts { reason }).unwrap())
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn drafts_subject_round_trips() {
    let subject = v1::DraftsSubject::new(&v1::CEILINGS, v1::DraftsSubjectParts { project: 3 }).unwrap();
    let watch = v1::Watching::new(&v1::CEILINGS, v1::WatchingParts { watch: v1::Watch::Drafts(subject) }).unwrap();
    let golden = include_bytes!("../../golden/draft_watch.bin");
    assert_eq!(crate::watching(&watch).as_ref(), golden);
    assert_eq!(crate::decode_watching(golden, &v1::CEILINGS), Some(watch));
    assert_eq!(crate::decode_watching(&fixtures::trailing(golden), &v1::CEILINGS), None);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn drafts_panels_round_trip_empty_and_full_at_every_bound() {
    for (count, golden) in [
        (0, include_bytes!("../../golden/draft_panel_smallest.bin").as_slice()),
        (64, include_bytes!("../../golden/draft_panel_full.bin").as_slice()),
    ] {
        let mut drafts = List::with_capacity(count);
        for _ in 0..count {
            drafts.push(landed()).unwrap();
        }
        let record = v1::DraftsPanel::new(&v1::CEILINGS, v1::DraftsPanelParts { project: 3, drafts }).unwrap();
        assert!(crate::drafts_panel(&record).as_ref() == golden, "drafts panel's golden bytes");
        assert_eq!(crate::decode_drafts_panel(golden, &v1::CEILINGS), Some(record));
        assert!(
            crate::decode_drafts_panel(&fixtures::trailing(golden), &v1::CEILINGS).is_none(),
            "trailing byte refused"
        );
    }
}

fn past_bound(bytes: &[u8], offset: usize, old_length: usize, count: u32, extra: &[u8]) -> Box<[u8]> {
    let field = offset.checked_add(4).unwrap();
    let end = field.checked_add(old_length).unwrap();
    let mut writer = skein_lib::Writer::new(bytes.len().checked_add(extra.len()).unwrap());
    writer.put(bytes.get(..offset).unwrap()).unwrap();
    writer.put(&count.to_be_bytes()).unwrap();
    writer.put(bytes.get(field..end).unwrap()).unwrap();
    writer.put(extra).unwrap();
    writer.put(bytes.get(end..).unwrap()).unwrap();
    writer.finish()
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn drafts_past_each_bound_are_refused_at_decode() {
    // Answer's version and tag, forge and four 1024-byte strings precede commit; title follows it.
    const COMMIT: usize = 2 + 1 + 2 + 4 * (4 + 1024);
    const TITLE: usize = COMMIT + 4 + 64;
    // Requesting's version, request tag, project and repository option precede the path length.
    let path = past_bound(
        include_bytes!("../../golden/draft_request_ready_chat_home.bin"),
        2 + 1 + 4 + 1,
        29,
        1025,
        &[b'p'; 1025 - 29],
    );
    assert_eq!(
        v1::Requesting::decode(&v1::CEILINGS, &mut Reader::new(&path)).unwrap_err(),
        v1::Problem { path: v1::Path::SetDraftPath, reason: skein_codec::Reason::Bound },
    );
    for (offset, old_length, count, path) in
        [(COMMIT, 64, 65, v1::Path::LandedDraftCommit), (TITLE, 200, 201, v1::Path::LandedDraftTitle)]
    {
        let bytes =
            past_bound(include_bytes!("../../golden/draft_answer_drafted.bin"), offset, old_length, count, b"x");
        assert_eq!(
            v1::Answer::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).unwrap_err(),
            v1::Problem { path, reason: skein_codec::Reason::Bound }
        );
    }
    let full = include_bytes!("../../golden/draft_panel_full.bin");
    let row = include_bytes!("../../golden/draft_answer_drafted.bin").get(3..).unwrap();
    let panel = past_bound(full, 2 + 4, full.len().checked_sub(10).unwrap(), 65, row);
    assert_eq!(
        v1::DraftsPanel::decode(&v1::CEILINGS, &mut Reader::new(&panel)).unwrap_err(),
        v1::Problem { path: v1::Path::DraftsPanelDrafts, reason: skein_codec::Reason::Bound },
    );
}
