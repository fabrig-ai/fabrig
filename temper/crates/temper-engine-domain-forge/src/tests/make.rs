//! Forge decisions and restart intent (temper/docs/design/draft/playground-reset.md, 3, 6.4).
use super::*;

fn application(d: &mut Domain, number: u64, ask: Ask) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number),
            retention: Duration::from_secs(60),
            ask,
            role: Some(jig_domain_people::Role::Owner),
            carried: None,
        },
    )
}

fn make(d: &mut Domain, number: u64, choice: ForgeChoice, answer: MakeAnswer) -> Box<[Request]> {
    let out = application(d, number, Ask::MakeForge { choice });
    assert_eq!(d.application_answer(Token::new(number)), Some(Answered::MakeForge(answer)));
    out
}

fn github(d: &mut Domain, number: u16) -> Box<[Request]> {
    make(d, u64::from(number), ForgeChoice::GitHub, MakeAnswer::Made { forge: number })
}

#[test]
fn the_first_make_is_forge_one_and_each_next_is_never_reused() {
    let mut d = setup_domain(&[], true);
    github(&mut d, 1);
    github(&mut d, 2);
    let second = d.records.get(&2).unwrap().as_ref().clone();
    let mut restored = setup_domain(&[], true);
    outputs(&mut restored, Event::Restore { record: Stored::Forge(second) });
    outputs(&mut restored, Event::Restore { record: Stored::ForgeNumbers { next: d.next_number } });
    outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    github(&mut restored, 3);
    assert_eq!(restored.next_number, 4);
    assert!(!restored.records.contains_key(&1));
}

#[test]
fn the_last_number_given_leaves_none() {
    let mut d = setup_domain(&[], true);
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumbers { next: u16::MAX } });
    let out = github(&mut d, u16::MAX);
    assert!(out.contains(&Request::Save {
        read_afresh: None,
        release: None,
        record: Stored::ForgeNumbers { next: 0 }
    }));
    assert_eq!(d.next_number, 0);
    make(&mut d, 100_000, ForgeChoice::GitHub, MakeAnswer::Refused(MakeRefusal::Full));
    let mut restored = setup_domain(&[], true);
    outputs(&mut restored, Event::Restore { record: Stored::ForgeNumbers { next: 0 } });
    make(&mut restored, 1, ForgeChoice::GitHub, MakeAnswer::Refused(MakeRefusal::Full));
    assert!(restored.records.is_empty());
    assert!(restored.forge_ended(u16::MAX));
}

#[test]
fn makes_past_the_forge_limit_are_full() {
    let mut d = setup_domain(&[], true);
    for number in 1..=u16::try_from(LIMITS.forges).unwrap() {
        github(&mut d, number);
    }
    let mut removing = d.records.get(&1).unwrap().as_ref().clone();
    removing.state = ForgeState::Removing;
    outputs(&mut d, Event::Restore { record: Stored::Forge(removing) });
    let out = make(&mut d, 5, ForgeChoice::GitHub, MakeAnswer::Refused(MakeRefusal::Full));
    assert_eq!(out.as_ref(), [Request::ApplicationDone { request: Token::new(5), token: Token::new(5) }]);
    assert_eq!(d.next_number, 5);
}

fn owned() -> ForgeRecord {
    ForgeRecord {
        number: 0,
        kind: Kind::Forgejo,
        mode: Mode::Owned,
        setup: Some(b"{}".as_slice().into()),
        ..ForgeRecord::github_com()
    }
}

#[test]
fn owned_is_offered_only_from_the_machine_under_the_box_and_once() {
    for under_box in [false, true] {
        for template in [false, true] {
            let mut d = setup_domain(&[], under_box);
            if template {
                d.machine.owned = Some(owned());
            }
            if under_box && template {
                make(&mut d, 1, ForgeChoice::Owned, MakeAnswer::Made { forge: 1 });
                assert_eq!(d.records.get(&1).expect("kept forge").as_ref(), &ForgeRecord { number: 1, ..owned() });
                make(&mut d, 2, ForgeChoice::Owned, MakeAnswer::Refused(MakeRefusal::NotOffered));
                let mut removing = d.records.get(&1).unwrap().as_ref().clone();
                removing.state = ForgeState::Removing;
                outputs(&mut d, Event::Restore { record: Stored::Forge(removing) });
                make(&mut d, 3, ForgeChoice::Owned, MakeAnswer::Refused(MakeRefusal::NotOffered));
            } else {
                make(&mut d, 1, ForgeChoice::Owned, MakeAnswer::Refused(MakeRefusal::NotOffered));
                assert!(d.records.is_empty());
            }
        }
    }
}

