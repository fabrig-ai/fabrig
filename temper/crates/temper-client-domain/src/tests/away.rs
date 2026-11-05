use super::fixtures as f;
use crate::{
    Domain, Event, Request,
    forge::{
        self,
        away::{self, AwayRow},
    },
};
use jig_client::{Ask, Down, Family, Link, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

use skein_lib::Wall;
const WHAT: AwayRow = AwayRow { project: f::PROJECT, since: Wall::EPOCH, until: None, kinds: 7, landed: 0, off: None };
const OTHER: AwayRow = AwayRow { project: 17, ..WHAT };

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn opening(out: &mut Queue<Request>) -> Option<Token> {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && subject == away::watch(&f::limits().forge).unwrap()
        {
            assert!(found.is_none(), "one Away watch");
            found = Some(watch);
        }
    }
    found
}
fn opened() -> (Domain, Token, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let frame = f::start(&mut root, List::with_capacity(0));
    let mut out = Queue::uncapped();
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    let watch = opening(&mut out).expect("Away watch");
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    up(
        &mut root,
        Up::FamilyServed {
            family: wire::FORGE,
            versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
        },
    );
    (root, watch, frame)
}
fn record(projects: &[u32]) -> Family {
    let mut rows = List::uncapped();
    for project in projects {
        rows.push(AwayRow { project: *project, ..WHAT }).unwrap();
    }
    row_record(rows.as_slice())
}
fn row_record(values: &[AwayRow]) -> Family {
    let mut rows = List::with_capacity(w::CEILINGS.aways_rows);
    for row in values {
        rows.push(
            w::AwayRow::new(
                &w::CEILINGS,
                w::AwayRowParts {
                    project: row.project,
                    since: row.since.as_nanos(),
                    until: nanos(row.until),
                    kinds: row.kinds,
                    landed: row.landed,
                    off: nanos(row.off),
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::aways(&w::Aways::new(&w::CEILINGS, w::AwaysParts { rows }).unwrap()),
    }
}

fn deliver(root: &mut Domain, watch: Token, projects: &[u32]) -> Queue<Request> {
    up(root, Up::FamilyDelivered { watch, record: record(projects) })
}
fn closes(out: &mut Queue<Request>, expected: Token) -> u32 {
    let mut count = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
            && watch == expected
        {
            count = count.checked_add(1).expect("bounded close count");
        }
    }
    count
}
fn link(root: &mut Domain, link: Link) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Link { link }, &mut out);
    out
}

#[test]
fn the_watch_opens_once_for_a_party() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let frame = f::start(&mut root, List::with_capacity(0));
    let mut child = Queue::uncapped();
    away::configure(&mut root.forge, f::limits().forge, &mut child);
    assert!(child.is_empty(), "no party yet");
    let mut out = Queue::uncapped();
    let Watchable::Family { scope, window, record } = away::watch(&f::limits().forge).unwrap() else {
        panic!("family watch")
    };
    assert_eq!(scope, c::Scope::Own);
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch(), &w::Watch::Away);
    assert_eq!(away::handle().asker, f::FORGE_ASKER);
    assert_eq!(away::handle().number, 4);
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    assert!(opening(&mut out).is_some());
    assert!(root.client().document(away::handle()).is_some());
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    assert_eq!(opening(&mut out), None, "same party opens no second watch");
    assert_eq!(opening(&mut link(&mut root, Link::Offline)), None);
    assert_eq!(opening(&mut link(&mut root, Link::Live)), None);
}

#[test]
fn rows_are_delivered_by_project() {
    let (mut root, watch, _) = opened();
    assert_eq!(root.forge().away(WHAT.project), None);
    let shown = forge::shown(root.forge());
    let row =
        AwayRow { since: minute(3), until: Some(minute(500)), kinds: 9, landed: 42, off: Some(minute(6)), ..WHAT };
    up(&mut root, Up::FamilyDelivered { watch, record: row_record(&[row, OTHER]) });
    assert_eq!(root.forge().away(WHAT.project), Some(row));
    assert_eq!(root.forge().away(OTHER.project), Some(OTHER));
    assert_eq!(root.forge().away(99), None);
    assert_ne!(forge::shown(root.forge()), shown);
}

