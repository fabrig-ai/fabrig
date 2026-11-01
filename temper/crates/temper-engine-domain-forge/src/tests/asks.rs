//! The requests' provisional refusals and setup choice (contracts, 5).
use super::*;
use jig_domain_core::Carried;

const REQUEST: Token = Token::new(91);
const ANSWER: Token = Token::new(92);
const VALUE: Carried = Carried { number: 1_u64 << 63_u32, digest: [17; 32] };

fn application(d: &mut Domain, ask: Ask, carried: Option<Carried>) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: REQUEST,
            token: ANSWER,
            retention: Duration::from_secs(60),
            ask,
            role: Some(jig_domain_people::Role::Owner),
            carried,
        },
    )
}
fn refused(ask: Ask, expected: Answered) {
    let mut d = domain();
    assert_eq!(application(&mut d, ask, None).as_ref(), [Request::ApplicationDone { request: REQUEST, token: ANSWER }]);
    assert_eq!(d.application_answer(ANSWER), Some(expected));
}
#[test]
fn an_absent_forge_cannot_be_removed() {
    refused(
        Ask::RemoveForge { forge: REPO.forge },
        Answered::RemoveForge(RemoveAnswer::Refused(RemoveRefusal::NoSuchForge)),
    );
}
#[test]
fn start_and_end_playground_are_refused_not_offered() {
    for (ask, expected) in [
        (Ask::StartPlayground, Answered::StartPlayground(StartAnswer::NotOffered)),
        (Ask::EndPlayground, Answered::EndPlayground(EndAnswer::NotOffered)),
    ] {
        refused(ask, expected);
    }
}
#[test]
fn connect_to_an_absent_kept_forge_is_refused_no_such_forge() {
    for forge in [ConnectForge::Existing(2), ConnectForge::Existing(3)] {
        let mut d = domain();
        let ask = Ask::Connect { forge, how: How::GitHubManifest { back: None } };
        assert_eq!(
            application(&mut d, ask, Some(VALUE)).as_ref(),
            [
                Request::CarriedUnused { number: VALUE.number },
                Request::ApplicationDone { request: REQUEST, token: ANSWER },
            ]
        );
        assert_eq!(
            d.application_answer(ANSWER),
            Some(Answered::Connect(ConnectAnswer::Refused(ForgeRefusal::NoSuchForge)))
        );
    }
}
fn setup_configured(number: u16, mode: Mode, asked: bool) -> Forge {
    forge_mode(number, Kind::Forgejo, mode, asked)
}
fn setup() -> Ask {
    Ask::SetUp {
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        project: SetUpProject::Existing(5),
        playground: false,
    }
}
#[test]
fn set_up_sets_up_the_asked_owned_forge() {
    for other in [Forge::connected(2, Kind::GitHub, &[]), setup_configured(4, Mode::Owned, true)] {
        let mut d = setup_domain(&[other, setup_configured(3, Mode::Owned, true)], true);
        outputs(&mut d, Event::Connection { forge: 3, state: Connection::NotSetUp });
        assert_eq!(crate::asks::set_up_forge(&d), Some(3));
        assert_eq!(
            application(&mut d, setup(), None).as_ref(),
            [
                Request::MarkedForges,
                Request::SetUp {
                    forge: 3,
                    request: REQUEST,
                    org: b"work".as_slice().into(),
                    repository: b"hello".as_slice().into(),
                    playground: false
                },
            ]
        );
    }
}
#[test]
fn set_up_without_an_owned_asked_forge_is_not_offered() {
    let rows: [Box<[Forge]>; 3] = [
        Box::new([]),
        Box::new([setup_configured(3, Mode::Owned, false)]),
        Box::new([setup_configured(3, Mode::Managed, true)]),
    ];
    for forges in rows {
        let mut d = setup_domain(&forges, true);
        assert_eq!(
            application(&mut d, setup(), None).as_ref(),
            [Request::ApplicationDone { request: REQUEST, token: ANSWER }]
        );
        assert_eq!(d.application_answer(ANSWER), Some(Answered::SetUp(SetUpAnswer::Refused(SetUpRefusal::NotOffered))));
    }
}
#[test]
fn a_row_is_finished_once_connected_with_a_repository() {
    for (connected, adopted, finished) in
        [(false, false, false), (true, false, false), (false, true, false), (true, true, true)]
    {
        let mut d = setup_domain(&[setup_configured(REPO.forge, Mode::Connected, false)], false);
        if connected {
            outputs(
                &mut d,
                Event::Connection {
                    forge: REPO.forge,
                    state: Connection::Connected { id: 7, login: b"writer".as_slice().into() },
                },
            );
        }
        if adopted {
            outputs(&mut d, Event::Restore { record: Stored::Repository(super::adopted()) });
        }
        assert_eq!(d.forges().rows[0].finished, finished);
    }
}
#[test]
fn the_demo_forge_row_says_playground() {
    let mut d =
        setup_domain(&[setup_configured(REPO.forge, Mode::Fake, true), setup_configured(3, Mode::Owned, true)], true);
    d.configure_demo(Some(DemoConfig {
        forge: None,
        provider: 1,
        adoption: fixtures::adoption(),
        deadline: Duration::from_secs(10),
    }));
    let rows = d.forges().rows;
    assert!(rows[0].playground);
    assert!(!rows[1].playground);
}
