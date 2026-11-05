//! What the client keeps (temper/docs/design/client/README.md, 3.5).
use crate::{
    Address, ChatsFilter,
    pages::{
        self, Pages, Request,
        needs::{self, Act, Label, Toasted},
    },
    tests::fixtures::{self, acts as f},
};
use jig_client::{
    Key, Outcome, Told,
    contract::{self as c, ObjectKey},
};
use skein_lib::{Duration, List, Queue, Time, Wall};
const FIRST: ObjectKey = f::QUESTION;
const SECOND: ObjectKey = ObjectKey::Question { task: f::TASK, message: 4 };
const THIRD: ObjectKey = ObjectKey::Question { task: f::TASK, message: 5 };
const OTHER_PROJECT: u32 = 2;
// Session's bound, README 3.5; one past it exercises eviction/refusal.
const CAPACITY: u32 = 64;
const OVER_CAPACITY: u32 = 65;
const BEFORE_TOAST: u64 = 4499;
const TOAST_DUE: u64 = 4500;
const BEFORE_HOLD: u64 = 4999;
const HOLD_DUE: u64 = 5000;
const SECOND_CHOICE: u64 = 1000;
const SECOND_HOLD: u64 = 6000;
const CHOICE: &[u8] = b"Use the first";
fn fixture() -> Pages {
    let mut pages = f::pages(FIRST, f::question());
    pages.inbox.as_mut().unwrap().cards.get_mut(0).unwrap().group = c::InboxGroup::Questions;
    pages
}
fn add(pages: &mut Pages, object: ObjectKey, project: u32) {
    let mut card = pages.inbox().unwrap().cards.get(0).unwrap().clone();
    card.object = object;
    card.project = Some(project);
    card.words = None;
    pages.inbox.as_mut().unwrap().cards.push(card).unwrap();
}
fn event(pages: &mut Pages, millis: u64, event: pages::Event) -> Queue<Request> {
    let limits = fixtures::limits();
    let mut env = fixtures::env(&limits);
    env.now = Time::ZERO.saturating_add(Duration::from_millis(millis));
    env.wall = Wall::from_nanos(Duration::from_millis(millis).as_nanos());
    let mut out = Queue::uncapped();
    pages::step(pages, &env, crate::unlinked(&limits, fixtures::SEED).client(), event, &mut out);
    out
}
fn act(pages: &mut Pages, millis: u64, act: Act) -> Queue<Request> {
    event(pages, millis, pages::Event::Act { act: pages::Act::Needs(act) })
}
fn choose(pages: &mut Pages, millis: u64, object: ObjectKey) -> Queue<Request> {
    act(pages, millis, Act::Choose { object, words: Box::from(CHOICE) })
}
fn fire(pages: &mut Pages, millis: u64) -> Queue<Request> {
    let limits = fixtures::limits();
    let mut env = fixtures::env(&limits);
    env.now = Time::ZERO.saturating_add(Duration::from_millis(millis));
    let mut out = Queue::uncapped();
    pages::fire(pages, &env, crate::unlinked(&limits, fixtures::SEED).client(), &mut out);
    out
}
fn lines(pages: &Pages) -> Box<[&needs::Answered]> {
    pages.needs().answered(Wall::EPOCH, needs::ANSWERED_ON_NEEDS, None)
}
fn open(pages: &Pages) -> Box<[ObjectKey]> {
    let mut objects = List::with_capacity(pages.inbox().unwrap().cards.len());
    for card in &pages.needs().open(pages.inbox().unwrap()) {
        objects.push(card.object).unwrap();
    }
    objects.into_boxed()
}
fn sent(out: &mut Queue<Request>) -> (Key, c::Ask) {
    let Some(Request::Client(jig_client::Event::Send { key, name: None, request: jig_client::Ask::Jig(ask) })) =
        out.pop()
    else {
        panic!("one held answer")
    };
    assert!(out.is_empty());
    (key, *ask)
}
fn expected(object: ObjectKey, words: &[u8], project: u32) -> c::Ask {
    let ObjectKey::Question { task, message } = object else { panic!("question fixture") };
    c::Ask::Answer {
        project,
        on: c::AnswerOn::Question {
            task,
            message,
            words: c::Text::new(Box::from(words), temper_client_protocol_wire::CUE_BYTES).unwrap(),
        },
    }
}
fn repeated(count: u32, suffix: &[u8]) -> Box<[u8]> {
    let mut words = List::with_capacity(count.checked_add(u32::try_from(suffix.len()).unwrap()).unwrap());
    for _ in 0..count {
        words.push(b'a').unwrap();
    }
    for byte in suffix {
        words.push(*byte).unwrap();
    }
    words.into_boxed()
}
fn reply(pages: &mut Pages, words: &[u8]) -> Queue<Request> {
    let mut limits = fixtures::limits();
    limits.client.domain.text = temper_client_protocol_wire::CUE_BYTES;
    limits.client.domain.long_text = temper_client_protocol_wire::CUE_BYTES;
    limits.client.domain.turn_text = temper_client_protocol_wire::CUE_BYTES;
    let env = fixtures::env(&limits);
    let client = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::uncapped();
    for act in [
        pages::Act::Needs(Act::OpenReply { object: FIRST }),
        pages::Act::CardWords { object: FIRST, words: Box::from(words) },
        pages::Act::Needs(Act::SendReply { object: FIRST }),
    ] {
        pages::step(pages, &env, client.client(), pages::Event::Act { act }, &mut out);
    }
    out
}

