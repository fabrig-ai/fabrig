use super::fixtures;
use crate::{
    Address,
    pages::{
        self, Act, Event, Pages, Request,
        needs::Toasted,
        start_goal::{self, Failure},
    },
};
use jig_client::{Key, Outcome, Told, contract as c};
use skein_lib::{List, Queue};

const WAY: u32 = 2;
const CHARTER: u32 = 7;
const CHAT: u64 = 4;
const GOAL: u64 = 42;
// The wall clock differs from monotone time zero when the create answer arrives.
const FACT_AT: u64 = 1234;
const PATH: &[u8] = b"docs/design/draft/away-mode.md";
// The whole specification fits, including its two copies of the path.
const TEXT: u32 = 256;
const EXTRA_BUDGET: u64 = 1;

pub(super) fn me() -> c::Me {
    let mut me = fixtures::chats::me();
    let member = me.projects.get_mut(0).unwrap();
    let template = member.templates.get_mut(0).unwrap();
    template.number = WAY;
    template.tracked = Some(1);
    template.executor = c::TemplateExecutor::Charter(CHARTER);
    template.budget = c::TemplateBudget::Given;
    template.flags.conversation = false;
    let creating = member.creating.as_mut().unwrap();
    creating.within.charters = List::from_boxed(Box::new([CHARTER]));
    me
}
pub(super) fn beyond(mut me: c::Me, beyond: c::Beyond) -> c::Me {
    let member = me.projects.get_mut(0).unwrap();
    let creating = member.creating.as_mut().unwrap();
    member.templates.get_mut(0).unwrap().budget =
        c::TemplateBudget::Amount(creating.within.budget.checked_add(EXTRA_BUDGET).unwrap());
    creating.beyond = beyond;
    me
}
pub(super) struct Harness {
    pub(super) root: crate::Domain,
    pub(super) pages: Pages,
    pub(super) limits: crate::Limits,
    pub(super) out: Queue<Request>,
}
impl Harness {
    pub(super) fn new(me: c::Me) -> Self {
        let mut limits = fixtures::limits();
        limits.client.domain.text = TEXT;
        limits.client.domain.long_text = TEXT;
        limits.client.domain.turn_text = TEXT;
        let mut root = crate::unlinked(&limits, fixtures::SEED);
        let watch = fixtures::start(&mut root, List::with_capacity(0));
        crate::step(
            &mut root,
            &fixtures::env(&limits),
            crate::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
            &mut Queue::uncapped(),
        );
        let mut h = Self { root, pages: Pages::new(), limits, out: Queue::uncapped() };
        h.event(Event::Went { address: Address::Home { project: fixtures::PROJECT } });
        h.out = Queue::uncapped();
        h
    }
    pub(super) fn event(&mut self, event: Event) {
        pages::step(&mut self.pages, &fixtures::env(&self.limits), self.root.client(), event, &mut self.out);
    }
    pub(super) fn act(&mut self, act: Act) {
        self.event(Event::Act { act });
    }
    pub(super) fn open(&mut self) {
        self.act(Act::OpenStart { draft: Some(PATH.into()), chat: Some(CHAT) });
    }
    pub(super) fn sent(&mut self) -> (Key, c::Ask) {
        while let Some(request) = self.out.pop() {
            if let Request::Client(jig_client::Event::Send { key, request: jig_client::Ask::Jig(ask), .. }) = request {
                return (key, *ask);
            }
        }
        panic!("one sent goal request")
    }
    pub(super) fn answer(&mut self, key: Key, outcome: Outcome) {
        self.event(Event::Told { told: Told::Answered { key, outcome } });
    }
}
pub(super) fn done(value: c::DocumentOutcome) -> Outcome {
    Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(value))))
}

