//! Kept-act translation (temper/docs/plan/playground-reset-r4/notes/contracts.md, D4).
use crate::boundary::Work;
use crate::decision::{Decision, Delivery, JournalOutput};
use crate::{Domain, Limits};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, Env, Queue, ReplyTo};
use temper_engine_domain_forge as forge;

const RESTART_STEPS: u32 = 16;

fn send(domain: &mut Domain, env: &Env<Limits>, row: forge::Kept) {
    let mut out = Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(env),
        forge::Event::Restore { record: forge::Stored::Kept(row) },
        &mut out,
    );
    out.push(forge::Request::SendKept(row));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::outputs(domain, env, &mut decision, &mut out);
    assert!(decision.empty());
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.core.counters,
        &env.limits.journal,
        decision,
        false,
    )
    .unwrap();
}

fn act(domain: &mut Domain, row: forge::Kept, expected_grace: Option<Duration>) -> ReplyTo {
    let Some(Work::Core(core::Event::Act { reply_to, connector, key, act, carried })) = domain.work.pop() else {
        panic!("kept act reaches the core")
    };
    assert_eq!(connector, domain.config.forge_connector);
    assert_eq!(key, row.key);
    assert!(carried.is_none());
    match (row.act, *act) {
        (
            forge::KeptAct::Cancel { project, task },
            people::Act::Cancel { project: actual_project, task: actual_task, grace, reason },
        ) => {
            assert_eq!((actual_project, actual_task), (project, task));
            assert_eq!(grace, expected_grace);
            assert_eq!(reason.as_ref(), b"Removed with its forge");
        }
        (forge::KeptAct::RemoveProject { project }, people::Act::RemoveProject { project: actual }) => {
            assert_eq!(actual, project);
        }
        (forge::KeptAct::RemoveAccount { account }, people::Act::RemoveAccount { account: actual }) => {
            assert_eq!(actual, account);
        }
        pair => panic!("wrong act: {pair:?}"),
    }
    assert!(domain.work.is_empty());
    reply_to
}

fn answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    outcome: people::Outcome,
    row: forge::Kept,
    keep: bool,
) -> Decision {
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::held::held(domain, env, &mut decision, core::Held::Acted { to, outcome });
    let Some(Work::Forge(event)) = domain.work.pop() else { panic!("kept answer reaches the connector") };
    assert_eq!(event, forge::Event::KeptAnswered { key: row.key, keep });
    let mut out = Queue::uncapped();
    forge::step(&mut domain.forge, &crate::translate::environment_forge(env), event, &mut out);
    if keep {
        assert!(out.is_empty());
    } else {
        assert_eq!(out.pop(), Some(forge::Request::Erase { key: forge::Key::Kept(row.key) }));
        assert!(out.is_empty());
    }
    decision
}

#[test]
fn a_kept_cancel_becomes_the_deployment_cancel_and_its_ended_answer_erases() {
    let mut config = crate::tests::resume_fixture::config(7);
    config.core.defaults.templates = Box::new([core::Template {
        number: 1,
        name: b"conversation".as_slice().into(),
        executor: people::TemplateExecutor::Chosen,
        contract: jig_domain_tasks::Contract::Report { words: 32 },
        shallower: 1,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::Given,
        tracked: None,
        flags: jig_domain_tasks::Flags {
            ended_by: jig_domain_tasks::EndedBy::Result,
            conversation: true,
            listed: true,
            retention: jig_domain_tasks::Retention::Kept,
        },
        label: 1,
        grace: Some(Duration::from_secs(3)),
        llm: None,
    }]);
    let (mut domain, env) = crate::tests::resume_fixture::root_with_config(config);
    let row = forge::Kept { key: [71; 16], act: forge::KeptAct::Cancel { project: 1, task: 7 } };
    send(&mut domain, &env, row);
    let to = act(&mut domain, row, Some(Duration::from_secs(3)));
    let decision = answer(
        &mut domain,
        &env,
        to,
        people::Outcome::Refused(people::Refusal::Ended { how: people::Ending::Done }),
        row,
        false,
    );
    assert!(decision.empty());
    assert!(domain.kept_replies.is_empty());
}

