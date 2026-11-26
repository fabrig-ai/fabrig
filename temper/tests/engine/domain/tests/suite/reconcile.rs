//! Missing required forge links refuse the start before the restore terminal and switch.
use skein_lib::Wall;
use temper_engine_domain::{self as engine, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};
use temper_engine_forge_world as forge_world;

const ABSENT_TASK: u64 = 999;
const ABSENT_ENTRY: u64 = 777;
const ABSENT_REPOSITORY: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 404 };
// Same full-restart horizon as switch.rs.
const STEPS: u32 = 256;

fn restart_with(row: forge::Stored) -> Driver {
    restart_with_rows(Box::new([row]))
}
fn restart_with_rows(rows: Box<[forge::Stored]>) -> Driver {
    let (running, _) = Driver::chat();
    restart_over(running, rows)
}
fn restart_over(mut running: Driver, rows: Box<[forge::Stored]>) -> Driver {
    let mut store: Store = running.take_store();
    store.seed(rows.into_vec().into_iter().map(|row| Record::Forge { row: Box::new(row) }).collect(), Box::new([]));
    store.crash(9761);
    Driver::cold(store, walking::config(91), &walking::limits())
}
fn refuses_before_switch(row: forge::Stored, expected: forge::MissingLink) {
    let key = engine::Key::Forge(forge::stored_key(&row));
    let mut driver = restart_with(row.clone());
    driver.broken("fixture removes a required forge link and refuses the start");
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert!(driver.stopped, "start refused");
    assert_eq!(driver.root.forge_missing_link(), Some(expected.clone()));
    assert_eq!(
        driver.root.start_stop(),
        Some(engine::StartStop::ForgeLink { phase: Some(jig_domain_core::start::Phase::Link), link: expected })
    );
    assert!(driver.switches.is_empty(), "no switch asked");
    assert!(!driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)), "no mark");
    assert_eq!(driver.root.start_phase(), Some(jig_domain_core::start::Phase::Link), "no adoption stage");
    assert!(!driver.root.ready(), "no decisions");
    assert!(driver.delivered.is_empty(), "no run adopted or delivered");
    assert!(driver.immediate.is_empty(), "no external requests");
    while let Some(request) = driver.out.pop() {
        assert!(
            !matches!(
                request,
                engine::Request::Forge { .. } | engine::Request::Connect { .. } | engine::Request::Worker(_)
            ),
            "no external wires before link: {request:?}"
        );
    }
    assert_eq!(driver.store.rows.get(&key), Some(&Record::Forge { row: Box::new(row) }), "row retained");
}
#[test]
fn a_missing_task_refuses_the_start_before_the_switch() {
    refuses_before_switch(
        forge::Stored::Call {
            key: forge::NamedCall { task: ABSENT_TASK, attempt: 1, completion: 1, position: 1 },
            answer: forge::CallPayload::Refused(skein_forge::api::Error::Refused),
        },
        forge::MissingLink {
            kind: forge::RecordKind::Call,
            key: forge::Key::Call(forge::NamedCall { task: ABSENT_TASK, attempt: 1, completion: 1, position: 1 }),
            need: forge::Need::Task { task: ABSENT_TASK, retired: None },
        },
    );
}
#[test]
fn a_missing_repository_refuses_the_start_before_the_switch() {
    refuses_before_switch(
        forge::Stored::Ci(forge::CiState {
            repository: ABSENT_REPOSITORY,
            head: [3; 32],
            status: skein_forge::api::Ci::Passed,
        }),
        forge::MissingLink {
            kind: forge::RecordKind::Ci,
            key: forge::Key::Ci { repository: ABSENT_REPOSITORY, head: [3; 32] },
            need: forge::Need::Repository(ABSENT_REPOSITORY),
        },
    );
}
#[test]
fn a_missing_flight_entry_refuses_the_start_before_the_switch() {
    refuses_before_switch(
        forge::Stored::Flight { task: ABSENT_TASK, entry: ABSENT_ENTRY },
        forge::MissingLink {
            kind: forge::RecordKind::Flight,
            key: forge::Key::Flight { task: ABSENT_TASK, entry: ABSENT_ENTRY },
            need: forge::Need::Entry { task: ABSENT_TASK, entry: ABSENT_ENTRY },
        },
    );
}
#[test]
fn a_missing_configured_forge_refuses_the_start_before_the_switch() {
    refuses_before_switch(
        forge::Stored::Writer(skein_forge_client::Writer { forge: 99, author: 7 }),
        forge::MissingLink {
            kind: forge::RecordKind::Writer,
            key: forge::Key::Writer(99),
            need: forge::Need::Forge(99),
        },
    );
}
pub(super) const UNMADE: u64 = 10030;

