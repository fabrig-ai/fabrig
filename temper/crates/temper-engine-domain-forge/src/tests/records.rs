//! Kept forge records (draft playground-reset.md, 3).
use super::*;
use skein_forge::forgejo::pin;

fn record(number: u16, kind: Kind, mode: Mode) -> ForgeRecord {
    ForgeRecord {
        number,
        kind,
        mode,
        bot: Bot {
            login: b"temper".as_slice().into(),
            name: b"Temper".as_slice().into(),
            email: b"temper@localhost".as_slice().into(),
        },
        ..ForgeRecord::github_com()
    }
}

#[test]
fn a_kept_record_projects_its_configured_forge() {
    for kind in [Kind::Forgejo, Kind::GitHub] {
        for mode in [Mode::Connected, Mode::Managed, Mode::Owned, Mode::Fake] {
            for asked in [false, true] {
                let mut row = record(7, kind, mode);
                row.read_only = true;
                if asked {
                    row.setup = Some(b"{}".as_slice().into());
                }
                let expected = Configured {
                    number: 7,
                    kind,
                    mode,
                    admin_connect: kind == Kind::Forgejo && mode == Mode::Managed,
                    bot: if asked { b"temper".as_slice().into() } else { Box::new([]) },
                    owned: if mode == Mode::Owned {
                        Some(Owned {
                            forgejo: pin::VERSION.as_bytes().into(),
                            runner: Some(pin::RUNNER_VERSION.as_bytes().into()),
                            password: b"forge-7/person_password".as_slice().into(),
                        })
                    } else {
                        None
                    },
                    git: row.git.clone(),
                    web: row.web.clone(),
                    read_only: true,
                    asked,
                };
                assert_eq!(row.configured(), expected);
                let mut d = setup_domain(&[], true);
                fixtures::restore_records(&mut d, core::slice::from_ref(&row));
                assert_eq!(d.configured(7), Some(&expected));
                assert_eq!(d.records.get(&7).expect("kept forge").as_ref(), &row);
                outputs(&mut d, Event::Connection { forge: 7, state: Connection::NotSetUp });
                assert_eq!(d.offer(7), if mode == Mode::Owned && asked { Offer::Offered } else { Offer::NotOffered });
                row.git = skein_lib::bytes::zeroed(usize::try_from(PULL_ADDRESS_BYTES).unwrap() + 1);
                assert!(row.configured().git.is_empty());
                let mut long = setup_domain(&[], true);
                fixtures::restore_records(&mut long, &[row]);
                assert!(long.configured(7).unwrap().git.is_empty(), "long git bases restore");
            }
        }
    }
}

#[test]
fn records_alone_project_the_list_and_removing_drops_the_projection() {
    let documents = configured();
    let mut d = setup_domain(&documents, true);
    let mut shared = record(documents[0].number, Kind::Forgejo, Mode::Managed);
    shared.setup = Some(b"{}".as_slice().into());
    let extra = record(9, Kind::GitHub, Mode::Connected);
    fixtures::restore_records(&mut d, &[extra.clone(), shared.clone()]);
    assert_eq!(d.configured(shared.number), Some(&shared.configured()));
    assert_eq!(d.configured(documents[1].number), Some(&fixtures::from_configured(&documents[1]).configured()));
    assert_eq!(d.configured(9), Some(&extra.configured()));
    let listed = d.forges();
    assert_eq!(listed.rows.len(), 3);
    assert_eq!(listed.rows[0].number, documents[0].number);
    assert_eq!(listed.rows[0].mode, Mode::Managed);
    assert_eq!(listed.rows[2].number, 9);
    assert_eq!(listed.rows[0].offer, d.offer(shared.number));
    assert_eq!(d.offer(shared.number), Offer::NotOffered);
    assert!(d.forge_ended(20));
    // A removing record leaves the configured list.
    shared.state = ForgeState::Removing;
    outputs(&mut d, Event::Restore { record: Stored::Forge(shared) });
    assert_eq!(d.configured(documents[0].number), None);
    assert_eq!(d.forges().rows.len(), 2);
}

