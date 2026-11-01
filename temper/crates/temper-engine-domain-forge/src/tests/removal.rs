//! Forge removal decisions (draft 4; contracts D3, D4).
use crate::removal::{remove, removing};
use crate::*;
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};

#[path = "fixtures.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the removal module reuses the shared builders without editing the shared test module"
)]
#[expect(dead_code, reason = "the shared change fixtures also provide unrelated fresh-read paths")]
mod fixtures;

const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };
const CLIENT: skein_forge_client::Limits = skein_forge_client::Limits {
    pending: 4,
    calls: 2,
    rate: 100,
    reserve: 0,
    window: Duration::from_secs(10),
    op_bytes: 512,
    answer_bytes: 4096,
    rows: 8,
    inbox: 16,
    read_attempts: 3,
    backoff: Duration::from_secs(1),
    backoff_max: Duration::from_secs(4),
    entries: 8,
    write_attempts: 3,
    lifetime: Duration::from_secs(10),
    clock_margin: Duration::from_secs(1),
    resources: 8,
    repositories: 2,
    poll: Duration::from_secs(5),
    poll_max: Duration::from_secs(30),
    hinted: Duration::from_secs(1),
    slow: Duration::from_secs(40),
    settle: Duration::from_secs(30),
    facts: 16,
};
const LIMITS: Limits = Limits {
    forges: 4,
    repositories: 2,
    tasks: 8,
    holds: 8,
    subscriptions: 8,
    entries: 8,
    named_calls: 16,
    changes: 8,
    issues: 8,
    resources_per_task: 4,
    paths_per_subscription: 4,
    // forge/host/owner/repository/branch/temper/root/marker-task.
    authority_segments: 8,
    name_bytes: 128,
    output: 80,
    facts: 16,
    adoptions: 2,
    applications: 4,
    bindings: 4,
    collaborators: 8,
    landings: 8,
    landing_members: 2,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    key_entries: 16,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    judge_projects: 2,
    judge_criteria: 8,
    brief_sections: 32,
    brief_bytes: 4096,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        plan_items: 8,
        milestones: 8,
        title_bytes: 128,
        body_bytes: 512,
        comment_bytes: 256,
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        repairs: 3,
        resolutions: 3,
        updates: 3,
        stall: Duration::from_secs(30),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
    client: CLIENT,
};

fn env() -> Env<Limits> {
    Env { now: Time::from_nanos(100_000_000_000), wall: Wall::from_nanos(100_000_000_000), limits: LIMITS }
}

fn domain() -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config {
            namespace: Box::from(&b"test"[..]),
            writers: Box::new([skein_forge_client::Writer { forge: 1, author: 7 }]),
        },
        &[Forge::connected(1, Kind::Forgejo, &[])],
        false,
    )
    .expect("valid connector")
}
fn outputs(d: &mut Domain, event: Event) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    step(d, &env(), event, &mut out);
    let mut values = List::with_capacity(out.len());
    for _ in 0..out.len() {
        values.push(out.pop().expect("counted output")).expect("sized from queue");
    }
    values.into_boxed()
}
fn ready(d: &mut Domain) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    resume(d, &env(), &mut out);
    let mut values = List::with_capacity(out.len());
    for _ in 0..out.len() {
        values.push(out.pop().expect("counted output")).expect("sized from queue");
    }
    values.into_boxed()
}
fn adopted() -> Repository {
    Repository {
        project: 5,
        home: true,
        provider: REPO,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}

fn record() -> Forge {
    Forge {
        number: REPO.forge,
        kind: Kind::Forgejo,
        api: record::Address::Socket(core::net::SocketAddr::from(([127, 0, 0, 1], 3000))),
        transport: record::Transport::Plaintext,
        git: b"https://forge.example".as_slice().into(),
        git_program: None,
        web: None,
        read_only: false,
        identity: record::Identity { number: 1, login: Box::new([]), name: Box::new([]), email: Box::new([]) },
        files: None,
        setup: None,
        first: None,
        playground: false,
        removing: false,
    }
}

fn new(row: Forge, under_box: bool) -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config { namespace: b"removal".as_slice().into(), writers: Box::new([]) },
        &[row],
        under_box,
    )
    .unwrap()
}

