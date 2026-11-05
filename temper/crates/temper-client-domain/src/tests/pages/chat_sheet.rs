use super::{
    chat::{draft_note, notes_page},
    fixtures,
    start_goal::{self, Harness, done},
};
use crate::{
    Address,
    pages::{
        Act, Event,
        chat::sheet::{self, WayKind},
        start_goal::Failure,
    },
};
use jig_client::{Readout, Told, contract as c};
use skein_lib::Queue;

const CHAT: u64 = fixtures::chat::TASK;
const BODY: &[u8] = b"# Our draft\nKeep the work whole.\n";
// The snapshot, its goal line and provenance fit in the long-text entrance.
const LONG_TEXT: u32 = 2048;
const SNAPSHOT: &[u8] = b"a-draft\n\nImplement this draft. First commit it, as it stands, at docs/design/draft/a-draft.md.\n\n# Our draft\nKeep the work whole.\n\n\nYou are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.\nDraft: docs/design/draft/a-draft.md\nChat: T9";

fn with_document(me: c::Me, body: &[u8]) -> Harness {
    let mut h = Harness::new(me);
    h.limits.client.domain.long_text = LONG_TEXT;
    h.event(Event::Went { address: Address::Chat { task: CHAT } });
    let handle = h.pages.chat().unwrap().document;
    h.event(Event::Told {
        told: Told::Page {
            handle,
            page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(fixtures::chat::task())))),
        },
    });
    let handle = jig_client::Handle { number: handle.number.checked_add(2).unwrap(), ..handle };
    h.event(Event::Told { told: Told::Page { handle, page: notes_page(&[draft_note(1, 1, body)]) } });
    h.out = Queue::uncapped();
    h.act(Act::OpenStart { draft: Some(b"ignored.md".as_slice().into()), chat: Some(CHAT) });
    h
}

#[test]
fn words_are_exact() {
    assert_eq!(sheet::words(b"Away", b"away-mode", b"# Away\nOne.", WayKind::TaskByTask).as_ref(), b"Away\n\nImplement this draft. First commit it, as it stands, at docs/design/draft/away-mode.md.\n\n# Away\nOne.\n\nYou are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.");
}

#[test]
fn agreed_adds_its_line() {
    assert_eq!(sheet::words(b"Away", b"away-mode", b"# Away\nOne.", WayKind::Agreed).as_ref(), b"Away\n\nImplement this draft. First commit it, as it stands, at docs/design/draft/away-mode.md.\n\n# Away\nOne.\n\nYou are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.\nThe work is to be agreed: before anything is changed, ask your requester your plan with message, as a question, and wait for the answer.");
}

#[test]
fn the_shared_sheet_takes_a_snapshot_and_provenance() {
    let mut h = with_document(start_goal::me(), BODY);
    let start = h.pages.start().unwrap();
    assert_eq!(start.name.as_ref(), b"a-draft");
    assert_eq!(start.draft.as_ref(), b"docs/design/draft/a-draft.md");
    assert_eq!(start.body.as_deref(), Some(BODY));
    // A read after the sheet opened cannot replace the snapshot it sends.
    let document = h.pages.chat().unwrap().document;
    let handle = jig_client::Handle { number: document.number.checked_add(2).unwrap(), ..document };
    h.event(Event::Told { told: Told::Page { handle, page: notes_page(&[draft_note(1, 2, b"# Changed")]) } });
    h.act(Act::StartGoal);
    let (_, ask) = h.sent();
    let c::Ask::Create { tasks, .. } = ask else { panic!("create from document") };
    let spec = tasks.get(0).unwrap().spec.words.bytes();
    assert_eq!(spec, SNAPSHOT);
}

#[test]
fn over_long_text_is_refused_in_place() {
    for fits in [false, true] {
        let length = u32::try_from(SNAPSHOT.len()).unwrap();
        let mut h = with_document(start_goal::me(), BODY);
        h.limits.client.domain.long_text = if fits { length } else { length.checked_sub(1).unwrap() };
        h.act(Act::StartGoal);
        let start = h.pages.start().unwrap();
        assert_eq!(start.pending.is_some(), fits);
        if !fits {
            let Some(Failure::TooLong) = start.error else { panic!("long-text refusal") };
            assert!(h.out.is_empty());
            assert_eq!(start.body.as_deref(), Some(BODY));
        }
    }
}

#[test]
fn propose_closes_with_the_line() {
    let to = c::Recipient::Party(fixtures::PARTY);
    let me = start_goal::beyond(start_goal::me(), c::Beyond::Proposed { to: to.clone() });
    let mut h = with_document(me, BODY);
    h.act(Act::StartGoal);
    let (key, _) = h.sent();
    h.answer(key, done(c::DocumentOutcome::Created { first: 42, last: 42, given: None }));
    assert!(h.pages.start().is_none());
    assert_eq!(h.pages.chat().unwrap().proposed, Some(to));
    assert!(h.out.is_empty());
}

#[test]
fn no_creating_or_empty_document_opens_no_sheet() {
    let mut me = start_goal::me();
    me.projects.get_mut(0).unwrap().creating = None;
    for (me, body) in [(me, BODY), (start_goal::me(), b"".as_slice())] {
        let h = with_document(me, body);
        assert!(h.pages.start().is_none());
        assert!(h.out.is_empty());
    }
}

#[test]
fn a_snapshot_keeps_the_asking_choice() {
    for always in [false, true] {
        let mut h = with_document(start_goal::me(), BODY);
        h.act(Act::StartAsks { always });
        h.act(Act::StartGoal);
        let (_, ask) = h.sent();
        let c::Ask::Create { tasks, .. } = ask else { panic!("create from document") };
        let spec = tasks.get(0).unwrap().spec.words.bytes();
        if always {
            assert_eq!(spec.strip_suffix(b"\nAsks: always").expect("the snapshot asks the person"), SNAPSHOT);
        } else {
            assert_eq!(spec, SNAPSHOT);
        }
    }
}