#[test]
fn create_sends_and_goes_to_its_goal() {
    let me = me();
    let budget = me.projects.get(0).unwrap().creating.as_ref().unwrap().within.budget;
    let mut h = Harness::new(me);
    h.open();
    let start = h.pages.start().unwrap();
    assert_eq!((start.project, start.way, start.chat), (fixtures::PROJECT, Some(WAY), Some(CHAT)));
    assert_eq!(start.name.as_ref(), b"away-mode");
    h.act(Act::StartGoal);
    let (key, ask) = h.sent();
    let c::Ask::Create { project, tasks } = ask else { panic!("create") };
    assert_eq!(project, fixtures::PROJECT);
    assert_eq!(tasks.len(), 1);
    let task = tasks.get(0).unwrap();
    assert_eq!(task.executor, c::Executor::Agent { charter: CHARTER });
    assert_eq!(task.authority.spend, budget);
    assert_eq!(task.authority.deadline, None);
    assert_eq!(task.llm, None);
    assert_eq!(task.spec.words.bytes(), start_goal::words(b"away-mode", PATH, Some(CHAT), false).as_ref());
    assert!(h.pages.start().is_some());
    let mut env = fixtures::env(&h.limits);
    env.wall = skein_lib::Wall::from_nanos(FACT_AT);
    pages::step(
        &mut h.pages,
        &env,
        h.root.client(),
        Event::Told {
            told: Told::Answered {
                key,
                outcome: done(c::DocumentOutcome::Created { first: GOAL, last: GOAL, given: None }),
            },
        },
        &mut h.out,
    );
    assert!(h.pages.start().is_none());
    let Some(Request::Client(jig_client::Event::Send { request: jig_client::Ask::Family { scope, record }, .. })) =
        h.out.pop()
    else {
        panic!("goal fact sent")
    };
    assert_eq!(scope, c::Scope::Project(fixtures::PROJECT));
    let request =
        temper_client_protocol_wire::decode_requesting(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
            .unwrap();
    let temper_client_protocol_wire::v1::Request::Dismiss(dismiss) = request.into_parts().request else {
        panic!("dismiss fact")
    };
    let temper_client_protocol_wire::v1::PersonFact::GoalFrom(fact) = dismiss.into_parts().what else {
        panic!("goal fact")
    };
    assert_eq!((fact.project(), fact.chat(), fact.goal(), fact.at()), (fixtures::PROJECT, CHAT, GOAL, FACT_AT));
    assert_eq!(h.out.pop(), Some(Request::Show { address: Address::Task { task: GOAL } }));
    let mut chosen = self::me();
    let member = chosen.projects.get_mut(0).unwrap();
    member.templates.get_mut(0).unwrap().executor = c::TemplateExecutor::Chosen;
    member.creating.as_mut().unwrap().within.charters = List::from_boxed(Box::new([1]));
    let mut h = Harness::new(chosen);
    h.open();
    h.act(Act::StartGoal);
    let (_, ask) = h.sent();
    let c::Ask::Create { tasks, .. } = ask else { panic!("chosen create") };
    assert_eq!(tasks.get(0).unwrap().executor, c::Executor::Agent { charter: 1 });
}
#[test]
fn propose_sends_closes_and_toasts() {
    let to = c::Recipient::Party(fixtures::PARTY);
    let mut h = Harness::new(beyond(me(), c::Beyond::Proposed { to: to.clone() }));
    h.open();
    h.act(Act::StartGoal);
    let (key, ask) = h.sent();
    let c::Ask::Propose { project, reason, action: c::ProposeAction::Batch { tasks } } = ask else {
        panic!("proposal")
    };
    assert_eq!(project, fixtures::PROJECT);
    assert_eq!(reason.bytes(), b"Started from its draft");
    assert_eq!(tasks.len(), 1);
    assert!(h.pages.start().is_some());
    assert_eq!(h.pages.needs().toast(), None);
    h.answer(key, done(c::DocumentOutcome::Created { first: GOAL, last: GOAL, given: None }));
    assert!(h.pages.start().is_none());
    assert_eq!(h.pages.needs().toast(), Some(&Toasted::Proposed { to }));
    assert_eq!(pages::next_deadline(&h.pages), Some(skein_lib::Time::ZERO.saturating_add(pages::needs::TOAST)));
    assert!(h.out.is_empty());
}
#[test]
fn refused_stays_open_with_jigs_words() {
    let why = c::Refusal::NotOffered;
    let mut h = Harness::new(beyond(me(), c::Beyond::Refused(why.clone())));
    h.open();
    h.act(Act::StartGoal);
    let Some(Failure::Refused(refusal)) = &h.pages.start().unwrap().error else { panic!("jig refusal") };
    assert_eq!(refusal, &why);
    assert!(h.out.is_empty());
}
#[test]
fn not_offered_stays_open() {
    let mut me = me();
    me.projects.get_mut(0).unwrap().creating = None;
    let mut h = Harness::new(me);
    h.open();
    h.act(Act::StartGoal);
    let Some(Failure::NotOffered(why)) = &h.pages.start().unwrap().error else { panic!("not offered") };
    assert_eq!(why, &jig_client::shapes::NotOffered::Creating);
    assert!(h.out.is_empty());
}
#[test]
fn a_refused_answer_stays_open() {
    for outcome in [
        Outcome::Lost,
        Outcome::Unknown,
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None })),
    ] {
        let mut h = Harness::new(me());
        h.open();
        h.act(Act::StartGoal);
        let (key, _) = h.sent();
        h.answer(key, outcome.clone());
        let start = h.pages.start().unwrap();
        assert_eq!(start.pending, None);
        let Some(Failure::Outcome(why)) = &start.error else { panic!("answer preserved") };
        assert_eq!(why, &outcome);
        assert!(h.out.is_empty());
    }
}
#[test]
fn start_needs_name_and_draft() {
    let mut h = Harness::new(me());
    for (draft, chat) in [(None, Some(CHAT)), (Some(Box::from(b" ".as_slice())), Some(CHAT)), (Some(PATH.into()), None)]
    {
        h.act(Act::OpenStart { draft, chat });
        assert!(h.pages.start().is_none(), "only a chat's draft opens the sheet");
    }
    for path in [b"".as_slice(), b" \t"] {
        let mut blank = Harness::new(me());
        blank.act(Act::OpenStart { draft: None, chat: None });
        blank.act(Act::StartDraft { words: path.into() });
        blank.act(Act::StartName { words: b"Named".as_slice().into() });
        blank.act(Act::StartGoal);
        assert!(blank.pages.start().unwrap().pending.is_none());
        assert!(blank.out.is_empty());
    }
    h.open();
    h.act(Act::StartName { words: b" \t".as_slice().into() });
    h.act(Act::StartGoal);
    assert!(h.pages.start().unwrap().pending.is_none());
    assert!(h.out.is_empty());
}
#[test]
fn a_double_press_joins_the_pending_request() {
    let mut h = Harness::new(me());
    h.open();
    h.act(Act::StartGoal);
    let (key, _) = h.sent();
    h.act(Act::StartGoal);
    h.act(Act::StartName { words: b"changed".as_slice().into() });
    h.act(Act::OpenStart { draft: Some(b"another.md".as_slice().into()), chat: Some(CHAT) });
    h.event(Event::Told { told: Told::Pending { key } });
    h.answer(Key { number: key.number.checked_add(1).unwrap(), ..key }, Outcome::Lost);
    assert_eq!(h.pages.start().unwrap().pending, Some(key));
    assert_eq!(h.pages.start().unwrap().name.as_ref(), b"away-mode");
    assert!(h.out.is_empty());
}
#[test]
fn the_words_read_back_through_draft_of() {
    for path in [PATH, b" docs/design/draft/away-mode.md "] {
        for chat in [None, Some(CHAT)] {
            let words = start_goal::words(b"Away mode", path, chat, false);
            let expected: &[u8] = if chat.is_some() {
                b"Away mode\n\nBuild docs/design/draft/away-mode.md.\nDraft: docs/design/draft/away-mode.md\nChat: T4"
            } else {
                b"Away mode\n\nBuild docs/design/draft/away-mode.md.\nDraft: docs/design/draft/away-mode.md"
            };
            assert_eq!(words.as_ref(), expected);
            assert_eq!(pages::goal::draft_of(&words), (Some(PATH.into()), chat));
        }
    }
}
#[test]
fn the_default_way_is_the_first_numbered() {
    let mut me = me();
    let templates = &mut me.projects.get_mut(0).unwrap().templates;
    let base = templates.get(0).unwrap().clone();
    *templates = List::from_boxed(Box::new([
        c::Template { number: 9, ..base.clone() },
        c::Template { number: 1, tracked: None, ..base.clone() },
        c::Template { number: 4, ..base },
    ]));
    assert_eq!(start_goal::default_way(&me, fixtures::PROJECT), Some(4));
    for (configured, expected) in [(None, Some(4)), (Some(9), Some(9)), (Some(1), Some(4)), (Some(99), Some(4))] {
        assert_eq!(start_goal::preferred_way(&me, fixtures::PROJECT, configured), expected);
    }
    assert_eq!(start_goal::default_way(&me, 99), None);
    me.projects.get_mut(0).unwrap().templates.clear();
    assert_eq!(start_goal::default_way(&me, fixtures::PROJECT), None);
}
#[test]
fn cancel_closes() {
    let mut h = Harness::new(me());
    h.open();
    h.act(Act::StartCancel);
    assert!(h.pages.start().is_none());
    assert!(h.out.is_empty());
}
#[test]
fn name_can_be_edited_but_the_given_draft_stays() {
    let mut h = Harness::new(me());
    h.open();
    h.act(Act::StartDraft { words: b"different.md".as_slice().into() });
    assert_eq!(h.pages.start().unwrap().name.as_ref(), b"away-mode");
    h.act(Act::StartName { words: b"Away".as_slice().into() });
    h.act(Act::StartDraft { words: b"different-again.md".as_slice().into() });
    let start = h.pages.start().unwrap();
    assert_eq!(start.name.as_ref(), b"Away");
    assert_eq!(start.draft.as_ref(), PATH);
    assert_eq!(start.given.as_deref(), Some(PATH));
}
#[test]
fn another_address_closes_even_when_only_the_filter_changes() {
    let mut h = Harness::new(me());
    let address = Address::Goals { project: fixtures::PROJECT, filter: crate::GoalsFilter::All };
    h.event(Event::Went { address: address.clone() });
    h.open();
    h.event(Event::Went { address });
    assert!(h.pages.start().is_some());
    h.event(Event::Went { address: Address::Goals { project: fixtures::PROJECT, filter: crate::GoalsFilter::Live } });
    assert!(h.pages.start().is_none());
}
#[test]
fn too_long_text_stays_open_without_sending() {
    for name in [true, false] {
        let mut h = Harness::new(me());
        let mut long = skein_lib::bytes::zeroed(usize::try_from(TEXT).unwrap().checked_add(1).unwrap());
        long.fill(b'x');
        if name {
            h.open();
            h.act(Act::StartName { words: long });
        } else {
            h.act(Act::OpenStart { draft: Some(long), chat: Some(CHAT) });
        }
        h.act(Act::StartGoal);
        let Some(Failure::TooLong) = h.pages.start().unwrap().error else { panic!("text limit refusal") };
        assert!(h.out.is_empty());
    }
    let length = u32::try_from(start_goal::words(b"away-mode", PATH, Some(CHAT), false).len()).unwrap();
    for bound in [length.checked_sub(1).unwrap(), length] {
        let mut h = Harness::new(me());
        h.open();
        h.limits.client.domain.text = bound;
        h.act(Act::StartGoal);
        assert_eq!(h.pages.start().unwrap().pending.is_some(), bound == length);
    }
}
#[test]
fn way_must_be_a_tracked_template() {
    let mut me = me();
    let templates = &mut me.projects.get_mut(0).unwrap().templates;
    let base = templates.get(0).unwrap().clone();
    *templates = List::from_boxed(Box::new([
        base.clone(),
        c::Template { number: 9, ..base.clone() },
        c::Template { number: 1, tracked: None, ..base },
    ]));
    let mut h = Harness::new(me);
    h.open();
    for template in [99, 1] {
        h.act(Act::StartWay { template });
        assert_eq!(h.pages.start().unwrap().way, Some(WAY));
    }
    h.act(Act::StartWay { template: 9 });
    assert_eq!(h.pages.start().unwrap().way, Some(9));
}
#[test]
fn missing_me_does_not_open() {
    let root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut pages = Pages::new();
    for draft in [None, Some(PATH.into())] {
        for chat in [None, Some(CHAT)] {
            pages::step(
                &mut pages,
                &fixtures::env(&fixtures::limits()),
                root.client(),
                Event::Act { act: Act::OpenStart { draft: draft.clone(), chat } },
                &mut Queue::uncapped(),
            );
            assert!(pages.start().is_none());
        }
    }
}

