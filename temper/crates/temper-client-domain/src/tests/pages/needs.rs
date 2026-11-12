//! What the client keeps (temper/docs/design/client/pages/README.md).
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
    let (mut root, watch) = fixtures::inbox_home(crate::address::Filter::ALL);
    fixtures::inbox_entries(&mut root, watch, List::with_capacity(0));
    let mut inbox = root.pages.inbox.take().unwrap();
    inbox.cards = pages.inbox.take().unwrap().cards;
    pages.inbox = Some(inbox);
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
fn answer_fixtures() -> [(Box<Pages>, ObjectKey); 2] {
    let mut person = fixture();
    let object = ObjectKey::PersonTask(f::TASK);
    add(&mut person, object, fixtures::PROJECT);
    let card = person.inbox.as_mut().unwrap().cards.get_mut(1).unwrap();
    card.page = Some(f::person(c::ActKind::Answer));
    card.group = c::InboxGroup::PersonTasks;
    [(Box::new(fixture()), FIRST), (Box::new(person), object)]
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
    let words = c::Text::new(Box::from(words), temper_client_protocol_wire::CUE_BYTES).unwrap();
    let on = match object {
        ObjectKey::Question { task, message } => c::AnswerOn::Question { task, message, words },
        ObjectKey::PersonTask(task) => {
            c::AnswerOn::Person { task, revision: f::REVISION, answer: c::PersonAnswer::Words(words) }
        }
        ObjectKey::Task(_)
        | ObjectKey::Proposal(_)
        | ObjectKey::Escalation(_)
        | ObjectKey::Result(_)
        | ObjectKey::Note { .. } => panic!("answer fixture"),
    };
    c::Ask::Answer { project, on }
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
    reply_to(pages, FIRST, words)
}
fn frame(pages: &mut Pages, env: &skein_lib::Env<crate::Limits>, client: &jig_client::Client) {
    pages::step(
        pages,
        env,
        client,
        pages::Event::Told { told: Told::Changed { handle: pages::me_handle() } },
        &mut Queue::uncapped(),
    );
}
fn reply_to(pages: &mut Pages, object: ObjectKey, words: &[u8]) -> Queue<Request> {
    let mut limits = fixtures::limits();
    limits.client.domain.text = temper_client_protocol_wire::CUE_BYTES;
    limits.client.domain.long_text = temper_client_protocol_wire::CUE_BYTES;
    limits.client.domain.turn_text = temper_client_protocol_wire::CUE_BYTES;
    let env = fixtures::env(&limits);
    let client = super::composer::root(&limits);
    frame(pages, &env, client.client());
    let mut out = Queue::uncapped();
    for act in [
        pages::Act::Needs(Act::OpenReply { object }),
        pages::Act::CardWords { object, words: Box::from(words) },
        pages::Act::Needs(Act::SendReply { object }),
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
    for (mut pages, object) in answer_fixtures() {
        choose(&mut pages, 0, object);
        for card in pages.inbox.as_mut().unwrap().cards.iter_mut() {
            if let Some(c::ReadValue::PersonTask(task)) = &mut card.page {
                task.revision = f::NEW_REVISION;
            }
        }
        went(&mut pages, chats());
        assert!(pages.inbox().is_none());
        assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(object, CHOICE, fixtures::PROJECT));
    }
}

