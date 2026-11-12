//! Session settings, project reads and immediate edits (client/pages/settings.md, 2).
use super::fixtures as f;
use crate::{
    Address, Domain, Event, Place, Request,
    forge::{
        self,
        ways::{self, Act, Asked, Control, Route, Ways},
    },
};
use jig_client::{Down, Link, Readable, Readout, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_protocol_reaches::{Class, Classes, Item, Kind, Subject, Way};
const OTHER: u32 = 17;
const NEWEST: u32 = 8;
const NEXT: u32 = 9;
const SHOWN: Ways = Ways { drafts: true, goals: true, changes: true, way: 3 };

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn go(root: &mut Domain, address: Address) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(address) })
}
fn opening(out: &Queue<Request>) -> Option<Token> {
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && *subject == Watchable::Jig(c::Subject::Settings)
        {
            return Some(*watch);
        }
    }
    None
}
fn reads(out: &Queue<Request>, kind: u32) -> Box<[(Token, u32)]> {
    let mut reads = List::with_capacity(f::limits().client.domain.reads);
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Read { read, what: Readable::Family { scope, record, window } },
                },
        } = request
            && let w::Read::Setting(value) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
            && value.kind() == kind
        {
            assert_eq!(*scope, c::Scope::Project(value.scope()));
            assert_eq!(*window, c::Window { rows: 0, after: None });
            assert_eq!((record.family, record.version, value.number()), (wire::FORGE, wire::CLIENT_VERSION, 1));
            reads.push((*read, value.scope())).unwrap();
        }
    }
    reads.into_boxed()
}
fn me() -> c::Me {
    let mut me = f::chats::me();
    let mut other = me.projects.get(0).unwrap().clone();
    other.project = OTHER;
    let mut projects = List::with_capacity(2);
    projects.push(me.projects.get(0).unwrap().clone()).unwrap();
    projects.push(other).unwrap();
    me.projects = projects;
    me
}
struct Fixture {
    root: Domain,
    settings: Token,
    me: Token,
    read: Token,
    classes_read: Token,
}
fn opened() -> (Fixture, Queue<Request>) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me_watch = f::start(&mut root, List::with_capacity(0));
    up(
        &mut root,
        Up::FamilyServed {
            family: wire::FORGE,
            versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
        },
    );
    let out = up(
        &mut root,
        Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me()))) },
    );
    let settings = opening(&out).expect("session settings");
    let read = reads(&out, 7).first().expect("project ways read").0;
    let classes_read = reads(&out, 8).first().expect("project classes read").0;
    (Fixture { root, settings, me: me_watch, read, classes_read }, out)
}
fn fixture() -> Fixture {
    let (mut f, _) = opened();
    document(&mut f, NEWEST, &[]);
    f
}
fn document(f: &mut Fixture, newest: u32, lines: &[(u32, u32, Option<u32>, c::SettingFollows)]) -> Queue<Request> {
    let mut settings = List::with_capacity(u32::try_from(lines.len()).unwrap());
    for (kind, project, last, follows) in lines {
        settings
            .push(c::SettingLine {
                setting: c::Setting::Application { kind: *kind, scope: *project, number: 1 },
                name: None,
                last: *last,
                follows: follows.clone(),
                application: None,
            })
            .unwrap();
    }
    up(
        &mut f.root,
        Up::Watched {
            watch: f.settings,
            event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest,
                settings,
                held: List::with_capacity(0),
                pending: List::with_capacity(0),
            }))),
        },
    )
}
fn value(asked: Asked) -> Box<[u8]> {
    let writer = match asked {
        Asked::Ways(value) => {
            let record = w::ProjectWays::new(
                &w::CEILINGS,
                w::ProjectWaysParts {
                    drafts: value.drafts,
                    goals: value.goals,
                    changes: value.changes,
                    way: value.way,
                },
            )
            .unwrap();
            let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
            record.encode(&mut writer).unwrap();
            writer
        }
        Asked::Classes(classes) => {
            let record = temper_protocol_reaches::classes_record(&classes).unwrap();
            let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
            record.encode(&mut writer).unwrap();
            writer
        }
        Asked::Questions(route) => {
            let record = w::ProjectQuestions::new(
                &w::CEILINGS,
                w::ProjectQuestionsParts {
                    route: match route {
                        Route::None => w::QuestionsRoute::None,
                        Route::Up => w::QuestionsRoute::Up,
                    },
                },
            )
            .unwrap();
            let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
            record.encode(&mut writer).unwrap();
            writer
        }
    };
    // A whole typed setting, inside the forge's setting envelope.
    let bytes = writer.finish();
    wire::setting_answer(&w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(bytes) }).unwrap())
}
fn page(root: &mut Domain, read: Token, record: Option<Box<[u8]>>) -> Queue<Request> {
    up(root, Up::ReadDone { read, result: Readout::Family { family: wire::FORGE, record } })
}
fn none() -> Box<[u8]> {
    wire::setting_answer(&w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: None }).unwrap())
}
fn set(root: &mut Domain, asked: Asked) -> Queue<forge::Request> {
    let mut out = Queue::uncapped();
    forge::step(
        &mut root.forge,
        &f::env(&f::limits()).with(f::limits().forge),
        forge::Event::Act { act: forge::Act::Ways(Act::Set { project: f::PROJECT, asked }) },
        &mut out,
    );
    out
}
fn sent(out: &mut Queue<forge::Request>) -> (jig_client::Key, c::ChangeSettings) {
    let Some(forge::Request::Send { request, name: None, ask: jig_client::Ask::Jig(ask) }) = out.pop() else {
        panic!("one settings request")
    };
    let c::Ask::ChangeSettings(change) = *ask else { panic!("change settings") };
    (jig_client::Key { asker: f::FORGE_ASKER, number: request.raw() }, change)
}
fn answered(root: &mut Domain, key: jig_client::Key, answer: c::Answer) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(
        root,
        &f::env(&f::limits()),
        Told::Answered { key, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) },
        &mut out,
    );
    out
}
fn stale() -> c::Answer {
    c::Answer::Refused { refusal: c::Refusal::Stale(c::RefusalStale { newest: NEXT }), at: None }
}

