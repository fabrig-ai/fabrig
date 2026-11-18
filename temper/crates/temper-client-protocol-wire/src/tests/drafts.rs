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
fn draft_requests_round_trip_each_change_and_repository() {
    let repository = v1::DraftRepository::new(
        &v1::CEILINGS,
        v1::DraftRepositoryParts { owner: b"acme".as_slice().into(), name: b"shop".as_slice().into() },
    )
    .unwrap();
    for change in [
        ready(Some(7)),
        ready(None),
        v1::DraftChange::NotReady,
        v1::DraftChange::Started(v1::StartedDraft::new(&v1::CEILINGS, v1::StartedDraftParts { goal: 9 }).unwrap()),
    ] {
        for repository in [None, Some(repository.clone())] {
            let record = request(change.clone(), repository);
            let bytes = crate::requesting(&record);
            assert_eq!(crate::decode_requesting(&bytes, &v1::CEILINGS), Some(record));
            assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &v1::CEILINGS), None);
        }
    }
}

fn request(change: v1::DraftChange, repository: Option<v1::DraftRepository>) -> v1::Requesting {
    let value = v1::SetDraft::new(
        &v1::CEILINGS,
        v1::SetDraftParts { project: 3, repository, path: b"docs/design/draft/catch-up.md".as_slice().into(), change },
    )
    .unwrap();
    v1::Requesting::new(&v1::CEILINGS, v1::RequestingParts { request: v1::Request::SetDraft(value) }).unwrap()
}

#[test]
fn draft_answers_round_trip_the_full_row_and_each_refusal() {
    for outcome in [
        v1::Outcome::Drafted(landed()),
        refusal(v1::DraftRefusal::NoRepository),
        refusal(v1::DraftRefusal::Path),
        refusal(v1::DraftRefusal::Missing),
        refusal(v1::DraftRefusal::NotAFile),
        refusal(v1::DraftRefusal::Unreachable),
        refusal(v1::DraftRefusal::Unknown),
        refusal(v1::DraftRefusal::Full),
    ] {
        let record = answer(outcome);
        let bytes = crate::answer(&record);
        let mut reader = Reader::new(&bytes);
        assert_eq!(v1::Answer::decode(&v1::CEILINGS, &mut reader), Ok(record));
        assert!(reader.is_empty());
    }
}
fn answer(outcome: v1::Outcome) -> v1::Answer {
    v1::Answer::new(&v1::CEILINGS, v1::AnswerParts { outcome }).unwrap()
}

fn ready(chat: Option<u64>) -> v1::DraftChange {
    v1::DraftChange::Ready(v1::ReadyDraft::new(&v1::CEILINGS, v1::ReadyDraftParts { chat }).unwrap())
}
fn refusal(reason: v1::DraftRefusal) -> v1::Outcome {
    v1::Outcome::DraftRefused(v1::DraftRefused::new(&v1::CEILINGS, v1::DraftRefusedParts { reason }).unwrap())
}

#[test]
fn drafts_subject_round_trips() {
    let subject = v1::DraftsSubject::new(&v1::CEILINGS, v1::DraftsSubjectParts { project: 3 }).unwrap();
    let watch = v1::Watching::new(&v1::CEILINGS, v1::WatchingParts { watch: v1::Watch::Drafts(subject) }).unwrap();
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &v1::CEILINGS), Some(watch));
    assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &v1::CEILINGS), None);
}

#[test]
fn drafts_panels_round_trip_empty_and_full_at_every_bound() {
    for count in [0, 64] {
        let record = panel(count);
        let bytes = crate::drafts_panel(&record);
        assert_eq!(crate::decode_drafts_panel(&bytes, &v1::CEILINGS), Some(record));
        assert!(
            crate::decode_drafts_panel(&fixtures::trailing(&bytes), &v1::CEILINGS).is_none(),
            "trailing byte refused"
        );
    }
}
fn panel(count: u32) -> v1::DraftsPanel {
    let mut drafts = List::with_capacity(count);
    for _ in 0..count {
        drafts.push(landed()).unwrap();
    }
    v1::DraftsPanel::new(&v1::CEILINGS, v1::DraftsPanelParts { project: 3, drafts }).unwrap()
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
fn drafts_past_each_bound_are_refused_at_decode() {
    // Answer's version and tag, forge and four 1024-byte strings precede commit; title follows it.
    const COMMIT: usize = 2 + 1 + 2 + 4 * (4 + 1024);
    const TITLE: usize = COMMIT + 4 + 64;
    // Requesting's version, request tag, project and repository option precede the path length.
    let path =
        past_bound(&crate::requesting(&request(ready(Some(7)), None)), 2 + 1 + 4 + 1, 29, 1025, &[b'p'; 1025 - 29]);
    assert_eq!(
        v1::Requesting::decode(&v1::CEILINGS, &mut Reader::new(&path)).unwrap_err(),
        v1::Problem { path: v1::Path::SetDraftPath, reason: skein_codec::Reason::Bound },
    );
    let drafted = crate::answer(&answer(v1::Outcome::Drafted(landed())));
    for (offset, old_length, count, path) in
        [(COMMIT, 64, 65, v1::Path::LandedDraftCommit), (TITLE, 200, 201, v1::Path::LandedDraftTitle)]
    {
        let bytes = past_bound(&drafted, offset, old_length, count, b"x");
        assert_eq!(
            v1::Answer::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).unwrap_err(),
            v1::Problem { path, reason: skein_codec::Reason::Bound }
        );
    }
    let full = crate::drafts_panel(&panel(64));
    let row = drafted.get(3..).unwrap();
    let panel = past_bound(&full, 2 + 4, full.len().checked_sub(10).unwrap(), 65, row);
    assert_eq!(
        v1::DraftsPanel::decode(&v1::CEILINGS, &mut Reader::new(&panel)).unwrap_err(),
        v1::Problem { path: v1::Path::DraftsPanelDrafts, reason: skein_codec::Reason::Bound },
    );
}
