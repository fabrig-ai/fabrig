use crate::{Domain, Event, Fact, Limits};
use jig_domain_core::start::{Action, Phase};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use temper_engine_domain_forge as forge;

const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };

fn begin(domain: &mut Domain, env: &Env<Limits>, phase: Phase) {
    domain.work.push(crate::boundary::Work::Start(Action::Step(phase)));
    crate::step(domain, env, Event::Resume);
}
fn at(env: &mut Env<Limits>, seconds: u64) {
    env.now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
}
fn waiting(domain: &mut Domain) -> Option<(Phase, Duration, Box<[u8]>)> {
    while let Some(fact) = domain.pop_fact() {
        if let Fact::StartWaiting { step, age, waits } = fact {
            return Some((step, age, waits));
        }
    }
    None
}
fn restore(domain: &mut Domain, env: &Env<Limits>, record: forge::Stored) {
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record },
        &mut Queue::uncapped(),
    );
}
fn repository() -> forge::Repository {
    forge::Repository {
        contexts: Box::new([]),
        landing: b"main".as_slice().into(),
        workflows: Box::default(),
        project: 1,
        home: true,
        provider: REPO,
        host: b"forge.example".as_slice().into(),
        owner: b"work".as_slice().into(),
        name: b"hello".as_slice().into(),
        prefix: b"temper/".as_slice().into(),
        role: forge::Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: forge::Kinds {
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
        protection: forge::Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: b"main".as_slice().into(),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}
fn branch(parts: &[&[u8]]) -> forge::Name {
    let mut names = skein_lib::List::with_capacity(u32::try_from(parts.len()).unwrap());
    for part in parts {
        names.push(Box::from(*part)).unwrap();
    }
    forge::Name { forge: 1, repository: 2, what: forge::What::Branch(names.into_boxed()) }
}
fn uncertain(number: u64, task: u64) -> skein_forge_client::Entry {
    skein_forge_client::Entry {
        number,
        task,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: skein_forge::api::Write::CreateBranch { branch: b"saved".as_slice().into(), commit: [3; 32] },
            condition: skein_forge_client::Condition::None,
        },
        start: Some(skein_forge_client::Position { at: Time::ZERO, comment: 0, review: 0, review_page: 1 }),
        attempt: Some(skein_forge_client::Attempt {
            sent: Time::ZERO,
            deadline: Time::ZERO.saturating_add(Duration::from_secs(600)),
            wall: Wall::EPOCH,
            expires: Wall::from_nanos(600_000_000_000),
        }),
        failures: 0,
        queued: false,
    }
}

#[test]
fn start_reports_name_fresh_reads_and_goal_branches_then_reset_for_settling() {
    let phase = Phase::ReadAfresh { connector: 0 };
    let (mut domain, mut env) = super::resume::at_phase(false, phase);
    restore(&mut domain, &env, forge::Stored::Repository(repository()));
    for (project, task) in [(1_u64, 40), (2, 10), (3, 30), (4, 20)] {
        let project = skein_lib::Decimal::of(project);
        let goal = skein_lib::bytes::concat(&[b"g", skein_lib::Decimal::of(task).as_bytes()]);
        restore(
            &mut domain,
            &env,
            forge::Stored::Hold(forge::Hold {
                name: branch(&[b"temper", project.as_bytes(), &goal]),
                task,
                writer: None,
                beside: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        );
    }
    restore(
        &mut domain,
        &env,
        forge::Stored::Names { task: 40, resources: Box::new([branch(&[b"main"]), branch(&[b"topic"])]) },
    );
    for (number, task) in [(9, 20), (2, 40), (7, 10), (5, 30)] {
        restore(&mut domain, &env, forge::Stored::Entry { entry: uncertain(number, task), held: false });
    }
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut Queue::uncapped(),
    );
    begin(&mut domain, &env, phase);
    let waits =
        b"2 fresh reads (work/hello branch main, work/hello branch topic), 4 goal branches (task 40, task 10, task 30)";
    for (seconds, expected) in [(29, false), (30, true), (45, false), (60, true)] {
        at(&mut env, seconds);
        crate::fire(&mut domain, &env);
        let fact = waiting(&mut domain);
        if expected {
            assert_eq!(fact, Some((phase, Duration::from_secs(seconds), waits.as_slice().into())));
        } else {
            assert_eq!(fact, None);
        }
        assert_eq!(waiting(&mut domain), None, "one report per bound");
    }
    // The fixture supplies the connector's terminal, as it supplies prior steps' terminals.
    at(&mut env, 61);
    crate::restart::connector_start_done(&mut domain, jig_domain_core::effects::connector::RestartStage::ReadAfresh);
    crate::step(&mut domain, &env, Event::Resume);
    assert_eq!(domain.start_phase(), Some(Phase::SettleOutbox { connector: 0 }));
    at(&mut env, 90);
    crate::fire(&mut domain, &env);
    assert_eq!(waiting(&mut domain), None, "the next step begins at 61, not zero");
    at(&mut env, 91);
    crate::fire(&mut domain, &env);
    assert_eq!(
        waiting(&mut domain),
        Some((
            Phase::SettleOutbox { connector: 0 },
            Duration::from_secs(30),
            b"4 uncertain writes (entry 2 of task 40, entry 5 of task 30, entry 7 of task 10)".as_slice().into()
        ))
    );
    crate::restart::connector_start_done(&mut domain, jig_domain_core::effects::connector::RestartStage::Settled);
    crate::step(&mut domain, &env, Event::Resume);
    assert_eq!(domain.start_phase(), None);
    at(&mut env, 300);
    crate::fire(&mut domain, &env);
    assert_eq!(waiting(&mut domain), None);
    assert!(domain.start_report.is_none());
}

#[test]
fn an_unanswered_load_wakes_for_its_report_without_any_input() {
    let (mut domain, mut env) = super::resume::at_phase(false, Phase::LoadCore);
    begin(&mut domain, &env, Phase::LoadCore);
    assert_eq!(domain.next_deadline(&env), Some(Time::ZERO.saturating_add(Duration::from_secs(30))));
    // Store credit is cumulative; four unanswered commits name the oldest three.
    domain.sent_commit = 9;
    domain.durable_commit = 5;
    at(&mut env, 29);
    crate::fire(&mut domain, &env);
    assert_eq!(waiting(&mut domain), None);
    at(&mut env, 30);
    crate::fire(&mut domain, &env);
    assert_eq!(
        waiting(&mut domain),
        Some((
            Phase::LoadCore,
            Duration::from_secs(30),
            b"1 store page (the core's records), 4 commits in flight (commit 6, commit 7, commit 8)".as_slice().into()
        ))
    );
    assert_eq!(domain.next_deadline(&env), Some(Time::ZERO.saturating_add(Duration::from_secs(60))));
}