fn configured(row: Forge, under_box: bool) -> Domain {
    let mut d = new(row, under_box);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

fn changes(d: &mut Domain) {
    outputs(d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(d, Event::Restore { record: Stored::Change(fixtures::opening_row()) });
}

fn removed(d: &mut Domain) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    assert_eq!(remove(d, &env(), REPO.forge, &mut out), Ok(()));
    let mut requests = List::with_capacity(out.len());
    for _ in 0..out.len() {
        requests.push(out.pop().unwrap()).unwrap();
    }
    requests.into_boxed()
}

fn cancel(d: &Domain) -> Kept {
    for (key, act) in &d.kept {
        if let KeptAct::Cancel { task: fixtures::CHANGE, .. } = act {
            return Kept { key: *key, act: *act };
        }
    }
    panic!("kept cancel")
}

#[test]
fn removing_a_forge_ends_it_at_once() {
    let mut d = configured(record(), false);
    changes(&mut d);
    let out = removed(&mut d);
    assert!(removing(&d, REPO.forge));
    assert!(d.forge_ended(REPO.forge));
    assert!(d.forges().rows.is_empty());
    assert!(out.contains(&Request::ForgeRemoved { forge: REPO.forge }));
    assert!(out.contains(&Request::Save {
        record: Stored::Forge(Forge { removing: true, ..record() }),
        release: None,
        read_afresh: None,
    }));
    assert!(
        outputs(
            &mut d,
            Event::Hint {
                hint: skein_forge::api::Hint { repository: REPO, change: skein_forge::api::Change::Item(1), key: None }
            }
        )
        .is_empty()
    );
    let mut out = Queue::uncapped();
    assert_eq!(remove(&mut d, &env(), REPO.forge, &mut out), Err(RemoveRefusal::NoSuchForge));
    assert!(out.is_empty());
}

#[test]
fn its_writer_flights_and_entries_are_erased_in_the_decision() {
    let (mut d, call) = fixtures::opening();
    outputs(&mut d, Event::Restore { record: Stored::Forge(record()) });
    let other = skein_forge::api::Repository { forge: 2, repository: 2 };
    let mut repository = adopted();
    repository.provider = other;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    assert!(writer(&mut d, &env(), skein_forge_client::Writer { forge: 2, author: 9 }, &mut Queue::uncapped()));
    let mut entry = d.entries.get(&fixtures::OPEN).unwrap().entry.clone();
    entry.number = 12;
    entry.task = 52;
    entry.repository = other;
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: true } });
    outputs(&mut d, Event::Restore { record: Stored::Flight { task: 52, entry: 12 } });
    let out = removed(&mut d);
    for key in [
        Key::Writer(REPO.forge),
        Key::Flight { task: fixtures::CHANGE, entry: fixtures::OPEN },
        Key::Entry(fixtures::OPEN),
    ] {
        let mut count = 0_u32;
        for request in &out {
            if *request == (Request::Erase { key: key.clone() }) {
                count += 1;
            }
        }
        assert_eq!(count, 1);
    }
    for request in &out {
        if let Request::Erase { key: Key::Client(skein_forge_client::Key::Writer(_)) } = request {
            panic!("writer erasures use the connector's family");
        }
    }
    assert!(!d.writers.contains_key(&REPO.forge));
    assert!(!d.entries.contains_key(&fixtures::OPEN));
    assert!(!d.flights.contains_key(&(fixtures::CHANGE, fixtures::OPEN)));
    assert!(d.writers.contains_key(&2));
    assert!(d.entries.contains_key(&12));
    assert!(d.flights.contains_key(&(52, 12)));
    assert!(fixtures::answer(&mut d, call, Ok(skein_forge::api::Answer::Created(1))).is_empty());
}

#[test]
fn each_live_change_on_it_is_kept_as_a_cancel() {
    let mut d = configured(record(), false);
    changes(&mut d);
    let mut second = fixtures::opening_row();
    second.task = 52;
    outputs(&mut d, Event::Restore { record: Stored::Change(second) });
    let mut terminal = fixtures::opening_row();
    terminal.task = 53;
    terminal.change.state = temper_engine_domain_forge_change::State::Ended;
    outputs(&mut d, Event::Restore { record: Stored::Change(terminal) });
    let mut other = fixtures::opening_row();
    other.task = 54;
    other.repository.forge = 2;
    outputs(&mut d, Event::Restore { record: Stored::Change(other) });
    let out = removed(&mut d);
    assert_eq!(d.kept.len(), 2);
    for task in [fixtures::CHANGE, 52] {
        let row = Kept { key: kept::key(b"cancel", &[5, task]), act: KeptAct::Cancel { project: 5, task } };
        assert!(out.contains(&Request::Save { record: Stored::Kept(row), release: None, read_afresh: None }));
        assert!(out.contains(&Request::SendKept(row)));
    }
}

fn setup(mode: record::SetupMode) -> record::Setup {
    record::Setup {
        directory: b"forge".as_slice().into(),
        bot: record::Account { login: b"temper".as_slice().into(), email: Box::new([]) },
        orgs: Box::new([]),
        mode,
        asked: true,
    }
}