pub(super) fn unmade_landing(changes: &[forge::ChangeRow]) -> Vec<forge::Stored> {
    let mut landing = forge_world::landings::landing_row(changes.first().expect("members"));
    landing.landing.task = UNMADE;
    landing.landing.project = 1;
    landing.landing.round = temper_engine_domain_forge_landing::Round::new(b"temper610030/l10030".as_slice().into());
    landing.landing.members = changes
        .iter()
        .enumerate()
        .map(|(index, change)| {
            let mut member = forge_world::landings::landing_row(change).landing.members[0].clone();
            member.branch = if index == 0 {
                b"temper610030/l10030".as_slice().into()
            } else {
                format!("temper610030/l10030-{}", index + 1).into_bytes().into()
            };
            member.gates = Box::new([]);
            member
        })
        .collect();

    let mut rows = Vec::new();
    for member in &landing.landing.members {
        rows.push(forge::Stored::Hold(forge::Hold {
            name: forge::Name {
                forge: member.repository.forge,
                repository: member.repository.repository,
                what: forge::What::Branch(member.branch.split(|byte| *byte == b'/').map(Into::into).collect()),
            },
            task: UNMADE,
            writer: None,
            design_writers: Box::new([]),
            beside: None,
            drift: None,
        }));
    }
    rows.push(forge::Stored::Landing(landing));
    for change in changes {
        rows.push(forge::Stored::Change(Box::new(change.clone())));
    }
    rows
}

fn unmade_rows() -> Box<[forge::Stored]> {
    let mut world = forge_world::World::new(17);
    world.adopt();
    let forge::Stored::Repository(mut repository) =
        world.stored().get(&forge::Key::Repository(forge_world::REPO)).expect("adopted").clone()
    else {
        panic!("repository")
    };
    repository.project = 1;
    repository.provider.repository = 7;
    let changes: Vec<_> = [8975, 9104, 9885]
        .into_iter()
        .enumerate()
        .map(|(index, task)| {
            let mut row = forge_world::changes::change_row();
            row.task = task;
            row.change.task = task;
            row.repository = repository.provider;
            row.pull = Some(task);
            let ready_since = Wall::from_nanos(u64::try_from(index + 1).expect("time"));
            row.change.ready_since = Some(ready_since);
            row.change.state = change::State::Ready { head: [3; 32], ready_since };
            row
        })
        .collect();
    let mut rows = unmade_landing(&changes);
    rows.push(forge::Stored::Repository(repository));
    rows.into_boxed_slice()
}

#[test]
fn an_unmade_landing_ends_at_start_and_its_members_wait_again() {
    let mut driver = restart_with_rows(unmade_rows());
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert!(driver.root.ready(), "start continues: {:?}", driver.root.start_stop());
    assert_unmade_members_and_warnings(&mut driver);
    assert_no_unmade_history(&driver);
    let store = driver.take_store();
    let mut again = Driver::cold(store, walking::config(91), &walking::limits());
    for _ in 0..STEPS {
        again.advance(true);
    }
    assert!(again.root.ready());
    assert!(again.root.pop_fact().is_none(), "a second start repairs and logs nothing");
    assert!(
        !again.transactions.iter().flatten().any(|write| matches!(
            write,
            Write::Erase(engine::Key::Forge(forge::Key::Landing(UNMADE) | forge::Key::Hold(_)))
                | Write::Save(Record::Forge { .. })
        )),
        "no repair writes on the second start"
    );
}

