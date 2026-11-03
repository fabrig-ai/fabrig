//! Forge-making acts keep their records and restartable intent (temper/docs/design/protocol/forge.md, 2.2; temper/docs/design/protocol/setup.md, 6.7).
#![expect(clippy::disallowed_methods, reason = "ordinary assertions inspect captured requests")]
#![expect(clippy::match_like_matches_macro, reason = "explicit test patterns follow the step subset")]
use super::*;
use core::num::NonZeroU64;
use jig_domain_core::Carried;
use jig_domain_people::Role as PartyRole;

const VALUE: Carried = Carried { number: 1_u64 << 63_u32, digest: [17; 32] };
fn place(host: &[u8]) -> Place {
    Place { api: Address { host: host.into(), port: 80, tls: None }, git: host.into(), web: Some(host.into()) }
}
fn fresh(under_box: bool) -> Domain {
    let mut d = setup_domain(&[], under_box);
    d.set_machine(Machine {
        owned: Some(place(b"owned.example")),
        fake: None,
        github: place(b"github.example"),
        github_git_program: Some(b"/data/bin/github-git".as_slice().into()),
    });
    d
}
fn setup(forge: Making) -> Ask {
    Ask::SetUp {
        forge,
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        project: SetUpProject::New(b"hello".as_slice().into()),
        playground: false,
    }
}
fn admin() -> How {
    How::ForgejoAdmin {
        first: Some(FirstAsk {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: SetUpProject::New(b"hello".as_slice().into()),
            label: b"docker".as_slice().into(),
        }),
    }
}
fn application(d: &mut Domain, number: u64, ask: Ask) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number),
            retention: Duration::from_secs(60),
            ask,
            role: Some(PartyRole::Owner),
            carried: Some(VALUE),
        },
    )
}
fn saved(out: &[Request]) -> ForgeRecord {
    out.iter()
        .find_map(
            |r| if let Request::Save { record: Stored::Forge(record), .. } = r { Some(record.clone()) } else { None },
        )
        .expect("the act saves its forge")
}
fn assert_setup_refusal(d: &Domain, token: u64, why: SetUpRefusal) {
    assert_eq!(d.application_answer(Token::new(token)), Some(Answered::SetUp(SetUpAnswer::Refused(why))));
}
#[test]
fn set_up_new_owned_makes_a_record_and_asks_the_box() {
    let mut d = fresh(true);
    let out = application(&mut d, 1, setup(Making::New(ForgeAsk::Owned)));
    let record = saved(&out);
    assert_eq!(record.number, 1);
    assert_eq!(record.kind, Kind::Forgejo);
    assert_eq!(record.mode, Mode::Owned);
    assert_eq!(record.state, ForgeState::Kept);
    assert_eq!(record.place, place(b"owned.example"));
    assert_eq!(record.bot.login.as_ref(), b"temper");
    assert!(record.asked);
    assert_eq!(record.first.as_ref().unwrap().org.as_ref(), b"work");
    assert_eq!(record.first.as_ref().unwrap().label.as_ref(), b"temper-host");
    assert_eq!(d.record(1), Some(&record));
    assert_eq!(d.forges().rows[0].first.as_ref().unwrap().state, FirstState::Making);
    assert!(out.contains(&Request::ForgeKept(record)));
    let saved_at = out
        .iter()
        .position(|r| if let Request::Save { record: Stored::Forge(_), .. } = r { true } else { false })
        .unwrap();
    let setup_at = out.iter().position(|r| if let Request::SetUp { forge: 1, .. } = r { true } else { false }).unwrap();
    assert!(saved_at < setup_at);
    assert!(out.contains(&Request::SetUp {
        forge: 1,
        request: Token::new(1),
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        playground: false
    }));
    outputs(&mut d, Event::SetUpAnswered { forge: 1, request: Token::new(1), answer: SetUpAnswer::SettingUp });
    assert_eq!(d.application_answer(Token::new(1)), Some(Answered::SetUp(SetUpAnswer::SettingUp)));
}
#[test]
fn a_second_owned_set_up_is_refused() {
    for (state, why) in [
        (Connection::SettingUp { step: None }, SetUpRefusal::Busy),
        (Connection::NotSetUp, SetUpRefusal::NotOffered),
        (Connection::SetUpFailed { why: b"failed".as_slice().into() }, SetUpRefusal::NotOffered),
        (Connection::Connected { id: 7, login: b"temper".as_slice().into() }, SetUpRefusal::NotOffered),
    ] {
        let mut d = fresh(true);
        application(&mut d, 1, setup(Making::New(ForgeAsk::Owned)));
        outputs(&mut d, Event::Connection { forge: 1, state });
        let out = application(&mut d, 2, setup(Making::New(ForgeAsk::Owned)));
        assert_setup_refusal(&d, 2, why);
        assert_eq!(d.records.len(), 1);
        assert!(!out.iter().any(|r| if let Request::Save { .. } | Request::SetUp { .. } = r { true } else { false }));
    }
}
#[test]
fn connect_new_github_makes_a_record_at_the_machines_place() {
    for how in [
        How::GitHubApp { app: NonZeroU64::new(7).unwrap(), read_only: false },
        How::GitHubApp { app: NonZeroU64::new(7).unwrap(), read_only: true },
        How::GitHubManifest { back: None },
    ] {
        let mut d = fresh(true);
        let out = application(&mut d, 1, Ask::Connect { forge: Making::New(ForgeAsk::GitHub), how: how.clone() });
        let record = saved(&out);
        assert_eq!(record.place, place(b"github.example"));
        assert_eq!(record.git_program.as_deref(), Some(b"/data/bin/github-git".as_slice()));
        assert_eq!(record.kind, Kind::GitHub);
        assert_eq!(record.mode, Mode::Connected);
        assert_eq!(record.first, None);
        assert!(!record.asked);
        match how {
            How::GitHubApp { read_only, .. } => {
                assert_eq!(record.read_only, read_only);
                assert!(out.contains(&Request::Connect { forge: 1, request: Token::new(1), how, carried: VALUE }));
            }
            How::GitHubManifest { .. } => {
                assert!(out.contains(&Request::Manifest { forge: 1, request: Token::new(1), back: None }));
            }
            How::ForgejoAdmin { .. } | How::ForgejoTokens => unreachable!(),
        }
    }
}
#[test]
fn connect_new_managed_makes_a_record_with_its_intent() {
    let mut d = fresh(false);
    let how = admin();
    let out = application(
        &mut d,
        1,
        Ask::Connect {
            forge: Making::New(ForgeAsk::Managed {
                place: place(b"managed.example"),
                git_program: Some(b"/data/bin/managed-git".as_slice().into()),
            }),
            how: how.clone(),
        },
    );
    let record = saved(&out);
    assert_eq!(record.mode, Mode::Managed);
    assert_eq!(record.kind, Kind::Forgejo);
    assert_eq!(record.place, place(b"managed.example"));
    assert_eq!(record.git_program.as_deref(), Some(b"/data/bin/managed-git".as_slice()));
    assert_eq!(record.first.as_ref().unwrap().label.as_ref(), b"docker");
    assert_eq!(record.first.as_ref().unwrap().project, SetUpProject::New(b"hello".as_slice().into()));
    assert!(out.contains(&Request::Connect { forge: 1, request: Token::new(1), how, carried: VALUE }));
}
#[test]
fn connect_new_connected_makes_a_record_with_its_place_and_bot() {
    let mut d = fresh(false);
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumber { last: 8 } });
    let out = application(
        &mut d,
        1,
        Ask::Connect {
            forge: Making::New(ForgeAsk::Connected {
                place: place(b"connected.example"),
                git_program: Some(b"/data/bin/connected-git".as_slice().into()),
            }),
            how: How::ForgejoTokens,
        },
    );
    let record = saved(&out);
    assert_eq!(record.number, 9);
    assert_eq!(record.kind, Kind::Forgejo);
    assert_eq!(record.mode, Mode::Connected);
    assert_eq!(record.state, ForgeState::Kept);
    assert_eq!(record.place, place(b"connected.example"));
    assert_eq!(record.git_program.as_deref(), Some(b"/data/bin/connected-git".as_slice()));
    assert_eq!(record.bot, domain::make::bot());
    assert!(!record.read_only);
    assert!(!record.asked);
    assert_eq!(record.first, None);
    assert_eq!(d.record(9), Some(&record));
    assert_eq!(d.connection(9), Connection::Connecting);
    assert!(out.contains(&Request::ForgeKept(record)));
    assert!(out.contains(&Request::Connect {
        forge: 9,
        request: Token::new(1),
        how: How::ForgejoTokens,
        carried: VALUE
    }));
}
#[test]
fn a_restart_before_the_first_repository_rebuilds_the_intent_from_the_record() {
    for owned in [true, false] {
        let mut d = fresh(true);
        let ask = if owned {
            setup(Making::New(ForgeAsk::Owned))
        } else {
            Ask::Connect {
                forge: Making::New(ForgeAsk::Managed { place: place(b"managed.example"), git_program: None }),
                how: admin(),
            }
        };
        let out = application(&mut d, 1, ask);
        let record = saved(&out);
        outputs(&mut d, Event::FirstFailed { forge: 1, step: CreateStep::File });
        assert_eq!(d.record(1).unwrap().first, record.first);
        let mut restarted = fresh(true);
        outputs(&mut restarted, Event::Restore { record: Stored::ForgeNumber { last: 1 } });
        outputs(&mut restarted, Event::Restore { record: Stored::Forge(record.clone()) });
        let first = restarted.forges().rows[0].first.clone().unwrap();
        assert_eq!(first.state, FirstState::Making);
        assert_eq!(first.org, record.first.as_ref().unwrap().org);
        assert_eq!(first.repository, record.first.as_ref().unwrap().repository);
        assert_eq!(first.project, record.first.as_ref().unwrap().project);
        assert_eq!(first.label, record.first.as_ref().unwrap().label);
        if !owned {
            let out = application(
                &mut restarted,
                2,
                Ask::Connect { forge: Making::Kept(1), how: How::ForgejoAdmin { first: None } },
            );
            assert!(out.contains(&Request::Connect { forge: 1, request: Token::new(2), how: admin(), carried: VALUE }));
            assert_eq!(restarted.forges().rows[0].first.as_ref().unwrap().state, FirstState::Making);
        }
    }
}
#[test]
fn kept_is_todays_path() {
    let mut d = setup_domain(&configured()[..1], true);
    outputs(&mut d, Event::Connection { forge: 1, state: Connection::NotSetUp });
    let out = application(&mut d, 1, setup(Making::Kept(1)));
    assert!(out.iter().any(|r| if let Request::SetUp { forge: 1, .. } = r { true } else { false }));
    assert!(d.records.is_empty());
    let mut d = fresh(true);
    let how = How::GitHubApp { app: NonZeroU64::new(7).unwrap(), read_only: false };
    application(&mut d, 1, Ask::Connect { forge: Making::New(ForgeAsk::GitHub), how: how.clone() });
    outputs(&mut d, Event::ConnectEnded { forge: 1, request: Token::new(1), connected: false, why: None, stop: None });
    let out = application(&mut d, 2, Ask::Connect { forge: Making::Kept(1), how: how.clone() });
    assert!(out.contains(&Request::Connect { forge: 1, request: Token::new(2), how, carried: VALUE }));
    assert!(!out.iter().any(|r| if let Request::ForgeKept(_) = r { true } else { false }));
}
#[test]
fn an_unoffered_machine_or_box_makes_nothing() {
    let mut no_box = fresh(false);
    application(&mut no_box, 1, setup(Making::New(ForgeAsk::Owned)));
    assert_setup_refusal(&no_box, 1, SetUpRefusal::NotUnderTheBox);
    assert!(no_box.records.is_empty());
    for machine in
        [None, Some(Machine { owned: None, fake: None, github: place(b"github.example"), github_git_program: None })]
    {
        let mut d = setup_domain(&[], true);
        d.machine = machine;
        let out = application(&mut d, 1, setup(Making::New(ForgeAsk::Owned)));
        assert_setup_refusal(&d, 1, SetUpRefusal::NotOffered);
        assert!(d.records.is_empty());
        assert!(!out.iter().any(|r| if let Request::Save { .. } = r { true } else { false }));
    }
    let mut d = setup_domain(&[], true);
    application(
        &mut d,
        1,
        Ask::Connect { forge: Making::New(ForgeAsk::GitHub), how: How::GitHubManifest { back: None } },
    );
    assert_eq!(
        d.application_answer(Token::new(1)),
        Some(Answered::Connect(ConnectAnswer::Refused(ForgeRefusal::NoSuchForge)))
    );
    assert!(d.records.is_empty());
}
#[test]
fn a_refused_connect_saves_no_record_and_a_failed_connect_keeps_it() {
    for (role, carried, how, why) in [
        (None, Some(VALUE), admin(), ForgeRefusal::Role),
        (Some(PartyRole::Owner), None, admin(), ForgeRefusal::NoKey),
        (
            Some(PartyRole::Owner),
            Some(VALUE),
            How::ForgejoAdmin {
                first: Some(FirstAsk {
                    org: b"Temper".as_slice().into(),
                    repository: b"hello".as_slice().into(),
                    project: SetUpProject::Existing(1),
                    label: b"docker".as_slice().into(),
                }),
            },
            ForgeRefusal::FirstIsBot,
        ),
    ] {
        let mut d = fresh(true);
        let out = outputs(
            &mut d,
            Event::Application {
                request: Token::new(1),
                token: Token::new(1),
                retention: Duration::from_secs(60),
                ask: Ask::Connect {
                    forge: Making::New(ForgeAsk::Managed { place: place(b"managed.example"), git_program: None }),
                    how,
                },
                role,
                carried,
            },
        );
        assert_eq!(d.application_answer(Token::new(1)), Some(Answered::Connect(ConnectAnswer::Refused(why))));
        assert!(d.records.is_empty());
        assert!(!out.iter().any(|r| if let Request::Save { .. } = r { true } else { false }));
    }
    let mut d = fresh(true);
    application(
        &mut d,
        1,
        Ask::Connect {
            forge: Making::New(ForgeAsk::Managed { place: place(b"managed.example"), git_program: None }),
            how: admin(),
        },
    );
    outputs(
        &mut d,
        Event::ConnectEnded {
            forge: 1,
            request: Token::new(1),
            connected: false,
            why: Some(b"failed".as_slice().into()),
            stop: None,
        },
    );
    assert!(d.record(1).is_some());
}
#[test]
fn the_kept_forge_bound_and_spent_numbers_refuse_without_saving() {
    for full in [false, true] {
        let mut d = fresh(true);
        if full {
            for number in 1..=LIMITS.forges {
                application(
                    &mut d,
                    u64::from(number),
                    Ask::Connect { forge: Making::New(ForgeAsk::GitHub), how: How::GitHubManifest { back: None } },
                );
                outputs(
                    &mut d,
                    Event::ConnectAnswered {
                        forge: u16::try_from(number).unwrap(),
                        request: Token::new(u64::from(number)),
                        answer: ConnectAnswer::Link(b"/app".as_slice().into()),
                    },
                );
            }
        } else {
            outputs(&mut d, Event::Restore { record: Stored::ForgeNumber { last: u16::MAX } });
        }
        let before = d.records.len();
        let last = d.forge_number;
        let out = application(
            &mut d,
            10,
            Ask::Connect { forge: Making::New(ForgeAsk::GitHub), how: How::GitHubManifest { back: None } },
        );
        assert_eq!(
            d.application_answer(Token::new(10)),
            Some(Answered::Connect(ConnectAnswer::Refused(ForgeRefusal::Busy)))
        );
        assert_eq!(d.records.len(), before);
        assert_eq!(d.forge_number, last);
        assert!(!out.iter().any(|r| if let Request::Save { .. } = r { true } else { false }));
    }
}