fn forgejo(api: &[u8]) -> ForgeChoice {
    ForgeChoice::Forgejo { api: api.into(), git: b"https://forge.example".as_slice().into(), web: None }
}

#[test]
fn a_forgejo_elsewhere_takes_the_machines_git_program() {
    let programs: [Option<Box<[u8]>>; 2] = [Some(b"data/bin/git".as_slice().into()), None];
    for git_program in programs {
        let mut d = setup_domain(&[], true);
        d.machine.github.git_program = git_program.clone();
        d.machine.owned = Some(owned());
        let out = make(&mut d, 1, forgejo(b"forge.example"), MakeAnswer::Made { forge: 1 });
        let record = saved_record(&out);
        assert_eq!(record.git_program, git_program);
        assert_eq!(d.records.get(&1).unwrap().as_ref(), &record);

        let out = make(&mut d, 2, ForgeChoice::Owned, MakeAnswer::Made { forge: 2 });
        let record = saved_record(&out);
        assert_eq!(record, ForgeRecord { number: 2, ..owned() });
        assert_eq!(d.records.get(&2).unwrap().as_ref(), &record);
    }
}

#[test]
fn a_forgejo_elsewhere_is_managed_from_its_addresses() {
    for (address, host, port, plaintext) in [
        (b"forge.example".as_slice(), b"forge.example".as_slice(), 443, false),
        (b"forge.example:65535", b"forge.example", 65535, false),
        (b"localhost:1", b"localhost", 1, true),
        (b"127.0.0.1", b"127.0.0.1", 443, true),
        (b"::1", b"::1", 443, true),
        (b"[::1]", b"::1", 443, true),
        (b"[::1]:3000", b"::1", 3000, true),
    ] {
        let mut d = setup_domain(&[], false);
        make(&mut d, 1, forgejo(address), MakeAnswer::Made { forge: 1 });
        let record = d.records.get(&1).unwrap();
        assert_eq!(
            record.api,
            Api {
                host: host.into(),
                port,
                transport: if plaintext {
                    ApiTransport::Plaintext
                } else {
                    ApiTransport::Tls { server_name: host.into(), trust: None }
                }
            }
        );
        assert_eq!(record.kind, Kind::Forgejo);
        assert_eq!(record.mode, Mode::Managed);
        assert_eq!(record.git.as_ref(), b"https://forge.example");
        assert_eq!(record.git_program, None);
        assert_eq!(record.web, None);
        assert!(!record.read_only && !record.playground);
        assert_eq!(record.first, None);
        assert_eq!(
            record.bot,
            Bot {
                login: b"temper".as_slice().into(),
                name: b"Temper".as_slice().into(),
                email: b"temper@localhost".as_slice().into()
            }
        );
        assert_eq!(
            record.setup.as_deref(),
            Some(br#"{"bot":{"login":"temper","email":"temper@localhost"},"mode":"managed"}"#.as_slice())
        );
        let config = d.configured(1).unwrap();
        assert!(config.admin_connect && config.asked);
        assert_eq!(config.bot.as_ref(), b"temper");
    }
    for address in [
        b"".as_slice(),
        b":443",
        b"host:",
        b"host:0",
        b"host:65536",
        b"host:abc",
        b"host:1:2",
        b"https://host",
        b"host/path",
        b"host name",
        b"[::1]:",
        b"[::1]suffix",
    ] {
        let mut d = setup_domain(&[], false);
        make(&mut d, 1, forgejo(address), MakeAnswer::Refused(MakeRefusal::NotOffered));
        assert_eq!(d.next_number, 1);
    }
    for length in [253, 254] {
        let mut d = setup_domain(&[], false);
        let host = [b'a'; 254];
        make(
            &mut d,
            1,
            forgejo(&host[..length]),
            if length == 253 { MakeAnswer::Made { forge: 1 } } else { MakeAnswer::Refused(MakeRefusal::NotOffered) },
        );
    }
    for (git, web, admitted) in [(0, 0, false), (1, 0, true), (1024, 1024, true), (1025, 0, false), (1, 1025, false)] {
        let mut d = setup_domain(&[], false);
        let choice = ForgeChoice::Forgejo {
            api: b"host".as_slice().into(),
            git: skein_lib::bytes::zeroed(git),
            web: Some(skein_lib::bytes::zeroed(web)),
        };
        make(
            &mut d,
            1,
            choice,
            if admitted { MakeAnswer::Made { forge: 1 } } else { MakeAnswer::Refused(MakeRefusal::NotOffered) },
        );
    }
}

#[test]
fn records_allow_another_forge_to_be_made() {
    let mut d = setup_domain(&configured(), true);
    github(&mut d, 3);
    assert_eq!(d.records.len(), 3);
    assert_eq!(d.next_number, 4);
}

#[test]
fn a_made_record_is_saved_registered_and_answered_in_one_decision() {
    let mut d = setup_domain(&[], false);
    let mut expected = d.machine.github.clone();
    expected.number = 1;
    let out = github(&mut d, 1);
    assert_eq!(
        out.as_ref(),
        [
            Request::Save { read_afresh: None, release: None, record: Stored::Forge(expected.clone()) },
            Request::Register(expected.clone()),
            Request::MarkedForges,
            Request::Save { read_afresh: None, release: None, record: Stored::ForgeNumbers { next: 2 } },
            Request::ApplicationDone { request: Token::new(1), token: Token::new(1) },
        ]
    );
    assert_eq!(d.records.get(&1).expect("kept forge").as_ref(), &expected);
    assert_eq!(d.configured(1), Some(&expected.configured()));
}

const VALUE: jig_domain_core::Carried = jig_domain_core::Carried { number: 1_u64 << 63_u32, digest: [17; 32] };

fn connect(d: &mut Domain, number: u64, how: How) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number),
            retention: Duration::from_secs(60),
            ask: Ask::Connect { forge: 1, how },
            role: Some(jig_domain_people::Role::Owner),
            carried: Some(VALUE),
        },
    )
}

