use crate::{Domain, Event, Fact, Limits};
use jig_domain_core::start::{Action, Phase};
use skein_lib::{Duration, Env, Queue, Time};
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
#[test]
fn catching_up_reports_after_opening_and_ends_without_another_start_step() {
    let (mut domain, mut env) = super::resume_fixture::root();
    assert!(domain.ready());
    domain.forge.restore_begin(env.wall);
    restore(&mut domain, &env, forge::Stored::Repository(repository()));
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut Queue::uncapped(),
    );
    crate::step(
        &mut domain,
        &env,
        Event::Forge(Box::new(forge::Event::Names {
            task: 7,
            resources: Box::new([forge::Name {
                forge: REPO.forge,
                repository: REPO.repository,
                what: forge::What::Branch(Box::new([b"main".as_slice().into()])),
            }]),
        })),
    );
    crate::step(&mut domain, &env, Event::Forge(Box::new(forge::Event::CatchUp)));
    flush(&mut domain, &env);
    assert_eq!(
        domain.forge.restart_waits(forge::RestartStage::ReadAfresh).as_ref(),
        b"1 fresh read (work/hello branch main)"
    );
    assert_eq!(domain.forge.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(30))));
    for (seconds, expected) in [(29, false), (30, true), (45, false), (60, true)] {
        at(&mut env, seconds);
        crate::fire(&mut domain, &env);
        flush(&mut domain, &env);
        let expected = if expected {
            Some(skein_lib::bytes::concat(&[
                b"the forge catching up for ",
                skein_lib::Decimal::of(seconds).as_bytes(),
                b" s: 1 fresh read (work/hello branch main)",
            ]))
        } else {
            None
        };
        assert_eq!(forge_line(&mut domain), expected);
        assert!(domain.ready(), "the report does not hold the opening");
        assert_eq!(domain.start_phase(), None);
    }
    crate::step(&mut domain, &env, Event::Forge(Box::new(forge::Event::Names { task: 7, resources: Box::new([]) })));
    flush(&mut domain, &env);
    assert_eq!(forge_line(&mut domain).as_deref(), Some(b"the forge has caught up".as_slice()));
    at(&mut env, 300);
    crate::fire(&mut domain, &env);
    assert_eq!(forge_line(&mut domain), None);
    assert!(domain.start_report.is_none());
}

fn flush(domain: &mut Domain, env: &Env<Limits>) {
    let mut out = Queue::uncapped();
    for _ in 0_u32..256 {
        crate::release(domain, env, &mut out);
        crate::release(domain, env, &mut out);
        let Some(request) = out.pop() else { return };
        if let crate::Request::Store(crate::StoreRequest::Commit { number, .. }) = request {
            crate::step(domain, env, Event::Store(crate::StoreInput::Committed { number, fill: 0 }));
        } else if let crate::Request::ToChild(released) = request {
            crate::step(domain, env, Event::Released(released));
        }
    }
    panic!("bounded fixture outputs");
}

fn forge_line(domain: &mut Domain) -> Option<Box<[u8]>> {
    let mut line = None;
    while let Some(fact) = domain.pop_fact() {
        if let Fact::StartWaiting { .. } = &fact {
            panic!("catching up is not a start step");
        }
        if let Fact::ForgeTable { words } = fact {
            assert!(line.is_none(), "one report per bound");
            line = Some(words);
        }
    }
    line
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