fn assert_unmade_members_and_warnings(driver: &mut Driver) {
    assert_cancelled_landing(&driver.store);
    let mut facts = Vec::new();
    while let Some(fact) = driver.root.pop_fact() {
        facts.push(fact);
    }
    let repository = skein_forge::api::Repository { forge: 1, repository: 7 };
    assert_eq!(facts.len(), 4, "one warning per landing and per hold: {facts:?}");
    assert_eq!(
        facts[0],
        engine::Fact::LandingUnmade {
            task: UNMADE,
            repository,
            branch: b"main".as_slice().into(),
            changes: Box::new([8975, 9104, 9885]),
            readback: false,
            landed: Box::new([]),
            unrecorded: Box::new([]),
        }
    );
    for (index, task) in [8975, 9104, 9885].into_iter().enumerate() {
        let row = driver.store.change(task).expect("member retained");
        let ready_since = Wall::from_nanos(u64::try_from(index + 1).expect("time"));
        assert_eq!(driver.root.forge().landing_of(row.task), None);
        assert_eq!(row.change.ready_since, Some(ready_since));
        assert_eq!(row.change.state, change::State::Ready { head: [3; 32], ready_since });
        let branch: Box<[u8]> = if index == 0 {
            b"temper610030/l10030".as_slice().into()
        } else {
            format!("temper610030/l10030-{}", index + 1).into_bytes().into()
        };
        let name = forge::Name {
            forge: 1,
            repository: 7,
            what: forge::What::Branch(branch.split(|byte| *byte == b'/').map(Into::into).collect()),
        };
        assert!(!driver.store.rows.contains_key(&engine::Key::Forge(forge::Key::Hold(name.clone()))));
        assert!(facts.contains(&engine::Fact::HoldDropped { name, task: UNMADE }));
    }
}

fn assert_no_unmade_history(driver: &Driver) {
    assert!(
        !driver.transactions.iter().flatten().any(|write| matches!(write,
            Write::Save(Record::Forge { row }) if matches!(row.as_ref(),
                forge::Stored::HistoryEvent(_) | forge::Stored::HistoryLanding(_) | forge::Stored::HistoryChange(_)
            )
        )),
        "an unmade landing has no history step"
    );
}

pub(super) fn cancelled_landing(store: &Store) -> bool {
    matches!(store.rows.get(&engine::Key::Forge(forge::Key::Landing(UNMADE))),
        Some(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Landing(row)
            if row.landing.phase == landing::Phase::Cancelled))
}

pub(super) fn assert_cancelled_landing(store: &Store) {
    assert!(cancelled_landing(store), "taskless row is retained cancelled");
}

fn saves_cancelled_landing(write: &Write) -> bool {
    matches!(write, Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Landing(row)
        if row.landing.task == UNMADE && row.landing.phase == landing::Phase::Cancelled))
}