fn saved_record(out: &[Request]) -> ForgeRecord {
    let mut saved = None;
    for request in out {
        if let Request::Save { record: Stored::Forge(record), .. } = request {
            assert!(saved.is_none(), "one saved record per update");
            saved = Some(record.clone());
        }
    }
    let record = saved.expect("the intent is saved in this decision");
    assert!(out.contains(&Request::Register(record.clone())));
    record
}

fn restored_intent(record: ForgeRecord, expected: Option<&First>) {
    assert_eq!(record.first.as_ref(), expected);
    let mut restored = setup_domain(&[], true);
    fixtures::restore_records(&mut restored, &[record]);
    assert_eq!(restored.forges().rows[0].first.as_ref(), expected);
}

#[test]
fn the_first_repositorys_intent_is_kept_and_restored() {
    for admin in [false, true] {
        let mut d = setup_domain(&[], true);
        let out = if admin {
            make(&mut d, 1, forgejo(b"host"), MakeAnswer::Made { forge: 1 });
            connect(
                &mut d,
                2,
                How::ForgejoAdmin {
                    first: Some(FirstAsk {
                        org: b"work".as_slice().into(),
                        repository: b"hello".as_slice().into(),
                        project: SetUpProject::New(b"Project".as_slice().into()),
                        label: b"docker".as_slice().into(),
                    }),
                },
            )
        } else {
            d.machine.owned = Some(owned());
            make(&mut d, 1, ForgeChoice::Owned, MakeAnswer::Made { forge: 1 });
            outputs(&mut d, Event::Connection { forge: 1, state: Connection::NotSetUp });
            outputs(&mut d, setup_application(1, Token::new(2), Token::new(2)))
        };
        let mut expected = First {
            playground: false,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: if admin { SetUpProject::New(b"Project".as_slice().into()) } else { SetUpProject::Existing(1) },
            state: FirstState::Making,
            label: if admin { b"docker".as_slice().into() } else { b"temper-host".as_slice().into() },
        };
        restored_intent(saved_record(&out), Some(&expected));
        assert_eq!(d.forges().rows[0].first.as_ref(), Some(&expected));
        let out = outputs(&mut d, Event::FirstMade { forge: 1, id: 42 });
        expected.state = FirstState::Made { id: 42 };
        restored_intent(saved_record(&out), Some(&expected));
        for step in [CreateStep::Admin, CreateStep::Org, CreateStep::Repository, CreateStep::File] {
            let out = outputs(&mut d, Event::FirstFailed { forge: 1, step });
            expected.state = FirstState::Failed { step };
            restored_intent(saved_record(&out), Some(&expected));
        }
        if admin {
            let mut restored = setup_domain(&[], true);
            let record = d.records.get(&1).unwrap().as_ref().clone();
            fixtures::restore_records(&mut restored, &[record]);
            let out = connect(&mut restored, 3, How::ForgejoAdmin { first: None });
            restored_intent(saved_record(&out), None);
            assert_eq!(restored.forges().rows[0].first, None);
        }
    }
}