fn chats() -> Address {
    Address::Chats { filter: ChatsFilter::All }
}
fn went(pages: &mut Pages, address: Address) {
    event(pages, 0, pages::Event::Went { address });
}
fn outcome(pages: &mut Pages, key: Key, outcome: Outcome) {
    event(pages, HOLD_DUE, pages::Event::Told { told: Told::Answered { key, outcome } });
}
fn done() -> Outcome {
    Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Amended {
        task: f::TASK,
    }))))
}
#[test]
fn choice_is_held_until_five_seconds_toast_until_four_and_half() {
    let mut pages = fixture();
    assert!(choose(&mut pages, 0, FIRST).is_empty());
    assert_eq!(open(&pages).as_ref(), []);
    assert!(pages.needs().held(FIRST));
    let answered = lines(&pages);
    assert_eq!(answered.len(), 1);
    assert_eq!(answered[0].label, Label::Choice(Box::from(CHOICE)));
    assert_eq!(answered[0].title.as_ref(), f::chip().title.bytes());
    assert_eq!(answered[0].at, Wall::EPOCH);
    assert_eq!(
        pages.needs().toast(),
        Some(&Toasted::Answered { object: FIRST, label: Label::Choice(Box::from(CHOICE)) })
    );
    assert_eq!(pages::next_deadline(&pages), Some(Time::ZERO.saturating_add(needs::TOAST)));
    assert!(fire(&mut pages, BEFORE_TOAST).is_empty());
    assert!(pages.needs().toast().is_some());
    let shown = pages::shown(&pages);
    assert!(fire(&mut pages, TOAST_DUE).is_empty());
    assert!(pages::shown(&pages) > shown);
    assert_eq!(pages.needs().toast(), None);
    assert_eq!(pages::next_deadline(&pages), Some(Time::ZERO.saturating_add(needs::HOLD)));
    assert!(fire(&mut pages, BEFORE_HOLD).is_empty());
    let (key, ask) = sent(&mut fire(&mut pages, HOLD_DUE));
    assert_eq!(key.asker, crate::owner::Owner::Pages.asker());
    assert_eq!(key.number, 1);
    assert_eq!(ask, expected(FIRST, CHOICE, fixtures::PROJECT));
    assert!(!pages.needs().held(FIRST));
    assert_eq!(lines(&pages).len(), 1);
    assert_eq!(pages::next_deadline(&pages), None);
    assert!(fire(&mut pages, SECOND_HOLD).is_empty());
}
#[test]
fn held_send_survives_navigation_without_an_inbox() {
    let mut pages = fixture();
    choose(&mut pages, 0, FIRST);
    went(&mut pages, chats());
    assert!(pages.inbox().is_none());
    assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(FIRST, CHOICE, fixtures::PROJECT));
}
#[test]
fn undo_restores_the_card_in_place_and_never_sends() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    choose(&mut pages, 0, FIRST);
    act(&mut pages, BEFORE_HOLD, Act::Undo { object: FIRST });
    assert_eq!(open(&pages).as_ref(), [FIRST, SECOND]);
    assert!(lines(&pages).is_empty());
    assert_eq!(pages.needs().toast(), None);
    assert!(fire(&mut pages, HOLD_DUE).is_empty());
    assert!(fire(&mut pages, SECOND_HOLD).is_empty());
}
#[test]
fn undo_at_the_deadline_cannot_retract_the_answer() {
    let mut pages = fixture();
    choose(&mut pages, 0, FIRST);
    act(&mut pages, HOLD_DUE, Act::Undo { object: FIRST });
    assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(FIRST, CHOICE, fixtures::PROJECT));
    act(&mut pages, SECOND_HOLD, Act::Undo { object: FIRST });
    assert_eq!(lines(&pages).len(), 1);
}
#[test]
fn reply_trims_words_and_sends_the_full_answer() {
    let mut pages = fixture();
    assert!(reply(&mut pages, b" \tFull reply\r\n").is_empty());
    assert!(!pages.needs().replying(FIRST));
    assert_eq!(pages.inbox().unwrap().cards.get(0).unwrap().words, None);
    assert_eq!(lines(&pages)[0].label, Label::Reply(Box::from(*b"Full reply")));
    assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(FIRST, b"Full reply", fixtures::PROJECT));
}
#[test]
fn reply_labels_cut_only_past_sixty_whole_characters() {
    // README 3.5: 60 whole characters, or the first 57 and an ellipsis.
    const WHOLE: u32 = 60;
    const OVER: u32 = 61;
    const PREFIX: u32 = 57;
    const BEFORE_MULTIBYTE: u32 = 56;
    for (words, label) in [
        (repeated(WHOLE, b""), repeated(WHOLE, b"")),
        (repeated(OVER, b""), repeated(PREFIX, "…".as_bytes())),
        (repeated(BEFORE_MULTIBYTE, "ébbbb".as_bytes()), repeated(BEFORE_MULTIBYTE, "é…".as_bytes())),
        (repeated(PREFIX, "ébbb".as_bytes()), repeated(PREFIX, "…".as_bytes())),
    ] {
        let mut pages = fixture();
        reply(&mut pages, &words);
        assert_eq!(lines(&pages)[0].label, Label::Reply(label));
        assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(FIRST, &words, fixtures::PROJECT));
    }
}
#[test]
fn blank_reply_keeps_the_form_and_sends_nothing() {
    let mut pages = fixture();
    assert!(reply(&mut pages, b" \t\r\n").is_empty());
    assert!(pages.needs().replying(FIRST));
    assert!(lines(&pages).is_empty());
    assert!(fire(&mut pages, HOLD_DUE).is_empty());
}
#[test]
fn undo_reply_restores_its_words_on_the_card() {
    let mut pages = fixture();
    reply(&mut pages, b"My answer");
    act(&mut pages, BEFORE_HOLD, Act::Undo { object: FIRST });
    assert_eq!(pages.inbox().unwrap().cards.get(0).unwrap().words.as_ref().unwrap().bytes(), b"My answer");
    assert!(fire(&mut pages, HOLD_DUE).is_empty());
}
#[test]
fn only_one_reply_form_opens_and_cancel_clears_its_words() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    act(&mut pages, 0, Act::OpenReply { object: FIRST });
    act(&mut pages, 0, Act::OpenReply { object: SECOND });
    assert!(!pages.needs().replying(FIRST));
    assert!(pages.needs().replying(SECOND));
    f::act(&mut pages, pages::Act::CardWords { object: SECOND, words: Box::from(*b"draft") });
    act(&mut pages, 0, Act::CancelReply);
    assert!(!pages.needs().replying(SECOND));
    assert_eq!(pages.inbox().unwrap().cards.get(1).unwrap().words, None);
}
#[test]
fn two_choices_have_separate_deadlines_and_ordered_keys() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    choose(&mut pages, 0, FIRST);
    choose(&mut pages, SECOND_CHOICE, SECOND);
    let first = sent(&mut fire(&mut pages, HOLD_DUE));
    assert_eq!(first.1, expected(FIRST, CHOICE, fixtures::PROJECT));
    assert!(pages.needs().held(SECOND));
    assert_eq!(pages::next_deadline(&pages), Some(Time::ZERO.saturating_add(Duration::from_millis(5500))));
    let second = sent(&mut fire(&mut pages, SECOND_HOLD));
    assert_eq!(second.1, expected(SECOND, CHOICE, fixtures::PROJECT));
    assert!(first.0.number < second.0.number);
}
#[test]
fn unsuccessful_outcomes_reopen_the_card_until_a_second_answer() {
    for refused in [
        Outcome::NotKept,
        Outcome::Unknown,
        Outcome::Limit,
        Outcome::Unconfirmed,
        Outcome::Held,
        Outcome::Full { held: CAPACITY, room_at: Wall::EPOCH },
        Outcome::Unsent,
        Outcome::Lost,
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None })),
    ] {
        let mut pages = fixture();
        choose(&mut pages, 0, FIRST);
        let key = sent(&mut fire(&mut pages, HOLD_DUE)).0;
        outcome(&mut pages, key, refused.clone());
        assert!(lines(&pages).is_empty());
        assert_eq!(open(&pages).as_ref(), [FIRST]);
        assert_eq!(pages.needs().refused(FIRST), Some(&refused));
        choose(&mut pages, SECOND_HOLD, FIRST);
        assert_eq!(pages.needs().refused(FIRST), None);
    }
}
#[test]
fn done_keeps_the_answered_line_and_does_not_reopen() {
    let mut pages = fixture();
    choose(&mut pages, 0, FIRST);
    let key = sent(&mut fire(&mut pages, HOLD_DUE)).0;
    outcome(&mut pages, key, done());
    assert_eq!(lines(&pages).len(), 1);
    assert_eq!(open(&pages).as_ref(), []);
    assert_eq!(pages.needs().refused(FIRST), None);
}
#[test]
fn later_moves_last_keeps_its_count_and_advances_shown() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    act(&mut pages, 0, Act::OpenReply { object: FIRST });
    act(&mut pages, 0, Act::Later { object: FIRST });
    assert_eq!(open(&pages).as_ref(), [SECOND, FIRST]);
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, SECOND);
    assert!(pages.needs().later(FIRST));
    assert!(!pages.needs().replying(FIRST));
    assert_eq!(pages.needs().toast(), Some(&Toasted::Later));
    choose(&mut pages, 0, SECOND);
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, FIRST);
    choose(&mut pages, 0, FIRST);
    assert!(!pages.needs().later(FIRST));
}
#[test]
fn open_selects_the_card_with_homes_project_filter() {
    for (address, project) in [(Address::Home { project: fixtures::PROJECT }, Some(fixtures::PROJECT)), (chats(), None)]
    {
        let mut pages = fixture();
        went(&mut pages, address);
        let mut out = act(&mut pages, 0, Act::Open { object: SECOND });
        assert_eq!(
            out.pop(),
            Some(Request::Show {
                address: Address::Inbox { filter: crate::address::Filter { project, ..crate::address::Filter::ALL } }
            })
        );
        went(&mut pages, Address::Inbox { filter: crate::address::Filter::ALL });
        let mut card = fixture().inbox().unwrap().cards.get(0).unwrap().clone();
        pages.inbox.as_mut().unwrap().cards.push(card.clone()).unwrap();
        card.object = SECOND;
        pages.inbox.as_mut().unwrap().cards.push(card).unwrap();
        assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, SECOND);
    }
}
#[test]
fn at_clears_on_answer_later_or_leaving_and_missing_falls_back() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    act(&mut pages, 0, Act::Open { object: SECOND });
    act(&mut pages, 0, Act::Later { object: SECOND });
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, FIRST);
    act(&mut pages, 0, Act::Open { object: SECOND });
    choose(&mut pages, 0, SECOND);
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, FIRST);
    act(&mut pages, 0, Act::Undo { object: SECOND });
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, FIRST);
    act(&mut pages, 0, Act::Open { object: SECOND });
    pages.inbox.as_mut().unwrap().cards.truncate(1);
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, FIRST);
    went(&mut pages, chats());
    went(&mut pages, Address::Inbox { filter: crate::address::Filter::ALL });
    let card = fixture().inbox().unwrap().cards.get(0).unwrap().clone();
    pages.inbox.as_mut().unwrap().cards.push(card).unwrap();
    add(&mut pages, SECOND, fixtures::PROJECT);
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, FIRST);
}
#[test]
fn navigation_closes_the_reply_form() {
    let mut pages = fixture();
    act(&mut pages, 0, Act::OpenReply { object: FIRST });
    went(&mut pages, chats());
    assert!(!pages.needs().replying(FIRST));
}
#[test]
fn answered_windows_exclude_the_edge_and_filter_project_in_order() {
    let mut pages = fixture();
    add(&mut pages, SECOND, OTHER_PROJECT);
    choose(&mut pages, 0, FIRST);
    choose(&mut pages, SECOND_CHOICE, SECOND);
    let mut objects = List::with_capacity(CAPACITY);
    for line in &lines(&pages) {
        objects.push(line.object).unwrap();
    }
    assert_eq!(objects.as_slice(), [FIRST, SECOND]);
    for window in [needs::ANSWERED_ON_HOME, needs::ANSWERED_ON_NEEDS] {
        let before = Wall::from_nanos(window.as_nanos() - 1);
        let edge = Wall::from_nanos(window.as_nanos());
        assert_eq!(pages.needs().answered(before, window, Some(fixtures::PROJECT)).len(), 1);
        assert!(pages.needs().answered(edge, window, Some(fixtures::PROJECT)).is_empty());
        assert_eq!(pages.needs().answered(edge, window, None)[0].object, SECOND);
    }
}
#[test]
fn signout_and_outofdate_drop_every_session_field_unsent() {
    for link in [jig_client::Link::SignedOut, jig_client::Link::OutOfDate] {
        let mut pages = fixture();
        add(&mut pages, SECOND, OTHER_PROJECT);
        add(&mut pages, THIRD, OTHER_PROJECT);
        choose(&mut pages, 0, FIRST);
        let key = sent(&mut fire(&mut pages, HOLD_DUE)).0;
        outcome(&mut pages, key, Outcome::Unsent);
        choose(&mut pages, SECOND_HOLD, SECOND);
        act(&mut pages, SECOND_HOLD, Act::Later { object: THIRD });
        act(&mut pages, SECOND_HOLD, Act::Open { object: SECOND });
        act(&mut pages, SECOND_HOLD, Act::OpenReply { object: FIRST });
        event(&mut pages, SECOND_HOLD, pages::Event::Link { link });
        assert!(lines(&pages).is_empty());
        assert!(!pages.needs().held(SECOND));
        assert!(!pages.needs().later(THIRD));
        assert!(!pages.needs().replying(FIRST));
        assert_eq!(pages.needs().refused(FIRST), None);
        assert_eq!(pages.needs().toast(), None);
        assert_eq!(pages::next_deadline(&pages), None);
        assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, FIRST);
        assert!(fire(&mut pages, SECOND_HOLD + HOLD_DUE).is_empty());
    }
}
#[test]
fn invalid_or_unoffered_cards_cannot_be_chosen() {
    let mut missing_project = fixture();
    missing_project.inbox.as_mut().unwrap().cards.get_mut(0).unwrap().project = None;
    let mut pending = fixture();
    pending.inbox.as_mut().unwrap().cards.get_mut(0).unwrap().key =
        Some(Key { asker: crate::owner::Owner::Pages.asker(), number: 1 });
    let mut task_page = fixture();
    went(&mut task_page, Address::Task { task: f::TASK });
    task_page.cards_mut().unwrap().push(fixture().inbox().unwrap().cards.get(0).unwrap().clone()).unwrap();
    let mut refused = fixture();
    let mut question = f::question();
    if let c::ReadValue::Question(question) = &mut question {
        question.acts = f::offered(c::ActKind::Answer, c::Admission::Refused(c::Refusal::NotOffered));
    }
    refused.inbox.as_mut().unwrap().cards.get_mut(0).unwrap().page = Some(question);
    for mut pages in [
        Box::new(missing_project),
        Box::new(pending),
        Box::new(task_page),
        Box::new(refused),
        Box::new(f::pages(FIRST, f::proposal(c::ActKind::Accept, c::Admission::Allowed))),
        Box::new(f::pages(ObjectKey::Proposal(f::PROPOSAL), f::question())),
    ] {
        let object = pages.cards().unwrap().get(0).unwrap().object;
        assert!(choose(&mut pages, 0, object).is_empty());
        assert!(lines(&pages).is_empty());
        assert_eq!(pages.needs().toast(), None);
        assert!(fire(&mut pages, HOLD_DUE).is_empty());
    }
}
#[test]
fn choice_words_hold_at_cue_bound_and_refuse_past_it() {
    for (length, count) in
        [(temper_client_protocol_wire::CUE_BYTES, 1), (temper_client_protocol_wire::CUE_BYTES + 1, 0)]
    {
        let mut pages = fixture();
        act(&mut pages, 0, Act::Choose { object: FIRST, words: repeated(length, b"") });
        assert_eq!(lines(&pages).len(), count);
    }
}
fn capacity_fixture() -> Pages {
    let mut pages = fixture();
    let template = pages.inbox().unwrap().cards.get(0).unwrap().clone();
    let mut cards = List::with_capacity(OVER_CAPACITY);
    for number in 0..u64::from(OVER_CAPACITY) {
        let mut card = template.clone();
        card.object = ObjectKey::Question { task: f::TASK, message: number };
        cards.push(card).unwrap();
    }
    pages.inbox.as_mut().unwrap().cards = cards;
    pages
}
fn object(number: u32) -> ObjectKey {
    ObjectKey::Question { task: f::TASK, message: u64::from(number) }
}
#[test]
fn full_answered_evicts_oldest_unheld_and_keeps_held_lines() {
    let mut pages = capacity_fixture();
    choose(&mut pages, 0, object(0));
    fire(&mut pages, HOLD_DUE);
    for number in 1..OVER_CAPACITY {
        choose(&mut pages, SECOND_HOLD, object(number));
    }
    assert_eq!(lines(&pages).len(), usize::try_from(CAPACITY).unwrap());
    assert_eq!(lines(&pages)[0].object, object(1));
    assert_eq!(open(&pages).as_ref(), [object(0)]);
    assert!(pages.needs().held(object(CAPACITY)));
}
#[test]
fn all_held_at_capacity_refuses_the_next_answer() {
    let mut pages = capacity_fixture();
    for number in 0..OVER_CAPACITY {
        choose(&mut pages, 0, object(number));
    }
    assert_eq!(lines(&pages).len(), usize::try_from(CAPACITY).unwrap());
    assert_eq!(open(&pages).as_ref(), [object(CAPACITY)]);
    assert!(!pages.needs().held(object(CAPACITY)));
}
#[test]
fn later_evicts_the_oldest_mark_at_capacity() {
    let mut pages = capacity_fixture();
    for number in 0..OVER_CAPACITY {
        act(&mut pages, 0, Act::Later { object: object(number) });
    }
    assert!(!pages.needs().later(object(0)));
    assert!(pages.needs().later(object(1)));
    assert!(pages.needs().later(object(CAPACITY)));
}
#[test]
fn refusals_evict_the_oldest_outcome_at_capacity() {
    let mut pages = capacity_fixture();
    for number in 0..OVER_CAPACITY {
        choose(&mut pages, 0, object(number));
        let key = sent(&mut fire(&mut pages, HOLD_DUE)).0;
        outcome(&mut pages, key, Outcome::Lost);
    }
    assert_eq!(pages.needs().refused(object(0)), None);
    assert_eq!(pages.needs().refused(object(1)), Some(&Outcome::Lost));
    assert_eq!(pages.needs().refused(object(CAPACITY)), Some(&Outcome::Lost));
}
#[test]
fn later_clears_the_refusal_and_the_chosen_at() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    choose(&mut pages, 0, FIRST);
    let key = sent(&mut fire(&mut pages, HOLD_DUE)).0;
    outcome(&mut pages, key, Outcome::NotKept);
    act(&mut pages, SECOND_HOLD, Act::Open { object: FIRST });
    act(&mut pages, SECOND_HOLD, Act::Later { object: FIRST });
    assert_eq!(pages.needs().refused(FIRST), None);
    assert_eq!(pages.needs().shown(pages.inbox().unwrap()).unwrap().object, SECOND);
}
#[test]
fn undo_preserves_another_cards_toast() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    choose(&mut pages, 0, FIRST);
    choose(&mut pages, SECOND_CHOICE, SECOND);
    act(&mut pages, SECOND_CHOICE, Act::Undo { object: FIRST });
    assert_eq!(
        pages.needs().toast(),
        Some(&Toasted::Answered { object: SECOND, label: Label::Choice(Box::from(CHOICE)) })
    );
}
#[test]
fn repeated_choice_joins_the_held_answer() {
    let mut pages = fixture();
    choose(&mut pages, 0, FIRST);
    choose(&mut pages, SECOND_CHOICE, FIRST);
    assert_eq!(lines(&pages).len(), 1);
    assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(FIRST, CHOICE, fixtures::PROJECT));
    assert!(fire(&mut pages, SECOND_HOLD).is_empty());
}

