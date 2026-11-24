//! The notes example's write-before-send path (skein/docs/design/ui/web/testing.md, 3).
use super::duties;
use skein_fake_person::{Face, Find, Found, TreeFace};
use skein_http::{
    Method, Version,
    server::{Body, Call},
};
use skein_lib::{Queue, Reader, Time, Wall, len_u64};
use skein_ui::{Act, Role, View};
use skein_ui_example_server::{State as Server, bundle::Bundle, routes};
use skein_ui_example_view::{
    Duties, Report, State, limits, places,
    records::{CEILINGS, Notes, Record as _},
};
use skein_ui_web::{
    fetch::Exchange,
    platform::Platform,
    store::{Record, Store},
    transport::{self, Carried, StreamDuty},
};
use std::collections::BTreeMap;

const TITLE: &[u8] = b"Groceries";
const SEED: u64 = 1;
// The first add uses the seed as its key and the first pending slot as exchange 1.
const ADD_KEY: &[u8] = b"add-0000000000000001";
const ADD_EXCHANGE: u32 = 1;

#[derive(Default)]
struct FakePlatform {
    records: Vec<Record>,
    sent: Vec<Exchange>,
}
impl Platform for FakePlatform {
    fn read_all(&self) -> Vec<Record> {
        self.records.clone()
    }
    fn write(&mut self, key: &[u8], value: &[u8]) -> bool {
        self.remove(key);
        self.records.push(Record { key: key.into(), value: value.into() });
        true
    }
    fn remove(&mut self, key: &[u8]) {
        self.records.retain(|record| record.key.as_ref() != key);
    }
    fn send(&mut self, exchange: &Exchange) {
        self.sent.push(exchange.clone());
    }
    fn navigate(&mut self, _: &[u8]) {
        unreachable!("the add does not navigate");
    }
    fn stream(&mut self, _: StreamDuty) {
        unreachable!("the add does not change the stream");
    }
    fn abandon(&mut self, _: u32) {
        unreachable!("the add does not abandon an exchange");
    }
    fn read_on(&mut self) {
        unreachable!("the add does not read the stream");
    }
}

fn step(state: &mut State, report: Report) -> Duties {
    let mut duties = duties::empty();
    state.step(report, Time::ZERO, Wall::EPOCH, &mut duties);
    duties
}

fn build(state: &State) -> View {
    let limits = limits::limits();
    let mut view = View::new(&limits);
    state.build(&mut view, &mut Queue::with_capacity(limits.patches));
    view
}

fn named(view: &View, role: Role, name: &[u8]) -> Found {
    TreeFace::new(view.tree()).find(&Find::exactly_named(role, name))
}

fn add(path: &[u8]) -> (Vec<Carried>, Vec<Exchange>) {
    let mut state = State::start(SEED, path, &[], 0, b"en");
    let mut store = Store::new(b"notes-", places::small_store_bound(path));
    let mut platform = FakePlatform::default();
    let mut reports = Vec::new();
    for (role, name, act) in
        [(Role::TextField, b"Title".as_slice(), Act::Input { value: TITLE.into() }), (Role::Button, b"Add", Act::Press)]
    {
        let view = build(&state);
        let Found::Node(node) = named(&view, role, name) else { unreachable!("the example's add control") };
        let act = view.act(node, act).expect("the add control is bound");
        transport::carry(duties::requests(step(&mut state, Report::Act(act))), &mut store, &mut platform, &mut reports);
    }
    (reports, platform.sent)
}

fn list(sent: &[Exchange]) -> routes::Answer {
    let mut server = Server::new();
    let bundle = Bundle::new(b"<html></html>".as_slice().into(), "0123456789abcdef".into(), BTreeMap::new())
        .expect("fixture bundle fits");
    for exchange in sent {
        let call = Call {
            method: Method::from_bytes(exchange.method.as_bytes()).expect("the view uses an HTTP method"),
            target: exchange.path.clone(),
            version: Version::Http11,
            headers: Box::from([]),
            body: Body::Length(len_u64(exchange.body.len())),
        };
        let _answer = routes::route(&mut server, &bundle, &call, &exchange.body);
    }
    let get = Call {
        method: Method::Get,
        target: b"/api/notes".as_slice().into(),
        version: Version::Http11,
        headers: Box::from([]),
        body: Body::None,
    };
    routes::route(&mut server, &bundle, &get, &[])
}

fn listing(sent: &[Exchange]) -> (Notes, Found) {
    let answer = list(sent);
    let notes = Notes::decode(&CEILINGS, &mut Reader::new(&answer.body)).expect("the server's list decodes");
    let mut reader = State::start(SEED, b"/", &[], 0, b"en");
    let _list_request = step(&mut reader, Report::StreamOpened);
    let _duties = step(&mut reader, Report::Answered { exchange: 0, status: answer.head.status, body: answer.body });
    (notes, named(&build(&reader), Role::Link, TITLE))
}

#[test]
fn a_refused_add_makes_no_note() {
    let (reports, sent) = add(b"/test/small-store");
    let (notes, link) = listing(&sent);
    assert_eq!(reports, [Carried::Refused { key: ADD_KEY.into() }, Carried::Failed { exchange: ADD_EXCHANGE }]);
    assert!(sent.is_empty(), "no exchange leaves the refused step");
    assert!(notes.notes.is_empty(), "the server's list is empty");
    assert_eq!(link, Found::Absent);
}

#[test]
fn an_unbounded_add_is_sent_and_listed_once() {
    let (_, sent) = add(b"/");
    let (notes, link) = listing(&sent);
    let titles: Vec<_> = notes.notes.iter().map(|note| note.title.as_ref()).collect();
    let exchanges: Vec<_> = sent.iter().map(|exchange| exchange.id).collect();
    assert_eq!(exchanges, [ADD_EXCHANGE], "one add exchange leaves the step");
    assert_eq!(titles, [TITLE]);
    assert!(matches!(link, Found::Node(_)), "the reader shows the added note");
}