#[test]
fn an_ungranted_kept_act_stays_and_notes_its_grant() {
    for (act_kind, line) in [
        (
            forge::KeptAct::Cancel { project: 1, task: 7 },
            b"kept act: task 7 not cancelled: the forge connector is not granted cancel (engine.acts)".as_slice(),
        ),
        (
            forge::KeptAct::RemoveProject { project: 1 },
            b"kept act: project 1 not removed: the forge connector is not granted remove_project (engine.acts)"
                .as_slice(),
        ),
        (
            forge::KeptAct::RemoveAccount { account: 3 },
            b"kept act: account 3 not removed: the forge connector is not granted remove_account (engine.acts)"
                .as_slice(),
        ),
    ] {
        let (mut domain, env) = crate::tests::resume_fixture::root();
        let row = forge::Kept { key: [72; 16], act: act_kind };
        send(&mut domain, &env, row);
        let to = act(&mut domain, row, None);
        let decision = answer(&mut domain, &env, to, people::Outcome::Refused(people::Refusal::NotOffered), row, true);
        crate::decision::accept_routed(
            &mut domain.journal,
            &mut domain.core.counters,
            &env.limits.journal,
            decision,
            false,
        )
        .unwrap();
        let mut out = Queue::uncapped();
        crate::decision::resume(&mut domain.journal, &mut out);
        assert_eq!(out.pop(), Some(JournalOutput::Deliver(Delivery::DemoNote { line: line.into() })));
        assert!(out.is_empty());
    }
}

#[test]
fn a_kept_box_remove_asks_the_box_link() {
    let (mut domain, env) = crate::tests::resume_fixture::root();
    let row = forge::Kept { key: [73; 16], act: forge::KeptAct::BoxRemove { forge: 1 } };
    send(&mut domain, &env, row);
    assert!(domain.work.is_empty());
    assert!(domain.kept_replies.is_empty());
    let mut out = Queue::uncapped();
    crate::decision::resume(&mut domain.journal, &mut out);
    assert_eq!(out.pop(), Some(JournalOutput::Now(crate::Request::BoxRemove { forge: 1 })));
    assert!(out.is_empty());
}

#[test]
fn kept_acts_go_again_after_a_restart() {
    let row = forge::Kept { key: [74; 16], act: forge::KeptAct::RemoveAccount { account: 3 } };
    let (mut before, env) = crate::tests::resume_fixture::root();
    send(&mut before, &env, row);
    let _lost = act(&mut before, row, None);
    let config = crate::tests::resume_fixture::config(7);
    let limits = env.limits;
    let mut policies = skein_lib::List::with_capacity(limits.authority.projects);
    let mut project_mark = 0;
    for project in &config.core.seed_projects {
        project_mark = project_mark.max(u64::from(project.number));
        policies
            .push(people::Stored::Policy {
                project: project.number,
                name: project.name.clone(),
                value: core::policy_snapshot(
                    &project.policy.policy,
                    &project.policy.permissions,
                    &project.policy.templates,
                )
                .unwrap(),
            })
            .unwrap();
    }
    let mut domain = Domain::new(config, &limits).unwrap();
    let mut request = domain.core.restart_begin(false);
    let mut deployment = domain.core.counters.deployment();
    deployment.projects = project_mark;
    crate::restart::restore_page_row(
        &mut domain,
        &env,
        crate::Record::Core(core::Record::Core(core::CoreRecord::Deployment(deployment))),
    );
    for policy in policies.into_boxed() {
        crate::restart::restore_page_row(&mut domain, &env, crate::Record::Core(core::Record::People(policy)));
    }
    let mut out = Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Kept(row) },
        &mut out,
    );
    for _ in 0..RESTART_STEPS {
        match request {
            core::RestartRequest::Step(core::RestartStep::Open) => {
                let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
                crate::restart::restart_request(&mut domain, &env, &mut decision, request);
                assert!(domain.ready());
                let Some(Work::Forge(event)) = domain.work.pop() else { panic!("restart sends kept rows") };
                assert_eq!(event, forge::Event::SendKept);
                forge::step(&mut domain.forge, &crate::translate::environment_forge(&env), event, &mut out);
                super::super::outputs(&mut domain, &env, &mut decision, &mut out);
                let to = act(&mut domain, row, None);
                crate::decision::accept_routed(
                    &mut domain.journal,
                    &mut domain.core.counters,
                    &env.limits.journal,
                    decision,
                    false,
                )
                .unwrap();
                let decision = answer(
                    &mut domain,
                    &env,
                    to,
                    people::Outcome::AccountRemoved { confirmed: true, failed_hosts: Box::new([]) },
                    row,
                    false,
                );
                assert!(decision.empty());
                return;
            }
            core::RestartRequest::Step(step) => {
                if step == core::RestartStep::Switch {
                    crate::tests::resume_fixture::check_switch(&mut domain, &env);
                }
                if step == core::RestartStep::LoadInbox {
                    let _requests = core::want_accounts(&mut domain.core, &crate::limits::environment_core(&env));
                }
                request = domain.core.restart_done(step);
            }
            other @ (core::RestartRequest::HeldPastLimit { .. }
            | core::RestartRequest::UnheldRevision { .. }
            | core::RestartRequest::Settings { .. }
            | core::RestartRequest::ProjectPolicy { .. }
            | core::RestartRequest::Unresolved { .. }
            | core::RestartRequest::Idle
            | core::RestartRequest::Refused { .. }) => panic!("restart did not open: {other:?}"),
        }
    }
    panic!("restart exceeded its steps");
}