#[test]
fn always_adds_the_line() {
    let mut h = Harness::new(me());
    h.open();
    h.act(Act::StartAsks { always: true });
    h.act(Act::StartGoal);
    let (_, ask) = h.sent();
    let c::Ask::Create { tasks, .. } = ask else { panic!("create") };
    assert!(tasks.get(0).unwrap().spec.words.bytes().ends_with(b"\nAsks: always"));
    h.act(Act::StartAsks { always: false });
    assert!(h.pages.start().unwrap().always, "the sent choice is held while pending");
}
#[test]
fn follow_adds_none() {
    let mut h = Harness::new(me());
    h.open();
    h.act(Act::StartAsks { always: true });
    h.act(Act::StartAsks { always: false });
    h.act(Act::StartGoal);
    let (_, ask) = h.sent();
    let c::Ask::Create { tasks, .. } = ask else { panic!("create") };
    assert!(skein_lib::bytes::find(tasks.get(0).unwrap().spec.words.bytes(), b"Asks:").is_none());
}
#[test]
fn the_default_follows() {
    let mut h = Harness::new(me());
    h.open();
    assert!(!h.pages.start().unwrap().always);
}

#[test]
fn name_follows_draft_until_edited() {
    let mut h = Harness::new(me());
    h.act(Act::OpenStart { draft: None, chat: None });
    for (path, name) in [(PATH, b"away-mode".as_slice()), (b" next/file.md ", b"file")] {
        h.act(Act::StartDraft { words: path.into() });
        let start = h.pages.start().unwrap();
        assert_eq!(start.name.as_ref(), name);
        assert_eq!(start.given, None);
    }
    h.act(Act::StartName { words: b"My goal".as_slice().into() });
    h.act(Act::StartDraft { words: PATH.into() });
    assert_eq!(h.pages.start().unwrap().name.as_ref(), b"My goal");
    h.act(Act::StartGoal);
    h.sent();
    h.act(Act::StartDraft { words: b"ignored.md".as_slice().into() });
    assert_eq!(h.pages.start().unwrap().draft.as_ref(), PATH);
}