#[test]
fn undo_restores_the_card_in_place_and_never_sends() {
    for (mut pages, object) in answer_fixtures() {
        add(&mut pages, SECOND, fixtures::PROJECT);
        choose(&mut pages, 0, object);
        assert!(pages.needs().held(object));
        act(&mut pages, BEFORE_HOLD, Act::Undo { object });
        assert!(open(&pages).contains(&object));
        assert!(open(&pages).contains(&SECOND));
        assert!(lines(&pages).is_empty());
        assert_eq!(pages.needs().toast(), None);
        assert!(fire(&mut pages, HOLD_DUE).is_empty());
        assert!(fire(&mut pages, SECOND_HOLD).is_empty());
    }
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
    for (mut pages, object) in answer_fixtures() {
        assert!(reply_to(&mut pages, object, b" \tFull reply\r\n").is_empty());
        assert!(!pages.needs().replying(object));
        assert_eq!(pages.inbox().unwrap().cards.get(u32::from(object != FIRST)).unwrap().words, None);
        assert_eq!(lines(&pages)[0].title.as_ref(), f::chip().title.bytes());
        assert_eq!(lines(&pages)[0].label, Label::Reply(Box::from(*b"Full reply")));
        assert_eq!(
            pages.needs().toast(),
            Some(&Toasted::Answered { object, label: Label::Reply(Box::from(*b"Full reply")) })
        );
        for card in pages.inbox.as_mut().unwrap().cards.iter_mut() {
            if let Some(c::ReadValue::PersonTask(task)) = &mut card.page {
                task.revision = f::NEW_REVISION;
            }
        }
        went(&mut pages, chats());
        assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(object, b"Full reply", fixtures::PROJECT));
    }
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
        for (mut pages, object) in answer_fixtures() {
            choose(&mut pages, 0, object);
            let key = sent(&mut fire(&mut pages, HOLD_DUE)).0;
            outcome(&mut pages, key, refused.clone());
            assert!(lines(&pages).is_empty());
            assert!(open(&pages).contains(&object));
            assert_eq!(pages.needs().refused(object), Some(&refused));
            choose(&mut pages, SECOND_HOLD, object);
            assert_eq!(pages.needs().refused(object), None);
        }
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
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        SECOND
    );
    assert!(pages.needs().later(FIRST));
    assert!(!pages.needs().replying(FIRST));
    assert_eq!(pages.needs().toast(), Some(&Toasted::Later));
    choose(&mut pages, 0, SECOND);
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        FIRST
    );
    choose(&mut pages, 0, FIRST);
    assert!(!pages.needs().later(FIRST));
}
#[test]
fn open_selects_the_card_with_homes_project_filter() {
    for (address, show) in [
        (Address::Home { project: fixtures::PROJECT }, Some(Some(fixtures::PROJECT))),
        (chats(), Some(None)),
        (Address::Inbox { filter: crate::address::Filter::ALL }, None),
        (
            Address::Inbox {
                filter: crate::address::Filter { project: Some(fixtures::PROJECT), ..crate::address::Filter::ALL },
            },
            None,
        ),
    ] {
        let mut pages = fixture();
        went(&mut pages, address.clone());
        let mut out = act(&mut pages, 0, Act::Open { object: SECOND });
        assert_eq!(
            out.pop(),
            match show {
                Some(project) => Some(Request::Show {
                    address: Address::Inbox {
                        filter: crate::address::Filter { project, ..crate::address::Filter::ALL }
                    }
                }),
                None => None,
            }
        );
        assert!(out.is_empty());
        assert_eq!(pages.address(), &address);
        went(&mut pages, Address::Inbox { filter: crate::address::Filter::ALL });
        pages.inbox = fixture().inbox;
        add(&mut pages, SECOND, fixtures::PROJECT);
        assert_eq!(
            pages
                .needs()
                .shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default())
                .unwrap()
                .object(),
            SECOND
        );
    }
}
#[test]
fn at_clears_on_answer_later_or_leaving_and_missing_falls_back() {
    let mut pages = fixture();
    add(&mut pages, SECOND, fixtures::PROJECT);
    act(&mut pages, 0, Act::Open { object: SECOND });
    act(&mut pages, 0, Act::Later { object: SECOND });
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        FIRST
    );
    act(&mut pages, 0, Act::Open { object: SECOND });
    choose(&mut pages, 0, SECOND);
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        FIRST
    );
    act(&mut pages, 0, Act::Undo { object: SECOND });
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        FIRST
    );
    act(&mut pages, 0, Act::Open { object: SECOND });
    pages.inbox.as_mut().unwrap().cards.truncate(1);
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        FIRST
    );
    went(&mut pages, chats());
    went(&mut pages, Address::Inbox { filter: crate::address::Filter::ALL });
    pages.inbox = fixture().inbox;
    add(&mut pages, SECOND, fixtures::PROJECT);
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        FIRST
    );
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
        assert_eq!(
            pages
                .needs()
                .shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default())
                .unwrap()
                .object(),
            FIRST
        );
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
    let mut role = f::person(c::ActKind::Answer);
    if let c::ReadValue::PersonTask(task) = &mut role {
        task.address = c::PersonAddress::Role(0);
    }
    for mut pages in [
        Box::new(f::pages(ObjectKey::PersonTask(f::TASK), role)),
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
    assert_eq!(
        pages.needs().shown(pages.inbox().unwrap(), &crate::forge::approvals::Approvals::default()).unwrap().object(),
        SECOND
    );
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
    let (root, inbox, _) = approval_root();
    (root, inbox)
}
fn approval_root() -> (crate::Domain, skein_lib::Token, skein_lib::Token) {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let me = fixtures::start(&mut root, List::with_capacity(0));
    let out = super::welcome::deliver(&mut root, me, c::Delivery::Me(Box::new(fixtures::chats::me())));
    let watches = opens(&out);
    assert_eq!(watches.len(), 1);
    let approvals = crate::tests::approvals::opening(&out).expect("session approvals");
    (root, *watches.get(0).unwrap(), approvals)
}
const NO_GOAL_CHANGE: u64 = 83;
type ApprovalCase = (
    &'static str,
    crate::address::Filter,
    &'static [ObjectKey],
    bool,
    Option<ObjectKey>,
    Option<ObjectKey>,
    &'static [ObjectKey],
    Option<ObjectKey>,
);
fn approval_cases() -> &'static [ApprovalCase] {
    use crate::{
        address::{Filter, Kind},
        tests::approvals as a,
    };
    const APPROVAL: ObjectKey = ObjectKey::Task(a::CHANGE);
    const OTHER_APPROVAL: ObjectKey = ObjectKey::Task(a::OTHER_CHANGE);
    const NO_GOAL: ObjectKey = ObjectKey::Task(NO_GOAL_CHANGE);
    &[
        (
            "all",
            Filter::ALL,
            &[FIRST, SECOND],
            true,
            None,
            None,
            &[FIRST, SECOND, APPROVAL, OTHER_APPROVAL, NO_GOAL],
            Some(FIRST),
        ),
        (
            "project",
            Filter { project: Some(fixtures::PROJECT), ..Filter::ALL },
            &[],
            true,
            None,
            None,
            &[APPROVAL, NO_GOAL],
            Some(APPROVAL),
        ),
        (
            "goal",
            Filter { goal: Some(a::GOAL), ..Filter::ALL },
            &[],
            true,
            None,
            None,
            &[APPROVAL, OTHER_APPROVAL],
            Some(APPROVAL),
        ),
        (
            "project and goal",
            Filter { project: Some(OTHER_PROJECT), goal: Some(a::GOAL), ..Filter::ALL },
            &[],
            true,
            None,
            None,
            &[OTHER_APPROVAL],
            Some(OTHER_APPROVAL),
        ),
        (
            "kind",
            Filter { kind: Some(Kind::Question), ..Filter::ALL },
            &[FIRST],
            true,
            None,
            None,
            &[FIRST],
            Some(FIRST),
        ),
        (
            "later approval",
            Filter::ALL,
            &[FIRST],
            true,
            Some(APPROVAL),
            Some(APPROVAL),
            &[FIRST, OTHER_APPROVAL, NO_GOAL, APPROVAL],
            Some(FIRST),
        ),
        (
            "later inbox",
            Filter::ALL,
            &[FIRST, SECOND],
            true,
            Some(FIRST),
            None,
            &[SECOND, APPROVAL, OTHER_APPROVAL, NO_GOAL, FIRST],
            Some(SECOND),
        ),
        (
            "open approval",
            Filter::ALL,
            &[FIRST],
            true,
            None,
            Some(OTHER_APPROVAL),
            &[FIRST, APPROVAL, OTHER_APPROVAL, NO_GOAL],
            Some(OTHER_APPROVAL),
        ),
        ("unknown inbox", Filter::ALL, &[FIRST], false, None, None, &[], None),
    ]
}
#[test]
fn needs_take_the_approvals_after_the_inbox_by_its_filter() {
    use crate::tests::approvals as a;
    for (name, filter, objects, known, later, at, expected, shown) in approval_cases() {
        let (mut root, _, watch) = approval_root();
        let mut cards = if *known { fixture() } else { f::pages(FIRST, f::question()) };
        let inbox = cards.inbox.as_mut().unwrap();
        let template = inbox.cards.get(0).unwrap().clone();
        inbox.cards.clear();
        inbox.filter = *filter;
        for object in *objects {
            let mut card = template.clone();
            card.object = *object;
            card.group = c::InboxGroup::Questions;
            inbox.cards.push(card).unwrap();
        }
        root.pages.inbox = cards.inbox.take();
        a::deliver(
            &mut root,
            watch,
            &[
                a::row(a::CHANGE, fixtures::PROJECT, Some(a::GOAL)),
                a::row(a::OTHER_CHANGE, OTHER_PROJECT, Some(a::GOAL)),
                a::row(NO_GOAL_CHANGE, fixtures::PROJECT, None),
            ],
            0,
        );
        if let Some(object) = at {
            root_event(
                &mut root,
                crate::Event::Act { action: crate::Action::Pages(pages::Act::Needs(Act::Open { object: *object })) },
            );
        }
        if let Some(object) = later {
            root_event(
                &mut root,
                crate::Event::Act { action: crate::Action::Pages(pages::Act::Needs(Act::Later { object: *object })) },
            );
            assert!(root.pages().needs().later(*object), "{name}");
        }
        let session = root.pages().needs();
        let inbox = root.pages().inbox().unwrap();
        let needs = session.needs(inbox, root.forge().approvals());
        let mut objects = List::with_capacity(u32::try_from(needs.len()).unwrap());
        for need in &needs {
            objects.push(need.object()).unwrap();
        }
        assert_eq!(objects.as_slice(), *expected, "{name}");
        let selected = match session.shown(inbox, root.forge().approvals()) {
            Some(need) => Some(need.object()),
            None => None,
        };
        assert_eq!(selected, *shown, "{name}");
    }
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
fn across_count_counts_shared_proposals_once_and_subtracts_a_held_answer() {
    let (mut root, watch) = count_root();
    let mut rows = List::with_capacity(3);
    for (object, project) in [(FIRST, fixtures::PROJECT), (SECOND, OTHER_PROJECT)] {
        rows.push(crate::tests::documents::entry(object, project, 1, Wall::EPOCH, None)).unwrap();
    }
    let mut proposal =
        crate::tests::documents::entry(ObjectKey::Proposal(f::PROPOSAL), fixtures::PROJECT, 1, Wall::EPOCH, None);
    proposal.project = None;
    proposal.projects = List::with_capacity(2);
    proposal.projects.push(fixtures::PROJECT).unwrap();
    proposal.projects.push(OTHER_PROJECT).unwrap();
    rows.push(proposal).unwrap();
    let inbox = crate::tests::documents::inbox(rows, 99, c::Count { value: 0, limited: false }, 0, false);
    super::welcome::deliver(&mut root, watch, c::Delivery::Inbox(Box::new(inbox)));
    assert_eq!(
        root.pages().needs().count_across(root.client(), root.forge().approvals()),
        needs::Count { count: 3, cut: false }
    );
    assert_eq!(root.pages().needs().count(root.client(), root.forge().approvals(), fixtures::PROJECT).count, 2);
    root.pages.inbox = fixture().inbox.take();
    root_event(
        &mut root,
        crate::Event::Act {
            action: crate::Action::Pages(pages::Act::Needs(Act::Choose { object: FIRST, words: Box::from(CHOICE) })),
        },
    );
    assert!(root.pages().needs().held(FIRST));
    assert_eq!(root.pages().needs().count_across(root.client(), root.forge().approvals()).count, 2);
    assert_eq!(root.pages().needs().count(root.client(), root.forge().approvals(), fixtures::PROJECT).count, 1);
}

#[test]
fn standing_count_filters_projects_subtracts_answered_keeps_later_and_reports_cut_and_end() {
    for (inbox_cut, more) in [(false, 1), (true, 0)] {
        let (mut root, watch, approvals) = approval_root();
        assert_eq!(
            root.pages().needs().count(root.client(), root.forge().approvals(), fixtures::PROJECT),
            needs::Count::default()
        );
        crate::tests::approvals::deliver(
            &mut root,
            approvals,
            &[
                crate::tests::approvals::row(crate::tests::approvals::CHANGE, fixtures::PROJECT, None),
                crate::tests::approvals::row(crate::tests::approvals::OTHER_CHANGE, OTHER_PROJECT, None),
            ],
            more,
        );
        assert_eq!(root.pages().needs().count_across(root.client(), root.forge().approvals()), needs::Count::default());
        let mut rows = List::with_capacity(4);
        for (object, project) in [
            (FIRST, fixtures::PROJECT),
            (SECOND, fixtures::PROJECT),
            (THIRD, OTHER_PROJECT),
            (ObjectKey::Result(f::TASK), fixtures::PROJECT),
        ] {
            rows.push(crate::tests::documents::entry(object, project, 1, Wall::EPOCH, None)).unwrap();
        }
        let inbox = crate::tests::documents::inbox(rows, 99, c::Count { value: 1, limited: false }, 0, inbox_cut);
        let before = pages::shown(root.pages());
        super::welcome::deliver(&mut root, watch, c::Delivery::Inbox(Box::new(inbox)));
        assert!(pages::shown(root.pages()) > before);
        assert_eq!(
            root.pages().needs().count(root.client(), root.forge().approvals(), fixtures::PROJECT),
            needs::Count { count: 3, cut: true }
        );
        assert_eq!(
            root.pages().needs().count_across(root.client(), root.forge().approvals()),
            needs::Count { count: 5, cut: true }
        );
        let mut cards = fixture();
        add(&mut cards, SECOND, fixtures::PROJECT);
        root.pages.inbox = cards.inbox.take();
        root_event(
            &mut root,
            crate::Event::Act {
                action: crate::Action::Pages(pages::Act::Needs(Act::Later {
                    object: ObjectKey::Task(crate::tests::approvals::CHANGE),
                })),
            },
        );
        assert_eq!(root.pages().needs().count(root.client(), root.forge().approvals(), fixtures::PROJECT).count, 3);
        assert_eq!(root.pages().needs().count_across(root.client(), root.forge().approvals()).count, 5);
        root_event(
            &mut root,
            crate::Event::Act {
                action: crate::Action::Pages(pages::Act::Needs(Act::Choose {
                    object: FIRST,
                    words: Box::from(CHOICE),
                })),
            },
        );
        assert_eq!(root.pages().needs().count(root.client(), root.forge().approvals(), fixtures::PROJECT).count, 2);
        assert_eq!(root.pages().needs().count_across(root.client(), root.forge().approvals()).count, 4);
        let before = pages::shown(root.pages());
        let out = root_event(
            &mut root,
            crate::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded },
                },
            },
        );
        assert!(pages::shown(root.pages()) > before);
        assert_eq!(
            root.pages().needs().count(root.client(), root.forge().approvals(), fixtures::PROJECT),
            needs::Count::default()
        );
        assert_eq!(root.pages().needs().count_across(root.client(), root.forge().approvals()), needs::Count::default());
        assert!(opens(&out).is_empty());
    }
}

