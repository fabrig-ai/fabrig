//! Limits are durable settings, never applied by the core (domain/engine/settings.md, 1).
use crate::settings_support::*;
use jig_domain_core::{self as core, documents, reads, settings as s};
use jig_domain_people as people;
use jig_domain_tasks as tasks;

fn value() -> s::Value {
    s::Value::Limits(Box::new([s::Override {
        name: b"tasks.tasks".as_slice().into(),
        value: 0,
        reason: b"shape alone; not applied".as_slice().into(),
    }]))
}
fn item(edit: s::Edit) -> s::Item {
    s::Item { setting: s::Setting::Limits, edit }
}
fn point(revision: u32, proposal: Option<(people::Proposer, u64)>) -> reads::Read {
    reads::Read::Setting { setting: s::Setting::Limits, revision, proposal }
}
#[test]
fn limits_start_empty_change_restart_and_reset_without_applying_their_figures() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let reads::Answer::Setting(default) = read(&mut world, signed, point(1, None)) else { panic!("default limits") };
    assert_eq!(default.revision, 1);
    assert_eq!(default.value, Some(s::Value::Limits(Box::default())));
    assert_eq!(world.domain.settings().lookup(s::Setting::Limits).1, Some((false, 1)));
    changed(&change(&mut world, signed, 1, 1, Box::new([item(s::Edit::Set(value()))])), 2);
    let reads::Answer::Setting(current) = read(&mut world, signed, point(2, None)) else { panic!("current limits") };
    assert_eq!(current.revision, 2);
    assert_eq!(current.value, Some(value()));
    let document = inventory(&mut world, signed);
    let line = document.settings.iter().find(|line| line.setting == s::Setting::Limits).expect("limits in inventory");
    assert_eq!(line.last, Some(2));
    restart(&mut world, configuration().0);
    assert_eq!(world.domain.settings().value(s::Setting::Limits), Some(&value()));
    assert_eq!(world.domain.settings().newest(), 2);
    let signed = sign_in(&mut world);
    changed(&change(&mut world, signed, 2, 2, Box::new([item(s::Edit::Reset)])), 3);
    assert_eq!(world.domain.settings().value(s::Setting::Limits), Some(&s::Value::Limits(Box::default())));
}
#[test]
fn removing_or_misshaping_limits_refuses_the_named_item() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let malformed = s::Value::Limits(Box::new([s::Override {
        name: b"Bad".as_slice().into(),
        value: 4,
        reason: b"reason".as_slice().into(),
    }]));
    for (key, edit) in [(1, s::Edit::Removed), (2, s::Edit::Set(malformed))] {
        let template = template_edit(&world, 2);
        let reply = change(&mut world, signed, key, 1, Box::new([template, item(edit)]));
        assert_eq!(
            reply,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
                at: 1,
                why: people::Malformed::Settings { setting: s::Setting::Limits, fault: s::Fault::Shape, detail: None }
            }))
        );
        assert_eq!(world.domain.settings().newest(), 1);
    }
}
#[test]
fn a_partys_limits_proposal_reviews_current_and_proposed_values_then_applies() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    let number = proposed(&propose(&mut world, signed, 1, 1, Box::new([item(s::Edit::Set(value()))])));
    let doc = proposal_document(&mut world, signed, OWNER, number);
    let documents::ProposalAction::Settings { items, review: Some(review), .. } = doc.action else {
        panic!("settings review")
    };
    assert_eq!(items.get(0).expect("limits head").setting, s::Setting::Limits);
    let reviewed = review.items.get(0).expect("limits review");
    assert_eq!(reviewed.setting, s::Setting::Limits);
    assert_eq!(reviewed.now, Some(s::Value::Limits(Box::default())));
    assert_eq!(reviewed.would_be, Some(value()));
    let reads::Answer::Setting(proposed) =
        read(&mut world, signed, point(0, Some((people::Proposer::Party(OWNER), number))))
    else {
        panic!("proposed limits")
    };
    assert_eq!(proposed.value, Some(value()));
    let reply =
        decide(&mut world, signed, 2, OWNER, number, people::ProposalDecision::Accept, Some(people::Running::Stays));
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::ProposalDecided { .. })), "{reply:?}");
    assert_eq!(world.domain.settings().value(s::Setting::Limits), Some(&value()));
    assert_eq!(world.domain.settings().newest(), 2);
}
#[test]
fn a_runs_limits_proposal_keeps_its_head_and_applies_after_an_owners_decision() {
    let (mut world, signed, task) = proposing_run(true);
    let core::CallPart::Proposed { proposal, .. } = run_propose(&mut world, 1, Box::new([item(s::Edit::Set(value()))]))
    else {
        panic!("run proposes limits")
    };
    let waits = world.domain.core().waits(task);
    let wait = waits.first.expect("the pending settings proposal is the task's first wait");
    assert_eq!(wait.kind, core::WaitKind::Party);
    assert_eq!(
        wait.object,
        core::WaitObject::Entry {
            kind: people::EntryKind::Proposal { proposer: people::Proposer::Task(task), number: proposal },
            at: tasks::EscalationHolder::Role { project: people::DEPLOYMENT, role: people::Role::Owner.number() },
        }
    );
    assert!(matches!(wait.bound, core::Bound::At { then: core::AtBound::Overdue, .. }));
    let reads::Answer::Document(doc) =
        read(&mut world, signed, reads::Read::Proposal { proposer: people::Proposer::Task(task), proposal })
    else {
        panic!("run proposal")
    };
    let documents::Document::Proposal(doc) = *doc else { panic!("proposal document") };
    let documents::ProposalAction::Settings { items, .. } = doc.action else { panic!("settings head") };
    assert_eq!(items.get(0).expect("limits head").setting, s::Setting::Limits);
    let reply = ask(
        &mut world,
        signed,
        2,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    );
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::ProposalDecided { .. })), "{reply:?}");
    assert_eq!(world.domain.settings().value(s::Setting::Limits), Some(&value()));
    assert_eq!(world.domain.settings().newest(), 2);
    // The engine claim is the proposing-run fixture's, not restored by its oracle.
    world.broken();
}
#[test]
fn largest_limits_fit_the_stocks_derived_settings_value_bound_and_round_trip() {
    let domain = jig_protocol_config::stock().core;
    let bound = jig_protocol_store::sizes::settings_value(&domain).expect("stock settings value bound");
    let mut values = Vec::new();
    for at in 0..s::OVERRIDES {
        let mut name = format!("{at:03}").into_bytes();
        name.resize(usize::try_from(s::OVERRIDE_NAME_BYTES).expect("name bound"), b'_');
        values.push(s::Override {
            name: name.into_boxed_slice(),
            value: u64::MAX,
            reason: vec![b'r'; usize::try_from(s::OVERRIDE_REASON_BYTES).expect("reason bound")].into_boxed_slice(),
        });
    }
    assert_eq!(s::check_overrides(&values), Ok(()));
    let record = s::Record::Changed {
        setting: s::Setting::Limits,
        changed: s::Changed { value: Some(s::Value::Limits(values.into_boxed_slice())), revision: 2 },
    };
    let limits = jig_protocol_store::codec::settings::limits(&domain).expect("stock codec limits");
    let encoded = jig_protocol_store::codec::settings::encode(&record, &limits, &[]).expect("maximum overrides encode");
    assert!(encoded.len() <= usize::try_from(bound).expect("value bound"), "encoded {} <= {bound}", encoded.len());
    let decoded = jig_protocol_store::codec::settings::decode(&record.key(), &encoded, &limits, &[])
        .expect("maximum overrides decode");
    assert!(decoded == record, "maximum limits round-trip without losing any member");
}
