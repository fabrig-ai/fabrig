#![expect(clippy::disallowed_types, clippy::disallowed_macros, reason = "tests collect and compare boundary actions")]
use super::fixtures;
use crate::{
    Action, Domain, Event, Place, Request,
    approve::{self, Act, State},
};
use jig_client::{Approval, ApprovalAct, ApprovalCode, ApprovalState as Read, Decided};
use skein_lib::{Duration, Queue, Wall};
fn code() -> ApprovalCode {
    ApprovalCode::parse(b"WDJBMJHT").unwrap()
}
fn step(root: &mut Domain, event: Event) -> Vec<ApprovalAct> {
    let mut out = Queue::uncapped();
    crate::step(root, &fixtures::env(&fixtures::limits()), event, &mut out);
    acts(&mut out)
}
fn acts(out: &mut Queue<Request>) -> Vec<ApprovalAct> {
    let mut acts = Vec::new();
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::ApprovalAct(act) } = request {
            acts.push(act);
        }
    }
    acts
}
fn open(root: &mut Domain, code: Option<ApprovalCode>) -> Vec<ApprovalAct> {
    step(root, Event::Went { place: Place::Approve { code } })
}
fn act(root: &mut Domain, act: Act) -> Vec<ApprovalAct> {
    step(root, Event::Act { action: Action::Approve(act) })
}
fn answer(root: &mut Domain, answer: Approval) -> Vec<ApprovalAct> {
    step(root, Event::Client { event: jig_client::Event::ApprovalAnswer { answer } })
}
fn read(root: &mut Domain, state: Read) -> Vec<ApprovalAct> {
    answer(root, Approval::Read { code: code(), state })
}
fn state(root: &Domain) -> State {
    root.approval().expect("approval page").state()
}
fn reading() -> Domain {
    let mut root = fixtures::on_adoption_page();
    open(&mut root, Some(code()));
    root
}
fn asking() -> Domain {
    let mut root = reading();
    read(&mut root, Read::Waiting { asked: Wall::EPOCH });
    root
}
#[test]
fn opening_the_page_reads_once_and_approves_nothing() {
    let mut root = fixtures::on_adoption_page();
    assert_eq!(open(&mut root, Some(code())), vec![ApprovalAct::Read { code: code() }]);
    assert_eq!(state(&root), State::Reading);
    let mut out = Queue::uncapped();
    crate::fire(&mut root, &fixtures::env(&fixtures::limits()), &mut out);
    assert!(acts(&mut out).is_empty());
    assert!(act(&mut root, Act::Approve).is_empty());
    assert!(open(&mut root, None).is_empty());
    assert_eq!(state(&root), State::Typing { unread: false });
}
#[test]
fn each_answer_draws_its_state() {
    let asked = Wall::from_nanos(123);
    for (answer, expected) in [
        (Read::Waiting { asked }, State::Asking { asked }),
        (Read::Approved, State::Approved),
        (Read::Taken, State::SignedIn),
        (Read::Refused, State::Refused),
        (Read::Expired, State::Expired),
        (Read::Unknown, State::Unknown),
        (Read::NotABrowser, State::SignedOut),
        (Read::SignedOut, State::SignedOut),
        (Read::NotReady, State::Busy),
        (Read::Busy, State::Busy),
        (Read::Lost, State::Busy),
    ] {
        let mut root = reading();
        let shown = crate::shown(&root);
        assert!(read(&mut root, answer).is_empty());
        assert_eq!(state(&root), expected, "{answer:?}");
        assert!(crate::shown(&root) > shown);
        assert_eq!(approve::next_deadline(&root).is_some(), expected == State::Approved);
    }
    for answer in [Read::NotReady, Read::Busy, Read::Lost] {
        let mut root = reading();
        read(&mut root, Read::Approved);
        let mut env = fixtures::env(&fixtures::limits());
        env.now = env.now.saturating_add(approve::AGAIN);
        crate::fire(&mut root, &env, &mut Queue::uncapped());
        read(&mut root, answer);
        assert_eq!(state(&root), State::Approved, "{answer:?}");
        assert!(approve::next_deadline(&root).is_some());
    }
    for (decided, expected) in [
        (Decided::Approved, State::Approved),
        (Decided::Refused, State::Refused),
        (Decided::Used, State::Used),
        (Decided::Expired, State::Expired),
        (Decided::Unknown, State::Unknown),
        (Decided::NotABrowser, State::SignedOut),
        (Decided::SignedOut, State::SignedOut),
        (Decided::NotReady, State::Busy),
        (Decided::Busy, State::Busy),
        (Decided::Lost, State::Busy),
    ] {
        let mut root = asking();
        act(&mut root, Act::Approve);
        assert!(answer(&mut root, Approval::Decided { code: code(), decided }).is_empty());
        assert_eq!(state(&root), expected, "{decided:?}");
        assert_eq!(approve::next_deadline(&root).is_some(), expected == State::Approved);
    }
}
#[test]
fn a_decision_is_sent_once() {
    for (decision, approve) in [(Act::Approve, true), (Act::Refuse, false)] {
        let mut root = asking();
        assert_eq!(act(&mut root, decision), vec![ApprovalAct::Decide { code: code(), approve }]);
        assert_eq!(state(&root), State::Deciding { approve, asked: Wall::EPOCH });
        assert!(act(&mut root, Act::Approve).is_empty());
        assert!(act(&mut root, Act::Refuse).is_empty());
        let mut out = Queue::uncapped();
        crate::fire(&mut root, &fixtures::env(&fixtures::limits()), &mut out);
        assert!(acts(&mut out).is_empty());
    }
}
#[test]
fn while_approved_it_reads_again_each_second_until_another_answer() {
    let mut root = reading();
    read(&mut root, Read::Approved);
    let mut env = fixtures::env(&fixtures::limits());
    let deadline = env.now.saturating_add(approve::AGAIN);
    assert_eq!(crate::next_deadline(&root), Some(deadline));
    let mut out = Queue::uncapped();
    env.now = env.now.saturating_add(Duration::from_millis(999));
    crate::fire(&mut root, &env, &mut out);
    assert!(acts(&mut out).is_empty());
    env.now = deadline;
    crate::fire(&mut root, &env, &mut out);
    assert_eq!(acts(&mut out), vec![ApprovalAct::Read { code: code() }]);
    crate::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::ApprovalAnswer { answer: Approval::Read { code: code(), state: Read::Busy } },
        },
        &mut out,
    );
    assert_eq!(state(&root), State::Approved);
    assert_eq!(crate::next_deadline(&root), Some(deadline.saturating_add(approve::AGAIN)));
    env.now = deadline.saturating_add(approve::AGAIN);
    crate::fire(&mut root, &env, &mut out);
    assert_eq!(acts(&mut out), vec![ApprovalAct::Read { code: code() }]);
    read(&mut root, Read::Taken);
    assert_eq!(state(&root), State::SignedIn);
    assert_eq!(approve::next_deadline(&root), None);
    crate::fire(&mut root, &env, &mut out);
    assert!(acts(&mut out).is_empty());
}
#[test]
fn a_typed_code_reads_or_stays_with_its_alert() {
    for (typed, reads) in [
        (b"wdjb mjht".as_slice(), true),
        (b"WDJB-MJHT", true),
        (b"WDJB-MJH", false),
        (b"AEIOUAEI", false),
        (b"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB", false),
        (b"WDJBMJHT                         ", false),
        (b"", false),
    ] {
        let mut root = fixtures::on_adoption_page();
        open(&mut root, None);
        assert!(act(&mut root, Act::Words { words: typed.into() }).is_empty());
        let out = act(&mut root, Act::Check);
        if reads {
            assert_eq!(out, vec![ApprovalAct::Read { code: code() }]);
            assert_eq!(state(&root), State::Reading);
        } else {
            assert!(out.is_empty());
            assert_eq!(state(&root), State::Typing { unread: true });
        }
        assert_eq!(
            root.approval().unwrap().words(),
            &typed[..typed.len().min(jig_client_protocol_wire::CODE_TYPED_BYTES)]
        );
    }
}
#[test]
fn an_answer_for_another_code_or_a_page_left_changes_nothing() {
    let mut root = reading();
    let shown = crate::shown(&root);
    assert!(
        answer(&mut root, Approval::Read { code: ApprovalCode::parse(b"BCDFGHJK").unwrap(), state: Read::Approved })
            .is_empty()
    );
    assert_eq!(state(&root), State::Reading);
    assert_eq!(crate::shown(&root), shown);
    // The unrelated answer must not release the in-flight read.
    let mut out = Queue::uncapped();
    crate::fire(&mut root, &fixtures::env(&fixtures::limits()), &mut out);
    assert!(acts(&mut out).is_empty());
    step(&mut root, Event::Went { place: Place::Address(crate::Address::Accounts) });
    for answer_value in [
        Approval::Read { code: code(), state: Read::Approved },
        Approval::Decided { code: code(), decided: Decided::Approved },
    ] {
        let shown = crate::shown(&root);
        assert!(answer(&mut root, answer_value).is_empty());
        assert!(root.approval().is_none());
        assert_eq!(crate::shown(&root), shown);
    }
}
#[test]
fn signed_out_draws_sign_in_first() {
    let mut root = fixtures::on_adoption_page();
    step(&mut root, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } });
    assert!(open(&mut root, Some(code())).is_empty());
    assert_eq!(state(&root), State::SignedOut);
    let mut root = reading();
    assert!(read(&mut root, Read::SignedOut).is_empty());
    assert_eq!(state(&root), State::SignedOut);
}
#[test]
fn try_again_reads_and_type_a_code_returns_to_the_field() {
    let mut root = reading();
    read(&mut root, Read::Busy);
    assert_eq!(act(&mut root, Act::Again), vec![ApprovalAct::Read { code: code() }]);
    read(&mut root, Read::Unknown);
    assert!(act(&mut root, Act::Type).is_empty());
    assert_eq!(state(&root), State::Typing { unread: false });
    assert_eq!(root.approval().unwrap().code(), None);
    assert!(root.approval().unwrap().words().is_empty());
}
