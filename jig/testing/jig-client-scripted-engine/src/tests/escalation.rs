use super::*;

#[test]
fn escalation_script_preserves_supplied_identity_scope_and_outer_offer_without_synthetic_terminal() {
    let mut setup = script();
    let mut task = task();
    task.context = None;
    task.acts = list(vec![Offered { effect: None, act: ActKind::Stop, admission: Admission::Allowed }]);
    let value = ReadValue::Escalation(Box::new(Escalation {
        task: Box::new(task),
        acts: list(vec![Offered { effect: None, act: ActKind::Release, admission: Admission::Allowed }]),
    }));
    setup.truth.objects.insert(ObjectKey::Escalation(8), value.clone());
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), PROJECT);
    let engine = Engine::new(setup.clone(), 1).expect("valid script");
    assert_eq!(engine.truth.objects[&ObjectKey::Escalation(8)], value);
    // The embedded project is authoritative even if an obsolete fallback differs.
    setup.truth.object_projects.insert(ObjectKey::Escalation(8), 2);
    assert!(Engine::new(setup.clone(), 1).is_ok());
    assert_eq!(
        crate::Change::Object { project: 2, key: ObjectKey::Escalation(8), value: value.clone() }
            .validate(&Bounds::default()),
        Err(Invalid::Identity)
    );
    setup.truth.objects.remove(&ObjectKey::Escalation(8));
    setup.truth.objects.insert(ObjectKey::Escalation(99), value);
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Identity));
}

const ANCESTOR: u64 = 6;
const PARENT: u64 = 7;
const HELD_TASK: u64 = 8;
const HOLD_REVISION: u64 = 1;
// The default script makes each commit durable after ten milliseconds.
const COMMIT_NANOS: u64 = 10_000_000;

fn waiting_escalation(holder: EscalationHolder) -> Escalation {
    let mut task = task();
    let mut held = hold(HoldCategory::Stopped, HOLD_REVISION);
    held.escalation = Some(HoldEscalation::Waiting { holder, entry: 1, since: Wall::EPOCH });
    task.hold = Some(Box::new(held));
    Escalation {
        task: Box::new(task),
        acts: list(vec![Offered { effect: None, act: ActKind::Release, admission: Admission::Allowed }]),
    }
}
fn release_escalation() -> Ask {
    Ask::Decide {
        project: PROJECT,
        on: DecideOn::Escalation {
            task: HELD_TASK,
            revision: Revision { value: HOLD_REVISION },
            decision: Decision::Released { again: None },
        },
    }
}
#[test]
fn escalation_records_walk_to_the_person_who_asked_instead_of_the_decider() {
    for (requester, expected) in
        [(Requester::Person(REQUESTER), REQUESTER), (Requester::Deployment { project: PROJECT }, 0)]
    {
        let mut setup = script();
        let mut parent = task();
        parent.chip.task = PARENT;
        parent.requester = Requester::Task(ANCESTOR);
        let mut ancestor = task();
        ancestor.chip.task = ANCESTOR;
        ancestor.requester = requester;
        setup.truth.objects.insert(ObjectKey::Task(ANCESTOR), ReadValue::Task(Box::new(ancestor)));
        setup.truth.objects.insert(ObjectKey::Task(PARENT), ReadValue::Task(Box::new(parent)));
        let mut escalation = waiting_escalation(EscalationHolder::Person(DECIDER));
        escalation.task.requester = Requester::Task(PARENT);
        escalation.task.lineage = list(vec![ANCESTOR, PARENT]);
        setup.truth.objects.insert(ObjectKey::Escalation(HELD_TASK), ReadValue::Escalation(Box::new(escalation)));
        let mut engine = Engine::new(setup, 1).expect("valid escalation");
        send(&mut engine, 0, DECIDER_SIGN_IN, 1, 1, release_escalation());
        let answers = engine.poll(at(COMMIT_NANOS));
        assert!(matches!(final_answer(&answers[0]), Answer::Done(_)));
        let ReadValue::Ended(ended) = &engine.truth.objects[&ObjectKey::Escalation(HELD_TASK)] else {
            panic!("retained escalation");
        };
        let EndedValue::Escalation { decision, .. } = &ended.value else { panic!("escalation record") };
        assert_eq!(decision.requester, expected);
        assert_eq!(decision.by, DECIDER);
    }
}
#[test]
fn a_task_holder_gives_its_requester_standing_only_for_a_proposal() {
    let mut setup = live_script();
    let escalation = waiting_escalation(EscalationHolder::Task(HELD_TASK));
    setup.truth.objects.insert(ObjectKey::Escalation(HELD_TASK), ReadValue::Escalation(Box::new(escalation)));
    let mut proposal = proposal();
    proposal.waiting.holder = HistoryHolder::Task(HELD_TASK);
    let ask = Ask::Decide {
        project: PROJECT,
        on: DecideOn::Proposal { proposal: proposal.key, decision: Decision::Accepted, running: None },
    };
    setup.truth.objects.insert(ObjectKey::Proposal(proposal.key), ReadValue::Proposal(Box::new(proposal)));
    let mut engine = Engine::new(setup, 1).expect("valid task holders");
    send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, release_escalation());
    let answers = engine.poll(at(COMMIT_NANOS));
    assert_eq!(
        final_answer(&answers[0]),
        &Answer::Refused { refusal: Refusal::Moved { now: Revision { value: 0 }, what: Moved::Waiting }, at: None }
    );
    assert!(matches!(engine.truth.objects[&ObjectKey::Escalation(HELD_TASK)], ReadValue::Escalation(_)));
    send(&mut engine, COMMIT_NANOS, REQUESTER_SIGN_IN, 2, 2, ask);
    let answers = engine.poll(at(COMMIT_NANOS * 2));
    assert!(matches!(
        final_answer(&answers[0]),
        Answer::Done(Outcome::Document(DocumentOutcome::ProposalDecided { by: REQUESTER, .. }))
    ));
}