#[test]
fn kept_answers_follow_the_saved_answer_rule() {
    for (outcome, keep, why) in [
        (people::Outcome::Again(people::Again::Busy), true, None),
        (people::Outcome::Again(people::Again::NotReady), true, None),
        (
            people::Outcome::Refused(people::Refusal::Role { held: None }),
            true,
            Some(b"no first owner configured".as_slice()),
        ),
        (people::Outcome::Refused(people::Refusal::Unknown), false, None),
        (people::Outcome::Compound { project: 1, operations: 1, first_message: 0 }, false, None),
        (people::Outcome::Refused(people::Refusal::KeyConflict), false, Some(b"KeyConflict".as_slice())),
        (people::Outcome::CompoundRefused { at: 0, why: people::Refusal::Unknown }, false, Some(b"Unknown".as_slice())),
    ] {
        let (mut domain, env) = crate::tests::resume_fixture::root();
        let row = forge::Kept { key: [75; 16], act: forge::KeptAct::Cancel { project: 1, task: 7 } };
        send(&mut domain, &env, row);
        let to = act(&mut domain, row, None);
        let decision = answer(&mut domain, &env, to, outcome, row, keep);
        assert_eq!(decision.empty(), why.is_none());
        crate::decision::accept_routed(
            &mut domain.journal,
            &mut domain.core.counters,
            &env.limits.journal,
            decision,
            false,
        )
        .unwrap();
        let mut out = Queue::uncapped();
        crate::decision::resume(&mut domain.journal, &mut out);
        if let Some(why) = why {
            let Some(JournalOutput::Deliver(Delivery::DemoNote { line })) = out.pop() else { panic!("refusal note") };
            assert!(line.starts_with(b"kept act: task 7 not cancelled: "));
            assert!(line.ends_with(why));
        }
        assert!(out.is_empty());
        assert!(domain.kept_replies.is_empty());
    }
}

#[test]
fn a_removed_project_answer_erases_its_kept_row() {
    let (mut domain, env) = crate::tests::resume_fixture::root();
    let row = forge::Kept { key: [76; 16], act: forge::KeptAct::RemoveProject { project: 1 } };
    send(&mut domain, &env, row);
    let to = act(&mut domain, row, None);
    let decision = answer(&mut domain, &env, to, people::Outcome::ProjectRemoved { project: 1 }, row, false);
    assert!(decision.empty());
}