fn controls() -> [(u32, Control, Asked); 2] {
    [(7, Control::Ways, Asked::Ways(SHOWN)), (8, Control::Classes, Asked::Classes(Classes::stock(Way::YouApprove)))]
}
fn initial_read(f: &Fixture, kind: u32, out: &Queue<Request>) -> Token {
    match kind {
        7 => f.read,
        8 => f.classes_read,
        6 => reads(out, kind)[0].0,
        _ => panic!("fixture setting kind"),
    }
}
fn stock_classes() -> Asked {
    Asked::Classes(Classes::stock(Way::AgentsReview))
}
#[test]
fn the_watch_is_held_for_the_session_and_home_reads_it() {
    let (mut f, _) = opened();
    let settings = f.root.forge().settings_handle();
    for address in [
        Address::Home { project: f::PROJECT },
        Address::Settings { project: f::PROJECT },
        Address::Home { project: OTHER },
    ] {
        let out = go(&mut f.root, address);
        assert!(opening(&out).is_none());
        assert!(f.root.client().document(settings).is_some());
        if let Some(home) = f.root.pages().home() {
            assert_eq!(home.settings, settings);
        }
        for request in &out {
            if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
                request
            {
                assert_ne!(*watch, f.settings);
            }
        }
    }
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let mut f = fixture();
        up(&mut f.root, Up::WatchEnded { watch: f.settings, end });
        assert!(
            opening(&up(
                &mut f.root,
                Up::Watched { watch: f.me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me()))) }
            ))
            .is_none()
        );
        let mut out = Queue::uncapped();
        for link in [Link::Offline, Link::Live] {
            crate::route::told(&mut f.root, &f::env(&f::limits()), Told::Link { link }, &mut out);
        }
        assert!(opening(&out).is_none(), "ended until next sign-in");
        up(&mut f.root, Up::SignedOut);
        let out = up(&mut f.root, Up::SignedIn);
        let mut next = opening(&out);
        let mut me_watch = f.me;
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) },
                    },
            } = request
            {
                me_watch = *watch;
            }
        }
        let out = up(
            &mut f.root,
            Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me()))) },
        );
        next = next.or(opening(&out));
        assert_ne!(next.expect("next sign-in reopens settings"), f.settings);
    }
}
#[test]
fn project_values_are_read_on_open_and_dropped_on_close() {
    for (kind, control, asked) in controls() {
        let mut f = fixture();
        let read = initial_read(&f, kind, &Queue::uncapped());
        page(&mut f.root, read, Some(value(asked.clone())));
        assert_eq!(held(&f.root, control), Some(asked));
        let out = go(&mut f.root, Address::Home { project: OTHER });
        let found = reads(&out, kind);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, OTHER);
        assert_eq!(held(&f.root, control), None);
        assert_eq!(f.root.forge().ways(OTHER), None);
        assert_eq!(f.root.forge().classes(OTHER), None);
        ways::progress(
            &mut f.root.forge,
            skein_lib::Time::ZERO,
            None,
            None,
            false,
            f::limits().forge,
            &mut Queue::uncapped(),
        );
        assert_eq!(f.root.forge().ways(OTHER), None);
        assert_eq!(f.root.forge().classes(OTHER), None);
    }
    for shown in [Some(OTHER), Some(f::PROJECT), None] {
        let mut f = fixture();
        let mut out = Queue::uncapped();
        ways::progress(
            &mut f.root.forge,
            skein_lib::Time::ZERO,
            Some(f::PROJECT),
            shown,
            false,
            f::limits().forge,
            &mut out,
        );
        if shown != Some(OTHER) {
            assert!(out.is_empty());
            continue;
        }
        let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, read } }) = out.pop() else {
            panic!("shown classes read")
        };
        let Readable::Family { scope, record, .. } = read else { panic!("family read") };
        assert_eq!(scope, c::Scope::Project(OTHER));
        let w::Read::Setting(setting) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().into_parts().read
        else {
            panic!("setting")
        };
        assert_eq!((setting.kind(), setting.scope()), (8, OTHER));
        assert!(out.is_empty());
        let classes = Classes::stock(Way::YouApprove);
        ways::told(
            &mut f.root.forge,
            skein_lib::Time::ZERO,
            &f.root.client,
            &Told::Page {
                handle,
                page: Readout::Family { family: wire::FORGE, record: Some(value(Asked::Classes(classes.clone()))) },
            },
            f::limits().forge,
            &mut Queue::uncapped(),
        );
        assert_eq!(f.root.forge().classes(OTHER), Some(&classes));
        for next in [Some(OTHER), None] {
            let mut out = Queue::uncapped();
            ways::progress(
                &mut f.root.forge,
                skein_lib::Time::ZERO,
                Some(f::PROJECT),
                next,
                false,
                f::limits().forge,
                &mut out,
            );
            assert!(out.is_empty());
            assert_eq!(f.root.forge().classes(OTHER), if next.is_some() { Some(&classes) } else { None });
        }
    }
}
#[test]
fn kind_six_is_read_on_the_tab_alone() {
    let (mut f, out) = opened();
    assert!(reads(&out, 6).is_empty());
    for project in [f::PROJECT, OTHER] {
        let out = go(&mut f.root, Address::Settings { project });
        let found = reads(&out, 6);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, project);
        page(&mut f.root, found[0].0, Some(value(Asked::Questions(Route::Up))));
        assert_eq!(f.root.forge().questions(project), Some(Route::Up));
        assert!(reads(&go(&mut f.root, Address::Repositories { project }), 6).is_empty());
    }
}
#[test]
fn no_value_is_the_stock() {
    let mut f = fixture();
    page(&mut f.root, f.read, Some(none()));
    assert_eq!(f.root.forge().ways(f::PROJECT), Some(Ways { drafts: true, goals: false, changes: false, way: 0 }));
    page(&mut f.root, f.classes_read, Some(none()));
    assert_eq!(f.root.forge().classes(f::PROJECT), Some(&Classes::stock(Way::AgentsReview)));
    let out = go(&mut f.root, Address::Settings { project: f::PROJECT });
    page(&mut f.root, reads(&out, 6)[0].0, Some(none()));
    assert_eq!(f.root.forge().questions(f::PROJECT), Some(Route::None));
}
#[test]
fn a_refused_read_hides_and_a_malformed_value_too() {
    let malformed = wire::setting_answer(
        &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(b"bad".as_slice().into()) }).unwrap(),
    );
    for result in [
        Readout::Limit,
        Readout::NotServed { family: Some(wire::FORGE), served: None },
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
        Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)),
        Readout::Family { family: wire::FORGE, record: None },
        Readout::Family { family: wire::FORGE, record: Some(malformed) },
    ] {
        for (kind, control, _) in controls() {
            let mut f = fixture();
            let read = initial_read(&f, kind, &Queue::uncapped());
            up(&mut f.root, Up::ReadDone { read, result: result.clone() });
            assert_eq!(held(&f.root, control), None, "kind {kind}: {result:?}");
        }
    }
    let invalid = Classes {
        items: Box::new([Item { subject: Subject::Kind(Kind::Money), words: Box::new([]), class: Class::Ask }]),
        sets: Box::new([]),
    };
    let mut f = fixture();
    page(&mut f.root, f.classes_read, Some(value(Asked::Classes(invalid))));
    assert_eq!(f.root.forge().classes(f::PROJECT), None, "a decodable table must pass check");
}
#[test]
fn a_changed_line_reads_again() {
    for kind in [7, 6, 8] {
        let mut f = fixture();
        let out = go(&mut f.root, Address::Settings { project: f::PROJECT });
        let read = initial_read(&f, kind, &out);
        page(&mut f.root, read, Some(none()));
        for (last, follows) in [
            (Some(NEWEST), c::SettingFollows::Change),
            (Some(NEXT), c::SettingFollows::Change),
            (Some(NEXT), c::SettingFollows::Default),
        ] {
            let lines = [(kind, f::PROJECT, last, follows)];
            let out = document(&mut f, NEXT, &lines);
            let found = reads(&out, kind);
            assert_eq!(found.len(), 1);
            page(&mut f.root, found[0].0, Some(none()));
            assert!(reads(&document(&mut f, NEXT, &lines), kind).is_empty(), "unchanged line");
        }
        assert_eq!(reads(&document(&mut f, NEXT, &[]), kind).len(), 1, "removing a line reads the stock");
    }
}
#[test]
fn one_read_at_a_time() {
    for (kind, control, asked) in controls() {
        let mut f = fixture();
        let read = initial_read(&f, kind, &Queue::uncapped());
        for last in [NEWEST, NEXT] {
            assert!(
                reads(&document(&mut f, last, &[(kind, f::PROJECT, Some(last), c::SettingFollows::Change)]), kind)
                    .is_empty()
            );
        }
        let out = page(&mut f.root, read, Some(none()));
        let found = reads(&out, kind);
        assert_eq!(found.len(), 1, "one follow-up for all changes");
        assert!(reads(&page(&mut f.root, found[0].0, Some(value(asked.clone()))), kind).is_empty());
        assert_eq!(held(&f.root, control), Some(asked.clone()));
        let mut f = fixture();
        let read = initial_read(&f, kind, &Queue::uncapped());
        assert!(reads(&go(&mut f.root, Address::Home { project: OTHER }), kind).is_empty());
        let out = page(&mut f.root, read, Some(value(asked)));
        assert_eq!(held(&f.root, control), None, "the old project's late answer is dropped");
        let found = reads(&out, kind);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, OTHER);
    }
}
#[test]
fn the_link_back_reads_again() {
    for (address, six) in
        [(Address::Repositories { project: f::PROJECT }, 0), (Address::Settings { project: f::PROJECT }, 1)]
    {
        let mut f = fixture();
        let out = go(&mut f.root, address);
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Family { record, .. } } },
            } = request
                && let w::Read::View(_) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
            {
                page(&mut f.root, *read, None);
            }
        }
        page(&mut f.root, f.read, Some(none()));
        page(&mut f.root, f.classes_read, Some(none()));
        for (read, _) in &*reads(&out, 6) {
            page(&mut f.root, *read, Some(none()));
        }
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Family { record, .. } } },
            } = request
                && let w::Read::View(_) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
            {
                page(&mut f.root, *read, None);
            }
        }
        let mut out = Queue::uncapped();
        for link in [Link::Offline, Link::Live] {
            crate::route::told(&mut f.root, &f::env(&f::limits()), Told::Link { link }, &mut out);
        }
        crate::route::ways_progress(&mut f.root, &f::env(&f::limits()), &mut out);
        assert_eq!(reads(&out, 7).len(), 1);
        assert_eq!(reads(&out, 8).len(), 1);
        assert_eq!(reads(&out, 6).len(), six);
    }
}
#[test]
fn a_set_sends_one_change_of_one_item_against_the_newest() {
    let mut f = fixture();
    for asked in [Asked::Ways(SHOWN), Asked::Questions(Route::Up), stock_classes()] {
        let mut out = set(&mut f.root, asked.clone());
        let (_, change) = sent(&mut out);
        assert!(out.is_empty());
        assert_eq!(change.against, NEWEST);
        assert_eq!(change.running, c::Running::Stays);
        assert_eq!(change.items.len(), 1);
        let item = change.items.get(0).unwrap();
        let expected = match asked {
            Asked::Ways(_) => 7,
            Asked::Questions(_) => 6,
            Asked::Classes(_) => 8,
        };
        assert_eq!(item.setting, c::Setting::Application { kind: expected, scope: f::PROJECT, number: 1 });
        let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(bytes) }) = &item.edit else {
            panic!("set application")
        };
        let envelope = wire::decode_setting_answer(&value(asked), &w::CEILINGS).unwrap();
        assert_eq!(Some(bytes), envelope.value().as_ref());
    }
}
#[test]
fn a_second_set_of_a_control_sends_nothing_while_one_is_out() {
    let mut f = fixture();
    for (control, asked) in [
        (Control::Ways, Asked::Ways(SHOWN)),
        (Control::Questions, Asked::Questions(Route::Up)),
        (Control::Classes, stock_classes()),
    ] {
        set(&mut f.root, asked.clone());
        assert!(set(&mut f.root, asked.clone()).is_empty());
        assert_eq!(f.root.forge().ways_asked(control), Some(&asked));
    }
}
#[test]
fn a_stale_refusal_is_sent_once_more_against_its_newest() {
    for (_, control, asked) in controls() {
        let mut f = fixture();
        let (key, _) = sent(&mut set(&mut f.root, asked.clone()));
        let mut child = Queue::uncapped();
        assert!(ways::told(
            &mut f.root.forge,
            skein_lib::Time::ZERO,
            &f.root.client,
            &Told::Answered { key, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(stale())) },
            f::limits().forge,
            &mut child
        ));
        let (next, change) = sent(&mut child);
        assert_eq!(change.against, NEXT);
        assert_ne!(key, next);
        assert_eq!(f.root.forge().ways_refused(control), None);
        assert_eq!(f.root.forge().ways_asked(control), Some(&asked));
    }
}
#[test]
fn a_second_stale_is_said() {
    for (_, control, asked) in controls() {
        let mut f = fixture();
        let (key, _) = sent(&mut set(&mut f.root, asked.clone()));
        let mut child = Queue::uncapped();
        ways::told(
            &mut f.root.forge,
            skein_lib::Time::ZERO,
            &f.root.client,
            &Told::Answered { key, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(stale())) },
            f::limits().forge,
            &mut child,
        );
        let (key, _) = sent(&mut child);
        assert!(answered(&mut f.root, key, stale()).is_empty());
        assert_eq!(f.root.forge().ways_asked(control), None);
        assert_eq!(f.root.forge().ways_refused(control), Some(&c::Refusal::Stale(c::RefusalStale { newest: NEXT })));
    }
}
#[test]
fn a_refusal_is_kept_until_the_next_act() {
    for (kind, control, asked) in controls() {
        let mut f = fixture();
        let (key, _) = sent(&mut set(&mut f.root, asked.clone()));
        answered(
            &mut f.root,
            key,
            c::Answer::Refused { refusal: c::Refusal::Role { held: Some(c::HeldRole::Member) }, at: None },
        );
        let refusal = c::Refusal::Role { held: Some(c::HeldRole::Member) };
        assert_eq!(f.root.forge().ways_refused(control), Some(&refusal));
        let read = initial_read(&f, kind, &Queue::uncapped());
        page(&mut f.root, read, Some(none()));
        assert_eq!(f.root.forge().ways_refused(control), Some(&refusal));
        set(&mut f.root, asked.clone());
        assert_eq!(f.root.forge().ways_refused(control), None);
    }
}
#[test]
fn an_answer_holds_the_value_asked() {
    for (asked, control, kind, stock) in [
        (Asked::Ways(SHOWN), Control::Ways, 7, Asked::Ways(Ways::STOCK)),
        (Asked::Questions(Route::Up), Control::Questions, 6, Asked::Questions(Route::None)),
        (Asked::Classes(Classes::stock(Way::YouApprove)), Control::Classes, 8, stock_classes()),
    ] {
        let mut f = fixture();
        let out = go(&mut f.root, Address::Settings { project: f::PROJECT });
        let read = initial_read(&f, kind, &out);
        page(&mut f.root, read, Some(none()));
        let (key, _) = sent(&mut set(&mut f.root, asked.clone()));
        let out = answered(
            &mut f.root,
            key,
            c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: NEXT })),
        );
        assert_eq!(held(&f.root, control), Some(asked));
        assert_eq!(f.root.forge().ways_asked(control), None);
        let found = reads(&out, kind);
        assert_eq!(found.len(), 1);
        page(&mut f.root, found[0].0, Some(none()));
        assert_eq!(held(&f.root, control), Some(stock));
    }
}
fn held(root: &Domain, control: Control) -> Option<Asked> {
    match control {
        Control::Classes => match root.forge().classes(f::PROJECT) {
            Some(classes) => Some(Asked::Classes(classes.clone())),
            None => None,
        },
        Control::Ways => match root.forge().ways(f::PROJECT) {
            Some(ways) => Some(Asked::Ways(ways)),
            None => None,
        },
        Control::Questions => match root.forge().questions(f::PROJECT) {
            Some(route) => Some(Asked::Questions(route)),
            None => None,
        },
    }
}
#[test]
fn default_way_uses_the_selected_tracked_template_or_the_lowest() {
    let mut me = me();
    let project = me.projects.get_mut(0).unwrap();
    let mut template = project.templates.get(0).unwrap().clone();
    template.tracked = Some(1);
    project.templates = List::with_capacity(2);
    for number in [3, 2] {
        template.number = number;
        project.templates.push(template.clone()).unwrap();
    }
    assert_eq!(ways::default_way(&me, f::PROJECT, &SHOWN), Some(3));
    assert_eq!(ways::default_way(&me, f::PROJECT, &Ways::STOCK), Some(2));
    assert_eq!(ways::default_way(&me, OTHER, &SHOWN), None);
}

