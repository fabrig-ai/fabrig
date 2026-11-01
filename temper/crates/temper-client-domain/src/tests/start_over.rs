//! A set up cut short is removed unconfirmed (temper/docs/design/draft/playground-reset.md, 6.4).
use super::{connect as fixtures, fixtures as f};
use crate::{
    Event, Request,
    forge::{
        self, connect,
        forges::{Connection, First, FirstState, Kind, Mode, Offer, Row, SetUpProject, State},
    },
};
use skein_lib::Queue;
use temper_client_protocol_wire::{self as wire, v1 as w};

fn row(connection: Connection) -> Row {
    Row {
        finished: false,
        playground: false,
        read_only: false,
        number: 3,
        kind: Kind::Forgejo,
        mode: Mode::Owned,
        connection,
        offer: Offer::Offered,
        first: None,
        installations: Box::new([]),
        ended: None,
        app: None,
        web: None,
        repositories: Box::new([]),
        owned: None,
        admin: false,
        ended_stop: None,
    }
}
fn connected() -> Connection {
    Connection::Connected { id: 42, login: b"temper".as_slice().into() }
}
#[test]
fn an_unfinished_forge_is_one_with_no_live_step() {
    let failed = row(Connection::SetUpFailed { why: b"fetch failed".as_slice().into(), at: None });
    let mut making = row(connected());
    making.first = Some(First {
        playground: false,
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        project: SetUpProject::Existing(f::PROJECT),
        state: FirstState::Making,
        label: b"work".as_slice().into(),
    });
    let mut finished = row(connected());
    finished.finished = true;
    let mut playground = failed.clone();
    playground.playground = true;
    let mut github = row(connected());
    github.kind = Kind::GitHub;
    let mut untouched_github = row(Connection::NotConnected);
    untouched_github.kind = Kind::GitHub;
    let mut not_offered = row(Connection::NotSetUp);
    not_offered.offer = Offer::NotOffered;
    let mut admin = row(Connection::NotConnected);
    admin.admin = true;
    for (name, row, live, unfinished) in [
        ("failed Forgejo", failed, false, true),
        ("setting up", row(Connection::SettingUp { step: None }), true, false),
        ("connecting", row(Connection::Connecting), true, false),
        ("making the first repository", making, true, false),
        ("finished", finished, false, false),
        ("playground", playground, false, false),
        ("GitHub awaiting installation", github, false, true),
        ("untouched GitHub", untouched_github, false, false),
        ("offered owned Forgejo", row(Connection::NotSetUp), false, false),
        ("unoffered owned Forgejo", not_offered, false, false),
        ("disconnected admin Forgejo", admin, false, false),
    ] {
        assert_eq!(connect::live(&row), live, "{name}");
        let rows = State::Rows(Box::new([row]));
        assert_eq!(connect::unfinished(&rows).is_some(), unfinished, "{name}");
    }
    assert!(connect::unfinished(&State::Unknown).is_none());
    assert!(connect::unfinished(&State::Ended).is_none());
    let mut first = row(connected());
    first.number = 2;
    first.kind = Kind::GitHub;
    let rows = State::Rows(Box::new([row(Connection::Connecting), first, row(connected())]));
    assert_eq!(connect::unfinished(&rows).expect("first idle unfinished row").number, 2);
}
#[test]
fn start_over_sends_the_removal_unconfirmed() {
    let (mut root, _, _) = fixtures::opened(true);
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &f::env(&f::limits()),
        Event::Forge(forge::Event::Act { act: forge::Act::StartOver { forge: 3 } }),
        &mut out,
    );
    let mut count = 0_u32;
    for request in &out {
        if let Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Send { request: jig_client::Ask::Family { scope, record }, .. },
                },
        } = request
        {
            count += 1;
            assert_eq!(*scope, jig_client::contract::Scope::Deployment);
            let w::Request::RemoveForge(value) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
            else {
                panic!("remove forge")
            };
            assert_eq!(value.forge(), 3);
        }
    }
    assert_eq!(count, 1);
    let Some(forge::remove::State::Removing { .. }) = root.forge().removal(3) else {
        panic!("removing without confirmation");
    };
    let mut again = Queue::uncapped();
    crate::step(
        &mut root,
        &f::env(&f::limits()),
        Event::Forge(forge::Event::Act { act: forge::Act::StartOver { forge: 3 } }),
        &mut again,
    );
    assert!(again.is_empty());
}