#[test]
fn reply_past_choice_bound_is_held_whole_for_the_engine() {
    let words = repeated(temper_client_protocol_wire::CUE_BYTES.checked_add(1).unwrap(), b"");
    let mut limits = fixtures::limits();
    limits.client.domain.long_text = 2048;
    let env = fixtures::env(&limits);
    let client = super::composer::root(&limits);
    for explicitly_open in [false, true] {
        for (mut pages, object) in answer_fixtures() {
            if explicitly_open {
                act(&mut pages, 0, Act::OpenReply { object });
            }
            frame(&mut pages, &env, client.client());
            let mut out = Queue::uncapped();
            for act in
                [pages::Act::CardWords { object, words: words.clone() }, pages::Act::Needs(Act::SendReply { object })]
            {
                pages::step(&mut pages, &env, client.client(), pages::Event::Act { act }, &mut out);
            }
            assert!(out.is_empty());
            assert!(pages.needs().held(object), "{object:?}: reply opened explicitly: {explicitly_open}");
            let (_, ask) = sent(&mut fire(&mut pages, HOLD_DUE));
            let sent = c::Text::new(words.clone(), limits.client.domain.long_text).unwrap();
            let on = if object == FIRST {
                c::AnswerOn::Question { task: f::TASK, message: f::MESSAGE, words: sent }
            } else {
                c::AnswerOn::Person { task: f::TASK, revision: f::REVISION, answer: c::PersonAnswer::Words(sent) }
            };
            assert_eq!(ask, c::Ask::Answer { project: fixtures::PROJECT, on });
        }
    }
    let object = ObjectKey::Proposal(f::PROPOSAL);
    let mut pages = f::pages(object, f::proposal(c::ActKind::Reject, c::Admission::Allowed));
    frame(&mut pages, &env, client.client());
    for (words, kept) in [
        (repeated(limits.client.domain.text, b""), true),
        (repeated(limits.client.domain.text.checked_add(1).unwrap(), b""), false),
    ] {
        let mut out = Queue::uncapped();
        pages::step(
            &mut pages,
            &env,
            client.client(),
            pages::Event::Act { act: pages::Act::CardWords { object, words: words.clone() } },
            &mut out,
        );
        assert!(out.is_empty());
        assert!(!pages.needs().replying(object));
        let card_words = pages.inbox().unwrap().cards.get(0).unwrap().words.as_ref().unwrap().bytes();
        if kept {
            assert_eq!(card_words, &*words);
        } else {
            assert_eq!(card_words.len(), usize::try_from(limits.client.domain.text).unwrap());
        }
    }
}

