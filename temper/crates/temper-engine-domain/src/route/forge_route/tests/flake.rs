//! Landing repair choices and their typed endings (forge/change/repairs.md).
use super::super::{Domain, Limits, forge, forge_change as change, tasks};
use super::*;
use skein_lib::{Env, Queue, bytes::find};
use temper_engine_domain_forge_landing as landing;

const FLAKE: &[u8] = b"\nFlake: if every failed test below is a flake (an open flake finding names it, or it passes when run again at this head) and nothing needs changing, finish with verdict 1, naming each test and how you saw it pass; a person then releases the change to run its gate once more.";

fn contract_fixture() -> (Domain, Env<Limits>, u64, forge::ReceiptKey) {
    let (mut domain, mut env, context) = change_start_fixture();
    env.limits.tasks.parameters = 5;
    env.limits.tasks.spec_bytes = 4096;
    let task = context.task;
    let mut row = change_row(Some(7));
    row.task = task;
    row.change.task = task;
    row.repository = adopted(1).provider;
    row.change.last_head = Some(SHA1);
    row.change.gates = Box::new([change::Gate {
        number: 0,
        kind: change::GateKind::Check,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: true,
    }]);
    let mut out = Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut out,
    );
    assert!(domain.forge.project_landing(
        context.project,
        change::Way::Turn,
        Box::new([forge::RepositoryGates {
            repository: adopted(1).provider,
            gates: Box::new([landing::Gate {
                name: b"gate".as_slice().into(),
                workflow: b"gate.yml".as_slice().into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
                report: None,
            }]),
            tiers: Box::new([]),
        }]),
        landing::Project::DEFAULT,
        forge::AfterLanding {
            wait: temper_engine_domain_forge_change::Wait::Cleared,
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
            },
            bound: skein_lib::Duration::from_secs(7200),
        }
    ));
    let (_, _, mut kept) = reserved_turn_landing();
    kept.landing.task = 900;
    kept.landing.members[0].change = task;
    kept.landing.members[0].left = Some(landing::Leave::Failed { gate: 0, run: 42 });
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Landing(kept) },
        &mut out,
    );
    let key = forge::ReceiptKey {
        project: context.project,
        repository: adopted(1).provider,
        check: forge::Check::Gate(b"gate".as_slice().into()),
        key: [9; 32],
    };
    (domain, env, task, key)
}

#[test]
fn only_landing_repairs_with_named_failed_tests_offer_the_flake_choice() {
    let (mut domain, env, task, key) = contract_fixture();
    let mut out = Queue::uncapped();
    for (report, offered) in [
        (
            forge::ReceiptReport::Tests(Box::new([forge::ReceiptTest {
                key: b"test a::t x".as_slice().into(),
                fuzzy: false,
            }])),
            true,
        ),
        (forge::ReceiptReport::Unset, false),
        (forge::ReceiptReport::Unread, false),
        (forge::ReceiptReport::Many { bound: 16 }, false),
        (forge::ReceiptReport::Tests(Box::new([])), false),
    ] {
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Restore {
                record: forge::Stored::Receipt {
                    key: key.clone(),
                    receipt: forge::Receipt {
                        passed: false,
                        run: forge::ReceiptRun { id: 41, number: 42, url: b"run/41".as_slice().into() },
                        candidate: SHA1,
                        base: Some(SHA256),
                        ended: env.wall,
                        jobs: Box::new([]),
                        logs: Box::new([]),
                        report,
                        failures: None,
                    },
                },
            },
            &mut out,
        );
        for kind in [
            change::Delegate::Repair(change::Repair::Landing { gate: 0 }),
            change::Delegate::Repair(change::Repair::LandingAt { gate: 0, candidate: SHA1, below: None }),
            change::Delegate::Repair(change::Repair::Ci),
            change::Delegate::Repair(change::Repair::Semantic),
            change::Delegate::Repair(change::Repair::Gate(0)),
            change::Delegate::Repair(change::Repair::Review { head: SHA1 }),
            change::Delegate::Produce,
            change::Delegate::Resolve { base: SHA256 },
        ] {
            let Some(delegate) = super::super::change_delegate(&domain, &env, task, kind) else {
                panic!("delegate {kind:?}")
            };
            let tasks::Contract::Change { choices, words, .. } = delegate.contract else { panic!("change contract") };
            let landing = match kind {
                change::Delegate::Repair(change::Repair::Landing { .. } | change::Repair::LandingAt { .. }) => true,
                change::Delegate::Repair(
                    change::Repair::Ci
                    | change::Repair::Semantic
                    | change::Repair::Gate(_)
                    | change::Repair::Review { .. },
                )
                | change::Delegate::Produce
                | change::Delegate::Close
                | change::Delegate::Gate { .. }
                | change::Delegate::Resolve { .. }
                | change::Delegate::Merge { .. } => false,
            };
            let expected = offered && landing;
            let expected_choices: Box<[tasks::Verdict]> = if expected {
                Box::from([tasks::Verdict { code: forge::REPAIR_FLAKE, words, followups: 0, fields: Box::new([]) }])
            } else {
                Box::new([])
            };
            assert_eq!(choices, expected_choices, "{kind:?}");
            let line = find(&delegate.spec.words, FLAKE);
            assert_eq!(line.is_some(), expected, "{kind:?}");
            if let Some(at) = line {
                assert!(delegate.spec.words[..at].ends_with(super::super::REPAIR_PRESERVATION));
                assert!(delegate.spec.words[at + FLAKE.len()..].starts_with(b"\nGate run:"));
            }
        }
    }
}

#[test]
fn an_inbox_repair_verdict_reaches_the_forge_as_a_flake_with_its_words() {
    let (mut domain, env, mut context) = change_start_fixture();
    let mut row = change_row(Some(7));
    row.task = context.task;
    row.change.task = context.task;
    row.delegate = Some((52, change::Delegate::Repair(change::Repair::Landing { gate: 0 })));
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
    context.inbox = Box::new([tasks::Word {
        number: 1,
        at: env.wall,
        hits: 1,
        eligible: true,
        from: tasks::Party::Task(52),
        kind: tasks::MessageKind::Result(tasks::ResultKind::Verdict {
            code: forge::REPAIR_FLAKE,
            fields: Box::new([]),
        }),
        words: b"test a::t x passed again".as_slice().into(),
    }]);
    super::super::change_results(&mut domain, &context, None);
    let Some(super::super::Work::Forge(forge::Event::DelegateResult {
        status, flake, no_change, failed, words, ..
    })) = domain.work.pop()
    else {
        panic!("typed terminal")
    };
    assert_eq!(status, change::Status::Passed);
    assert!(flake);
    assert!(!failed && !no_change);
    assert_eq!(words.as_ref(), b"test a::t x passed again");
}