#[test]
fn create_without_chat_sends_and_goes_to_its_goal() {
    for always in [false, true] {
        let mut h = Harness::new(me());
        h.act(Act::OpenStart { draft: None, chat: None });
        h.act(Act::StartDraft { words: b" docs/design/draft/away-mode.md ".as_slice().into() });
        h.act(Act::StartAsks { always });
        h.act(Act::StartGoal);
        let (key, ask) = h.sent();
        let c::Ask::Create { tasks, .. } = ask else { panic!("create") };
        assert_eq!(
            tasks.get(0).unwrap().spec.words.bytes(),
            start_goal::words(b"away-mode", PATH, None, always).as_ref()
        );
        h.answer(key, done(c::DocumentOutcome::Created { first: GOAL, last: GOAL, given: None }));
        assert_eq!(h.out.pop(), Some(Request::Show { address: Address::Task { task: GOAL } }));
        assert!(h.out.is_empty(), "no GoalFrom fact without a chat");
    }
}

#[test]
fn no_funds_blocks_unless_the_way_has_an_amount() {
    for (budget, template, reserved, sends) in [
        (0, c::TemplateBudget::Given, 0, false),
        (0, c::TemplateBudget::Given, 1, false),
        (0, c::TemplateBudget::AsAllowed { cap: None }, 0, false),
        (0, c::TemplateBudget::AsAllowed { cap: Some(1) }, 1, false),
        (0, c::TemplateBudget::Amount(1), 1, true),
        (1, c::TemplateBudget::Given, 1, true),
    ] {
        for given in [false, true] {
            let mut me = me();
            let member = me.projects.get_mut(0).unwrap();
            member.creating.as_mut().unwrap().within.budget = budget;
            member.creating.as_mut().unwrap().beyond = c::Beyond::Proposed { to: c::Recipient::Party(fixtures::PARTY) };
            member.templates.get_mut(0).unwrap().budget = template;
            member.pool = Some(c::PoolLedger {
                source: c::Funder::Pool { project: fixtures::PROJECT, person: fixtures::PARTY, period: 1 },
                parent: None,
                numbers: c::Numbers { budget: 1, spent: 0, spent_below: 0, reserved },
                left: budget,
                closed: false,
                began: None,
                next: None,
            });
            let mut h = Harness::new(me);
            if given {
                h.open();
            } else {
                h.act(Act::OpenStart { draft: None, chat: None });
                h.act(Act::StartDraft { words: PATH.into() });
            }
            h.act(Act::StartGoal);
            assert_eq!(h.pages.start().unwrap().pending.is_some(), sends, "{budget} {template:?} {reserved} {given}");
            assert_eq!(h.out.is_empty(), !sends);
        }
    }
}