#[test]
fn a_choice_pressed_while_the_inbox_reads_again_is_held_and_sent() {
    for filter in [
        crate::address::Filter { project: Some(fixtures::PROJECT), ..crate::address::Filter::ALL },
        crate::address::Filter::ALL,
    ] {
        let (mut root, watch) = fixtures::inbox_home(filter);
        let rows = refresh_rows();
        fixtures::inbox_entries(&mut root, watch, rows.clone());
        if filter.project.is_some() {
            let queries = root.pages().inbox().unwrap().queries;
            for (index, handle) in queries.into_iter().enumerate() {
                let rows = if index == 0 { rows.clone() } else { List::with_capacity(0) };
                refresh_page(
                    &mut root,
                    handle,
                    c::ReadValue::Inbox(Box::new(c::InboxPage {
                        rows,
                        cut: false,
                        before: None,
                        after: None,
                        selection: None,
                    })),
                );
            }
        }
        let handle = root.pages().inbox().unwrap().cards.get(0).unwrap().handle;
        let mut question = f::question();
        if let c::ReadValue::Question(question) = &mut question {
            question.words = c::DocumentText::complete(f::text(b"Which?\nChoice: Use the first"));
        }
        refresh_page(&mut root, handle, question);
        choose(&mut root.pages, 0, FIRST);
        act(&mut root.pages, 0, Act::Undo { object: FIRST });
        fixtures::inbox_entries(&mut root, watch, refresh_rows());
        assert!(choose(&mut root.pages, 0, FIRST).is_empty());
        assert!(root.pages().needs().held(FIRST), "choice during refresh at {filter:?}");
        assert!(fire(&mut root.pages, BEFORE_HOLD).is_empty());
        assert_eq!(sent(&mut fire(&mut root.pages, HOLD_DUE)).1, expected(FIRST, CHOICE, fixtures::PROJECT));
        assert!(fire(&mut root.pages, SECOND_HOLD).is_empty());
    }
}
fn refresh_rows() -> List<c::InboxEntry> {
    let mut rows = List::with_capacity(1);
    rows.push(fixtures::inbox_entry(FIRST, c::InboxGroup::Questions, fixtures::PROJECT, 1)).unwrap();
    rows
}
fn refresh_page(root: &mut crate::Domain, handle: jig_client::Handle, value: c::ReadValue) {
    crate::route::told(
        root,
        &fixtures::env(&fixtures::limits()),
        Told::Page { handle, page: jig_client::Readout::Jig(c::ReadResult::Value(value)) },
        &mut Queue::uncapped(),
    );
}

#[test]
fn later_reply_undo_choose_sends_the_choice_once() {
    let mut pages = fixture();
    act(&mut pages, 0, Act::Later { object: FIRST });
    reply(&mut pages, b"Reply words");
    act(&mut pages, SECOND_CHOICE, Act::Undo { object: FIRST });
    choose(&mut pages, SECOND_CHOICE, FIRST);
    // A late completion for an earlier send cannot take a new held line.
    outcome(&mut pages, Key { asker: crate::owner::Owner::Pages.asker(), number: 1 }, Outcome::Lost);
    assert!(fire(&mut pages, HOLD_DUE).is_empty());
    assert!(pages.needs().held(FIRST));
    assert_eq!(sent(&mut fire(&mut pages, SECOND_HOLD)).1, expected(FIRST, CHOICE, fixtures::PROJECT));
    assert!(fire(&mut pages, SECOND_HOLD + HOLD_DUE).is_empty());
}
#[test]
fn a_refused_choice_or_full_session_says_the_answer_was_not_sent() {
    let mut cue = fixture();
    act(&mut cue, 0, Act::Choose { object: FIRST, words: repeated(temper_client_protocol_wire::CUE_BYTES + 1, b"") });
    let mut full = capacity_fixture();
    for number in 0..CAPACITY {
        choose(&mut full, 0, object(number));
    }
    choose(&mut full, 0, object(CAPACITY));
    for (pages, object, count) in
        [(&mut cue, FIRST, 0), (&mut full, object(CAPACITY), usize::try_from(CAPACITY).unwrap())]
    {
        assert_eq!(lines(pages).len(), count);
        assert!(!pages.needs().held(object));
        assert_eq!(pages.needs().toast(), Some(&Toasted::NotSent));
    }
    assert!(fire(&mut cue, HOLD_DUE).is_empty());
    let mut out = fire(&mut full, HOLD_DUE);
    let mut sends = 0;
    while let Some(request) = out.pop() {
        if let Request::Client(jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. }) = request {
            assert_ne!(*ask, expected(object(CAPACITY), CHOICE, fixtures::PROJECT));
            sends += 1;
        }
    }
    assert_eq!(sends, CAPACITY);
}

fn composer_act(pages: &mut Pages, root: &crate::Domain, limits: &crate::Limits, act: pages::Act) -> Queue<Request> {
    let mut out = Queue::uncapped();
    pages::step(pages, &fixtures::env(limits), root.client(), pages::Event::Act { act }, &mut out);
    out
}
fn composer_reply(words: Box<[u8]>) -> (Pages, crate::Domain, crate::Limits) {
    let limits = super::composer::limits();
    let root = super::composer::root(&limits);
    let mut pages = fixture();
    composer_act(&mut pages, &root, &limits, pages::Act::CardWords { object: FIRST, words });
    (pages, root, limits)
}
fn composer_words(pages: &Pages) -> &[u8] {
    pages.inbox().unwrap().cards.get(0).unwrap().words.as_ref().unwrap().bytes()
}
#[test]
fn a_kilobyte_pasted_is_sent_whole() {
    let words = super::composer::words(super::composer::BOUND);
    let (mut pages, root, limits) = composer_reply(words.clone());
    assert_eq!(composer_words(&pages), &*words);
    composer_act(&mut pages, &root, &limits, pages::Act::Needs(Act::SendReply { object: FIRST }));
    assert!(pages.needs().held(FIRST));
    let c::Ask::Answer { on: c::AnswerOn::Question { words: sent, .. }, .. } = sent(&mut fire(&mut pages, HOLD_DUE)).1
    else {
        panic!("reply")
    };
    assert_eq!(sent.bytes(), &*words);
}
#[test]
fn a_paste_past_the_bound_stays_and_sends_nothing() {
    for &(typed, kept, beyond, excess) in super::composer::PAST {
        let (mut pages, root, limits) = composer_reply(super::composer::words(typed));
        let words = super::composer::words(kept);
        assert_eq!(composer_words(&pages), &*words);
        assert_eq!(pages.needs().reply_beyond, beyond);
        assert!(
            composer_act(&mut pages, &root, &limits, pages::Act::Needs(Act::SendReply { object: FIRST })).is_empty()
        );
        assert!(lines(&pages).is_empty());
        assert!(!pages.needs().held(FIRST));
        assert_eq!(pages.needs().toast(), None);
        assert_eq!(composer_words(&pages), &*words);
        assert_eq!(
            pages::composer::over(composer_words(&pages), pages.needs().reply_beyond, super::composer::BOUND),
            excess
        );
        assert!(fire(&mut pages, HOLD_DUE).is_empty());
        composer_act(&mut pages, &root, &limits, pages::Act::CardWords { object: FIRST, words: Box::from(*b"short") });
        assert_eq!(pages.needs().reply_beyond, 0);
        composer_act(&mut pages, &root, &limits, pages::Act::Needs(Act::SendReply { object: FIRST }));
        assert!(pages.needs().held(FIRST));
        assert_eq!(sent(&mut fire(&mut pages, HOLD_DUE)).1, expected(FIRST, b"short", fixtures::PROJECT));
    }
}

