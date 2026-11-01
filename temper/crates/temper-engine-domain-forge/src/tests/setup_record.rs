//! Owned setup from kept records and the machine template (temper/docs/plan/playground-reset-r4/notes/contracts.md, 5 and 6).
use super::*;
use crate::record as r;

fn owned(number: u16) -> Forge {
    Forge {
        number,
        kind: Kind::Forgejo,
        identity: r::Identity { number: u32::from(number), ..records::forge().identity },
        setup: Some(r::Setup {
            directory: b"forge".as_slice().into(),
            bot: r::Account { login: b"temper".as_slice().into(), email: Box::new([]) },
            orgs: Box::new([]),
            asked: true,
            mode: r::SetupMode::Owned(r::OwnedEntry {
                public: None,
                fetch: r::Fetch { forgejo: true, runner: false, mirror: None },
                forgejo: b"forgejo".as_slice().into(),
                runner: None,
                listen: ([127, 0, 0, 1], 3000).into(),
                person: r::Account { login: b"person".as_slice().into(), email: Box::new([]) },
            }),
        }),
        ..records::forge()
    }
}
fn started(seed: &[Forge], template: bool, under_box: bool) -> Domain {
    let mut d = Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config { namespace: b"setup".as_slice().into(), writers: Box::new([]) },
        seed,
        under_box,
    )
    .unwrap();
    if template {
        d.configure_owned(Some(owned(0)));
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    for row in seed {
        outputs(&mut d, Event::Connection { forge: row.number, state: Connection::NotSetUp });
    }
    d
}
fn ask(d: &mut Domain, serial: u64, org: &[u8]) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(serial),
            token: Token::new(serial.checked_add(1).unwrap()),
            retention: Duration::from_secs(60),
            role: None,
            carried: None,
            ask: Ask::SetUp {
                org: org.into(),
                repository: b"hello".as_slice().into(),
                project: SetUpProject::Existing(1),
                playground: false,
            },
        },
    )
}
fn refused(d: &mut Domain, serial: u64, org: &[u8], why: SetUpRefusal) {
    let count = d.forge_records.len();
    let next = d.next_forge();
    assert_eq!(
        ask(d, serial, org).as_ref(),
        [Request::ApplicationDone { request: Token::new(serial), token: Token::new(serial.checked_add(1).unwrap()) }]
    );
    assert_eq!(
        d.application_answer(Token::new(serial.checked_add(1).unwrap())),
        Some(Answered::SetUp(SetUpAnswer::Refused(why)))
    );
    assert_eq!(d.forge_records.len(), count);
    assert_eq!(d.next_forge(), next);
}
#[test]
fn set_up_with_only_the_template_makes_and_sets_up_its_forge() {
    let mut d = started(&[records::forge()], true, true);
    let emitted = ask(&mut d, 91, b"work");
    let mut expected = owned(3);
    expected.first = Some(r::FirstIntent {
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        project: SetUpProject::Existing(1),
        label: b"temper-host".as_slice().into(),
    });
    expected.identity.number = 2;
    assert_eq!(d.forge_record(3), Some(&expected));
    assert_eq!(
        emitted.as_ref(),
        [
            Request::Save {
                record: Stored::Forge(Forge { first: None, ..expected.clone() }),
                release: None,
                read_afresh: None
            },
            Request::Save { record: Stored::Forges { next: 4 }, release: None, read_afresh: None },
            Request::MarkedForges,
            Request::ForgeKept { forge: Box::new(Forge { first: None, ..expected.clone() }) },
            Request::Save { record: Stored::Forge(expected), release: None, read_afresh: None },
            Request::MarkedForges,
            Request::SetUp {
                forge: 3,
                request: Token::new(91),
                org: b"work".as_slice().into(),
                repository: b"hello".as_slice().into(),
                playground: false,
            },
        ]
    );
    assert_eq!(d.forges().rows[1].first.as_ref().unwrap().project, SetUpProject::Existing(1));
}
#[test]
fn set_up_sets_up_a_kept_owned_forge_without_making_one() {
    for template in [false, true] {
        let mut d = started(&[owned(3)], template, true);
        let emitted = ask(&mut d, 91, b"work");
        let mut expected = owned(3);
        expected.first = Some(r::FirstIntent {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: SetUpProject::Existing(1),
            label: b"temper-host".as_slice().into(),
        });
        assert_eq!(
            emitted.as_ref(),
            &[
                Request::Save { record: Stored::Forge(expected.clone()), release: None, read_afresh: None },
                Request::MarkedForges,
                setup_request(3, 91)
            ]
        );
        assert_eq!(d.forge_record(3), Some(&expected));
        assert_eq!(d.forge_records.len(), 1);
        assert_eq!(d.next_forge(), 4);
    }
}
#[test]
fn set_up_with_neither_is_not_offered() {
    refused(&mut started(&[], false, true), 91, b"work", SetUpRefusal::NotOffered);
}
#[test]
fn a_refused_set_up_makes_no_forge() {
    refused(&mut started(&[], true, true), 91, b"bad/name", SetUpRefusal::Invalid);
    refused(&mut started(&[], true, false), 91, b"work", SetUpRefusal::NotUnderTheBox);
    let mut d = started(&[], true, true);
    outputs(&mut d, Event::Connection { forge: 9, state: Connection::SettingUp { step: None } });
    refused(&mut d, 91, b"work", SetUpRefusal::Busy);
}
#[test]
fn a_second_set_up_while_one_sets_up_is_busy() {
    let mut d = started(&[], true, true);
    assert_eq!(ask(&mut d, 91, b"work").last(), Some(&setup_request(1, 91)));
    outputs(&mut d, Event::Connection { forge: 1, state: Connection::SettingUp { step: None } });
    refused(&mut d, 93, b"work", SetUpRefusal::Busy);
}
#[test]
fn removing_records_do_not_take_the_owned_candidate() {
    let mut d = started(&[Forge { removing: true, ..owned(3) }], true, true);
    assert_eq!(ask(&mut d, 91, b"work").last(), Some(&setup_request(4, 91)));
}
#[test]
fn make_refusals_leave_no_new_forge() {
    let mut full = started(&[], true, true);
    for number in 1..=LIMITS.forges {
        outputs(
            &mut full,
            Event::Restore {
                record: Stored::Forge(Forge { number: u16::try_from(number).unwrap(), ..records::forge() }),
            },
        );
    }
    refused(&mut full, 91, b"work", SetUpRefusal::NotOffered);
    let mut exhausted = started(&[], true, true);
    outputs(&mut exhausted, Event::Restore { record: Stored::Forges { next: u16::MAX } });
    refused(&mut exhausted, 91, b"work", SetUpRefusal::NotOffered);
    let mut invalid = started(&[], true, true);
    invalid.make.owned.as_mut().unwrap().git = skein_lib::bytes::zeroed(r::PATH_BYTES + 1);
    refused(&mut invalid, 91, b"work", SetUpRefusal::NotOffered);
    let mut busy = owned(1);
    busy.setup.as_mut().unwrap().asked = false;
    refused(&mut started(&[busy], true, true), 91, b"work", SetUpRefusal::Busy);
}

fn setup_request(forge: u16, serial: u64) -> Request {
    Request::SetUp {
        forge,
        request: Token::new(serial),
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        playground: false,
    }
}
