use crate::state::store;
use crate::tests::fixtures::Profile;
use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{AnsweredByEngine, Fixture, ReadPage, Watched};
pub(in crate::tests) use crate::tests::kit::text;
use crate::tests::state::domain::spine::{LIMITS, drain, frame, open_token};
use crate::{
    Event, Request,
    state::{
        address::{Address, Place},
        intents::send_card,
        objects::Source,
        store::{Record, STORE_VERSION},
    },
    tree::notices::kept::Kind,
};
use jig_client::contract::*;
use skein_lib::{Duration, Env, List, Queue, Time, Wall};
fn limits() -> Profile {
    Profile { view: crate::Config { ..(LIMITS).view }, client: jig_client::DomainLimits { ..(LIMITS).client } }
}
fn drive(
    domain: &mut Fixture,
    limits: &Profile,
    now: Time,
    event: impl crate::tests::fixtures::Input,
) -> Box<[Request]> {
    domain.ui.limits = limits.view;
    domain.drive(now, event)
}
fn tick(domain: &mut Fixture, limits: &Profile, now: Time) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    crate::fire(&mut domain.ui, &Env { now, wall: Wall::EPOCH, limits: limits.view }, &mut out, &domain.client);
    drain(out)
}

#[test]
fn a_minute_counts_a_change_only_while_the_page_shows_wall_times() {
    let limits = limits();
    let mut domain = Fixture::new(&limits);
    start(&mut domain, &limits, List::with_capacity(0));
    assert_eq!(domain.next_deadline(), None);
    domain.timed_page(true);
    assert_eq!(domain.next_deadline(), Some(Time::from_nanos(Duration::from_secs(60).as_nanos())));
    let shown = domain.shown();
    assert!(tick(&mut domain, &limits, Time::from_nanos(Duration::from_secs(59).as_nanos())).is_empty());
    assert_eq!(domain.shown(), shown);
    assert!(tick(&mut domain, &limits, Time::from_nanos(Duration::from_secs(60).as_nanos())).is_empty());
    assert_eq!(domain.shown(), shown.checked_add(1).unwrap());
    drive(
        &mut domain,
        &limits,
        Time::from_nanos(Duration::from_secs(60).as_nanos()),
        Event::Went { place: Place::Elsewhere },
    );
    let shown = domain.shown();
    assert!(tick(&mut domain, &limits, Time::from_nanos(Duration::from_secs(120).as_nanos())).is_empty());
    assert_eq!(domain.shown(), shown);
}

#[test]
fn hidden_answers_have_notices_and_lingers_and_coalesced_saves_end_on_deadlines() {
    let limits = limits();
    let mut domain = Fixture::new(&limits);
    start(&mut domain, &limits, List::with_capacity(0));
    let key = ObjectKey::Task(8);
    domain.objects.name(key, Source::Window { number: 1 }).unwrap();
    let mut out = Queue::uncapped();
    let request = send_card(
        &mut domain.ui,
        key,
        ActKind::Stop,
        Box::new(Ask::Stop { project: 1, task: 8 }),
        &mut out,
        &domain.client,
    )
    .unwrap();
    drive(&mut domain, &limits, Time::ZERO, Event::Went { place: Place::Elsewhere });
    drive(&mut domain, &limits, Time::ZERO, AnsweredByEngine { request, answer: refusal_answer(Refusal::NotOffered) });
    let notice = *domain.notices().ids().first().unwrap().0;
    let Kind::Answer { object, .. } = &domain.notices().get(notice).unwrap().kind else {
        panic!("hidden card outcome");
    };
    assert_eq!(*object, key);
    assert_eq!(domain.notices().get(notice).unwrap().until, Time::from_nanos(Duration::from_secs(5).as_nanos()));
    let id = domain.objects.name(ObjectKey::Task(9), Source::Window { number: 1 }).unwrap();
    assert!(domain.leave(id, Ending::Cancelled { by: Actor::Party(7), reason: Some(text(b"done")), at: None }));
    out = Queue::uncapped();
    assert!(store::coalesce(
        &mut domain,
        Record::Draft { version: STORE_VERSION, party: 7, task: 8, words: text(b"first"), touched: 1 },
        &mut out
    ));
    drive(&mut domain, &limits, Time::from_nanos(500_000_000), crate::tests::fixtures::Tick);
    let newest = Record::Draft { version: STORE_VERSION, party: 7, task: 8, words: text(b"newest"), touched: 1 };
    assert!(store::coalesce(&mut domain, newest.clone(), &mut out));
    assert!(out.is_empty());
    assert!(tick(&mut domain, &limits, Time::from_nanos(1_499_999_999)).is_empty());
    assert_eq!(
        tick(&mut domain, &limits, Time::from_nanos(1_500_000_000)).as_ref(),
        &[Request::Save { records: crate::tests::fixtures::one(newest) }]
    );
    tick(&mut domain, &limits, Time::from_nanos(5_000_000_000));
    tick(&mut domain, &limits, Time::from_nanos(5_000_000_000));
    assert!(domain.notices().ids().is_empty());
    assert!(domain.objects().find(ObjectKey::Task(9)).is_none());
}

fn start(domain: &mut Fixture, limits: &Profile, kept: List<Record>) -> (jig_client::Handle, jig_client::Handle) {
    let f = open_token(&drive(
        domain,
        limits,
        Time::ZERO,
        Event::Start { seed: 8, place: Place::Jig(Address::Inbox { project: None }), records: kept },
    ));
    let mut source = frame(7);
    source.party.login = DocumentText::complete(text(b"L"));
    let initial = drive(
        domain,
        limits,
        Time::ZERO,
        Watched { watch: f, event: WatchEvent::Delivered(Delivery::Me(Box::new(source))) },
    );
    let page = open_token(&initial);
    for request in &initial {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Inbox { .. }) },
        } = request
        {
            drive(
                domain,
                limits,
                Time::ZERO,
                ReadPage {
                    read: *read,
                    result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                        selection: None,
                        cut: false,
                        rows: List::with_capacity(0),
                        before: None,
                        after: None,
                    }))),
                },
            );
        }
    }
    domain.reclaim();
    (f, page)
}