fn why_root(state: &str, rows: &[(u64, u64, temper_protocol_reaches::Reach)]) -> crate::Domain {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let me = fixtures::start(&mut root, List::with_capacity(0));
    fixtures::confirm(&mut root, me, fixtures::PARTY, &mut Queue::uncapped());
    if state == "unwatched" {
        return root;
    }
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &env,
        crate::Event::Went { place: crate::Place::Address(Address::Home { project: fixtures::PROJECT }) },
        &mut out,
    );
    let subject = crate::forge::decided::watch(fixtures::PROJECT, &limits.forge).unwrap();
    let mut watch = None;
    for request in &out {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch: found, subject: opened } },
        } = request
            && *opened == subject
        {
            watch = Some(*found);
        }
    }
    let watch = watch.unwrap();
    if state == "unknown" {
        return root;
    }
    let up = if state == "ended" {
        jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed }
    } else {
        let mut asked = List::with_capacity(16);
        for (index, (task, at, reach)) in rows.iter().enumerate() {
            asked
                .push(
                    w::Asked::new(
                        &w::CEILINGS,
                        w::AskedParts {
                            number: u32::try_from(index.checked_add(1).unwrap()).unwrap(),
                            task: *task,
                            at: *at,
                            reach: temper_protocol_reaches::reach_record(reach),
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let document = w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project: fixtures::PROJECT, rows: List::with_capacity(0), asked, cut: state == "cut" },
        )
        .unwrap();
        jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::decisions(&document),
            },
        }
    };
    crate::step(&mut root, &env, crate::Event::Client { event: jig_client::Event::Up { up } }, &mut Queue::uncapped());
    root
}
#[test]
#[expect(clippy::too_many_lines, reason = "each row names one question matching or document readiness rule")]
fn why_matches_the_asking_task_and_time_and_waits_for_its_document() {
    use temper_protocol_reaches::{Class, Kind, Reach, Reason};
    let reach = Reach {
        kind: Kind::Implementation,
        class: Class::Ask,
        reason: Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    };
    let other = Reach { kind: Kind::Tests, ..reach };
    let expected = Some(needs::Why::Reach { project: fixtures::PROJECT, reach });
    let mut question = f::question();
    if let c::ReadValue::Question(q) = &mut question {
        q.from = Some(c::Requester::Task(42));
        q.since = Wall::from_nanos(20);
    }
    let mut own = question.clone();
    if let c::ReadValue::Question(q) = &mut own {
        q.from = None;
    }
    let mut person = f::person(c::ActKind::Answer);
    if let c::ReadValue::PersonTask(p) = &mut person {
        p.requester = c::Requester::Task(42);
    }
    let mut timed_person = person.clone();
    if let c::ReadValue::PersonTask(p) = &mut timed_person {
        p.since = Some(Wall::from_nanos(20));
    }
    let mut funds = f::held(c::ActKind::Release);
    if let c::ReadValue::Escalation(e) = &mut funds {
        e.task.hold.as_mut().unwrap().reason = c::HoldCategory::Budget;
    }
    let mut ordinary = person.clone();
    if let c::ReadValue::PersonTask(p) = &mut ordinary {
        p.address = c::PersonAddress::Role(1);
    }
    for (name, page, group, state, rows, why, ready) in [
        (
            "newest eligible",
            question.clone(),
            c::InboxGroup::Questions,
            "delivered",
            Box::from([(42, 20, reach), (42, 10, other)].as_slice()),
            expected,
            true,
        ),
        (
            "later ignored",
            question.clone(),
            c::InboxGroup::Questions,
            "delivered",
            Box::from([(42, 20, reach), (42, 21, other)].as_slice()),
            expected,
            true,
        ),
        (
            "other task ignored",
            question.clone(),
            c::InboxGroup::Questions,
            "delivered",
            Box::from([(42, 20, reach), (43, 20, other)].as_slice()),
            expected,
            true,
        ),
        (
            "own task",
            own,
            c::InboxGroup::Questions,
            "delivered",
            Box::from([(f::TASK, 20, reach)].as_slice()),
            expected,
            true,
        ),
        (
            "person requester newest",
            person,
            c::InboxGroup::PersonTasks,
            "delivered",
            Box::from([(42, 50, reach), (42, 20, other)].as_slice()),
            expected,
            true,
        ),
        (
            "person timestamp",
            timed_person,
            c::InboxGroup::PersonTasks,
            "delivered",
            Box::from([(42, 20, reach), (42, 21, other)].as_slice()),
            expected,
            true,
        ),
        (
            "direct",
            question.clone(),
            c::InboxGroup::Questions,
            "delivered",
            Box::from([].as_slice()),
            Some(needs::Why::Direct),
            true,
        ),
        ("cut", question.clone(), c::InboxGroup::Questions, "cut", Box::from([].as_slice()), None, true),
        (
            "cut with row",
            question.clone(),
            c::InboxGroup::Questions,
            "cut",
            Box::from([(42, 20, reach)].as_slice()),
            expected,
            true,
        ),
        ("unwatched", question.clone(), c::InboxGroup::Questions, "unwatched", Box::from([].as_slice()), None, true),
        ("ended", question.clone(), c::InboxGroup::Questions, "ended", Box::from([].as_slice()), None, true),
        ("unknown", question.clone(), c::InboxGroup::Questions, "unknown", Box::from([].as_slice()), None, false),
        (
            "decided",
            question,
            c::InboxGroup::Questions,
            "delivered",
            Box::from([(42, 20, Reach { class: Class::Decide, ..reach })].as_slice()),
            Some(needs::Why::Direct),
            true,
        ),
        (
            "invalid reach",
            f::question(),
            c::InboxGroup::Questions,
            "delivered",
            Box::from([(f::TASK, 0, Reach { reason: Reason::Set(0), ..reach })].as_slice()),
            Some(needs::Why::Direct),
            true,
        ),
        (
            "proposal",
            f::proposal(c::ActKind::Accept, c::Admission::Allowed),
            c::InboxGroup::Proposals,
            "unknown",
            Box::from([].as_slice()),
            Some(needs::Why::Proposal),
            true,
        ),
        (
            "held",
            f::held(c::ActKind::Release),
            c::InboxGroup::Escalations,
            "unknown",
            Box::from([].as_slice()),
            Some(needs::Why::Held),
            true,
        ),
        (
            "funds",
            funds,
            c::InboxGroup::Escalations,
            "unknown",
            Box::from([].as_slice()),
            Some(needs::Why::Funds),
            true,
        ),
        ("ordinary person", ordinary, c::InboxGroup::PersonTasks, "unknown", Box::from([].as_slice()), None, true),
    ] {
        let root = why_root(state, &rows);
        let mut pages = fixture();
        let card = pages.inbox.as_mut().unwrap().cards.get_mut(0).unwrap();
        card.page = Some(page);
        card.group = group;
        assert_eq!(needs::why(&root, needs::Need::Card(card)), why, "{name}");
        assert_eq!(needs::ready(&root, needs::Need::Card(card)), ready, "{name}");
    }
    let root = why_root("unknown", &[]);
    let approval = crate::forge::approvals::Row {
        task: f::TASK,
        project: fixtures::PROJECT,
        goal: None,
        forge: 1,
        kind: crate::forge::forges::Kind::Forgejo,
        owner: b"work".as_slice().into(),
        name: b"hello".as_slice().into(),
        pull: 1,
        address: Box::default(),
        title: Box::default(),
        since: Wall::EPOCH,
        scope: fixtures::PROJECT,
        rule: 1,
        role: 1,
        required: 1,
        counted: 0,
        sets: Box::new([]),
        whole: true,
        yours: crate::forge::approvals::Yours::Counts,
        size: None,
    };
    assert_eq!(needs::why(&root, needs::Need::Approval(&approval)), None, "approval");
    assert!(needs::ready(&root, needs::Need::Approval(&approval)), "approval");
}

