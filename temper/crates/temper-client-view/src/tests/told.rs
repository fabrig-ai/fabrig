//! Telling you's rows and refusal lines (temper/docs/design/client/pages/told.md).
#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "ordinary view tests inspect semantic nodes")]
use super::fixtures as f;
use jig_client::{Down, Up, contract as c};
use skein_lib::{Env, List, Queue, Time};
use skein_ui::{Act as UiAct, Role};
use temper_client_domain::{self as d, Address, Event, Request, pages::{self, told::{Act, Browser, Permission, Refusal, Report}}};
use temper_client_protocol_wire::{self as wire, v1 as w};
const ENDPOINT: &[u8] = b"https://fcm.googleapis.com/subscription/private";
fn drive(root: &mut d::Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() }, event, &mut out);
    out
}
fn up(root: &mut d::Domain, value: Up) -> Queue<Request> { drive(root, Event::Client { event: jig_client::Event::Up { up: value } }) }
fn append(out: &mut Queue<Request>, mut added: Queue<Request>) { while let Some(request) = added.pop() { out.push(request); } }
fn root(push: bool, browser: Browser, count: u32) -> d::Domain {
    let seed = f::root_at(Address::Home { project: f::PROJECT }, &f::domain_limits()).0;
    let me = seed.pages().me(seed.client()).unwrap().clone();
    let mut root = d::unlinked(&f::domain_limits(), 13);
    let mut out = drive(&mut root, Event::Start { address: Address::Told, kept: List::uncapped(), with: Some(jig_client::With::Transport) });
    append(&mut out, up(&mut root, Up::Served { served: true, versions: None }));
    while let Some(request) = out.pop() {
        let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        match down {
            Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) } => append(&mut out, up(&mut root, Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me.clone()))) })),
            Down::Open { watch, subject: jig_client::Watchable::Family { record, scope, .. } } if *wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch() == w::Watch::Devices => {
                assert_eq!(scope, c::Scope::Own);
                let mut rows = List::with_capacity(w::CEILINGS.devices_rows);
                for number in 1..=count {
                    rows.push(w::DeviceRow::new(&w::CEILINGS, w::DeviceRowParts { number, label: if number == 1 { b"Chrome on Android".as_slice().into() } else { b"Firefox on Linux".as_slice().into() }, at: f::NOW.as_nanos(), endpoint: if number == 1 { ENDPOINT.into() } else { b"https://updates.push.services.mozilla.com/sub/private".as_slice().into() } }).unwrap()).unwrap();
                }
                let document = w::Devices::new(&w::CEILINGS, w::DevicesParts { rows }).unwrap();
                append(&mut out, up(&mut root, Up::FamilyDelivered { watch, record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::devices(&document) } }));
            }
            Down::Read { read, what: jig_client::Readable::Family { record, .. } } if *wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() == w::Read::Deployment => {
                let answer = w::DeploymentAnswer::new(&w::CEILINGS, w::DeploymentAnswerParts { welcome: false, owned: true, program_login: None, push: push.then_some([4; 65]) }).unwrap();
                append(&mut out, up(&mut root, Up::ReadDone { read, result: jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::deployment_answer(&answer)) } }));
            }
            Down::Save { .. } | Down::Erase { .. } | Down::Send { .. } | Down::Open { .. } | Down::Continue { .. } | Down::Close { .. } | Down::Read { .. } => {}
        }
    }
    drive(&mut root, Event::Push(Report::State(browser)));
    root
}
fn browser() -> Browser { Browser { capable: true, permission: Permission::Granted, endpoint: None, user_agent: b"Chrome/120 Android".as_slice().into() } }
fn texts(view: &crate::View) -> Vec<Box<[u8]>> {
    let nodes = view.tree().nodes();
    let start = nodes.iter().position(|n| n.role == Role::Main).unwrap();
    let size = usize::try_from(nodes.get(start).unwrap().size).unwrap();
    nodes.get(start..start.checked_add(size).unwrap()).unwrap().iter().filter_map(|n| n.name.clone()).collect()
}
#[test]
fn told_draws_the_browser_state_and_only_the_deployments_real_rows() {
    for (push, capable, permission, endpoint, expected, act, count) in [
        (false, true, Permission::Granted, None, "Not set up", None, 2),
        (true, false, Permission::Ask, None, "This browser can't be pushed to.", None, 2),
        (true, true, Permission::Denied, None, "Notifications are blocked for this site in the browser's settings.", None, 0),
        (true, true, Permission::Granted, None, "Off", Some(("Turn on", Act::TurnOn)), 2),
        (true, true, Permission::Granted, Some(ENDPOINT), "On since Today", Some(("Turn off", Act::TurnOff)), 2),
    ] {
        let root = root(push, Browser { capable, permission, endpoint: endpoint.map(Into::into), user_agent: b"iPhone Safari/605".as_slice().into() }, count);
        let mut view = f::browser_page_view(&root);
        let words = texts(&view);
        assert!(words.iter().any(|text| text.starts_with(expected.as_bytes())), "missing {expected}: {words:?}");
        assert_eq!(words.iter().any(|text| text.as_ref() == b"Your devices"), push);
        assert_eq!(words.iter().any(|text| text.as_ref() == b"Chrome on Android"), push && count > 0);
        assert_eq!(words.iter().filter(|text| text.as_ref() == b"This browser").count(), if endpoint.is_some() { 2 } else { usize::from(push) });
        if push && !capable { assert!(words.iter().any(|text| text.as_ref() == b"Add temper to the Home Screen, then open it from there and turn this on.")); }
        if let Some((name, act)) = act {
            let node = f::find(view.tree(), Role::Button, name.as_bytes()).unwrap();
            assert_eq!(crate::act(&mut view, node, UiAct::Press), Some(d::Action::Pages(pages::Act::Told(act))));
        } else { for name in [b"Turn on".as_slice(), b"Turn off"] { assert!(f::find(view.tree(), Role::Button, name).is_none()); } }
        if push && count > 0 {
            let node = f::find(view.tree(), Role::Button, b"Remove").unwrap();
            assert_eq!(crate::act(&mut view, node, UiAct::Press), Some(d::Action::Pages(pages::Act::Told(Act::Remove { number: 1 }))));
            assert!(!words.iter().any(|text| text.windows(b"/subscription/private".len()).any(|part| part == b"/subscription/private")));
        }
    }
}
#[test]
fn told_refusals_stay_under_this_browser_and_off_replaces_the_rows() {
    for (why, expected) in [
        (Refusal::Off, "Not set up"),
        (Refusal::Service, "This browser's push service isn't one this deployment sends to."),
        (Refusal::TooMany, "You have 8 devices: remove one below first."),
        (Refusal::Malformed, "This browser gave a subscription temper can't use."),
        (Refusal::Person, "Only a person is pushed to."),
        (Refusal::Blocked, "Notifications are blocked for this site in the browser's settings."),
    ] {
        let mut root = root(true, browser(), 2);
        let out = drive(&mut root, Event::Act { action: d::Action::Pages(pages::Act::Told(Act::TurnOn)) });
        let token: u64 = out.iter().find_map(|request| if let Request::Push(pages::told::Duty::Subscribe { token, .. }) = request { Some(*token) } else { None }).unwrap();
        let waiting = f::browser_page_view(&root);
        assert!(texts(&waiting).iter().any(|text| text.as_ref() == "Turning on…".as_bytes()));
        drive(&mut root, Event::Push(Report::Refused { token, why }));
        let view = f::browser_page_view(&root);
        assert!(texts(&view).iter().any(|text| text.as_ref() == expected.as_bytes()), "{why:?}");
        assert_eq!(texts(&view).iter().any(|text| text.as_ref() == b"Your devices"), why != Refusal::Off);
    }
}