#[test]
fn removing_and_removed_forges_are_ended_and_their_repositories_stay_readable_by_tag() {
    let mut d = domain();
    assert_eq!(d.next_number, 2);
    let mut removing = record(3, Kind::Forgejo, Mode::Owned);
    removing.state = ForgeState::Removing;
    for number in [2, 3] {
        let mut repository = adopted();
        repository.provider.forge = number;
        outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    }
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumbers { next: 4 } });
    fixtures::restore_records(&mut d, &[removing]);
    assert_eq!(d.next_number, 4);
    for number in [2, 3] {
        assert!(d.forge_ended(number));
        let provider = skein_forge::api::Repository { forge: number, ..REPO };
        let repository = d.repository_tag(5, tag(provider.forge, provider.repository)).unwrap();
        assert_eq!(repository.provider, provider);
        assert!(crate::reconcile::references_resolve(&d, &LIMITS, &Stored::Repository(repository.clone()), &[]));
        let row = Stored::Ci(CiState { repository: provider, head: [3; 32], status: skein_forge::api::Ci::Passed });
        assert!(crate::reconcile::references_resolve(&d, &LIMITS, &row, &[]));
    }
    assert!(d.readable_repositories(5).is_empty());
    assert!(d.writable_repositories(5).is_empty());
    assert!(d.context_repositories(5, 2).unwrap().is_empty());
    assert!(d.home(5).is_none());
    assert_eq!(d.forges().rows.len(), 1);
    assert_eq!(d.forges().rows[0].number, 1);
    assert!(d.forge_ended(0));
    assert!(!d.forge_ended(1), "a kept record names this forge");
    assert!(d.forge_ended(4));
    assert!(d.forge_ended(5));
    let documents = configured();
    let mut named = setup_domain(&documents, true);
    outputs(&mut named, Event::Restore { record: Stored::ForgeNumbers { next: 10 } });
    for document in documents {
        assert!(!named.forge_ended(document.number));
    }
}

#[test]
fn restoration_registers_every_record_in_number_order_before_restored() {
    let mut d = domain();
    let mut removing = record(2, Kind::GitHub, Mode::Managed);
    removing.state = ForgeState::Removing;
    let rows = [
        record(4, Kind::Forgejo, Mode::Owned),
        removing,
        record(3, Kind::Forgejo, Mode::Fake),
        record(1, Kind::GitHub, Mode::Connected),
    ];
    for row in &rows {
        assert!(outputs(&mut d, Event::Restore { record: Stored::Forge(row.clone()) }).is_empty());
    }
    let out = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let mut numbers = skein_lib::List::with_capacity(LIMITS.forges);
    let mut finished = false;
    for request in out {
        if let Request::Register(row) = &request {
            assert!(!finished);
            numbers.push(row.number).unwrap();
            assert!(rows.contains(row));
        }
        if let Request::RestartDone { stage: RestartStage::Restored } = request {
            finished = true;
            assert_eq!(numbers.as_slice(), [1, 2, 3, 4]);
        }
    }
    assert!(finished);
}

#[test]
fn more_forge_rows_than_the_limit_do_not_fit() {
    let mut d = domain();
    for number in 1..=u16::try_from(LIMITS.forges).unwrap() {
        outputs(&mut d, Event::Restore { record: Stored::Forge(record(number, Kind::Forgejo, Mode::Owned)) });
    }
    // The same key replaces one row without consuming more room.
    outputs(&mut d, Event::Restore { record: Stored::Forge(record(1, Kind::GitHub, Mode::Connected)) });
    assert!(d.restore_fit_failure().is_none());
    outputs(&mut d, Event::Restore { record: Stored::Forge(record(9, Kind::Forgejo, Mode::Owned)) });
    let fit = d.restore_fit_failure().unwrap();
    assert_eq!(fit.family, Some(RecordKind::Forge));
    assert_eq!(fit.set, RestoreSet::Rows(RecordKind::Forge));
    assert_eq!((fit.held, fit.additional, fit.limit), (4, 1, 4));
    assert!(outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }).is_empty());
    assert!(!valid(&Limits { forges: 0, ..LIMITS }));
}

#[test]
fn every_records_identity_is_its_number() {
    for state in [ForgeState::Kept, ForgeState::Removing] {
        let mut d = domain();
        let mut numbered = record(7, Kind::GitHub, Mode::Connected);
        numbered.state = state;
        let mut named = numbered.clone();
        named.number = 1;
        fixtures::restore_records(&mut d, &[numbered, named]);
        assert_eq!(d.git_identity(7), Some(7));
        assert_eq!(d.git_identity(1), Some(1));
        assert_eq!(d.git_identity(8), None);
        let mut repository = adopted();
        repository.provider.forge = 7;
        let provider = repository.provider;
        outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
        let item = d
            .workspace_item(items::Seed {
                repository: provider,
                task: 55,
                parent: None,
                run_branch: Box::new([]),
                saved_branch: None,
            })
            .unwrap();
        assert_eq!(item.credential, Some(7));
        assert!(crate::reconcile::references_resolve(
            &d,
            &LIMITS,
            &Stored::Writer(skein_forge_client::Writer { forge: 7, author: 2 }),
            &[]
        ));
    }
}

#[test]
fn release_project_is_answered_at_once() {
    let mut d = domain();
    for project in [5, 99, 5] {
        assert_eq!(outputs(&mut d, Event::ReleaseProject { project }).as_ref(), [Request::ProjectReleased { project }]);
    }
}