#[test]
fn a_later_document_replaces_the_facts() {
    let (mut root, watch, _) = opened();
    deliver(&mut root, watch, &[f::PROJECT]);
    deliver(&mut root, watch, &[17]);
    assert_eq!(root.forge().away(WHAT.project), None);
    assert_eq!(root.forge().away(OTHER.project), Some(OTHER));
    deliver(&mut root, watch, &[]);
    assert_eq!(root.forge().away(WHAT.project), None);
    assert_eq!(root.forge().away(OTHER.project), None);
}

#[test]
fn an_undecodable_document_ends_the_facts() {
    for bad in [
        Family { bytes: b"bad".as_slice().into(), ..record(&[]) },
        Family { family: 99, ..record(&[]) },
        Family { version: wire::CLIENT_VERSION + 1, ..record(&[]) },
    ] {
        let (mut root, watch, frame) = opened();
        deliver(&mut root, watch, &[f::PROJECT]);
        let mut out = up(&mut root, Up::FamilyDelivered { watch, record: bad });
        assert_eq!(root.forge().away(WHAT.project), None);
        assert_eq!(closes(&mut out, watch), 1);
        f::confirm(&mut root, frame, f::PARTY, &mut out);
        assert_eq!(opening(&mut out), None, "invalid documents stay ended");
        deliver(&mut root, watch, &[f::PROJECT]);
        assert_eq!(root.forge().away(WHAT.project), None, "late documents stay ended");
    }
}

#[test]
fn a_refused_watch_stays_refused() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let (mut root, watch, frame) = opened();
        deliver(&mut root, watch, &[f::PROJECT]);
        let mut out = up(&mut root, Up::WatchEnded { watch, end });
        assert_eq!(root.forge().away(WHAT.project), None, "{end:?}");
        assert_eq!(opening(&mut out), None);
        f::confirm(&mut root, frame, f::PARTY, &mut out);
        assert_eq!(opening(&mut out), None, "ended watch does not reopen");
        assert_eq!(f::sends(&mut act(&mut root, A::Toggle { project: f::PROJECT }, Wall::EPOCH)), 0);
        assert!(!root.pages().away_busy());
        assert_eq!(root.pages().needs().toast(), None);
        assert_eq!(opening(&mut link(&mut root, Link::Offline)), None);
        assert_eq!(opening(&mut link(&mut root, Link::Live)), None);
        up(&mut root, Up::SignedOut);
        assert_eq!(root.forge().away(WHAT.project), None);
        let mut signed_in = up(&mut root, Up::SignedIn);
        let mut frame = None;
        let mut next = None;
        while let Some(request) = signed_in.pop() {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } },
            } = request
            {
                if subject == Watchable::Jig(c::Subject::Me) {
                    frame = Some(watch);
                }
                if subject == away::watch(&f::limits().forge).unwrap() {
                    next = Some(watch);
                }
            }
        }
        let frame = frame.expect("next sign-in opens me");
        f::confirm(&mut root, frame, f::PARTY, &mut out);
        let next = opening(&mut out).or(next).expect("next sign-in opens Away");
        assert_ne!(next, watch);
        deliver(&mut root, next, &[]);
        assert_eq!(root.forge().away(WHAT.project), None);
    }
}

#[test]
fn signing_out_closes_the_watch_and_clears_the_facts() {
    for ending in [Link::SignedOut, Link::OutOfDate] {
        let (mut root, watch, _) = opened();
        deliver(&mut root, watch, &[f::PROJECT]);
        let mut out = link(&mut root, ending);
        assert_eq!(root.forge().away(WHAT.project), None);
        assert_eq!(closes(&mut out, watch), 1);
        assert_eq!(opening(&mut link(&mut root, ending)), None);
    }
}

use crate::pages::{away::Act as A, needs::Toasted};
use jig_client::{Key, Outcome};

