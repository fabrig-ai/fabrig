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
    let sends = sends(out);
    assert_eq!(sends.len(), 1, "one SetAway");
    sends.get(0).expect("SetAway sent").clone()
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
fn told(root: &mut Domain, key: Key, outcome: Outcome) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut out);
    out
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
    let mut root = three_projects(&[]);
    act(&mut root, A::Toggle { project: f::PROJECT }, Wall::EPOCH);
    for next in [
        A::ToggleAll,
        A::Toggle { project: f::PROJECT },
        A::Kinds { project: 17, kinds: 4 },
        A::Until { project: 17, eight: false },
    ] {
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

// The frame lists the open project, OTHER's project, and a third project.
const THIRD: u32 = 23;
const PROJECTS: [u32; 3] = [f::PROJECT, OTHER.project, THIRD];
fn three_projects(rows: &[AwayRow]) -> Domain {
    let (mut root, watch, frame) = opened();
    let mut me = f::chats::me();
    let owner = me.projects.get(0).unwrap().clone();
    me.projects = List::uncapped();
    for project in PROJECTS {
        let mut member = owner.clone();
        member.project = project;
        me.projects.push(member).unwrap();
    }
    up(&mut root, Up::Watched { watch: frame, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) });
    up(&mut root, Up::FamilyDelivered { watch, record: row_record(rows) });
    root
}
fn sends(out: &Queue<Request>) -> List<w::SetAwayParts> {
    let mut found = List::uncapped();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Send { request: Ask::Family { record, scope }, value: payload, .. },
                },
        } = request
        {
            let w::Request::SetAway(value) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
            else {
                continue;
            };
            assert!(payload.is_none());
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            assert_eq!(*scope, c::Scope::Project(value.project()));
            found.push(value.into_parts()).expect("sent requests grow");
        }
    }
    found
}
#[test]
fn across_counts_the_projects_on() {
    use crate::pages::away_all::{Across, across};
    let all = [
        AwayRow { until: Some(minute(480)), ..WHAT },
        OTHER,
        AwayRow { project: THIRD, until: Some(minute(600)), ..WHAT },
    ];
    for (name, rows, wall, expected) in [
        ("no rows", Some([].as_slice()), minute(479), Across { projects: 3, on: 0, until: None, shown: false }),
        ("one on", Some([WHAT].as_slice()), minute(479), Across { projects: 3, on: 1, until: None, shown: true }),
        (
            "all on",
            Some(all.as_slice()),
            minute(479),
            Across { projects: 3, on: 3, until: Some(minute(480)), shown: true },
        ),
        (
            "at until",
            Some(all.as_slice()),
            minute(480),
            Across { projects: 3, on: 2, until: Some(minute(600)), shown: true },
        ),
        ("me undelivered", None, minute(479), Across { projects: 0, on: 0, until: None, shown: false }),
    ] {
        let root = match rows {
            Some(rows) => three_projects(rows),
            None => crate::unlinked(&f::limits(), f::SEED),
        };
        assert_eq!(across(&root, wall), expected, "{name}");
    }
}
fn away_answers(count: u32) -> Box<[Outcome]> {
    let mut answers = List::uncapped();
    for _ in 0..count {
        answers.push(answer(w::Outcome::Away)).expect("fixture answers grow");
    }
    answers.into_boxed()
}
#[test]
fn toggle_all_sets_each_project_that_differs() {
    let mut all = List::uncapped();
    for project in PROJECTS {
        all.push(AwayRow { project, until: Some(minute(480)), kinds: 8, ..WHAT }).expect("fixture projects grow");
    }
    let refused = || {
        answer(w::Outcome::ForgeRefused(
            w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::TooLarge }).unwrap(),
        ))
    };
    for (name, rows, projects, state, outcomes, expected) in [
        (
            "none on",
            Box::from([]),
            PROJECTS.as_slice(),
            (true, Some(minute(480).as_nanos()), 7),
            away_answers(3),
            Toasted::AwayAll { on: true, changed: 3, sent: 3 },
        ),
        (
            "one on",
            Box::from([WHAT]),
            [OTHER.project, THIRD].as_slice(),
            (true, Some(minute(480).as_nanos()), 7),
            away_answers(2),
            Toasted::AwayAll { on: true, changed: 2, sent: 2 },
        ),
        (
            "all on",
            all.into_boxed(),
            PROJECTS.as_slice(),
            (false, Some(minute(480).as_nanos()), 8),
            away_answers(3),
            Toasted::AwayAll { on: false, changed: 3, sent: 3 },
        ),
        (
            "one refused",
            Box::from([]),
            PROJECTS.as_slice(),
            (true, Some(minute(480).as_nanos()), 7),
            Box::from([answer(w::Outcome::Away), answer(w::Outcome::Unknown), answer(w::Outcome::Away)]),
            Toasted::AwayAll { on: true, changed: 2, sent: 3 },
        ),
        (
            "none changed",
            Box::from([]),
            PROJECTS.as_slice(),
            (true, Some(minute(480).as_nanos()), 7),
            Box::from([Outcome::Lost, answer(w::Outcome::Dismissed), refused()]),
            Toasted::AwayRefused,
        ),
    ] {
        let mut root = three_projects(&rows);
        let first = root.pages.serial.checked_add(1).unwrap();
        let mut out = act(&mut root, A::ToggleAll, minute(479));
        let mut collected = List::uncapped();
        for (index, outcome) in outcomes.into_iter().enumerate() {
            assert!(root.pages().away_busy(), "{name}: busy until the last answer");
            assert_eq!(root.pages().needs().toast(), None, "{name}: no early toast");
            for next in [
                A::Toggle { project: f::PROJECT },
                A::ToggleAll,
                A::Kinds { project: OTHER.project, kinds: 4 },
                A::Until { project: THIRD, eight: false },
            ] {
                assert!(act(&mut root, next, minute(479)).is_empty(), "{name}: competing act waits");
            }
            let key = Key { asker: crate::owner::Owner::Pages.asker(), number: first + u64::try_from(index).unwrap() };
            assert!(told(&mut root, Key { number: key.number + 99, ..key }, answer(w::Outcome::Away)).is_empty());
            assert!(root.pages().away_busy(), "{name}: unrelated answer");
            assert_eq!(root.pages().needs().toast(), None, "{name}: unrelated answer");
            let batch = sends(&out);
            assert_eq!(batch.len(), 1, "{name}: one request admitted at a time");
            let send = batch.get(0).expect("one request sent");
            assert_eq!(
                send.project,
                *projects.get(index).expect("one project per answer"),
                "{name}: changed projects only"
            );
            assert_eq!((send.on, send.until, send.kinds), state, "{name}: request state");
            let mut lone = three_projects(&rows);
            assert_eq!(
                *send,
                sent(&act(&mut lone, A::Toggle { project: send.project }, minute(479))),
                "{name}: same project request"
            );
            collected.push(send.clone()).expect("sent requests grow");
            out = told(&mut root, key, outcome);
        }
        assert!(out.is_empty(), "{name}: no requests left");
        assert_eq!(usize::try_from(collected.len()).unwrap(), projects.len(), "{name}");
        assert_eq!(root.pages().needs().toast(), Some(&expected), "{name}");
        assert!(!root.pages().away_busy(), "{name}: settled");
    }
}