#[test]
fn a_taskless_proving_landing_closes_its_round_in_link() {
    use forge::history::{LandingOutcome, RoundCause, What};
    let mut rows = unmade_rows();
    for row in &mut rows {
        if let forge::Stored::Landing(row) = row {
            row.landing.preparation = 1;
            row.landing.phase = landing::Phase::Proving;
            for member in &mut row.landing.members {
                member.base = Some([2; 32]);
                member.candidate = Some([3; 32]);
                member.round = Some(1);
            }
        }
    }
    let mut driver = restart_with_rows(rows);
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert_unmade_members_and_warnings(&mut driver);
    let link = driver
        .transactions
        .iter()
        .position(|commit| commit.iter().any(saves_cancelled_landing))
        .expect("cancelled row in Link");
    let switch =
        driver.transactions.iter().position(|commit| commit.contains(&Write::Wires)).expect("the start switches");
    assert!(link < switch, "ending commits before the switch");
    for change in [8975, 9104, 9885] {
        let history: Vec<_> = driver.transactions[link]
            .iter()
            .filter_map(|write| match write {
                Write::Save(Record::Forge { row }) => match row.as_ref() {
                    forge::Stored::HistoryEvent(event)
                        if event.change == change && event.landing.is_some_and(|l| l.task == UNMADE) =>
                    {
                        Some(event.what.clone())
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(
            history,
            [
                What::RoundEnded { cause: RoundCause::Cancelled },
                What::LandingEnded { outcome: LandingOutcome::Cancelled }
            ]
        );
    }
}

#[test]
fn an_empty_unmade_landing_ends_without_a_warning() {
    let mut landing = unmade_rows()
        .into_vec()
        .into_iter()
        .find_map(|row| match row {
            forge::Stored::Landing(row) => Some(row),
            _ => None,
        })
        .expect("landing");
    landing.landing.members = Box::new([]);
    landing.previous_asks = Box::new([]);
    let mut driver = restart_with(forge::Stored::Landing(landing));
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert!(driver.root.ready());
    assert_cancelled_landing(&driver.store);
    assert!(driver.root.pop_fact().is_none(), "no repository to name in a warning");
}

#[test]
fn unmade_members_leave_judging_and_keep_landed_records_and_other_links() {
    const FIRST_MEMBER: u64 = 8975;
    const LAST_MEMBER: u64 = 9885;
    let resolve = change::Ask::Resolve { base: [7; 32] };
    let converted = change::Handed { number: 1, landing: UNMADE, round: 0, ask: resolve.clone() };
    for already_landed in [false, true] {
        let mut rows = unmade_rows().into_vec();
        for row in &mut rows {
            match row {
                forge::Stored::Landing(row) => {
                    row.landing.members[1].landed = Some([9; 32]);
                    row.previous_asks = Box::new([Some(resolve.clone()), None, None]);
                }
                forge::Stored::Change(row) => match row.task {
                    FIRST_MEMBER => {
                        row.change.state = change::State::Judging {
                            head: [3; 32],
                            ready_since: Wall::from_nanos(1),
                            asked: Box::new([]),
                            target: change::Judgement {
                                landing: UNMADE,
                                round: 0,
                                pull: 1,
                                branch: b"temper610030/l10030".as_slice().into(),
                                base: b"main".as_slice().into(),
                                prefix: [3; 32],
                            },
                        }
                    }
                    9104 if already_landed => {
                        row.change.state = change::State::Landed { merge: [9; 32], since: Wall::from_nanos(2) }
                    }
                    LAST_MEMBER => row.previous = Some(forge::PreviousLink { landing: Some(ABSENT_TASK), ask: None }),
                    _ => {}
                },
                _ => {}
            }
        }
        let mut driver = restart_with_rows(rows.into_boxed_slice());
        for _ in 0..STEPS {
            driver.advance(true);
        }
        assert!(driver.root.ready());
        let first = driver.store.change(FIRST_MEMBER).expect("judging member");
        assert_eq!(driver.root.forge().landing_of(first.task), None);
        let ready = change::State::Ready { head: [3; 32], ready_since: Wall::from_nanos(1) };
        assert_eq!(first.change.state, ready);
        assert_eq!(first.asks.as_ref(), std::slice::from_ref(&converted));
        let commit = driver
            .transactions
            .iter()
            .find(|commit| commit.iter().any(saves_cancelled_landing))
            .expect("unmade Link commit");
        assert!(commit.iter().any(|write| matches!(write,
            Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::ChangeHot(change)
                if change.task == FIRST_MEMBER && change.change.state == ready && change.asks.as_ref() == std::slice::from_ref(&converted))
        )), "the ready change and its converted queue are saved in the Link commit");
        let landed = driver.store.change(9104).expect("recorded landed member");
        assert_eq!(driver.root.forge().landing_of(landed.task), None);
        assert!(matches!(landed.change.state, change::State::Landed { merge, .. } if merge == [9; 32]));
        if already_landed {
            assert_eq!(landed.change.state, change::State::Landed { merge: [9; 32], since: Wall::from_nanos(2) });
        }
        assert_eq!(
            driver.root.pop_fact(),
            Some(engine::Fact::LinkDisagreed { change: LAST_MEMBER, named: ABSENT_TASK, landing: Some(UNMADE) })
        );
        let other = driver.store.change(LAST_MEMBER).expect("member linked elsewhere");
        assert_eq!(other.previous, None, "Link clears the previous build's stored link");
        assert_eq!(driver.root.forge().landing_of(other.task), None);
    }
}

#[test]
fn an_unmade_landings_hold_with_a_live_writer_still_refuses_the_start() {
    for design in [false, true] {
        let (running, writer) = Driver::chat();
        let mut rows = unmade_rows().into_vec();
        let hold = rows
            .iter_mut()
            .find_map(|row| match row {
                forge::Stored::Hold(hold) => Some(hold),
                _ => None,
            })
            .expect("hold");
        if design {
            hold.design_writers =
                Box::new([forge::DesignWriter { task: writer.task, attempt: writer.attempt, rereading: false }]);
        } else {
            hold.writer = Some(forge::Writer::Run { task: writer.task, attempt: writer.attempt });
        }
        let name = hold.name.clone();
        let mut driver = restart_over(running, rows.into_boxed_slice());
        driver.broken("fixture keeps a hold written under by a task jig holds");
        for _ in 0..STEPS {
            driver.advance(true);
        }
        assert!(driver.stopped, "a live writer still refuses the start");
        let expected = forge::MissingLink {
            kind: forge::RecordKind::Hold,
            key: forge::Key::Hold(name.clone()),
            need: forge::Need::Task { task: UNMADE, retired: Some(1) },
        };
        assert_eq!(
            driver.root.start_stop(),
            Some(engine::StartStop::ForgeLink { phase: Some(jig_domain_core::start::Phase::Link), link: expected })
        );
        assert!(!driver.root.ready());
        assert!(driver.switches.is_empty(), "no switch asked");
        assert!(!driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)));
        assert!(driver.store.rows.contains_key(&engine::Key::Forge(forge::Key::Hold(name.clone()))));
        while let Some(fact) = driver.root.pop_fact() {
            assert!(!matches!(fact, engine::Fact::HoldDropped { name: dropped, .. } if dropped == name));
        }
    }
}

#[test]
fn a_taskless_landing_with_an_outbox_entry_still_refuses_the_start() {
    let mut rows = unmade_rows().into_vec();
    rows.retain(|row| !matches!(row, forge::Stored::Hold(_)));
    let landing = rows
        .iter_mut()
        .find_map(|row| match row {
            forge::Stored::Landing(row) => Some(row),
            _ => None,
        })
        .expect("landing");
    landing.entries = Box::new([forge::LandingEntry {
        entry: ABSENT_ENTRY,
        member: 0,
        gate: 0,
        operation: forge::Operation::Delete,
        run: None,
        preparation: None,
        base: None,
    }]);
    let original = landing.clone();
    rows.push(forge::Stored::Entry {
        entry: skein_forge_client::Entry {
            number: ABSENT_ENTRY,
            task: UNMADE,
            repository: skein_forge::api::Repository { forge: 1, repository: 7 },
            effect: skein_forge_client::Effect {
                write: skein_forge::api::Write::DeleteBranch { branch: b"temper610030/l10030".as_slice().into() },
                condition: skein_forge_client::Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
        held: true,
    });
    let mut driver = restart_with_rows(rows.into_boxed_slice());
    driver.broken("fixture keeps an outbox entry whose task jig does not hold");
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert!(driver.stopped, "an outbox entry still refuses the start");
    assert_eq!(
        driver.root.start_stop(),
        Some(engine::StartStop::ForgeLink {
            phase: Some(jig_domain_core::start::Phase::Link),
            link: forge::MissingLink {
                kind: forge::RecordKind::Entry,
                key: forge::Key::Entry(ABSENT_ENTRY),
                need: forge::Need::EntryOwner { task: UNMADE, entry: ABSENT_ENTRY },
            },
        })
    );
    assert_eq!(
        driver.store.rows.get(&engine::Key::Forge(forge::Key::Landing(UNMADE))),
        Some(&Record::Forge { row: Box::new(forge::Stored::Landing(original)) })
    );
    assert!(driver.root.pop_fact().is_none());
    assert!(!driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)));
}

#[test]
fn a_taskless_hold_on_an_ended_forge_is_not_dropped_or_warned() {
    let rows = unmade_rows();
    let repository = rows.iter().find(|row| matches!(row, forge::Stored::Repository(_))).expect("repository").clone();
    let hold = rows
        .iter()
        .find_map(|row| match row {
            forge::Stored::Hold(hold) => Some(hold.clone()),
            _ => None,
        })
        .expect("hold");
    let ended = forge::ForgeRecord {
        number: 1,
        person: None,
        git_program: None,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Owned,
        state: forge::ForgeState::Ended { secrets_erased: true, box_removed: true },
        place: forge::Place {
            api: forge::Address { host: b"forge.example".as_slice().into(), port: 3000, tls: None },
            git: b"http://forge.example:3000".as_slice().into(),
            web: None,
        },
        read_only: false,
        bot: forge::Bot { login: b"temper".as_slice().into(), name: b"Temper".as_slice().into(), email: Box::new([]) },
        asked: true,
        first: None,
    };
    let mut driver = restart_with_rows(Box::new([
        forge::Stored::Hold(hold.clone()),
        forge::Stored::Forge(ended),
        forge::Stored::ForgeNumber { last: 1 },
        repository,
    ]));
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert!(driver.root.ready(), "ended forge links resolve: {:?}", driver.root.start_stop());
    assert!(driver.store.rows.contains_key(&engine::Key::Forge(forge::Key::Hold(hold.name))));
    assert!(driver.root.pop_fact().is_none(), "no repair warning for an ended forge");
}

#[test]
fn landing_history_outlives_its_task_and_the_start_goes_on() {
    let mut driver = restart_with(forge::Stored::Landed { commit: [9; 32], task: ABSENT_TASK });
    for _ in 0..STEPS {
        driver.advance(true);
    }
    assert!(!driver.stopped, "historical task need not be live or retained");
    assert_eq!(driver.root.start_stop(), None);
    assert!(driver.switches.is_empty(), "no pending generation needs a store switch");
    assert!(driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)), "mark committed");
    assert!(driver.root.start_phase().is_none(), "adoption stage completed");
    assert!(driver.root.ready(), "start continues into decisions");
}