fn fake() -> record::SetupMode {
    record::SetupMode::Fake(record::FakeEntry {
        listen: core::net::SocketAddr::from(([127, 0, 0, 1], 3001)),
        git: b"fake".as_slice().into(),
        owner: b"playground".as_slice().into(),
        repositories: Box::new([]),
    })
}

#[test]
fn an_owned_forge_under_the_box_keeps_its_box_remove() {
    let owned = record::SetupMode::Owned(record::OwnedEntry {
        public: None,
        fetch: record::Fetch { forgejo: true, runner: false, mirror: None },
        forgejo: b"forgejo".as_slice().into(),
        runner: None,
        listen: core::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
        person: record::Account { login: b"person".as_slice().into(), email: Box::new([]) },
    });
    for (mode, under_box, kept) in [
        (owned, true, true),
        (fake(), true, true),
        (fake(), false, false),
        (record::SetupMode::Managed { admin_token: None }, true, false),
    ] {
        let mut d = configured(Forge { setup: Some(setup(mode)), ..record() }, under_box);
        let out = removed(&mut d);
        let act =
            Kept { key: kept::key(b"box", &[u64::from(REPO.forge)]), act: KeptAct::BoxRemove { forge: REPO.forge } };
        assert_eq!(out.contains(&Request::SendKept(act)), kept);
        assert_eq!(d.forge_record(REPO.forge).is_some(), kept);
    }
}

#[test]
fn the_row_is_erased_once_its_kept_acts_are_answered() {
    let mut d = configured(Forge { setup: Some(setup(fake())), ..record() }, true);
    changes(&mut d);
    removed(&mut d);
    let row = cancel(&d);
    assert!(outputs(&mut d, Event::KeptAnswered { key: row.key, keep: true }).is_empty());
    assert!(outputs(&mut d, Event::BoxRemoved { forge: REPO.forge, removed: false }).is_empty());
    assert_eq!(
        outputs(&mut d, Event::KeptAnswered { key: row.key, keep: false }).as_ref(),
        &[Request::Erase { key: Key::Kept(row.key) }]
    );
    assert!(removing(&d, REPO.forge));
    let out = outputs(&mut d, Event::BoxRemoved { forge: REPO.forge, removed: true });
    assert!(out.contains(&Request::Erase { key: Key::Forge(REPO.forge) }));
    assert!(d.forge_record(REPO.forge).is_none());
    assert!(d.configured(REPO.forge).is_none());
    assert!(d.forge_ended(REPO.forge));
    assert!(outputs(&mut d, Event::BoxRemoved { forge: REPO.forge, removed: true }).is_empty());
}

#[test]
fn a_removal_with_nothing_to_keep_erases_its_row_at_once() {
    let mut d = configured(record(), false);
    let out = removed(&mut d);
    assert!(out.contains(&Request::Erase { key: Key::Forge(REPO.forge) }));
    assert!(d.forge_record(REPO.forge).is_none());
    assert!(d.forges().rows.is_empty());
}

#[test]
fn the_resets_remove_forge_row_is_answered_by_the_finish() {
    let mut d = configured(record(), false);
    changes(&mut d);
    let reset = Kept { key: [9; 16], act: KeptAct::RemoveForge { forge: REPO.forge } };
    outputs(&mut d, Event::Restore { record: Stored::Kept(reset) });
    removed(&mut d);
    assert!(d.kept.contains_key(&reset.key));
    let row = cancel(&d);
    let out = outputs(&mut d, Event::KeptAnswered { key: row.key, keep: false });
    assert!(out.contains(&Request::Erase { key: Key::Forge(REPO.forge) }));
    assert!(out.contains(&Request::Erase { key: Key::Kept(reset.key) }));
    assert!(d.kept.is_empty());
}

#[test]
fn a_restart_resumes_a_removal() {
    let mut d = configured(record(), false);
    changes(&mut d);
    removed(&mut d);
    let kept = cancel(&d);
    let row = d.forge_record(REPO.forge).unwrap().clone();
    let mut restarted = new(record(), false);
    for record in [
        Stored::Forges { next: 2 },
        Stored::Forge(row),
        Stored::Repository(adopted()),
        Stored::Change(fixtures::opening_row()),
        Stored::Kept(kept),
    ] {
        outputs(&mut restarted, Event::Restore { record });
    }
    let out = outputs(&mut restarted, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(restarted.forge_ended(REPO.forge));
    assert!(restarted.forges().rows.is_empty());
    for request in &out {
        if let Request::ForgeKept { .. } = request {
            panic!("a removing forge is not told kept");
        }
    }
    assert_eq!(outputs(&mut restarted, Event::SendKept).as_ref(), &[Request::SendKept(kept)]);
    let out = outputs(&mut restarted, Event::KeptAnswered { key: kept.key, keep: false });
    assert!(out.contains(&Request::Erase { key: Key::Forge(REPO.forge) }));
    assert!(restarted.forge_record(REPO.forge).is_none());
}

fn ask(d: &mut Domain, forge: u16, role: Option<jig_domain_people::Role>) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(90),
            token: Token::new(91),
            retention: Duration::from_secs(60),
            ask: Ask::RemoveForge { forge },
            role,
            carried: None,
        },
    )
}