#[test]
fn a_read_started_before_the_set_cannot_replace_its_answer() {
    for (asked, control, kind) in [
        (Asked::Ways(SHOWN), Control::Ways, 7),
        (Asked::Questions(Route::Up), Control::Questions, 6),
        (stock_classes(), Control::Classes, 8),
    ] {
        let mut f = fixture();
        let out = go(&mut f.root, Address::Settings { project: f::PROJECT });
        let prior = initial_read(&f, kind, &out);
        let (key, _) = sent(&mut set(&mut f.root, asked.clone()));
        let out = answered(
            &mut f.root,
            key,
            c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: NEXT })),
        );
        assert!(reads(&out, kind).is_empty(), "the prior read still owns the kind");
        let out = page(&mut f.root, prior, Some(none()));
        assert_eq!(held(&f.root, control), Some(asked), "a prior read cannot undo the accepted setting");
        assert_eq!(reads(&out, kind).len(), 1, "the answer starts one fresh read");
    }
}

const PROPOSAL: u64 = 91;
fn proposal_cards(key: u64, kind: u32) -> List<crate::pages::inbox::Card> {
    let items = List::with_capacity(0);
    let key = c::ProposalKey { proposer: c::Proposer::Task(12), number: key };
    let mut cards = List::with_capacity(1);
    cards
        .push(crate::pages::inbox::Card {
            whole: None,
            object: c::ObjectKey::Proposal(key),
            group: c::InboxGroup::Proposals,
            project: Some(f::PROJECT),
            position: None,
            unread: None,
            handle: jig_client::Handle { asker: f::FORGE_ASKER, number: 0 },
            page: Some(c::ReadValue::Proposal(Box::new(c::Proposal {
                projects: List::with_capacity(0),
                key,
                project: f::PROJECT,
                reason: c::DocumentText::complete(c::Text::new(Box::from(*b"How I work"), 32).unwrap()),
                action: c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
                    against: 0,
                    items,
                    review: Some(c::SettingsReview {
                        key: Some(key.number.checked_add(1000).unwrap()),
                        newest: 0,
                        items: List::from_boxed(Box::new([c::ReviewItem {
                            setting: c::Setting::Application { kind, scope: f::PROJECT, number: 1 },
                            edit: c::RevisionItemEdit::Set,
                            now: c::ReviewValue::None,
                            origin: c::SettingOrigin::NotOffered,
                            would_be: c::ReviewValue::None,
                            application: None,
                        }])),
                        cut: false,
                    }),
                    made: None,
                })),
                as_holder: true,
                waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(73), since: skein_lib::Wall::EPOCH },
                funding: List::with_capacity(0),
                acts: List::with_capacity(0),
            }))),
            words: None,
            key: None,
            outcome: None,
        })
        .unwrap();
    cards
}
fn proposal_read(root: &mut Domain, key: u64) -> Queue<forge::Request> {
    proposal_read_kind(root, key, 7)
}
fn proposal_read_kind(root: &mut Domain, key: u64, kind: u32) -> Queue<forge::Request> {
    let mut out = Queue::uncapped();
    ways::proposals(&mut root.forge, Some(&proposal_cards(key, kind)), f::limits().forge, &mut out);
    out
}
fn proposal_handle(out: &Queue<forge::Request>, key: u64) -> jig_client::Handle {
    let mut found = List::with_capacity(1);
    for request in out {
        if let forge::Request::Client {
            asked: jig_client::Event::Read { handle, read: Readable::Family { scope, window, record } },
        } = request
        {
            assert_eq!(*scope, c::Scope::Deployment);
            assert_eq!(*window, c::Window { rows: 1, after: None });
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
            let w::Read::ProposalItems(read) = reading.read() else { panic!("proposal items") };
            assert_eq!(read.key(), key.checked_add(1000).unwrap());
            found.push(*handle).unwrap();
        }
    }
    assert_eq!(found.len(), 1);
    *found.get(0).unwrap()
}
fn proposed_page(asked: Asked) -> Readout {
    let kind = match &asked {
        Asked::Ways(_) => 7,
        Asked::Questions(_) => 6,
        Asked::Classes(_) => 8,
    };
    let value = wire::decode_setting_answer(&value(asked), &w::CEILINGS).unwrap().into_parts().value.unwrap();
    let mut items = List::with_capacity(1);
    items
        .push(
            w::ProposalItem::new(
                &w::CEILINGS,
                w::ProposalItemParts {
                    kind,
                    scope: f::PROJECT,
                    number: 1,
                    edit: w::SettingEdit::Set(
                        w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        )
        .unwrap();
    let answer =
        w::ProposalItemsAnswer::new(&w::CEILINGS, w::ProposalItemsAnswerParts { items, owned_approval: false })
            .unwrap();
    Readout::Family { family: wire::FORGE, record: Some(wire::proposal_items_answer(&answer)) }
}
fn proposal_told(root: &mut Domain, told: Told) {
    let mut out = Queue::uncapped();
    ways::told(&mut root.forge, skein_lib::Time::ZERO, &root.client, &told, f::limits().forge, &mut out);
}
#[test]
fn a_proposals_items_are_read_once_and_bounded() {
    let mut f = fixture();
    let handle = proposal_handle(&proposal_read(&mut f.root, PROPOSAL), PROPOSAL);
    assert!(proposal_read(&mut f.root, PROPOSAL).is_empty());
    proposal_told(&mut f.root, Told::Page { handle, page: proposed_page(Asked::Ways(SHOWN)) });
    assert_eq!(
        f.root.forge.proposed(PROPOSAL),
        Some([ways::Proposed::Ways { project: f::PROJECT, ways: SHOWN }].as_slice())
    );
    assert!(proposal_read(&mut f.root, PROPOSAL).is_empty());
    let mut ended = proposal_cards(PROPOSAL, 7);
    let card = ended.get_mut(0).unwrap();
    card.page = Some(c::ReadValue::Ended(Box::new(c::Ended {
        key: card.object,
        at: None,
        by: None,
        value: c::EndedValue::Proposal(c::ProposalHistory {
            number: PROPOSAL,
            proposer: 12,
            as_holder: true,
            reason: c::DocumentText::complete(c::Text::new(b"How I work".as_slice().into(), 32).unwrap()),
            state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(73) },
            action: c::RetainedAction::Settings(c::RetainedActionSettings {
                projects: List::with_capacity(0),
                against: 0,
                items: List::with_capacity(0),
                made: Some(1),
            }),
        }),
    })));
    let mut out = Queue::uncapped();
    ways::proposals(&mut f.root.forge, Some(&ended), f::limits().forge, &mut out);
    assert!(out.is_empty(), "decided proposals ask nothing");
    proposal_told(&mut f.root, Told::Link { link: Link::Offline });
    proposal_told(&mut f.root, Told::Link { link: Link::Live });
    ways::proposals(&mut f.root.forge, Some(&ended), f::limits().forge, &mut out);
    assert!(out.is_empty(), "a decided proposal is not read after reconnection");
    assert_eq!(
        f.root.forge.proposed(PROPOSAL),
        Some([ways::Proposed::Ways { project: f::PROJECT, ways: SHOWN }].as_slice()),
        "a decided proposal keeps the values it read"
    );
    proposal_handle(&proposal_read(&mut f.root, PROPOSAL), PROPOSAL);
    // Fill the bound, then admit one beyond it: the oldest is evicted.
    for offset in 1..forge::forges::ROWS {
        let key = PROPOSAL.checked_add(u64::from(offset)).unwrap();
        let handle = proposal_handle(&proposal_read(&mut f.root, key), key);
        proposal_told(&mut f.root, Told::Page { handle, page: proposed_page(Asked::Ways(SHOWN)) });
    }
    assert!(proposal_read(&mut f.root, PROPOSAL).is_empty(), "oldest remains at the bound");
    let beyond = PROPOSAL.checked_add(u64::from(forge::forges::ROWS)).unwrap();
    proposal_handle(&proposal_read(&mut f.root, beyond), beyond);
    proposal_handle(&proposal_read(&mut f.root, PROPOSAL), PROPOSAL);
    proposal_told(&mut f.root, Told::Page { handle, page: proposed_page(Asked::Ways(SHOWN)) });
    assert_eq!(f.root.forge.proposed(PROPOSAL), None, "an evicted read cannot publish into its replacement");
}
fn rule_page(owned_approval: bool) -> Readout {
    edited_page(2, w::SettingEdit::Removed, owned_approval)
}
fn edited_page(kind: u32, edit: w::SettingEdit, owned_approval: bool) -> Readout {
    let items = List::from_boxed(Box::new([w::ProposalItem::new(
        &w::CEILINGS,
        w::ProposalItemParts { kind, scope: f::PROJECT, number: 1, edit },
    )
    .unwrap()]));
    let answer =
        w::ProposalItemsAnswer::new(&w::CEILINGS, w::ProposalItemsAnswerParts { items, owned_approval }).unwrap();
    Readout::Family { family: wire::FORGE, record: Some(wire::proposal_items_answer(&answer)) }
}
#[test]
fn proposal_reads_keep_owned_approval_and_refusals() {
    for (kind, page, expected, owned) in [
        (2, rule_page(true), Some(Box::<[ways::Proposed]>::from([])), true),
        (2, rule_page(false), Some(Box::<[ways::Proposed]>::from([])), false),
        (1, rule_page(true), Some(Box::<[ways::Proposed]>::from([])), true),
        (
            8,
            proposed_page(stock_classes()),
            Some(Box::<[ways::Proposed]>::from([ways::Proposed::Classes {
                project: f::PROJECT,
                classes: Classes::stock(Way::AgentsReview),
            }])),
            false,
        ),
        (
            8,
            edited_page(8, w::SettingEdit::Removed, false),
            Some(Box::<[ways::Proposed]>::from([ways::Proposed::Classes {
                project: f::PROJECT,
                classes: Classes::stock(Way::AgentsReview),
            }])),
            false,
        ),
        (
            8,
            edited_page(8, w::SettingEdit::Reset, false),
            Some(Box::<[ways::Proposed]>::from([ways::Proposed::Classes {
                project: f::PROJECT,
                classes: Classes::stock(Way::AgentsReview),
            }])),
            false,
        ),
        (7, Readout::Limit, None, false),
    ] {
        let mut f = fixture();
        assert!(!f.root.forge().owned_approval(PROPOSAL));
        let handle = proposal_handle(&proposal_read_kind(&mut f.root, PROPOSAL, kind), PROPOSAL);
        assert!(!f.root.forge().owned_approval(PROPOSAL));
        proposal_told(&mut f.root, Told::Page { handle, page });
        assert_eq!(f.root.forge.proposed(PROPOSAL), expected.as_deref());
        assert_eq!(f.root.forge().owned_approval(PROPOSAL), owned);
        assert!(proposal_read_kind(&mut f.root, PROPOSAL, kind).is_empty(), "read once, including a refusal");
    }
}

#[test]
fn a_transient_settings_read_backs_off_and_recovers_without_a_revision_change() {
    use skein_lib::{Duration, Time};
    for kind in [7, 6, 8] {
        for failure in [
            Readout::Limit,
            Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)),
            Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
            Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut)),
        ] {
            let mut f = fixture();
            let out = go(&mut f.root, Address::Settings { project: f::PROJECT });
            let mut read = initial_read(&f, kind, &out);
            let limits = f::limits();
            let mut env = f::env(&limits);
            for seconds in [1, 2, 4, 8, 16, 30, 30] {
                let mut out = Queue::uncapped();
                crate::step(
                    &mut f.root,
                    &env,
                    Event::Client {
                        event: jig_client::Event::Up { up: Up::ReadDone { read, result: failure.clone() } },
                    },
                    &mut out,
                );
                assert!(reads(&out, kind).is_empty(), "no immediate retry loop");
                let due = env.now.saturating_add(Duration::from_secs(seconds));
                assert_eq!(forge::next_deadline(f.root.forge()), Some(due), "retry must wake the shell");
                crate::fire(&mut f.root, &env, &mut out);
                assert!(reads(&out, kind).is_empty(), "wait until due");
                env.now = due;
                crate::fire(&mut f.root, &env, &mut out);
                let retry = reads(&out, kind);
                assert_eq!(retry.len(), 1, "one retry of the missing setting");
                read = retry[0].0;
            }
            let (control, asked) = match kind {
                7 => (Control::Ways, Asked::Ways(Ways::STOCK)),
                6 => (Control::Questions, Asked::Questions(Route::None)),
                8 => (Control::Classes, stock_classes()),
                _ => panic!("fixture setting kind"),
            };
            let mut out = Queue::uncapped();
            crate::step(
                &mut f.root,
                &env,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::ReadDone {
                            read,
                            result: Readout::Family { family: wire::FORGE, record: Some(none()) },
                        },
                    },
                },
                &mut out,
            );
            assert_eq!(forge::next_deadline(f.root.forge()), None);
            assert!(reads(&out, kind).is_empty());
            assert_eq!(held(&f.root, control), Some(asked.clone()));
            assert!(!set(&mut f.root, asked).is_empty(), "the recovered control can send its change");
            assert!(env.now > Time::ZERO);
        }
    }
}
#[test]
fn leaving_the_settings_scope_cancels_its_read_retry() {
    for kind in [7, 6, 8] {
        let mut f = fixture();
        let out = go(&mut f.root, Address::Settings { project: f::PROJECT });
        let read = initial_read(&f, kind, &out);
        up(&mut f.root, Up::ReadDone { read, result: Readout::Limit });
        assert!(forge::next_deadline(f.root.forge()).is_some());
        if kind == 6 {
            go(&mut f.root, Address::Home { project: f::PROJECT });
        } else {
            up(&mut f.root, Up::SignedOut);
        }
        assert_eq!(forge::next_deadline(f.root.forge()), None);
    }
}