fn minute(value: u64) -> Wall {
    Wall::from_nanos(value.checked_mul(60_000_000_000).unwrap())
}
#[test]
fn on_at_reads_off_and_until() {
    assert!(WHAT.on_at(Wall::EPOCH));
    assert!(!AwayRow { off: Some(minute(3)), ..WHAT }.on_at(minute(2)));
    let row = AwayRow { until: Some(minute(480)), ..WHAT };
    assert!(row.on_at(minute(479)));
    assert!(!row.on_at(minute(480)));
    assert!(!row.on_at(minute(481)));
}
#[test]
fn ended_is_off_or_until_once_past() {
    assert_eq!(WHAT.ended(minute(500)), None);
    let row = AwayRow { until: Some(minute(480)), ..WHAT };
    assert_eq!(row.ended(minute(479)), None);
    assert_eq!(row.ended(minute(480)), Some(minute(480)));
    assert_eq!(row.ended(minute(481)), Some(minute(480)));
    assert_eq!(AwayRow { off: Some(minute(450)), ..row }.ended(minute(500)), Some(minute(450)));
}
#[test]
fn next_eight_is_todays_before_eight_and_tomorrows_after() {
    assert_eq!(away::next_eight(minute(479), 0), minute(480));
    assert_eq!(away::next_eight(Wall::from_nanos(minute(480).as_nanos() - 1), 0), minute(480));
    assert_eq!(away::next_eight(minute(480), 0), minute(1920));
    assert_eq!(away::next_eight(minute(481), 0), minute(1920));
}
#[test]
fn next_eight_reads_the_zone() {
    // Minutes in universal time, immediately before and at each zone's 08:00.
    for (zone, eight) in [(120_i32, 360), (-300_i32, 780), (345_i32, 135)] {
        assert_eq!(away::next_eight(minute(eight - 1), zone), minute(eight));
        assert_eq!(away::next_eight(minute(eight), zone), minute(eight + 1440));
    }
    // The negative zone is still in the day before the epoch.
    assert_eq!(away::next_eight(Wall::EPOCH, -300), minute(780));
    assert_eq!(away::next_eight(Wall::EPOCH, 345), minute(135));
    // A positive zone has already crossed into tomorrow.
    assert_eq!(away::next_eight(minute(1439), 120), minute(1800));
}
fn ready(rows: &[AwayRow]) -> Domain {
    let (mut root, watch, _) = opened();
    up(&mut root, Up::FamilyDelivered { watch, record: row_record(rows) });
    root
}
fn act(root: &mut Domain, act: A, wall: Wall) -> Queue<Request> {
    let mut env = f::env(&f::limits());
    env.wall = wall;
    let mut out = Queue::uncapped();
    crate::step(root, &env, Event::Act { action: crate::Action::Pages(crate::pages::Act::Away(act)) }, &mut out);
    out
}
fn sent(out: &Queue<Request>) -> w::SetAwayParts {
    let mut found = None;
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Send { request: Ask::Family { scope, record }, value, .. } },
        } = request
        {
            assert!(found.is_none(), "one SetAway");
            assert!(value.is_none());
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            let w::Request::SetAway(value) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
            else {
                panic!("SetAway request")
            };
            assert_eq!(*scope, c::Scope::Project(value.project()));
            found = Some(value.into_parts());
        }
    }
    found.expect("SetAway sent")
}
fn pending_key(root: &Domain) -> Key {
    Key { asker: crate::owner::Owner::Pages.asker(), number: root.pages.serial }
}
fn answer(outcome: w::Outcome) -> Outcome {
    Outcome::Answered(jig_client::Answered::Family(Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
    }))
}
fn told(root: &mut Domain, key: Key, outcome: Outcome) {
    crate::route::told(root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut Queue::uncapped());
}
#[test]
fn toggle_on_sends_defaults_and_the_next_eight() {
    let mut root = ready(&[]);
    root.set_zone_minutes(120);
    let send = sent(&act(&mut root, A::Toggle { project: f::PROJECT }, minute(359)));
    assert_eq!(send, w::SetAwayParts { project: f::PROJECT, on: true, until: Some(minute(360).as_nanos()), kinds: 7 });
    assert!(root.pages().away_busy());
    assert_eq!(root.pages().needs().toast(), None, "toast waits for the answer");
    assert!(!root.away_on(f::PROJECT, minute(359)), "no optimistic row");
}
#[test]
fn toggle_on_keeps_a_row_whose_until_is_none() {
    let mut root = ready(&[AwayRow { off: Some(minute(1)), kinds: 12, ..WHAT }]);
    let send = sent(&act(&mut root, A::Toggle { project: f::PROJECT }, minute(500)));
    assert_eq!((send.on, send.until, send.kinds), (true, None, 12));
}
#[test]
fn toggle_off_keeps_the_row() {
    let mut root = ready(&[AwayRow { until: Some(minute(480)), kinds: 8, ..WHAT }]);
    let send = sent(&act(&mut root, A::Toggle { project: f::PROJECT }, minute(479)));
    assert_eq!((send.on, send.until, send.kinds), (false, Some(minute(480).as_nanos()), 8));
}
#[test]
fn a_second_toggle_waits() {
    let mut root = ready(&[]);
    act(&mut root, A::Toggle { project: f::PROJECT }, Wall::EPOCH);
    for next in
        [A::Toggle { project: f::PROJECT }, A::Kinds { project: 17, kinds: 4 }, A::Until { project: 17, eight: false }]
    {
        assert!(act(&mut root, next, Wall::EPOCH).is_empty());
    }
    assert!(root.pages().away_busy());
    let key = pending_key(&root);
    told(&mut root, key, answer(w::Outcome::Away));
    assert!(!root.pages().away_busy());
    assert!(sent(&act(&mut root, A::Toggle { project: f::PROJECT }, Wall::EPOCH)).on);
}
#[test]
fn kinds_keep_on_as_it_reads() {
    for (row, wall, on, until) in [
        (Some(AwayRow { until: Some(minute(480)), ..WHAT }), minute(479), true, Some(minute(480))),
        (Some(AwayRow { until: Some(minute(480)), ..WHAT }), minute(480), false, Some(minute(480))),
        (Some(AwayRow { off: Some(minute(1)), ..WHAT }), minute(2), false, None),
        (None, minute(479), false, Some(minute(480))),
    ] {
        let mut root = ready(row.as_slice());
        let send = sent(&act(&mut root, A::Kinds { project: f::PROJECT, kinds: 10 }, wall));
        assert_eq!((send.on, send.until, send.kinds), (on, nanos(until), 10));
    }
}
#[test]
fn until_sends_the_choice() {
    for row in [
        None,
        Some(WHAT),
        Some(AwayRow { off: Some(minute(1)), kinds: 12, ..WHAT }),
        Some(AwayRow { until: Some(minute(480)), ..WHAT }),
    ] {
        for eight in [true, false] {
            let mut root = ready(row.as_slice());
            root.set_zone_minutes(-300);
            let send = sent(&act(&mut root, A::Until { project: f::PROJECT, eight }, minute(480)));
            assert_eq!(
                send.on,
                match row {
                    Some(row) => row.on_at(minute(480)),
                    None => false,
                }
            );
            assert_eq!(send.until, if eight { Some(minute(780).as_nanos()) } else { None });
            assert_eq!(
                send.kinds,
                match row {
                    Some(row) => row.kinds,
                    None => 7,
                }
            );
        }
    }
}
#[test]
fn past_until_toggle_turns_on() {
    let mut root = ready(&[AwayRow { until: Some(minute(480)), kinds: 3, ..WHAT }]);
    let send = sent(&act(&mut root, A::Toggle { project: f::PROJECT }, minute(480)));
    assert_eq!((send.on, send.until, send.kinds), (true, Some(minute(1920).as_nanos()), 3));
}
#[test]
fn the_toast_follows_the_answer() {
    for (rows, outcome, expected) in [
        (Box::from([]), answer(w::Outcome::Away), Toasted::AwayOn { project: f::PROJECT, until: Some(minute(480)) }),
        (
            Box::from([AwayRow { off: Some(minute(1)), ..WHAT }]),
            answer(w::Outcome::Away),
            Toasted::AwayOn { project: f::PROJECT, until: None },
        ),
        (Box::from([WHAT]), answer(w::Outcome::Away), Toasted::AwayOff),
        (Box::from([]), Outcome::Unknown, Toasted::AwayRefused),
        (Box::from([]), answer(w::Outcome::Unknown), Toasted::AwayRefused),
        (Box::from([]), Outcome::Lost, Toasted::AwayRefused),
        (
            Box::from([]),
            answer(w::Outcome::ForgeRefused(
                w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::TooLarge }).unwrap(),
            )),
            Toasted::AwayRefused,
        ),
        (Box::from([]), answer(w::Outcome::Dismissed), Toasted::AwayRefused),
    ] {
        let mut root = ready(&rows);
        act(&mut root, A::Toggle { project: f::PROJECT }, minute(2));
        let key = pending_key(&root);
        told(&mut root, Key { number: key.number + 1, ..key }, answer(w::Outcome::Away));
        assert_eq!(root.pages().needs().toast(), None, "another answer makes no toast");
        assert!(root.pages().away_busy());
        told(&mut root, key, outcome);
        assert_eq!(root.pages().needs().toast(), Some(&expected));
        assert!(!root.pages().away_busy());
        // Duplicate answers cannot replace the settled toast.
        told(&mut root, key, Outcome::Lost);
        assert_eq!(root.pages().needs().toast(), Some(&expected));
    }
}
#[test]
fn limits_raises_the_focus_count() {
    let mut root = ready(&[]);
    assert_eq!(root.pages.away_focus(), 0);
    let mut out = act(&mut root, A::Limits { project: 17 }, Wall::EPOCH);
    assert_eq!(out.pop(), Some(Request::Show { address: crate::Address::Settings { project: 17 }, replace: false }));
    assert!(out.is_empty());
    event(&mut root, Event::Went { place: crate::Place::Address(crate::Address::Settings { project: 17 }) });
    assert_eq!(root.pages.away_focus(), 1);
    assert_eq!(root.pages.away_focus(), 1);
    act(&mut root, A::Limits { project: 17 }, Wall::EPOCH);
    assert_eq!(root.pages.away_focus(), 2);
}
#[test]
fn shown_by_goal_by_on_or_by_ways() {
    let (mut root, away_watch, me_watch) = opened();
    assert!(!root.away_shown(f::PROJECT, Wall::EPOCH));
    up(
        &mut root,
        Up::FamilyDelivered { watch: away_watch, record: row_record(&[AwayRow { until: Some(minute(480)), ..WHAT }]) },
    );
    assert!(root.away_shown(f::PROJECT, minute(479)));
    assert!(!root.away_shown(f::PROJECT, minute(480)));
    assert!(!root.away_shown(17, minute(479)));
    let mut me = f::chats::me();
    let template = me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap();
    template.number = jig_client::shapes::GOAL;
    template.tracked = Some(1);
    let mut out =
        up(&mut root, Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) });
    let mut goals = None;
    let mut ways = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request {
            match down {
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { window: 1, .. }) } => {
                    goals = Some(watch);
                }
                Down::Read { read, what: jig_client::Readable::Family { record, .. } } => {
                    if let w::Read::Setting(setting) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
                        && setting.kind() == 7
                    {
                        ways = Some(read);
                    }
                }
                Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. }
                | Down::Open { .. }
                | Down::Continue { .. }
                | Down::Close { .. }
                | Down::Read { .. } => {}
            }
        }
    }
    let goal_watch = goals.expect("goals fact");
    let mut rows = List::with_capacity(1);
    rows.push(goal_line()).unwrap();
    up(
        &mut root,
        Up::Watched {
            watch: goal_watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false }))),
        },
    );
    assert!(root.has_goals(f::PROJECT));
    assert!(root.away_shown(f::PROJECT, minute(480)), "goal alone");
    up(
        &mut root,
        Up::Watched {
            watch: goal_watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                rows: List::with_capacity(0),
                more: false,
            }))),
        },
    );
    assert!(!root.away_shown(f::PROJECT, minute(480)));
    let project_ways =
        w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { skein_bools: [false, false, false, true], way: 0 })
            .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(project_ways.measure()).unwrap());
    project_ways.encode(&mut writer).unwrap();
    let record = wire::setting_answer(
        &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) }).unwrap(),
    );
    up(
        &mut root,
        Up::ReadDone {
            read: ways.expect("ways read"),
            result: jig_client::Readout::Family { family: wire::FORGE, record: Some(record) },
        },
    );
    assert!(root.away_shown(f::PROJECT, minute(480)), "Ways alone");
    assert!(!root.away_shown(17, minute(480)));
}

fn goal_line() -> c::TaskLine {
    let task = f::chat::task();
    let budget = task.chip.details.details.budget.clone();
    c::TaskLine {
        chip: task.chip,
        project: f::PROJECT,
        requester: c::Requester::Person(f::PARTY),
        tracked: Some(1),
        label: f::chat::LABEL,
        flags: c::TaskFlags {
            ended_by: c::EndedBy::Result,
            conversation: false,
            listed: true,
            retention: c::Retention::Kept,
        },
        changed: Wall::EPOCH,
        last_activity: Wall::EPOCH,
        ended_at: None,
        budget,
        waits_on: c::WaitsOn::Nothing,
        phases: c::PhaseCounts::ZERO,
        delegates_done: 0,
        delegates_total: 0,
        held_below: 0,
        conversation: None,
    }
}

fn nanos(wall: Option<Wall>) -> Option<u64> {
    match wall {
        Some(wall) => Some(wall.as_nanos()),
        None => None,
    }
}