const STOP_DAY: u64 = 86_400_000_000_000;
const STOP_WALL: Wall = Wall::from_nanos(864_000_000_000_000);
const STOP_NEWEST: u32 = 8;
fn stop_limits() -> crate::Limits {
    let mut limits = fixtures::limits();
    limits.client.domain.text = 512;
    limits.client.domain.long_text = 512;
    limits.client.domain.turn_text = 512;
    limits
}
fn stop_reach() -> temper_protocol_reaches::Reach {
    use temper_protocol_reaches::{Class, Kind, Reach, Reason};
    Reach {
        kind: Kind::Implementation,
        class: Class::Ask,
        reason: Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    }
}
fn stop_watch(out: &Queue<crate::Request>, subject: &jig_client::Watchable) -> skein_lib::Token {
    for request in out {
        if let crate::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject: found } },
        } = request
            && found == subject
        {
            return *watch;
        }
    }
    panic!("stop asking watch")
}
fn stop_classes_read(out: &Queue<crate::Request>) -> Option<skein_lib::Token> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    for request in out {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && let w::Read::Setting(setting) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
            && setting.kind() == 8
        {
            return Some(*read);
        }
    }
    None
}
struct StopFixture {
    root: crate::Domain,
    settings: skein_lib::Token,
    dismissed: skein_lib::Token,
    decided: skein_lib::Token,
}
fn stop_fixture(
    owner: bool,
    reach: temper_protocol_reaches::Reach,
    read: bool,
    delivered: bool,
    keep: Option<(temper_protocol_reaches::Subject, u64)>,
) -> StopFixture {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    use temper_protocol_reaches::{Classes, Way};
    let mut root = crate::unlinked(&stop_limits(), fixtures::SEED);
    let me_watch = fixtures::start(&mut root, List::with_capacity(0));
    let mut me = fixtures::chats::me();
    me.projects.get_mut(0).unwrap().role = if owner { c::HeldRole::Owner } else { c::HeldRole::Member };
    let initial = super::welcome::deliver(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    let settings = stop_watch(&initial, &jig_client::Watchable::Jig(c::Subject::Settings));
    let dismissed = stop_watch(&initial, &crate::forge::dismissed::watch(&fixtures::limits().forge).unwrap());
    let out = root_event(
        &mut root,
        crate::Event::Went { place: crate::Place::Address(Address::Home { project: fixtures::PROJECT }) },
    );
    let decided =
        stop_watch(&out, &crate::forge::decided::watch(fixtures::PROJECT, &fixtures::limits().forge).unwrap());
    let classes_read = stop_classes_read(&initial).or(stop_classes_read(&out)).unwrap();
    super::welcome::deliver(
        &mut root,
        settings,
        c::Delivery::Settings(Box::new(c::SettingsDocument {
            newest: STOP_NEWEST,
            settings: List::with_capacity(0),
            held: List::with_capacity(0),
            pending: List::with_capacity(0),
        })),
    );
    if read {
        let record = temper_protocol_reaches::classes_record(&Classes::stock(Way::YouApprove)).unwrap();
        let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut writer).unwrap();
        let bytes = wire::setting_answer(
            &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) }).unwrap(),
        );
        super::welcome::up(
            &mut root,
            jig_client::Up::ReadDone {
                read: classes_read,
                result: jig_client::Readout::Family { family: wire::FORGE, record: Some(bytes) },
            },
        );
    }
    let mut asked = List::with_capacity(1);
    asked
        .push(
            w::Asked::new(
                &w::CEILINGS,
                w::AskedParts { number: 1, task: f::TASK, at: 0, reach: temper_protocol_reaches::reach_record(&reach) },
            )
            .unwrap(),
        )
        .unwrap();
    let decisions = w::Decisions::new(
        &w::CEILINGS,
        w::DecisionsParts { project: fixtures::PROJECT, rows: List::with_capacity(0), asked, cut: false },
    )
    .unwrap();
    super::welcome::up(
        &mut root,
        jig_client::Up::FamilyDelivered {
            watch: decided,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::decisions(&decisions),
            },
        },
    );
    root.pages.inbox = fixture().inbox.take();
    add(&mut root.pages, SECOND, fixtures::PROJECT);
    let mut fixture = StopFixture { root, settings, dismissed, decided };
    if delivered {
        stop_dismissals(&mut fixture, keep);
    }
    fixture
}
fn stop_dismissals(fixture: &mut StopFixture, keep: Option<(temper_protocol_reaches::Subject, u64)>) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut dismissed = List::with_capacity(1);
    if let Some((subject, at)) = keep {
        dismissed
            .push(w::PersonFact::KeepAsking(
                w::KeepAsking::new(
                    &w::CEILINGS,
                    w::KeepAskingParts {
                        project: fixtures::PROJECT,
                        subject: temper_protocol_reaches::subject_record(subject),
                        at,
                    },
                )
                .unwrap(),
            ))
            .unwrap();
    }
    let document = w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap();
    super::welcome::up(
        &mut fixture.root,
        jig_client::Up::FamilyDelivered {
            watch: fixture.dismissed,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::dismissals(&document),
            },
        },
    );
}
fn stop_act(root: &mut crate::Domain, act: pages::stop_asking::Act) -> Queue<crate::Request> {
    let limits = stop_limits();
    let mut env = fixtures::env(&limits);
    env.wall = STOP_WALL;
    let mut out = Queue::uncapped();
    crate::route::event(
        root,
        &env,
        crate::Event::Act { action: crate::Action::Pages(pages::Act::StopAsking(act)) },
        &mut out,
    );
    out
}
fn stop_sent(out: &Queue<crate::Request>) -> Box<[(Key, jig_client::Ask)]> {
    let mut sent = List::with_capacity(out.len());
    for request in out {
        if let crate::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { record } },
        } = request
        {
            sent.push((record.request.key, record.request.request.clone())).unwrap();
        }
    }
    sent.into_boxed()
}
fn stop_answer(root: &mut crate::Domain, key: Key, outcome: Outcome) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &fixtures::env(&fixtures::limits()), Told::Answered { key, outcome }, &mut out);
    out
}
fn stop_show_elsewhere(root: &mut crate::Domain) {
    use crate::forge::{self, ways};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut child = Queue::uncapped();
    ways::progress(
        &mut root.forge,
        Time::ZERO,
        Some(OTHER_PROJECT),
        Some(fixtures::PROJECT),
        false,
        fixtures::limits().forge,
        &mut child,
    );
    let mut shown = None;
    for request in &child {
        if let forge::Request::Client {
            asked:
                jig_client::Event::Read {
                    handle,
                    read: jig_client::Readable::Family { scope: c::Scope::Project(project), record, .. },
                },
        } = request
            && *project == fixtures::PROJECT
            && let w::Read::Setting(setting) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
            && setting.kind() == 8
        {
            shown = Some(*handle);
        }
    }
    let classes = temper_protocol_reaches::classes_record(&temper_protocol_reaches::Classes::default()).unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(classes.measure()).unwrap());
    classes.encode(&mut writer).unwrap();
    let record = wire::setting_answer(
        &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) }).unwrap(),
    );
    ways::told(
        &mut root.forge,
        Time::ZERO,
        &root.client,
        &Told::Page {
            handle: shown.expect("shown project's classes read"),
            page: jig_client::Readout::Family { family: wire::FORGE, record: Some(record) },
        },
        fixtures::limits().forge,
        &mut Queue::uncapped(),
    );
}
fn stop_item(classes: &temper_protocol_reaches::Classes) -> c::SettingItem {
    let record = temper_protocol_reaches::classes_record(classes).unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    c::SettingItem {
        setting: c::Setting::Application { kind: 8, scope: fixtures::PROJECT, number: 1 },
        edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(writer.finish()) }),
    }
}
enum Closing {
    Offers(pages::stop_asking::Act),
    Needs(Act),
    AnswerWords(ObjectKey),
    Leave(Address),
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table of Stop asking rules includes readiness, closing and its three sends"
)]
fn stop_asking_offers_closes_and_sends_the_narrowed_change_or_person_fact() {
    use pages::stop_asking::{self, Act as Stop};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    use temper_protocol_reaches::{Class, Classes, Item, Kind, Offer, Reason, Subject};
    let reach = stop_reach();
    let subject = Subject::Kind(Kind::Implementation);
    let offer = Offer::Decide(subject);
    for (name, reach, read, delivered, keep, expected) in [
        ("asked", reach, true, true, None, true),
        ("decided", temper_protocol_reaches::Reach { class: Class::Decide, ..reach }, true, true, None, false),
        ("away decided", temper_protocol_reaches::Reach { away: true, ..reach }, true, true, None, false),
        ("fixed", temper_protocol_reaches::Reach { kind: Kind::Money, ..reach }, true, true, None, false),
        ("always", temper_protocol_reaches::Reach { reason: Reason::Always, ..reach }, true, true, None, false),
        ("classes unread", reach, false, true, None, false),
        ("dismissals undelivered", reach, true, false, None, false),
        (
            "six days",
            reach,
            true,
            true,
            Some((subject, STOP_WALL.as_nanos().checked_sub(STOP_DAY.checked_mul(6).unwrap()).unwrap())),
            false,
        ),
        (
            "seven days",
            reach,
            true,
            true,
            Some((subject, STOP_WALL.as_nanos().checked_sub(STOP_DAY.checked_mul(7).unwrap()).unwrap())),
            true,
        ),
        (
            "eight days",
            reach,
            true,
            true,
            Some((subject, STOP_WALL.as_nanos().checked_sub(STOP_DAY.checked_mul(8).unwrap()).unwrap())),
            true,
        ),
        ("another subject", reach, true, true, Some((Subject::Kind(Kind::Tests), STOP_WALL.as_nanos())), true),
    ] {
        let mut fixture = stop_fixture(true, reach, read, delivered, keep);
        let card = fixture.root.pages().inbox().unwrap().cards.get(0).unwrap();
        assert_eq!(
            stop_asking::offered(&fixture.root, needs::Need::Card(card), STOP_WALL),
            expected.then_some(Box::from([offer].as_slice())),
            "{name}"
        );
        assert_eq!(
            stop_sent(&stop_act(&mut fixture.root, Stop::Keep { object: FIRST })).len(),
            usize::from(expected),
            "{name}: Keep guard"
        );
    }
    for (name, closing, expected) in [
        ("toggle", Closing::Offers(Stop::Open { object: FIRST }), None),
        ("other offers", Closing::Offers(Stop::Open { object: SECOND }), Some(SECOND)),
        ("other card", Closing::Needs(Act::Open { object: SECOND }), None),
        ("answer", Closing::Needs(Act::Choose { object: FIRST, words: Box::from(CHOICE) }), None),
        ("terminal answer", Closing::AnswerWords(FIRST), None),
        ("later", Closing::Needs(Act::Later { object: FIRST }), None),
        ("undo", Closing::Needs(Act::Undo { object: FIRST }), None),
        ("leave", Closing::Leave(chats()), None),
    ] {
        let mut fixture = stop_fixture(true, reach, true, true, None);
        stop_act(&mut fixture.root, Stop::Open { object: FIRST });
        assert_eq!(fixture.root.pages().stop_asking().unwrap().object(), FIRST);
        stop_act(&mut fixture.root, Stop::Words { words: b"wire changes".as_slice().into() });
        assert_eq!(fixture.root.pages().stop_asking().unwrap().words(), b"wire changes");
        match closing {
            Closing::Offers(act) => {
                stop_act(&mut fixture.root, act);
            }
            Closing::Needs(act) => {
                root_event(
                    &mut fixture.root,
                    crate::Event::Act { action: crate::Action::Pages(pages::Act::Needs(act)) },
                );
            }
            Closing::AnswerWords(object) => {
                for act in [
                    pages::Act::CardWords { object, words: Box::from(CHOICE) },
                    pages::Act::Decide { object, revision: None, act: c::ActKind::Answer },
                ] {
                    root_event(&mut fixture.root, crate::Event::Act { action: crate::Action::Pages(act) });
                }
            }
            Closing::Leave(address) => {
                root_event(&mut fixture.root, crate::Event::Went { place: crate::Place::Address(address) });
            }
        }
        let opened = match fixture.root.pages().stop_asking() {
            Some(open) => Some(open.object()),
            None => None,
        };
        assert_eq!(opened, expected, "{name}");
    }
    for (owner, shown) in [(true, false), (false, false), (true, true)] {
        let mut fixture = stop_fixture(owner, reach, true, true, None);
        if shown {
            stop_show_elsewhere(&mut fixture.root);
        }
        assert_eq!(stop_asking::owner(&fixture.root, fixtures::PROJECT), owner);
        stop_act(&mut fixture.root, Stop::Open { object: FIRST });
        let wrong = if owner { Stop::Propose { offer } } else { Stop::Apply { offer } };
        assert!(stop_sent(&stop_act(&mut fixture.root, wrong)).is_empty(), "role guard");
        let apply = if owner { Stop::Apply { offer } } else { Stop::Propose { offer } };
        let sent = stop_sent(&stop_act(&mut fixture.root, apply.clone()));
        assert_eq!(sent.len(), 1);
        assert!(fixture.root.pages().stop_asking().unwrap().busy());
        assert!(stop_sent(&stop_act(&mut fixture.root, apply)).is_empty(), "one request while out");
        let table = Classes {
            items: Box::new([Item { subject, words: Box::default(), class: Class::Decide }]),
            sets: Box::default(),
        };
        let mut items = List::with_capacity(1);
        items.push(stop_item(&table)).unwrap();
        let expected = if owner {
            c::Ask::ChangeSettings(c::ChangeSettings { against: STOP_NEWEST, items, running: c::Running::Stays })
        } else {
            c::Ask::Propose { project: 0, action: c::ProposeAction::Settings(c::ProposeSettings { against: STOP_NEWEST, items }), reason: c::Text::new(Box::from(b"Stop asking in Project 1: let agents decide implementation choices, then tell whoever they would have asked.".as_slice()), 512).unwrap() }
        };
        assert_eq!(sent[0].1, jig_client::Ask::Jig(Box::new(expected)), "owner {owner}");
        let key = if owner {
            let retried = stop_sent(&stop_answer(
                &mut fixture.root,
                sent[0].0,
                Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                    refusal: c::Refusal::Stale(c::RefusalStale { newest: 9 }),
                    at: None,
                })),
            ));
            assert_eq!(retried.len(), 1, "stale is retried once");
            let jig_client::Ask::Jig(ask) = &retried[0].1 else { panic!("settings retry") };
            let c::Ask::ChangeSettings(change) = &**ask else { panic!("settings retry") };
            assert_eq!(change.against, 9);
            assert_eq!(change.items.get(0), Some(&stop_item(&table)));
            assert!(fixture.root.pages().stop_asking().unwrap().busy());
            retried[0].0
        } else {
            sent[0].0
        };
        let out = stop_answer(
            &mut fixture.root,
            key,
            Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::NotOffered,
                at: None,
            })),
        );
        assert!(stop_sent(&out).is_empty());
        assert_eq!(fixture.root.pages().stop_asking().unwrap().refused(), Some(&c::Refusal::NotOffered));
        assert!(!fixture.root.pages().stop_asking().unwrap().busy());
        let apply = if owner { Stop::Apply { offer } } else { Stop::Propose { offer } };
        let sent = stop_sent(&stop_act(&mut fixture.root, apply));
        let answer = if owner {
            c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 9 })
        } else {
            c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 31, to: 1 })
        };
        stop_answer(
            &mut fixture.root,
            sent[0].0,
            Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(answer))),
        );
        assert!(fixture.root.pages().stop_asking().is_none());
        assert!(!fixture.root.pages().needs().held(FIRST), "the current need still waits");
        if owner {
            assert_eq!(fixture.root.forge().classes(fixtures::PROJECT), Some(&table));
        }
    }
    for owner in [true, false] {
        let mut fixture = stop_fixture(owner, reach, true, true, None);
        super::welcome::up(
            &mut fixture.root,
            jig_client::Up::WatchEnded { watch: fixture.settings, end: c::WatchEnd::NotAllowed },
        );
        stop_act(&mut fixture.root, Stop::Open { object: FIRST });
        let apply = if owner { Stop::Apply { offer } } else { Stop::Propose { offer } };
        assert!(stop_sent(&stop_act(&mut fixture.root, apply)).is_empty(), "newest revision unread");
    }
    let mut fixture = stop_fixture(true, reach, true, true, None);
    stop_act(&mut fixture.root, Stop::Open { object: FIRST });
    assert!(
        stop_sent(&stop_act(&mut fixture.root, Stop::Apply { offer: Offer::Decide(Subject::Kind(Kind::Money)) }))
            .is_empty(),
        "narrowed refusal sends nothing"
    );
    assert!(fixture.root.pages().stop_asking().unwrap().refused().is_some());
    let dismissed = Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Dismissed }).unwrap()),
    }));
    let refused =
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }));
    for (name, outcome, toast, offered_after) in [
        ("kept before watch", dismissed, Some(Toasted::KeptAsking), false),
        ("unknown follows subject", Outcome::Unknown, None, true),
        ("lost", Outcome::Lost, Some(Toasted::KeepAskingRefused), true),
        ("refused", refused, Some(Toasted::KeepAskingRefused), true),
    ] {
        let mut fixture = stop_fixture(true, reach, true, true, None);
        stop_act(&mut fixture.root, Stop::Open { object: FIRST });
        let sent = stop_sent(&stop_act(&mut fixture.root, Stop::Keep { object: FIRST }));
        assert_eq!(sent.len(), 1);
        assert_eq!(
            sent[0].1,
            crate::forge::dismissed::ask(
                crate::forge::dismissed::PersonFact::KeepAsking { project: fixtures::PROJECT, subject, at: STOP_WALL },
                &fixtures::limits().forge
            )
            .unwrap()
        );
        assert!(fixture.root.pages().stop_asking().is_none());
        let card = fixture.root.pages().inbox().unwrap().cards.get(0).unwrap();
        assert!(stop_asking::offered(&fixture.root, needs::Need::Card(card), STOP_WALL).is_none(), "held subject");
        stop_act(&mut fixture.root, Stop::Open { object: SECOND });
        assert!(fixture.root.pages().stop_asking().is_none(), "another card of held subject");
        assert!(stop_sent(&stop_act(&mut fixture.root, Stop::Keep { object: FIRST })).is_empty());
        stop_answer(&mut fixture.root, sent[0].0, outcome);
        assert_eq!(fixture.root.pages().needs().toast(), toast.as_ref(), "{name}");
        let card = fixture.root.pages().inbox().unwrap().cards.get(0).unwrap();
        assert_eq!(
            stop_asking::offered(&fixture.root, needs::Need::Card(card), STOP_WALL).is_some(),
            offered_after,
            "{name}"
        );
        stop_dismissals(&mut fixture, Some((subject, STOP_WALL.as_nanos())));
        let card = fixture.root.pages().inbox().unwrap().cards.get(0).unwrap();
        assert!(
            stop_asking::offered(&fixture.root, needs::Need::Card(card), STOP_WALL).is_none(),
            "held fact answers {name}"
        );
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "the table follows two cards' reaches through Keep and its queued fact")]
fn keep_asking_uses_its_own_open_card_and_suppresses_its_subject_at_once() {
    use pages::stop_asking::{self, Act as Stop};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    use temper_protocol_reaches::{Kind, Offer, Reach, Subject};
    let first_reach = stop_reach();
    let second_reach = Reach { kind: Kind::Tests, ..first_reach };
    for (name, object, opened, busy, answered, sent_subject) in [
        ("closed", SECOND, None, false, false, Some(Subject::Kind(Kind::Tests))),
        ("own open", SECOND, Some(SECOND), false, false, Some(Subject::Kind(Kind::Tests))),
        ("other open", SECOND, Some(FIRST), false, false, Some(Subject::Kind(Kind::Tests))),
        ("missing", THIRD, Some(FIRST), false, false, None),
        ("answered", FIRST, None, false, true, None),
        ("busy", SECOND, Some(FIRST), true, false, None),
    ] {
        let mut fixture = stop_fixture(true, first_reach, true, true, None);
        let card = fixture.root.pages.inbox.as_mut().unwrap().cards.get_mut(1).unwrap();
        let Some(c::ReadValue::Question(question)) = &mut card.page else { panic!("question") };
        question.from = Some(c::Requester::Task(42));
        let mut asked = List::with_capacity(2);
        for (number, task, reach) in [(1, f::TASK, first_reach), (2, 42, second_reach)] {
            asked
                .push(
                    w::Asked::new(
                        &w::CEILINGS,
                        w::AskedParts { number, task, at: 0, reach: temper_protocol_reaches::reach_record(&reach) },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let document = w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project: fixtures::PROJECT, rows: List::with_capacity(0), asked, cut: false },
        )
        .unwrap();
        super::welcome::up(
            &mut fixture.root,
            jig_client::Up::FamilyDelivered {
                watch: fixture.decided,
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::decisions(&document),
                },
            },
        );
        if answered {
            root_event(
                &mut fixture.root,
                crate::Event::Act {
                    action: crate::Action::Pages(pages::Act::Needs(Act::Choose { object, words: Box::from(CHOICE) })),
                },
            );
        }
        if let Some(object) = opened {
            stop_act(&mut fixture.root, Stop::Open { object });
        }
        if busy {
            stop_act(&mut fixture.root, Stop::Apply { offer: Offer::Decide(Subject::Kind(Kind::Implementation)) });
            assert!(fixture.root.pages().stop_asking().unwrap().busy());
        }
        let sent = stop_sent(&stop_act(&mut fixture.root, Stop::Keep { object }));
        if let Some(subject) = sent_subject {
            assert_eq!(sent.len(), 1, "{name}");
            assert_eq!(
                sent[0].1,
                crate::forge::dismissed::ask(
                    crate::forge::dismissed::PersonFact::KeepAsking {
                        project: fixtures::PROJECT,
                        subject,
                        at: STOP_WALL
                    },
                    &stop_limits().forge,
                )
                .unwrap(),
                "{name}"
            );
            assert!(fixture.root.pages().stop_asking().is_none(), "{name}");
            for card in &fixture.root.pages().inbox().unwrap().cards {
                assert_eq!(
                    stop_asking::offered(&fixture.root, needs::Need::Card(card), STOP_WALL).is_some(),
                    card.object == FIRST,
                    "{name}"
                );
            }
            assert!(stop_sent(&stop_act(&mut fixture.root, Stop::Keep { object })).is_empty(), "already keeping");
        } else {
            assert!(sent.is_empty(), "{name}");
            assert_eq!(
                match fixture.root.pages().stop_asking() {
                    Some(open) => Some(open.object()),
                    None => None,
                },
                opened,
                "{name}"
            );
        }
    }
    for reach in [
        Reach { kind: Kind::Money, ..first_reach },
        Reach { reason: temper_protocol_reaches::Reason::Always, ..first_reach },
    ] {
        let mut fixture = stop_fixture(true, reach, true, true, None);
        assert!(stop_sent(&stop_act(&mut fixture.root, Stop::Keep { object: FIRST })).is_empty(), "nothing offered");
    }
}