#[test]
fn a_non_owner_and_an_unknown_forge_are_refused() {
    for (role, forge, why) in [
        (None, REPO.forge, RemoveRefusal::Role),
        (Some(jig_domain_people::Role::Owner), 99, RemoveRefusal::NoSuchForge),
    ] {
        let mut d = configured(record(), false);
        assert_eq!(
            ask(&mut d, forge, role).as_ref(),
            &[Request::ApplicationDone { request: Token::new(90), token: Token::new(91) }]
        );
        assert_eq!(d.application_answer(Token::new(91)), Some(Answered::RemoveForge(RemoveAnswer::Refused(why))));
        assert!(!removing(&d, REPO.forge));
    }
    let mut d = configured(record(), false);
    ask(&mut d, REPO.forge, Some(jig_domain_people::Role::Owner));
    assert_eq!(d.application_answer(Token::new(91)), Some(Answered::RemoveForge(RemoveAnswer::Removing)));
}

#[test]
fn the_running_playgrounds_forge_is_not_offered() {
    let mut d = configured(Forge { playground: true, ..record() }, false);
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Playground(Playground {
                state: PlaygroundState::Running,
                forge: Some(REPO.forge),
                project: Some(5),
                account: Some(3),
            }),
        },
    );
    ask(&mut d, REPO.forge, Some(jig_domain_people::Role::Owner));
    assert_eq!(
        d.application_answer(Token::new(91)),
        Some(Answered::RemoveForge(RemoveAnswer::Refused(RemoveRefusal::NotOffered)))
    );
    assert!(!removing(&d, REPO.forge));
    assert!(d.forge_record(REPO.forge).is_some());
    removed(&mut d);
    assert!(d.forge_record(REPO.forge).is_none(), "reset calls the decision directly");
}

#[test]
fn its_repository_rows_stay_readable() {
    let mut d = configured(record(), false);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let out = removed(&mut d);
    assert_eq!(d.repository(REPO), Some(&adopted()));
    assert!(d.readable_repositories(5).is_empty());
    assert!(d.forge_ended(REPO.forge));
    for request in &out {
        if let Request::Erase { key: Key::Repository(_) } = request {
            panic!("historical repositories remain readable");
        }
    }
}

#[test]
fn a_start_finishes_a_removal_with_no_acts_left() {
    let mut d = new(record(), false);
    let reset = Kept { key: [8; 16], act: KeptAct::RemoveForge { forge: REPO.forge } };
    for record in [Stored::Forges { next: 2 }, Stored::Forge(Forge { removing: true, ..record() }), Stored::Kept(reset)]
    {
        outputs(&mut d, Event::Restore { record });
    }
    let out = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(out.contains(&Request::Erase { key: Key::Forge(REPO.forge) }));
    assert!(out.contains(&Request::Erase { key: Key::Kept(reset.key) }));
    for request in &out {
        if let Request::ForgeKept { .. } = request {
            panic!("a removing forge is not told kept");
        }
    }
    assert!(d.forge_record(REPO.forge).is_none());
}

#[test]
fn a_removed_forges_repositories_stay_ended_after_restart() {
    let mut d = new(record(), false);
    for record in [Stored::Forges { next: 2 }, Stored::Repository(adopted())] {
        outputs(&mut d, Event::Restore { record });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(d.forge_ended(REPO.forge));
    assert_eq!(d.repository(REPO), Some(&adopted()));
    assert!(d.readable_repositories(5).is_empty());
    assert!(d.forges().rows.is_empty());
}

#[test]
fn a_removed_forges_waiting_read_is_retired_without_a_demo() {
    let mut d = new(record(), false);
    let name = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"7".as_slice().into()])),
    };
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: name.clone(),
                task: 7,
                writer: Some(Writer::Lost { task: 9, attempt: 1 }),
                drift: None,
            }),
        },
    );
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    fixtures::answer_next(&mut d, Err(skein_forge::api::Error::NotConnected));
    assert!(!d.is_ready());
    removed(&mut d);
    assert!(
        outputs(
            &mut d,
            Event::Connection {
                forge: REPO.forge,
                state: Connection::Connected { id: 7, login: b"writer".as_slice().into() }
            }
        )
        .is_empty()
    );
    assert!(!d.is_ready());
    assert!(ready(&mut d).is_empty());
    assert_eq!(d.hold(&name).unwrap().writer, Some(Writer::Lost { task: 9, attempt: 1 }));
}