#[test]
fn a_pasted_app_connects_a_managed_github() {
    let mut d = setup_domain(&[], false);
    github(&mut d, 1);
    let app = core::num::NonZeroU64::new(7).unwrap();
    let out = connect(&mut d, 2, How::GitHubApp { app });
    let record = saved_record(&out);
    assert_eq!(record.mode, Mode::Connected);
    assert_eq!(d.configured(1).unwrap().mode, Mode::Connected);
    assert!(out.contains(&Request::Connect {
        forge: 1,
        request: Token::new(2),
        how: How::GitHubApp { app },
        carried: VALUE
    }));
    assert_eq!(record, ForgeRecord { number: 1, mode: Mode::Connected, ..d.machine.github.clone() });
    let mut d = setup_domain(&[], false);
    github(&mut d, 1);
    let out = connect(&mut d, 2, How::GitHubManifest { back: None });
    assert!(out.contains(&Request::Manifest { forge: 1, request: Token::new(2), back: None }));
    assert_eq!(d.records.get(&1).unwrap().mode, Mode::Managed);
    for request in out {
        if let Request::Save { record: Stored::Forge(_), .. } = request {
            panic!("a manifest does not change the mode");
        }
    }
}

fn apply(store: &mut skein_lib::Map<Key, Stored>, out: &[Request]) {
    for request in out {
        if let Request::Save { record, .. } = request {
            store.insert(stored_key(record), record.clone()).unwrap();
        }
        if let Request::Erase { key } = request {
            store.remove(key);
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one release checks the complete durable erasure, surviving rows and restart boundary"
)]
fn a_released_project_leaves_no_row_or_adoption_answer() {
    let mut d = domain();
    let mut store = skein_lib::Map::with_capacity(32);
    let other = skein_forge::api::Repository { forge: 1, repository: 3 };
    let name = branch();
    let other_name = Name { repository: other.repository, ..name.clone() };
    let mut other_row = adopted();
    other_row.project = 6;
    other_row.provider = other;
    let receipt =
        ReceiptKey { project: 5, repository: REPO, check: Check::Gate(b"gate".as_slice().into()), key: [7; 32] };
    let answer = Stored::Application {
        token: 1,
        until: Wall::from_nanos(160_000_000_000),
        outcome: ApplicationOutcome::Adopted(AdoptedRepository {
            repository: REPO,
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into(),
            role: Role::Owned,
            home: true,
            kinds: adopted().kinds,
            protection: Protection::Absent,
        }),
    };
    for row in [
        Stored::Repository(adopted()),
        Stored::Repository(other_row),
        Stored::BranchHead(BranchHead { name: name.clone(), commit: [3; 32], owned: true }),
        Stored::PullState(PullState {
            name: name.clone(),
            head: [3; 32],
            ci: skein_forge::api::Ci::Passed,
            state: skein_forge::api::State::Open,
        }),
        Stored::Ci(CiState { repository: REPO, head: [3; 32], status: skein_forge::api::Ci::Passed }),
        Stored::Receipt {
            key: receipt.clone(),
            receipt: Receipt {
                passed: true,
                run: ReceiptRun { id: 1, number: 1, url: Box::new([]) },
                candidate: [3; 32],
                ended: env().wall,
            },
        },
        answer.clone(),
        Stored::Names { task: 88, resources: Box::new([other_name]) },
    ] {
        store.insert(stored_key(&row), row.clone()).unwrap();
        outputs(&mut d, Event::Restore { record: row });
    }
    apply(&mut store, &outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }));
    let resource = skein_forge_client::Resource {
        repository: REPO,
        what: skein_forge_client::What::Branch(b"temper/7".as_slice().into()),
    };
    let surviving_resource = skein_forge_client::Resource { repository: other, what: resource.what.clone() };
    apply(
        &mut store,
        &outputs(
            &mut d,
            Event::Client(skein_forge_client::Event::Keep {
                owner: Token::new(99),
                watches: Box::new([
                    skein_forge_client::Watch { resource: resource.clone(), participating: false },
                    skein_forge_client::Watch { resource: surviving_resource.clone(), participating: false },
                ]),
            }),
        ),
    );
    let live = Key::Client(skein_forge_client::Key::Live(resource));
    let client_repository = Key::Client(skein_forge_client::Key::Repository(REPO));
    assert!(store.contains_key(&live) && store.contains_key(&client_repository));
    apply(&mut store, &outputs(&mut d, Event::ReadAfresh));
    apply(&mut store, &outputs(&mut d, Event::SettleOutbox));
    let pending = ready(&mut d);
    let out = outputs(&mut d, Event::ReleaseProject { project: 5 });
    assert_eq!(out.last(), Some(&Request::ProjectReleased { project: 5 }));
    let removed = [
        Key::Repository(REPO),
        Key::BranchHead(name.clone()),
        Key::PullState(name),
        Key::Ci { repository: REPO, head: [3; 32] },
        Key::Receipt(receipt),
        live,
        client_repository,
    ];
    for key in &removed {
        assert!(out.contains(&Request::Erase { key: key.clone() }), "{key:?}");
    }
    apply(&mut store, &out);
    for key in removed {
        assert!(!store.contains_key(&key));
    }
    assert_eq!(store.get(&Key::Application(1)), None);
    assert!(store.contains_key(&Key::Repository(other)));
    assert!(store.contains_key(&Key::Client(skein_forge_client::Key::Live(surviving_resource))));
    assert!(store.contains_key(&Key::Client(skein_forge_client::Key::Repository(other))));
    assert!(d.repository(REPO).is_none());
    let mut late_calls = 0_u32;
    for request in pending {
        if let Request::Call { call, repository, .. } = request
            && repository == REPO
        {
            late_calls += 1;
            let late = fixtures::answer(&mut d, call, Err(skein_forge::api::Error::Missing));
            assert!(late.is_empty(), "a departed watch never saves its late answer");
        }
    }
    assert!(late_calls > 0, "exercise a read in flight at release");
    assert_eq!(
        outputs(&mut d, Event::ReleaseProject { project: 5 }).as_ref(),
        [Request::ProjectReleased { project: 5 }]
    );
    let mut restored = domain();
    for (_, row) in &store {
        outputs(&mut restored, Event::Restore { record: row.clone() });
    }
    outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(restored.repository(REPO).is_none());
    assert!(restored.repository(other).is_some());
    assert_eq!(restored.application_answer(Token::new(1)), None);
}

