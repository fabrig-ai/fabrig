//! A full forge outbox defers projections and ends the root's step
//! (`temper/docs/design/domain/forge/effects.md`, 1 and `temper/docs/design/domain/forge/issues.md`).
use skein_lib::{Env, Queue, Wall};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_issues as issues;

fn adopted(provider: skein_forge::api::Repository) -> forge::Repository {
    let mut repository = super::owned_repository(provider.forge);
    repository.project = 1;
    repository.home = true;
    repository.provider = provider;
    repository.prefix = Box::new([]);
    repository.ci = false;
    repository.settings.squash = false;
    repository
}

fn full_outbox() -> (crate::Domain, Env<crate::Limits>, u64, u64) {
    let (mut domain, env, goal) = super::projection_settled::live_goal(None);
    let repository = skein_forge::api::Repository { forge: 1, repository: 1 };
    domain.forge.restore_begin(env.wall);
    let mut out = Queue::uncapped();
    for record in [
        forge::Stored::Repository(adopted(repository)),
        forge::Stored::Issue(forge::IssueRow {
            goal,
            repository,
            state: issues::Projected::default(),
            number: None,
            pending: None,
            before: None,
            failed: false,
            desired: Some(issues::GoalView {
                phase: issues::Phase::Waiting,
                goal,
                repository: u64::from(repository.repository),
                title: "Work".into(),
                goal_text: "Complete the work".into(),
                plan: Box::new([]),
                milestones: Box::new([]),
                finished: None,
            }),
        }),
    ] {
        forge::step(&mut domain.forge, &env.with(env.limits.forge), forge::Event::Restore { record }, &mut out);
    }
    let mut first = None;
    for _ in 0..env.limits.forge.entries {
        let number = crate::fresh(&mut domain.engine.core_mut().counters, crate::Family::ConnectorRow).unwrap();
        if first.is_none() {
            first = Some(number);
        }
        let entry = skein_forge_client::Entry {
            number,
            task: goal,
            repository,
            effect: skein_forge_client::Effect {
                write: skein_forge::api::Write::Status {
                    commit: [3; 32],
                    context: b"test".as_slice().into(),
                    check: skein_forge::api::Check::Passed,
                    description: Box::new([]),
                    link: Box::new([]),
                },
                condition: skein_forge_client::Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        };
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Restore { record: forge::Stored::Entry { entry, held: false } },
            &mut out,
        );
    }
    for event in [forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, forge::Event::CatchUp]
    {
        forge::step(&mut domain.forge, &env.with(env.limits.forge), event, &mut out);
    }
    assert!(!domain.forge.effect_room(&env.limits.forge));
    assert!(!domain.forge.full_outbox_met(), "restoration leaves the step unmarked");
    assert!(domain.work.is_empty(), "fixture has no unrelated root work");
    let entry = crate::fresh(&mut domain.engine.core_mut().counters, crate::Family::ConnectorRow).unwrap();
    domain.work.push(crate::Work::Forge(forge::Event::ProjectDesired { entry, goal }));
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_into(&mut domain, &env, &mut decision);
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        true,
    )
    .expect("fixture closes its projection decision");
    (domain, env, goal, first.expect("fixture fills the outbox"))
}

fn after_interval(env: &Env<crate::Limits>) -> Wall {
    Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.forge.issue_policy.interval.as_nanos()))
}

#[test]
fn a_projection_ending_without_an_entry_returns_its_reserved_room() {
    use jig_domain_core::effects::connector::{Ask, EffectRefusal};
    for translate in [true, false] {
        let (mut domain, mut env, goal, first) = full_outbox();
        let mut out = Queue::uncapped();
        forge::step(&mut domain.forge, &env.with(env.limits.forge), forge::Event::Withdraw { entry: first }, &mut out);
        out.clear();
        let entry = crate::fresh(&mut domain.engine.core_mut().counters, crate::Family::ConnectorRow).unwrap();
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::ProjectDesired { entry, goal },
            &mut out,
        );
        assert!(!domain.forge.effect_room(&env.limits.forge), "projection reserves the last room");
        assert_eq!(domain.forge.inspection_reservations().as_ref(), &[goal]);
        env.limits.authority.segments = if translate { 8 } else { 0 };
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::forge_route::outputs(&mut domain, &env, &mut decision, &mut out);
        if translate {
            let Some(crate::Work::Core(jig_domain_core::Event::EffectStart { owner, .. })) = domain.work.pop() else {
                panic!("projection starts its core flight")
            };
            let connector = domain.config.forge_connector;
            crate::route::forge_route::effect_ask(
                &mut domain,
                &env,
                &mut decision,
                connector,
                Ask::Drop {
                    owner,
                    answer: jig_domain_authority::Answer::Refuse,
                    findings: Box::new([]),
                    why: EffectRefusal::Authority,
                },
            );
        }
        let Some(crate::Work::Forge(event @ forge::Event::DropProjection { .. })) = domain.work.pop() else {
            panic!("projection returns its room, whether translation or core refused it")
        };
        forge::step(&mut domain.forge, &env.with(env.limits.forge), event, &mut out);
        assert!(domain.forge.effect_room(&env.limits.forge));
        assert!(domain.forge.inspection_reservations().is_empty());
        assert!(domain.forge_effects.is_empty());
    }
}

#[test]
fn a_projection_meeting_a_full_outbox_ends_the_step() {
    let (domain, env, goal, _) = full_outbox();
    assert!(domain.work.is_empty());
    assert_eq!(domain.forge.next_wall_deadline(), Some(after_interval(&env)));
    let issue = domain.forge.issue(goal).expect("live goal's issue");
    assert!(!issue.failed);
    assert!(issue.pending.is_none());
    assert!(!domain.forge.effect_room(&env.limits.forge));
    assert!(domain.forge.full_outbox_met());
}

#[test]
fn a_projection_asked_at_once_waits_in_the_step_that_met_the_full_outbox() {
    for (name, next_step, room, fire) in [
        ("met", false, false, false),
        ("next step", true, false, false),
        ("room", false, true, false),
        ("fire", false, false, true),
    ] {
        let (mut domain, mut env, goal, first) = full_outbox();
        assert!(domain.forge.full_outbox_met(), "{name}: fixture met the full outbox");
        env.wall = Wall::from_nanos(env.wall.as_nanos().saturating_add(1));
        if next_step {
            domain.forge.begin_step();
        }
        if room {
            forge::step(
                &mut domain.forge,
                &env.with(env.limits.forge),
                forge::Event::Withdraw { entry: first },
                &mut Queue::uncapped(),
            );
            assert!(domain.forge.effect_room(&env.limits.forge), "{name}: withdrawal frees room");
        }
        if fire {
            crate::fire(&mut domain, &env);
            assert!(!domain.forge.full_outbox_met(), "{name}: public timer entry begins a new step");
        }
        let mut out = Queue::uncapped();
        out.push(forge::Request::ProjectAfter { goal, when: None });
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::forge_route::outputs(&mut domain, &env, &mut decision, &mut out);
        if next_step || room || fire {
            let Some(crate::Work::Forge(forge::Event::ProjectDesired { goal: received, .. })) = domain.work.pop()
            else {
                panic!("{name}: projection is queued");
            };
            assert_eq!(received, goal, "{name}");
        } else {
            assert!(domain.work.is_empty(), "{name}: no projection requeued in the full-outbox step");
            assert_eq!(domain.forge.next_wall_deadline(), Some(after_interval(&env)), "{name}");
        }
        assert!(domain.work.is_empty(), "{name}: no extra work");
    }
}
