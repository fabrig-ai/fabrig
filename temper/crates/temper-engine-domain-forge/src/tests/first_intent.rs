//! Intent persistence at admission, completion and restore (contracts, 2).
use super::*;
use crate::record as r;

fn intent() -> r::FirstIntent {
    r::FirstIntent {
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        project: SetUpProject::New(b"Default".as_slice().into()),
        label: b"temper-host".as_slice().into(),
    }
}
fn owned() -> Forge {
    Forge {
        number: 1,
        kind: Kind::Forgejo,
        setup: Some(r::Setup {
            directory: b"forge".as_slice().into(),
            asked: true,
            orgs: Box::new([]),
            bot: r::Account { login: b"temper".as_slice().into(), email: Box::new([]) },
            mode: r::SetupMode::Owned(r::OwnedEntry {
                public: None,
                fetch: r::Fetch { forgejo: true, runner: false, mirror: None },
                forgejo: b"forgejo".as_slice().into(),
                runner: None,
                listen: ([127, 0, 0, 1], 3000).into(),
                person: r::Account { login: b"owner".as_slice().into(), email: Box::new([]) },
            }),
        }),
        ..records::forge()
    }
}
fn cold(seed: &[Forge]) -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config { namespace: b"first-intent".as_slice().into(), writers: Box::new([]) },
        &[],
        &[],
        seed,
        true,
    )
    .unwrap()
}
fn started(seed: &[Forge]) -> Domain {
    let mut d = cold(seed);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    outputs(&mut d, Event::Connection { forge: 1, state: Connection::NotSetUp });
    d
}
fn ask(d: &mut Domain, ask: Ask, carried: Option<jig_domain_core::Carried>) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(91),
            token: Token::new(92),
            retention: Duration::from_secs(60),
            role: Some(jig_domain_people::Role::Owner),
            carried,
            ask,
        },
    )
}
fn admitted() -> Domain {
    let mut d = started(&[owned()]);
    let first = intent();
    let out = ask(
        &mut d,
        Ask::SetUp { org: first.org, repository: first.repository, project: first.project, playground: false },
        None,
    );
    let mut expected = owned();
    expected.first = Some(intent());
    assert!(out.contains(&Request::Save { record: Stored::Forge(expected.clone()), release: None, read_afresh: None }));
    assert_eq!(d.forge_record(1), Some(&expected));
    d
}
#[test]
fn set_up_keeps_its_intent_in_the_record() {
    let d = admitted();
    assert_eq!(d.forges().rows[0].first.as_ref().unwrap().state, FirstState::Making);
}
#[test]
fn a_made_first_repository_clears_the_intent() {
    let mut d = admitted();
    let out = outputs(&mut d, Event::FirstMade { forge: 1, id: 7 });
    assert_eq!(d.forge_record(1).unwrap().first, None);
    assert!(out.contains(&Request::Save { record: Stored::Forge(owned()), release: None, read_afresh: None }));
    assert_eq!(d.forges().rows[0].first.as_ref().unwrap().state, FirstState::Made { id: 7 });
}
#[test]
fn a_failed_first_repository_keeps_the_intent() {
    let mut d = admitted();
    let out = outputs(&mut d, Event::FirstFailed { forge: 1, step: CreateStep::File });
    assert_eq!(d.forge_record(1).unwrap().first, Some(intent()));
    assert_eq!(out.as_ref(), [Request::MarkedForges]);
}
#[test]
fn a_restored_intent_is_making_again() {
    let mut old = admitted();
    outputs(&mut old, Event::FirstFailed { forge: 1, step: CreateStep::File });
    let row = old.forge_record(1).unwrap().clone();
    let mut d = cold(&[]);
    outputs(&mut d, Event::Restore { record: Stored::Forge(row) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    let first = d.forges().rows[0].first.clone().unwrap();
    assert_eq!(first.state, FirstState::Making);
    assert_eq!(
        (first.org, first.repository, first.project, first.label),
        (intent().org, intent().repository, intent().project, intent().label)
    );
}
#[test]
fn an_admin_connect_keeps_its_intent_and_a_reconnect_preserves_it() {
    let mut row = owned();
    row.setup.as_mut().unwrap().mode = r::SetupMode::Managed { admin_token: None };
    let mut d = started(&[row]);
    let first = intent();
    let out = ask(
        &mut d,
        Ask::Connect {
            forge: ConnectForge::Existing(1),
            how: How::ForgejoAdmin {
                first: Some(FirstAsk {
                    org: first.org,
                    repository: first.repository,
                    project: first.project,
                    label: first.label,
                }),
            },
        },
        Some(jig_domain_core::Carried { number: 1_u64 << 63_u32, digest: [7; 32] }),
    );
    assert_eq!(d.forge_record(1).unwrap().first, Some(intent()));
    assert!(out.contains(&Request::Save {
        record: Stored::Forge(d.forge_record(1).unwrap().clone()),
        release: None,
        read_afresh: None
    }));
    let record = d.forge_record(1).unwrap().clone();
    let mut d = cold(&[]);
    outputs(&mut d, Event::Restore { record: Stored::Forge(record) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    ask(
        &mut d,
        Ask::Connect { forge: ConnectForge::Existing(1), how: How::ForgejoAdmin { first: None } },
        Some(jig_domain_core::Carried { number: 1_u64 << 63_u32, digest: [7; 32] }),
    );
    assert_eq!(d.forge_record(1).unwrap().first, Some(intent()));
    assert!(d.forges().rows[0].first.is_some());
}

#[test]
fn retained_repositories_take_the_project_even_when_the_forge_is_ended_or_removing() {
    let mut d = admitted();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    assert!(d.project_has_repository(5));
    assert!(!d.project_has_repository(6));
    let mut record = d.forge_record(1).unwrap().clone();
    record.removing = true;
    outputs(&mut d, Event::Restore { record: Stored::Forge(record) });
    assert!(d.project_has_repository(5));
    d.configure_demo(Some(DemoConfig {
        forge: None,
        provider: 1,
        adoption: fixtures::adoption(),
        deadline: Duration::from_secs(10),
    }));
    outputs(&mut d, Event::Restore { record: Stored::Demo(Demo { forge: 1, state: DemoState::Ended }) });
    assert!(d.forge_ended(1));
    assert!(d.project_has_repository(5));
}
#[test]
fn setup_refusals_share_the_setup_answer_room_and_expiry() {
    let mut d = started(&[owned()]);
    let mut out = Queue::uncapped();
    for number in 1..=LIMITS.applications {
        d.refuse_set_up(
            &env(),
            Token::new(u64::from(number)),
            Token::new(u64::from(number)),
            Duration::from_secs(1),
            SetUpRefusal::NewProject,
            &mut out,
        );
        assert_eq!(
            out.pop(),
            Some(Request::ApplicationDone {
                request: Token::new(u64::from(number)),
                token: Token::new(u64::from(number)),
            })
        );
    }
    d.refuse_set_up(&env(), Token::new(91), Token::new(92), Duration::from_secs(1), SetUpRefusal::NewProject, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::ApplicationAgain { request: Token::new(91), why: jig_domain_people::Again::Busy })
    );
    assert_eq!(d.application_answer(Token::new(92)), None);
    let mut later = env();
    later.wall = Wall::from_nanos(later.wall.as_nanos().checked_add(Duration::from_secs(1).as_nanos()).unwrap());
    d.refuse_set_up(&later, Token::new(91), Token::new(92), Duration::from_secs(1), SetUpRefusal::NewProject, &mut out);
    assert_eq!(out.pop(), Some(Request::ApplicationDone { request: Token::new(91), token: Token::new(92) }));
    assert_eq!(
        d.application_answer(Token::new(92)),
        Some(Answered::SetUp(SetUpAnswer::Refused(SetUpRefusal::NewProject)))
    );
    assert!(out.is_empty());
}