#[test]
fn a_make_returns_a_carried_value_unused() {
    let mut d = setup_domain(&[], false);
    let out = outputs(
        &mut d,
        Event::Application {
            request: Token::new(1),
            token: Token::new(2),
            retention: Duration::from_secs(60),
            ask: Ask::MakeForge { choice: ForgeChoice::GitHub },
            role: Some(jig_domain_people::Role::Owner),
            carried: Some(VALUE),
        },
    );
    assert_eq!(out.first(), Some(&Request::CarriedUnused { number: VALUE.number }));
    assert_eq!(out.last(), Some(&Request::ApplicationDone { request: Token::new(1), token: Token::new(2) }));
    assert_eq!(d.application_answer(Token::new(2)), Some(Answered::MakeForge(MakeAnswer::Made { forge: 1 })));
}

#[test]
fn every_kept_forge_can_restore_its_first_intent() {
    let mut d = setup_domain(&[], true);
    for number in 1..=u16::try_from(LIMITS.forges).unwrap() {
        let row = ForgeRecord {
            number,
            first: Some(First {
                playground: false,
                org: b"work".as_slice().into(),
                repository: b"hello".as_slice().into(),
                project: SetUpProject::Existing(1),
                state: FirstState::Making,
                label: b"docker".as_slice().into(),
            }),
            ..owned()
        };
        outputs(&mut d, Event::Restore { record: Stored::Forge(row) });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert_eq!(d.forges().rows.len(), usize::try_from(LIMITS.forges).unwrap());
    for row in d.forges().rows {
        assert_eq!(row.first.as_ref().unwrap().state, FirstState::Making);
    }
}