fn landed() -> pages::work::Landed {
    pages::work::Landed { forge: 3, owner: b"author".as_slice().into(), name: b"project".as_slice().into(), path: PATH.into(),
        at: skein_lib::Wall::from_nanos(FACT_AT), title: b"Away mode".as_slice().into(), listed: true }
}
fn open_landed(h: &mut Harness, start: bool) {
    h.event(Event::Went { address: Address::Work { project: fixtures::PROJECT, filter: crate::WorkFilter::All } });
    h.out = Queue::uncapped(); h.act(Act::OpenLanded { draft: landed(), start });
}
#[test]
fn another_way_holds_the_landed_path_and_create_marks_it_started_after_the_sheet_closes() {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut h = Harness::new(me()); open_landed(&mut h, false);
    let start = h.pages.start().unwrap();
    assert_eq!(start.landed, Some(landed())); assert_eq!(start.given.as_deref(), Some(PATH)); assert_eq!(start.chat, None);
    assert!(start.body.is_none()); assert!(start.pending.is_none()); assert!(h.out.is_empty());
    h.act(Act::StartGoal); let (key, ask) = h.sent();
    let c::Ask::Create { tasks, .. } = ask else { panic!("create from path") };
    assert_eq!(tasks.get(0).unwrap().spec.words.bytes(), b"away-mode\n\nBuild docs/design/draft/away-mode.md.\nDraft: docs/design/draft/away-mode.md");
    h.answer(key, done(c::DocumentOutcome::Created { first: GOAL, last: GOAL, given: None }));
    let Some(Request::Client(jig_client::Event::Send { key: started, name: None, request: jig_client::Ask::Family { scope, record } })) = h.out.pop() else { panic!("keyed started request") };
    assert_ne!(key, started); assert_eq!(scope, c::Scope::Project(fixtures::PROJECT));
    let w::Request::SetDraft(value) = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request else { panic!("SetDraft") };
    assert_eq!(value.project(), fixtures::PROJECT); assert_eq!(value.repository().as_ref().unwrap().owner(), b"author");
    assert_eq!(value.repository().as_ref().unwrap().name(), b"project"); assert_eq!(value.path(), PATH);
    let w::DraftChange::Started(value) = value.change() else { panic!("started") }; assert_eq!(value.goal(), GOAL);
    assert_eq!(h.out.pop(), Some(Request::Show { address: Address::Task { task: GOAL } })); assert!(h.pages.start().is_none());
    h.event(Event::Went { address: Address::Task { task: GOAL } }); h.out = Queue::uncapped();
    h.answer(started, Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None })));
    assert!(h.out.is_empty()); assert!(h.pages.start().is_none()); assert_eq!(h.pages.needs().toast(), None);
}
#[test]
fn recommended_landed_start_sends_once_only_when_the_sheet_is_enabled() {
    for funded in [true, false] {
        let mut me = me(); if !funded { me.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within.budget = 0; }
        let mut h = Harness::new(me); open_landed(&mut h, true);
        assert_eq!(h.pages.start().unwrap().way, Some(WAY)); assert_eq!(h.pages.start().unwrap().pending.is_some(), funded);
        if funded { h.sent(); } else { assert_eq!(start_goal::unfunded(h.pages.me(h.root.client()).unwrap(), fixtures::PROJECT, Some(WAY)), Some(start_goal::Unfunded::Empty)); }
        h.act(Act::StartGoal); assert!(h.out.is_empty());
    }
}
#[test]
fn a_proposed_goal_from_a_landed_draft_does_not_mark_it_started() {
    let to = c::Recipient::Party(fixtures::PARTY);
    let mut h = Harness::new(beyond(me(), c::Beyond::Proposed { to: to.clone() })); open_landed(&mut h, true);
    let (key, ask) = h.sent(); let c::Ask::Propose { .. } = ask else { panic!("proposed") };
    h.answer(key, done(c::DocumentOutcome::Created { first: GOAL, last: GOAL, given: None }));
    assert!(h.pages.start().is_none()); assert_eq!(h.pages.needs().toast(), Some(&Toasted::Proposed { to })); assert!(h.out.is_empty());
}
#[test]
fn the_landed_recommendation_chooses_kind_sevens_way_before_sending() {
    use jig_client::{Down, Readable, Readout, Up};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut limits = fixtures::limits(); limits.client.domain.text = TEXT; limits.client.domain.long_text = TEXT; limits.client.domain.turn_text = TEXT;
    limits.client.domain.handles = 64; limits.client.domain.reads = 32; limits.client.exchanges = 32;
    let mut root = crate::unlinked(&limits, fixtures::SEED); let watch = fixtures::start(&mut root, List::uncapped());
    let mut frame = me(); let templates = &mut frame.projects.get_mut(0).unwrap().templates;
    let mut preferred = templates.get(0).unwrap().clone(); preferred.number = 9; preferred.label = 9; *templates = List::from_boxed(Box::new([templates.get(0).unwrap().clone(), preferred]));
    let out = root_event(&mut root, &limits, crate::Event::Client { event: jig_client::Event::Up { up: Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) } } }); let mut setting = None;
    for request in &out {
        if let crate::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Family { record, .. } } } } = request
            && let w::Read::Setting(key) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() && key.kind() == 7 { setting = Some(*read); }
    }
    let ways = w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { drafts: true, goals: true, changes: true, way: 9 }).unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(ways.measure()).unwrap()); ways.encode(&mut writer).unwrap();
    let answer = w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) }).unwrap();
    root_event(&mut root, &limits, crate::Event::Client { event: jig_client::Event::Up { up: Up::ReadDone {
        read: setting.unwrap(), result: Readout::Family { family: wire::FORGE, record: Some(wire::setting_answer(&answer)) },
    }}});
    root_event(&mut root, &limits, crate::Event::Went { place: crate::Place::Address(Address::Work { project: fixtures::PROJECT, filter: crate::WorkFilter::All }) });
    let out = root_event(&mut root, &limits, crate::Event::Act { action: crate::Action::Pages(Act::OpenLanded { draft: landed(), start: true }) });
    let start = root.pages().start().unwrap(); assert_eq!(start.way, Some(9)); assert!(start.pending.is_some()); let mut sent = false;
    for request in &out {
        if let crate::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { request: jig_client::Ask::Jig(ask), .. } } } = request {
            let c::Ask::Create { tasks, .. } = ask.as_ref() else { panic!("create goal") }; assert_eq!(tasks.get(0).unwrap().label, 9); sent = true;
        }
    }
    assert!(sent);
}

fn root_event(root: &mut crate::Domain, limits: &crate::Limits, event: crate::Event) -> Queue<crate::Request> {
    let mut out = Queue::uncapped(); crate::step(root, &fixtures::env(limits), event, &mut out); out
}