fn root_event(root: &mut crate::Domain, event: crate::Event) -> Queue<crate::Request> {
    super::welcome::event(root, event)
}
fn opens(out: &Queue<crate::Request>) -> List<skein_lib::Token> {
    let mut watches = List::with_capacity(out.len());
    for request in out {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) },
                },
        } = request
        {
            watches.push(*watch).unwrap();
        }
    }
    watches
}
fn standing(out: &Queue<Request>, watching: bool) -> u32 {
    let mut count = 0_u32;
    for request in out {
        if let Request::Client(event) = request {
            if watching {
                if let jig_client::Event::Watch {
                    handle,
                    subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }),
                } = event
                    && *handle == needs::handle()
                {
                    count = count.checked_add(1).unwrap();
                }
            } else if *event == (jig_client::Event::Unwatch { handle: needs::handle() }) {
                count = count.checked_add(1).unwrap();
            }
        }
    }
    count
}
fn page_event(pages: &mut Pages, client: &jig_client::Client, event: pages::Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    pages::step(pages, &fixtures::env(&fixtures::limits()), client, event, &mut out);
    out
}
fn count_root() -> (crate::Domain, skein_lib::Token) {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let me = fixtures::start(&mut root, List::with_capacity(0));
    let out = super::welcome::deliver(&mut root, me, c::Delivery::Me(Box::new(fixtures::chats::me())));
    let watches = opens(&out);
    assert_eq!(watches.len(), 1);
    (root, *watches.get(0).unwrap())
}
#[test]
fn standing_watch_opens_once_for_project_closes_on_welcome_and_signout_and_reopens() {
    let (root, _) = count_root();
    let mut pages = Pages::new();
    let client = root.client();
    let project = pages::Act::Project { project: fixtures::PROJECT };
    let welcome = Address::Welcome { step: crate::Step::Ready };
    assert_eq!(needs::handle().number, u64::MAX - 3);
    assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Link { link: jig_client::Link::Live }), true), 0);
    assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Act { act: project.clone() }), true), 1);
    assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Act { act: project.clone() }), true), 0);
    assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Went { address: welcome.clone() }), false), 1);
    assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Act { act: project.clone() }), true), 0);
    for link in [jig_client::Link::SignedOut, jig_client::Link::OutOfDate] {
        assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Went { address: chats() }), true), 1);
        assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Link { link }), false), 1);
        assert_eq!(
            standing(&page_event(&mut pages, client, pages::Event::Link { link: jig_client::Link::Live }), true),
            0
        );
        assert_eq!(standing(&page_event(&mut pages, client, pages::Event::Act { act: project.clone() }), true), 1);
        page_event(&mut pages, client, pages::Event::Went { address: welcome.clone() });
    }
}
#[test]
fn standing_count_filters_projects_subtracts_answered_keeps_later_and_reports_cut_and_end() {
    let (mut root, watch) = count_root();
    assert_eq!(root.pages().needs().count(root.client(), fixtures::PROJECT), needs::Count::default());
    let mut rows = List::with_capacity(4);
    for (object, project) in [
        (FIRST, fixtures::PROJECT),
        (SECOND, fixtures::PROJECT),
        (THIRD, OTHER_PROJECT),
        (ObjectKey::Result(f::TASK), fixtures::PROJECT),
    ] {
        rows.push(crate::tests::documents::entry(object, project, 1, Wall::EPOCH, None)).unwrap();
    }
    let inbox = crate::tests::documents::inbox(rows, 99, c::Count { value: 1, limited: false }, 0, true);
    let before = pages::shown(root.pages());
    super::welcome::deliver(&mut root, watch, c::Delivery::Inbox(Box::new(inbox)));
    assert!(pages::shown(root.pages()) > before);
    assert_eq!(root.pages().needs().count(root.client(), fixtures::PROJECT), needs::Count { count: 2, cut: true });
    let mut cards = fixture();
    add(&mut cards, SECOND, fixtures::PROJECT);
    root.pages.inbox = cards.inbox.take();
    root_event(
        &mut root,
        crate::Event::Act { action: crate::Action::Pages(pages::Act::Needs(Act::Later { object: FIRST })) },
    );
    assert_eq!(root.pages().needs().count(root.client(), fixtures::PROJECT).count, 2);
    root_event(
        &mut root,
        crate::Event::Act {
            action: crate::Action::Pages(pages::Act::Needs(Act::Choose { object: FIRST, words: Box::from(CHOICE) })),
        },
    );
    assert_eq!(root.pages().needs().count(root.client(), fixtures::PROJECT).count, 1);
    let before = pages::shown(root.pages());
    let out = root_event(
        &mut root,
        crate::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded } },
        },
    );
    assert!(pages::shown(root.pages()) > before);
    assert_eq!(root.pages().needs().count(root.client(), fixtures::PROJECT), needs::Count::default());
    assert!(opens(&out).is_empty());
}

#[test]
fn reply_past_choice_bound_is_held_whole_for_the_engine() {
    let mut pages = fixture();
    let words = repeated(temper_client_protocol_wire::CUE_BYTES.checked_add(1).unwrap(), b"");
    let mut limits = fixtures::limits();
    limits.client.domain.text = 2048;
    limits.client.domain.long_text = 2048;
    limits.client.domain.turn_text = 2048;
    let env = fixtures::env(&limits);
    let client = crate::unlinked(&limits, fixtures::SEED);
    let mut out = Queue::uncapped();
    for act in [
        pages::Act::CardWords { object: FIRST, words: words.clone() },
        pages::Act::Needs(Act::SendReply { object: FIRST }),
    ] {
        pages::step(&mut pages, &env, client.client(), pages::Event::Act { act }, &mut out);
    }
    assert!(pages.needs().held(FIRST));
    let (_, ask) = sent(&mut fire(&mut pages, HOLD_DUE));
    let c::Ask::Answer { on: c::AnswerOn::Question { words: sent, .. }, .. } = ask else { panic!("question reply") };
    assert_eq!(sent.bytes(), &*words);
}
