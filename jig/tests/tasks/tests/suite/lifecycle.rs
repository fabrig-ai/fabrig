use jig_domain_tasks::{
    Active, Cancel, Cause, Class, Contract, End, Ending, Event, Funder, Hold, Key, Party, Phase, Proposal,
    ProposalAction, ProposalDecision, ProposalHolder, ProposalState, Refusal, Release, ResultFollowups, Stage,
    TaskResult, Was,
};
use jig_tasks_world::{LIMITS, Reply, World, task};

fn refused(reply: &Reply, why: Refusal) {
    assert!(matches!(reply, Reply::Refused(problem) if problem.why == why), "expected {why:?}, got {reply:?}");
}

fn result_followup_proposal(w: &World) -> Proposal {
    let mut child = task(2, &[]);
    child.funder = Funder::Task(1);
    child.numbers.budget = 5;
    child.authority.budget.spend = 5;
    Proposal {
        number: 100,
        proposer: 1,
        project: 1,
        action: ProposalAction::Batch(Box::new([child])),
        reason: Box::new([1]),
        as_holder: false,
        state: ProposalState::Pending { holder: ProposalHolder::Person(1), since: w.env.wall },
    }
}

fn start_result_followup(w: &mut World) {
    let mut first = task(1, &[]);
    first.contract = Contract::Verdict {
        choices: Box::new([jig_domain_tasks::Verdict { fields: Box::new([]), code: 7, words: 32, followups: 1 }]),
    };
    assert_eq!(w.make(Party::Person(1), vec![first, task(3, &[1])]), Reply::Made(vec![1, 3]));
    w.claim_budget(1, 1, 10);
    let proposal = result_followup_proposal(w);
    assert_eq!(
        w.terminal_cause(
            1,
            End::FinishedWithFollowups {
                result: TaskResult::Verdict { fields: Box::new([]), code: 7, words: Box::new([1]) },
                followups: ResultFollowups::Proposal(Box::new(proposal)),
            },
            Cause::Priced { cumulative: 0 },
        ),
        Reply::Acknowledged(jig_domain_tasks::Accepted::New)
    );
    assert!(matches!(w.record(1).phase, Phase::Closing(jig_domain_tasks::Closing { stage: Stage::Delegates, .. })));
    assert!(w.results.is_empty());
    assert!(!w.activations.contains(&3));
}

#[test]
fn a_verdicts_proposal_is_accepted_and_its_followups_run_before_result_delivery() {
    let mut w = World::new(201, LIMITS);
    start_result_followup(&mut w);
    w.restart();
    let mut child = task(2, &[]);
    child.funder = Funder::Task(1);
    child.numbers.budget = 5;
    child.authority.budget.spend = 5;
    let to = w.to();
    let key = to.into_token().raw();
    w.send(Event::MakeResultFollowups {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(key)),
        proposer: 1,
        proposal: 100,
        batch: Box::new([child]),
    });
    assert_eq!(w.replies[&key], Reply::Made(vec![2]));
    assert_eq!(w.record(1).result_followups.as_ref(), [2]);
    w.restart();
    assert_eq!(w.record(1).result_followups.as_ref(), [2]);
    let to = w.to();
    w.send(Event::DecideProposal {
        reply_to: to,
        proposer: 1,
        proposal: 100,
        message: Some(101),
        by: Party::Person(1),
        decision: ProposalDecision::Accept,
    });
    assert!(w.results.is_empty());
    assert_eq!(w.record(1).delegates.as_ref(), [2]);
    w.claim_budget(2, 2, 5);
    w.finish(2);
    w.settle(2);
    assert!(w.results.contains_key(&2) && !w.results.contains_key(&1));
    assert!(w.record(1).delegates.is_empty());
    assert_eq!(w.record(1).result_followups.as_ref(), [2]);
    w.settle(1);
    assert!(w.results.contains_key(&1));
    let jig_domain_tasks::Stored::Ended(row) = &w.records[&Key::Ended(1)] else { panic!() };
    assert_eq!(row.result_followups.as_ref(), [2]);
    assert!(w.activations.contains(&3));
}

#[test]
fn a_verdicts_proposal_is_rejected_and_its_result_is_delivered() {
    let mut w = World::new(202, LIMITS);
    start_result_followup(&mut w);
    w.restart();
    let to = w.to();
    w.send(Event::DecideProposal {
        reply_to: to,
        proposer: 1,
        proposal: 100,
        message: Some(101),
        by: Party::Person(1),
        decision: ProposalDecision::Reject { reason: Box::new([2]) },
    });
    assert!(w.closing.contains(&1));
    assert!(!w.activations.contains(&2));
    assert!(w.record(1).result_followups.is_empty());
    w.settle(1);
    let jig_domain_tasks::Stored::Ended(row) = &w.records[&Key::Ended(1)] else { panic!() };
    assert!(row.result_followups.is_empty());
    assert!(matches!(w.results.get(&1), Some(Ending::Done(TaskResult::Verdict { code: 7, .. }))));
    assert!(w.activations.contains(&3));
}

#[test]
fn cancelling_a_closing_result_withdraws_its_pending_followup_proposal() {
    let mut w = World::new(206, LIMITS);
    start_result_followup(&mut w);
    let to = w.to();
    w.send(Event::Cancel {
        reply_to: to,
        by: Party::Person(1),
        operation: Cancel { task: 1, reason: Box::new([9]), grace: None, message: None },
    });
    assert!(w.record(1).proposal.is_none());
    assert!(!w.record(1).result_proposal);
    assert!(w.closing.contains(&1));
    w.settle(1);
    assert!(matches!(w.results.get(&1), Some(Ending::Cancelled { .. })));
}

#[test]
fn an_authorized_verdict_makes_followups_in_its_result_commit_and_waits_for_them() {
    let mut w = World::new(203, LIMITS);
    let mut first = task(1, &[]);
    first.contract = Contract::Verdict {
        choices: Box::new([jig_domain_tasks::Verdict { fields: Box::new([]), code: 7, words: 32, followups: 1 }]),
    };
    assert_eq!(w.make(Party::Person(1), vec![first, task(3, &[1])]), Reply::Made(vec![1, 3]));
    w.claim_task(1);
    let mut child = task(2, &[]);
    child.funder = Funder::Task(1);
    child.numbers.budget = 5;
    child.authority.budget.spend = 5;
    assert_eq!(
        w.terminal_cause(
            1,
            End::FinishedWithFollowups {
                result: TaskResult::Verdict { fields: Box::new([]), code: 7, words: Box::new([1]) },
                followups: ResultFollowups::Delegates(Box::new([child])),
            },
            Cause::Priced { cumulative: 0 },
        ),
        Reply::Acknowledged(jig_domain_tasks::Accepted::New)
    );
    assert_eq!(w.record(1).delegates.as_ref(), [2]);
    assert!(w.record(2).created_at == w.record(1).created_at);
    w.restart();
    assert_eq!(w.record(1).result_followups.as_ref(), [2]);
    assert!(w.results.is_empty() && !w.activations.contains(&3));
    w.claim_budget(2, 2, 5);
    w.finish(2);
    w.settle(2);
    assert!(w.record(1).delegates.is_empty());
    assert_eq!(w.record(1).result_followups.as_ref(), [2]);
    w.settle(1);
    assert!(w.results.contains_key(&1) && w.activations.contains(&3));
    w.restart();
    let jig_domain_tasks::Stored::Ended(row) = &w.records[&Key::Ended(1)] else { panic!() };
    assert_eq!(row.result_followups.as_ref(), [2]);
}

#[test]
fn a_verdict_cannot_make_followups_its_contract_did_not_allow() {
    let mut w = World::new(205, LIMITS);
    let mut first = task(1, &[]);
    first.contract = Contract::Verdict {
        choices: Box::new([jig_domain_tasks::Verdict { fields: Box::new([]), code: 7, words: 32, followups: 0 }]),
    };
    w.make(Party::Person(1), vec![first]);
    w.claim_task(1);
    let mut child = task(2, &[]);
    child.funder = Funder::Task(1);
    child.numbers.budget = 5;
    child.authority.budget.spend = 5;
    assert_eq!(
        w.terminal_cause(
            1,
            End::FinishedWithFollowups {
                result: TaskResult::Verdict { fields: Box::new([]), code: 7, words: Box::new([1]) },
                followups: ResultFollowups::Delegates(Box::new([child])),
            },
            Cause::Priced { cumulative: 0 },
        ),
        Reply::Acknowledged(jig_domain_tasks::Accepted::New)
    );
    assert!(!w.records.contains_key(&Key::Live(2)));
    assert_eq!(w.record(1).invalid_result, Some(jig_domain_tasks::InvalidResult::Followups));
    assert_eq!(w.record(1).tries.invalid, 1);
}

#[test]
fn a_plan_of_reports_a_choice_and_procedure_tasks_runs_in_dependency_order() {
    let mut w = World::new(204, LIMITS);
    let first = task(1, &[]);
    let mut choice = task(2, &[1]);
    choice.contract = Contract::Verdict {
        choices: Box::new([jig_domain_tasks::Verdict { fields: Box::new([]), code: 1, words: 32, followups: 0 }]),
    };
    let mut procedure = task(3, &[2]);
    procedure.executor = jig_domain_tasks::Executor::Procedure { connector: 1, code: 1 };
    let last = task(4, &[3]);
    assert_eq!(w.make(Party::Person(1), vec![first, choice, procedure, last]), Reply::Made(vec![1, 2, 3, 4]));
    assert!(w.activations.contains(&1));
    assert!(!w.activations.contains(&2) && !w.activations.contains(&3) && !w.activations.contains(&4));
    w.finish_task(1);
    assert!(w.activations.contains(&2) && !w.activations.contains(&3));
    w.claim_task(2);
    w.terminal(
        2,
        End::Finished {
            result: TaskResult::Verdict { fields: Box::new([]), code: 1, words: Box::new([9]) },
            cancel_delegates: false,
        },
    );
    w.settle(2);
    assert!(w.activations.contains(&3) && !w.activations.contains(&4));
    let to = w.to();
    let key = to.into_token().raw();
    w.send(Event::Procedure {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(key)),
        task: 3,
        step: 1,
        read: None,
        decision: jig_domain_tasks::ProcedureDecision::new(jig_domain_tasks::ProcedureEnd::Result(
            TaskResult::Report { words: Box::new([8]) },
        )),
    });
    assert_eq!(w.replies[&key], Reply::Done);
    w.settle(3);
    assert!(w.activations.contains(&4));
    w.finish_task(4);
    assert_eq!(w.results.len(), 4);
}

#[test]
fn batch_is_atomic_and_cycles_and_limits_refuse_at_entrance() {
    let batch_bytes =
        u32::try_from(jig_domain_tasks::batch_bytes(&[task(1, &[]), task(2, &[])]).expect("fixture batch bytes"))
            .expect("fixture batch fits u32");
    for (limits, why) in [
        (jig_domain_tasks::Limits { batch: 1, ..LIMITS }, Refusal::Batch),
        (jig_domain_tasks::Limits { batch_bytes: batch_bytes - 1, ..LIMITS }, Refusal::Batch),
        (jig_domain_tasks::Limits { tasks: 1, ..LIMITS }, Refusal::Live),
        (jig_domain_tasks::Limits { project_tasks: 1, ..LIMITS }, Refusal::Project),
    ] {
        let mut w = World::new(1, limits);
        let before = w.records.clone();
        refused(&w.make(Party::Person(1), vec![task(1, &[]), task(2, &[])]), why);
        assert_eq!(w.records, before);
    }
    let mut w = World::new(1, LIMITS);
    refused(&w.make(Party::Person(1), vec![task(1, &[2]), task(2, &[1])]), Refusal::Circle { cut: false });
    refused(&w.make(Party::Person(1), vec![task(1, &[]), task(1, &[])]), Refusal::Duplicate);
    refused(&w.make(Party::Person(1), vec![]), Refusal::Empty);
    for (new, why) in [
        (
            {
                let mut n = task(2, &[]);
                n.spec.words = vec![1; 65].into_boxed_slice();
                n
            },
            Refusal::Spec,
        ),
        (
            {
                let mut n = task(2, &[]);
                n.contract = Contract::Report { words: 33 };
                n
            },
            Refusal::Contract,
        ),
        (
            {
                let mut n = task(2, &[]);
                n.executor = jig_domain_tasks::Executor::Agent { charter: 99 };
                n
            },
            Refusal::Executor,
        ),
        (task(2, &[99]), Refusal::Dependencies),
        (
            {
                let mut n = task(2, &[]);
                n.authority.delegation.kinds = vec![jig_domain_tasks::AuthorityExecutor::Role(1); 4].into_boxed_slice();
                n
            },
            Refusal::AuthorityShape,
        ),
    ] {
        let before = w.records.clone();
        refused(&w.make(Party::Person(1), vec![task(1, &[]), new]), why);
        assert_eq!(w.records, before);
    }
}

#[test]
fn bounded_retained_inputs_are_left_for_the_roots_archive_check() {
    let mut w = World::new(101, LIMITS);
    let mut next = task(2, &[]);
    next.spec.inputs = Box::new([99]);
    assert_eq!(w.make(Party::Person(1), vec![task(1, &[]), next]), Reply::Made(vec![1, 2]));
}

#[test]
fn a_dependency_on_an_introduced_sibling_is_accepted_and_a_cross_subtree_wait_cycle_refused() {
    for cycle in [false, true] {
        let mut w = World::new(102 + u64::from(cycle), LIMITS);
        w.make(Party::Person(1), vec![task(1, &[])]);
        let right = if cycle { task(3, &[2]) } else { task(3, &[]) };
        w.make(Party::Task(1), vec![task(2, &[]), right]);
        refused(&w.make(Party::Task(2), vec![task(4, &[3])]), Refusal::Dependencies);
        let reply_to = w.to();
        let key = reply_to.into_token().raw();
        w.send(Event::Introduce {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(key)),
            by: 1,
            left: 2,
            right: 3,
        });
        assert_eq!(w.replies[&key], Reply::Done);
        assert_eq!(w.record(2).references.as_ref(), [3]);
        assert_eq!(w.record(3).references.as_ref(), [2]);
        let answer = w.make(Party::Task(2), vec![task(4, &[3])]);
        if cycle {
            refused(&answer, Refusal::Circle { cut: false });
        } else {
            assert_eq!(answer, Reply::Made(vec![4]));
        }
        w.restart();
    }
}

#[test]
fn tree_depth_and_delegate_limits_are_atomic() {
    for (limits, why) in [
        (jig_domain_tasks::Limits { tree_tasks: 1, ..LIMITS }, Refusal::Tree),
        (jig_domain_tasks::Limits { depth: 0, ..LIMITS }, Refusal::Depth),
        (jig_domain_tasks::Limits { delegates: 0, ..LIMITS }, Refusal::Delegates),
    ] {
        let mut w = World::new(2, limits);
        w.make(Party::Person(1), vec![task(1, &[])]);
        let before = w.records.clone();
        refused(&w.make(Party::Task(1), vec![task(2, &[])]), why);
        assert_eq!(w.records, before);
    }
}

#[test]
fn dependency_order_negative_verdict_starts_but_failure_holds() {
    for fail in [false, true] {
        let mut w = World::new(3, LIMITS);
        let mut first = task(1, &[]);
        first.contract = Contract::Verdict {
            choices: Box::new([jig_domain_tasks::Verdict { fields: Box::new([]), code: 0, words: 32, followups: 0 }]),
        };
        w.make(Party::Person(1), vec![first, task(2, &[1])]);
        assert!(!w.activations.contains(&2));
        w.claim_task(1);
        let result = if fail {
            TaskResult::Failure { reason: Box::new([1]) }
        } else {
            TaskResult::Verdict { fields: Box::new([]), code: 0, words: Box::new([]) }
        };
        w.terminal(1, End::Finished { result, cancel_delegates: false });
        assert!(!w.activations.contains(&2));
        w.settle(1);
        if fail {
            assert_eq!(w.record(2).phase, Phase::Held { was: Was::Waiting, why: Hold::Dependency(1), source: None });
            w.restart();
            assert_eq!(w.record(2).phase, Phase::Held { was: Was::Waiting, why: Hold::Dependency(1), source: None });
        } else {
            w.restart();
            w.finish_task(2);
        }
    }
}

#[test]
fn ended_dependency_keeps_a_bounded_stub_until_its_last_live_reader_ends() {
    let mut w = World::new(103, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[]), task(2, &[1])]);
    w.finish_task(1);
    assert!(!w.records.contains_key(&Key::Live(1)));
    assert!(
        matches!(w.records.get(&Key::Stub(1)), Some(jig_domain_tasks::Stored::Stub(stub)) if stub.task == 1 && stub.result.raw() == 1)
    );
    w.restart();
    assert!(w.records.contains_key(&Key::Stub(1)));
    w.finish_task(2);
    assert!(!w.records.contains_key(&Key::Stub(1)));
    assert!(w.records.contains_key(&Key::Ended(1)));
}

#[test]
fn ended_reference_survives_restart_as_a_stub_until_its_peer_ends() {
    let mut w = World::new(104, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.make(Party::Task(1), vec![task(2, &[]), task(3, &[])]);
    let reply_to = w.to();
    w.send(Event::Introduce { reply_to, by: 1, left: 2, right: 3 });
    w.finish_task(2);
    assert!(w.records.contains_key(&Key::Stub(2)));
    assert_eq!(w.record(3).references.as_ref(), [2]);
    w.restart();
    assert_eq!(w.record(3).references.as_ref(), [2]);
    w.finish_task(3);
    assert!(!w.records.contains_key(&Key::Stub(2)));
}

#[test]
fn infrastructure_owns_twelve_retries_and_does_not_spend_a_transient_try() {
    use jig_domain_tasks::{HoldSource, Retry};
    use skein_lib::Duration;
    let mut limits = LIMITS;
    limits.retries.transient = Retry { retries: 3, base: Duration::from_secs(1), max: Duration::from_secs(30) };
    limits.retries.infrastructure = Retry { retries: 12, base: Duration::from_secs(10), max: Duration::from_secs(300) };
    for last_class in [Class::Infrastructure, Class::Transient] {
        let mut w = World::new(24, limits);
        w.make(Party::Person(1), vec![task(1, &[])]);
        let reason: Box<[u8]> = Box::from(b"fetch: TimedOut".as_slice());
        for attempt in 1..=12 {
            w.claim(1, attempt);
            w.terminal(1, End::FailedWithReason { class: Class::Infrastructure, reason: reason.clone() });
            assert!(matches!(w.record(1).phase, Phase::Active(Active::BackingOff { .. })));
            assert_eq!(w.record(1).tries.infrastructure, u32::try_from(attempt).unwrap());
            assert_eq!(w.record(1).tries.transient, 0);
            w.advance();
        }
        w.claim(1, 13);
        w.terminal(1, End::FailedWithReason { class: last_class, reason: reason.clone() });
        if last_class == Class::Infrastructure {
            assert_eq!(
                w.record(1).phase,
                Phase::Held {
                    was: Was::Active(Active::Due),
                    why: Hold::Failures(Class::Infrastructure),
                    source: Some(Box::new(HoldSource::Preparation { words: reason })),
                }
            );
            assert_eq!(w.record(1).tries.infrastructure, 13);
            assert_eq!(w.record(1).tries.transient, 0);
        } else {
            assert!(matches!(w.record(1).phase, Phase::Active(Active::BackingOff { .. })));
            assert_eq!(w.record(1).tries.infrastructure, 12);
            assert_eq!(w.record(1).tries.transient, 1);
        }
        let records = w.records.clone();
        w.restart();
        assert_eq!(w.records, records);
    }
}

#[test]
fn every_failure_class_holds_after_its_retry_budget() {
    for class in [
        Class::Transient,
        Class::Permanent,
        Class::Run,
        Class::Agent,
        Class::Lost,
        Class::Invalid,
        Class::Infrastructure,
    ] {
        let mut w = World::new(4, LIMITS);
        w.make(Party::Person(1), vec![task(1, &[])]);
        for attempt in 1..=3 {
            w.claim(1, attempt);
            w.terminal(1, End::Failed(class));
            if attempt < 3 {
                w.advance();
            }
        }
        assert_eq!(
            w.record(1).phase,
            Phase::Held { was: Was::Active(Active::Due), why: Hold::Failures(class), source: None }
        );
        assert_eq!(w.record(1).tries.of(class), 3);
        let before = w.records.clone();
        w.restart();
        assert_eq!(w.records, before);
        assert!(w.activations.is_empty());
    }
}

#[test]
fn a_delegate_failing_past_its_tries_is_held_and_released_finishes() {
    let mut w = World::new(105, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.make(Party::Task(1), vec![task(2, &[])]);
    for attempt in 1..=3 {
        w.claim(2, attempt);
        w.terminal(2, End::Failed(Class::Run));
        if attempt < 3 {
            w.advance();
        }
    }
    assert_eq!(
        w.record(2).phase,
        Phase::Held { was: Was::Active(Active::Due), why: Hold::Failures(Class::Run), source: None }
    );
    assert_eq!(w.record(2).tries.run, 3);
    w.restart();
    let reply_to = w.to();
    let call = reply_to.into_token().raw();
    w.send(Event::Release {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        by: Party::Task(1),
        operation: Release { task: 2, again: None, message: None },
    });
    assert_eq!(w.replies[&call], Reply::Done);
    assert_eq!(w.record(2).tries, jig_domain_tasks::Tries::NONE);
    w.claim(2, 4);
    w.finish(2);
    w.settle(2);
    assert!(matches!(w.results.get(&2), Some(Ending::Done(_))));
    w.claim(1, 5);
    w.finish(1);
    w.settle(1);
}

#[test]
fn hold_reasons_survive_restart_and_release_without_losing_task_state() {
    let reasons = [
        Hold::Stalled,
        Hold::EffectFailed,
        Hold::Uncertain { entry: 7 },
        Hold::HoldsWaited,
        Hold::PoolLost { pool: 8 },
        Hold::Procedure,
        Hold::Placed { hold: 91 },
    ];
    for (index, reason) in reasons.into_iter().enumerate() {
        let mut w = World::new(106 + u64::try_from(index).expect("bounded reason index"), LIMITS);
        w.make(Party::Person(1), vec![task(1, &[])]);
        if let Hold::Placed { hold } = reason {
            let reply_to = w.to();
            w.send(Event::Place {
                reply_to,
                hold: jig_domain_tasks::PlacedHold {
                    number: hold,
                    project: 1,
                    owner: jig_domain_tasks::Owner::Party { person: 1 },
                    mode: jig_domain_tasks::Mode::Stop,
                    scope: jig_domain_tasks::Scope::Subtree { task: 1 },
                    reason: Box::new([]),
                    placed_at: 0,
                    revision: 1,
                    exempt: Box::new([]),
                    lifted: false,
                },
            });
        } else {
            w.send(Event::Hold { task: 1, why: reason });
        }
        assert_eq!(w.record(1).phase, Phase::Held { was: Was::Active(Active::Due), why: reason, source: None });
        w.restart();
        assert_eq!(w.record(1).phase, Phase::Held { was: Was::Active(Active::Due), why: reason, source: None });
        let reply_to = w.to();
        let call = reply_to.into_token().raw();
        w.send(Event::Release {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
            by: Party::Person(1),
            operation: Release {
                task: 1,
                again: if matches!(reason, Hold::Uncertain { .. }) { Some(true) } else { None },
                message: None,
            },
        });
        assert_eq!(w.replies[&call], Reply::Done);
        assert_eq!(w.record(1).phase, Phase::Active(Active::Due));
    }
}

#[test]
fn releasing_a_held_closing_task_retries_its_unsettled_effects() {
    let mut w = World::new(114, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim_task(1);
    w.finish(1);
    w.send(Event::Hold { task: 1, why: Hold::EffectFailed });
    assert!(matches!(w.record(1).phase, Phase::Held { was: Was::Closing(_), why: Hold::EffectFailed, .. }));
    w.restart();
    let reply_to = w.to();
    let call = reply_to.into_token().raw();
    w.send(Event::Release {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        by: Party::Person(1),
        operation: Release { task: 1, again: None, message: None },
    });
    assert_eq!(w.replies[&call], Reply::Done);
    assert!(matches!(w.record(1).phase, Phase::Closing(jig_domain_tasks::Closing { stage: Stage::Effects, .. })));
    assert!(w.closing.contains(&1), "the close obligation is reissued after release");
    w.settle(1);
    assert!(matches!(w.results.get(&1), Some(Ending::Done(_))));
}

#[test]
fn refusals_and_preparation_failure_do_not_spend_tries() {
    let mut w = World::new(5, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    let reply_to = w.to();
    w.send(Event::Prepare { reply_to, task: 1 });
    w.send(Event::PreparationPaused { task: 1, reason: b"brief: no brief slot is free".as_slice().into() });
    assert_eq!(w.record(1).attempt, 0);
    w.advance();
    for attempt in 1..=5 {
        w.claim(1, attempt);
        w.terminal(1, End::Refused);
        w.advance();
        assert_eq!(w.record(1).tries, jig_domain_tasks::Tries::NONE);
    }
    w.claim(1, 6);
    w.finish(1);
    w.settle(1);
}

#[test]
fn held_running_task_waits_for_terminal_and_retains_its_hold() {
    let mut w = World::new(6, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim_task(1);
    w.send(Event::Hold { task: 1, why: Hold::Stopped });
    assert!(w.stops.contains(&(1, 1)));
    w.terminal(1, End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: false });
    assert!(w.closing.is_empty());
    assert!(matches!(w.record(1).phase, Phase::Held { was: Was::Closing(_), .. }));
    let before = w.records.clone();
    w.restart();
    assert_eq!(w.records, before);
    assert!(w.closing.is_empty());
    assert!(w.results.is_empty());
    assert!(matches!(
        w.record(1).phase,
        Phase::Held {
            was: Was::Closing(jig_domain_tasks::Closing { stage: Stage::Delegates, .. }),
            why: Hold::Stopped,
            ..
        }
    ));
}

#[test]
fn corrected_finish_cancels_delegates_and_closes_deepest_first() {
    let mut w = World::new(7, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim_task(1);
    w.make(Party::Task(1), vec![task(2, &[])]);
    w.claim_task(2);
    w.make(Party::Task(2), vec![task(3, &[])]);
    w.claim_task(3);
    let finish = End::Finished { result: TaskResult::Report { words: Box::new([9]) }, cancel_delegates: false };
    assert_eq!(w.terminal(1, finish), Reply::Acknowledged(jig_domain_tasks::Accepted::New));
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert_eq!(w.record(1).delegates.as_ref(), [2]);
    assert_eq!(w.record(1).tries, jig_domain_tasks::Tries::NONE);
    assert!(w.results.is_empty());
    let reply_to = w.to();
    w.send(Event::Message {
        reply_to,
        project: 1,
        operation: jig_domain_tasks::Message {
            task: 1,
            word: jig_domain_tasks::Word {
                number: 1,
                from: Party::Person(1),
                kind: jig_domain_tasks::MessageKind::Words,
                words: b"Cancel the delegates".as_slice().into(),
                at: w.env.wall,
                hits: 1,
                eligible: true,
            },
        },
    });
    w.claim(1, 4);
    w.observe(jig_tasks_world::referee::Seen::Cancelled { task: 2 });
    w.terminal(1, End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: true });
    assert!(w.closing.is_empty());
    w.terminal(3, End::Finished { result: TaskResult::Report { words: Box::new([9]) }, cancel_delegates: false });
    w.terminal(2, End::Parked);
    assert_eq!(w.closing.iter().copied().collect::<Vec<_>>(), [3]);
    w.settle(3);
    assert!(w.closing.contains(&2));
    w.settle(2);
    assert!(w.closing.contains(&1));
    w.settle(1);
    assert!(
        matches!(w.results.get(&3),Some(Ending::Cancelled { result:Some(TaskResult::Report { words }),.. }) if words.as_ref()==[9])
    );
    assert!(matches!(w.results.get(&1), Some(Ending::Done(_))));
    w.referee.assert_passed(7);
}

#[test]
fn a_cancel_closes_a_tree_of_three_levels_deepest_first_with_runs_live_and_effects_in_flight() {
    let mut w = World::new(71, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim_task(1);
    w.make(Party::Task(1), vec![task(2, &[])]);
    w.claim_task(2);
    w.make(Party::Task(2), vec![task(3, &[])]);
    w.claim_task(3);

    w.terminal(1, End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: true });
    assert!(w.stops.contains(&(2, 2)) && w.stops.contains(&(3, 3)));
    w.terminal(3, End::Parked);
    w.terminal(2, End::Parked);
    assert_eq!(w.closing.iter().copied().collect::<Vec<_>>(), [3]);
    w.settle_effects(3);
    assert!(w.results.is_empty(), "effect settlement alone does not end the deepest task");
    w.release(3);
    assert_eq!(w.closing.iter().copied().collect::<Vec<_>>(), [2]);
    w.settle_effects(2);
    assert!(w.results.contains_key(&3) && !w.results.contains_key(&2));
    w.release(2);
    assert_eq!(w.closing.iter().copied().collect::<Vec<_>>(), [1]);
    w.settle(1);
    assert_eq!(w.results.len(), 3);
    assert!(matches!(w.results.get(&1), Some(Ending::Done(_))));
    assert!(matches!(w.results.get(&2), Some(Ending::Cancelled { .. })));
    assert!(matches!(w.results.get(&3), Some(Ending::Cancelled { .. })));
}

#[test]
fn a_dependent_starts_only_after_its_dependency_has_closed_and_its_effects_settled() {
    let mut w = World::new(72, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[]), task(2, &[1])]);
    assert!(w.activations.contains(&1) && !w.activations.contains(&2));
    w.claim_task(1);
    w.finish(1);
    assert!(w.closing.contains(&1));
    assert!(!w.activations.contains(&2));
    w.settle_effects(1);
    assert!(!w.activations.contains(&2), "resource cleanup still follows effect settlement");
    w.release(1);
    assert!(w.activations.contains(&2));
}

#[test]
fn lifetime_tree_limit_survives_delegate_ending() {
    let mut w = World::new(8, jig_domain_tasks::Limits { tree_tasks: 2, ..LIMITS });
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.make(Party::Task(1), vec![task(2, &[])]);
    w.claim(2, 1);
    w.finish(2);
    w.settle(2);
    assert_eq!(w.record(1).made, 2);
    refused(&w.make(Party::Task(1), vec![task(3, &[])]), Refusal::Tree);
}

#[test]
fn invalid_results_spend_invalid_tries_and_refused_held_closing_make_is_atomic() {
    let mut w = World::new(9, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim_task(1);
    w.terminal(
        1,
        End::Finished {
            result: TaskResult::Verdict { fields: Box::new([]), code: 0, words: Box::new([]) },
            cancel_delegates: false,
        },
    );
    assert_eq!(w.record(1).tries.invalid, 1);
    assert_eq!(w.record(1).invalid_result, Some(jig_domain_tasks::InvalidResult::Form));
    w.restart();
    w.advance();
    assert_eq!(w.contexts[&1].invalid_result, Some(jig_domain_tasks::InvalidResult::Form));
    w.claim(1, 2);
    assert_eq!(w.record(1).invalid_result, None);
    w.finish(1);
    w.send(Event::Hold { task: 1, why: Hold::Effects });
    let before = w.records.clone();
    refused(&w.make(Party::Task(1), vec![task(2, &[]), task(3, &[2])]), Refusal::State);
    assert_eq!(w.records, before);
    w.restart();
    w.settle(1);
    assert!(w.records.contains_key(&Key::Live(1)));
    assert!(w.results.is_empty());
    let before = w.records.clone();
    w.restart();
    assert_eq!(w.records, before);
}

#[test]
fn cancelling_unfinished_dependency_siblings_finishes_in_either_settlement_order() {
    for order in [[2, 3], [3, 2]] {
        let mut w = World::new(11, LIMITS);
        w.make(Party::Person(1), vec![task(1, &[])]);
        w.claim_task(1);
        w.make(Party::Task(1), vec![task(2, &[]), task(3, &[2])]);
        w.claim_task(2);
        w.observe(jig_tasks_world::referee::Seen::Cancelled { task: 2 });
        w.observe(jig_tasks_world::referee::Seen::Cancelled { task: 3 });
        w.terminal(1, End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: true });
        w.terminal(2, End::Parked);
        assert!(w.closing.contains(&2) && w.closing.contains(&3));
        for task in order {
            w.settle(task);
            assert!(matches!(w.results.get(&task), Some(Ending::Cancelled { .. })));
        }
        w.settle(1);
        assert_eq!(w.live(), 0);
        w.referee.assert_passed(11);
    }
}

#[test]
fn lifecycle_history_has_independent_positions_and_restore_does_not_repeat_a_phase() {
    let mut world = World::new(173, LIMITS);
    world.make(Party::Person(1), vec![task(1, &[])]);
    let first = world.record(1).milestone;
    let revision = world.record(1).revision;
    world.send(Event::Hold { task: 1, why: Hold::Effects });
    let held = world.record(1).milestone;
    assert_eq!(held, first + 1);
    assert_eq!(world.record(1).revision, revision, "lifecycle does not consume semantic revisions");
    world.send(Event::Hold { task: 1, why: Hold::Effects });
    assert_eq!(world.record(1).milestone, held, "the same phase has no second milestone");
    world.restart();
    assert_eq!(world.record(1).milestone, held, "restoration keeps the immutable history position");
    let reply_to = world.to();
    world.send(Event::Release {
        reply_to,
        by: Party::Person(1),
        operation: Release { task: 1, again: None, message: None },
    });
    assert_eq!(world.record(1).milestone, held + 1);
    assert_eq!(world.record(1).revision, revision + 1, "release also has its semantic history row");
    assert!(
        matches!(world.records.get(&Key::Milestone { task: 1, position: held }), Some(jig_domain_tasks::Stored::Milestone(row)) if matches!(row.phase, Phase::Held { why: Hold::Effects, .. }))
    );
    assert!(
        matches!(world.records.get(&Key::Milestone { task: 1, position: held + 1 }), Some(jig_domain_tasks::Stored::Milestone(row)) if row.phase == world.record(1).phase)
    );
}

#[path = "../lifecycle/goals.rs"]
mod goals;

#[path = "../lifecycle/delegates.rs"]
mod delegates;

#[path = "../lifecycle/grace.rs"]
mod grace;

#[path = "../lifecycle/compound.rs"]
mod compound;

fn open_task() -> jig_domain_tasks::New {
    let mut value = task(1, &[]);
    value.flags.conversation = true;
    value.flags.ended_by = jig_domain_tasks::EndedBy::Requester;
    value
}

#[test]
fn open_discards_every_raw_finish_before_result_judgment() {
    let ends = [
        End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: true },
        End::Finished { result: TaskResult::Failure { reason: Box::new([1]) }, cancel_delegates: false },
        End::Finished {
            result: TaskResult::Verdict { fields: Box::new([]), code: 0, words: Box::new([]) },
            cancel_delegates: false,
        },
        End::Finished {
            result: TaskResult::Report { words: vec![1; skein_lib::length::usize_of(LIMITS.result_bytes) + 1].into() },
            cancel_delegates: false,
        },
        End::FinishedWithFollowups {
            result: TaskResult::Failure { reason: Box::new([]) },
            followups: ResultFollowups::Delegates(Box::new([task(2, &[])])),
        },
    ];
    for end in ends {
        let mut w = World::new(326, LIMITS);
        w.make(Party::Person(1), vec![open_task()]);
        w.claim_task(1);
        assert_eq!(
            w.terminal_cause(1, end, Cause::Priced { cumulative: 0 }),
            Reply::Acknowledged(jig_domain_tasks::Accepted::New)
        );
        assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
        assert_eq!(w.record(1).tries.invalid, 0);
        assert_eq!(w.record(1).invalid_result, None);
        assert!(w.record(1).delegates.is_empty());
        assert!(w.results.is_empty());
        w.restart();
        assert_eq!(w.record(1).flags.ended_by, jig_domain_tasks::EndedBy::Requester);
        assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    }
}

#[test]
fn open_runtime_failures_keep_the_existing_retry_policy() {
    let mut w = World::new(326, LIMITS);
    w.make(Party::Person(1), vec![open_task()]);
    w.claim_task(1);
    w.terminal(1, End::Failed(Class::Invalid));
    assert_eq!(w.record(1).tries.invalid, 1);
    assert!(matches!(w.record(1).phase, Phase::Active(Active::BackingOff { .. })));
}

#[test]
fn open_park_keeps_saved_charge_and_delegates_then_cancel_wins() {
    let mut w = World::new(326, LIMITS);
    w.make(Party::Person(1), vec![open_task()]);
    w.claim_task(1);
    w.make(Party::Task(1), vec![task(2, &[])]);
    let saved = Box::new([jig_domain_tasks::SavedResource { connector: 1, path: Box::new([Box::from([3])]) }]);
    let reply_to = w.to();
    w.send(Event::Activation {
        reply_to,
        task: 1,
        attempt: 1,
        cause: Cause::Priced { cumulative: 3 },
        end: End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: true },
        saved: Some(saved.clone()),
    });
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert_eq!(w.record(1).saved.as_ref(), saved.as_ref());
    assert_eq!(w.record(1).numbers.spent, 3);
    assert_eq!(w.record(1).delegates.as_ref(), [2]);
    let reply_to = w.to();
    w.send(Event::Cancel {
        reply_to,
        by: Party::Person(1),
        operation: Cancel { task: 1, reason: Box::new([4]), grace: None, message: None },
    });
    w.complete_cancel();
    assert!(matches!(w.results.get(&1), Some(Ending::Cancelled { .. })));
}

#[test]
fn open_still_refuses_bad_saved_work_before_normalizing_a_finish() {
    let mut w = World::new(326, LIMITS);
    w.make(Party::Person(1), vec![open_task()]);
    w.claim_task(1);
    let before = w.record(1).clone();
    let reply_to = w.to();
    let call = reply_to.into_token().raw();
    w.send(Event::Activation {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        task: 1,
        attempt: 1,
        cause: Cause::Priced { cumulative: 0 },
        end: End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: false },
        saved: Some(
            vec![
                jig_domain_tasks::SavedResource { connector: 1, path: Box::new([]) };
                skein_lib::length::usize_of(LIMITS.saved_resources) + 1
            ]
            .into(),
        ),
    });
    refused(&w.replies[&call], Refusal::Contract);
    assert_eq!(w.record(1), &before);
}

const PREPARING_TASK: u64 = 1;
const PREPARATION_PARTY: u64 = 1;

fn prepare_reason_task(w: &mut World) {
    let reply_to = w.to();
    w.send(Event::Prepare { reply_to, task: PREPARING_TASK });
}

#[test]
fn preparing_failures_hold_past_the_allowance_with_the_reason() {
    let mut w = World::new(327, LIMITS);
    w.make(Party::Person(PREPARATION_PARTY), vec![task(PREPARING_TASK, &[])]);
    let allowance = LIMITS.retries.permanent.retries;
    for count in 0..=allowance {
        prepare_reason_task(&mut w);
        w.send(Event::PreparationCounted {
            task: PREPARING_TASK,
            class: Class::Permanent,
            reason: b"brief: required section cannot fit".as_slice().into(),
        });
        if count < allowance {
            w.advance();
        }
    }
    assert_eq!(
        w.record(PREPARING_TASK).phase,
        Phase::Held {
            was: Was::Active(Active::Due),
            why: Hold::Failures(Class::Permanent),
            source: Some(Box::new(jig_domain_tasks::HoldSource::Preparation {
                words: b"brief: required section cannot fit".as_slice().into(),
            })),
        }
    );
    assert_eq!(
        w.record(PREPARING_TASK).retry_notice,
        Some(jig_domain_tasks::RetryNotice::Failed {
            class: Class::Permanent,
            count: allowance + 1,
            allowance,
            reason: Some(b"brief: required section cannot fit".as_slice().into()),
        })
    );
    assert_eq!(w.record(PREPARING_TASK).attempt, 0);
    let before = w.records.clone();
    w.restart();
    assert_eq!(w.records, before);
}

#[test]
fn preparation_pauses_keep_the_reason_without_spending_tries() {
    let mut w = World::new(328, LIMITS);
    w.make(Party::Person(PREPARATION_PARTY), vec![task(PREPARING_TASK, &[])]);
    // More pauses than the transient allowance still spend no failure try.
    let pauses = LIMITS.retries.transient.retries + 2;
    for count in 1..=pauses {
        prepare_reason_task(&mut w);
        w.send(Event::PreparationPaused { task: PREPARING_TASK, reason: b"brief: no slot is free".as_slice().into() });
        assert_eq!(
            w.record(PREPARING_TASK).retry_notice,
            Some(jig_domain_tasks::RetryNotice::Refused {
                count,
                reason: Some(b"brief: no slot is free".as_slice().into()),
            })
        );
        assert_eq!(w.record(PREPARING_TASK).tries, jig_domain_tasks::Tries::NONE);
        assert_eq!(w.record(PREPARING_TASK).attempt, 0);
        let before = w.records.clone();
        w.restart();
        assert_eq!(w.records, before);
        w.advance();
    }
}

#[test]
fn preparation_failures_back_off_with_their_class_and_reason() {
    for class in [Class::Transient, Class::Permanent] {
        let mut w = World::new(329, LIMITS);
        w.make(Party::Person(PREPARATION_PARTY), vec![task(PREPARING_TASK, &[])]);
        prepare_reason_task(&mut w);
        w.send(Event::PreparationCounted {
            task: PREPARING_TASK,
            class,
            reason: b"brief: section unavailable".as_slice().into(),
        });
        assert!(matches!(w.record(PREPARING_TASK).phase, Phase::Active(Active::BackingOff { .. })));
        assert_eq!(w.record(PREPARING_TASK).tries.of(class), 1);
        assert_eq!(
            w.record(PREPARING_TASK).retry_notice,
            Some(jig_domain_tasks::RetryNotice::Failed {
                class,
                count: 1,
                allowance: LIMITS.retries.of(class).retries,
                reason: Some(b"brief: section unavailable".as_slice().into()),
            })
        );
        let before = w.records.clone();
        w.restart();
        assert_eq!(w.records, before);
    }
}

#[test]
fn preparation_reasons_cut_at_whole_characters() {
    // 66 three-byte characters plus two ASCII bytes: 200 bytes, cut at 126.
    const INPUT_CHARACTERS: usize = 66;
    const KEPT_CHARACTERS: usize = 42;
    let words = format!("{}ab", "界".repeat(INPUT_CHARACTERS));
    let expected = "界".repeat(KEPT_CHARACTERS).into_bytes().into_boxed_slice();
    for event in [
        Event::PreparationPaused { task: PREPARING_TASK, reason: words.as_bytes().into() },
        Event::PreparationCounted { task: PREPARING_TASK, class: Class::Permanent, reason: words.as_bytes().into() },
    ] {
        let mut w = World::new(330, LIMITS);
        w.make(Party::Person(PREPARATION_PARTY), vec![task(PREPARING_TASK, &[])]);
        prepare_reason_task(&mut w);
        w.send(event);
        let reason = match &w.record(PREPARING_TASK).retry_notice {
            Some(
                jig_domain_tasks::RetryNotice::Failed { reason, .. }
                | jig_domain_tasks::RetryNotice::Refused { reason, .. },
            ) => reason.as_ref().expect("preparation reason"),
            None => panic!("preparation notice"),
        };
        assert_eq!(reason, &expected);
        assert!(reason.len() <= usize::try_from(jig_domain_tasks::REASON_BYTES).expect("byte bound"));
        assert!(std::str::from_utf8(reason).is_ok());
    }
}

#[test]
fn preparation_notifications_ignore_tasks_outside_preparing_and_restore() {
    let events: [fn() -> Event; 2] = [
        || Event::PreparationPaused { task: PREPARING_TASK, reason: b"brief: busy".as_slice().into() },
        || Event::PreparationCounted {
            task: PREPARING_TASK,
            class: Class::Permanent,
            reason: b"brief: unavailable".as_slice().into(),
        },
    ];
    for event in events {
        let mut w = World::new(331, LIMITS);
        w.send(event());
        w.make(Party::Person(PREPARATION_PARTY), vec![task(PREPARING_TASK, &[])]);
        let before = w.records.clone();
        w.send(event());
        assert_eq!(w.records, before);
        let mut domain = jig_tasks_world::domain(&LIMITS, 331);
        let mut out = skein_lib::Queue::uncapped();
        let mut preparing = w.record(PREPARING_TASK).clone();
        preparing.phase = Phase::Active(Active::Preparing);
        jig_domain_tasks::step(
            &mut domain,
            &w.env,
            Event::Restore { record: jig_domain_tasks::Stored::Live(Box::new(preparing.clone())) },
            &mut out,
        );
        jig_domain_tasks::step(&mut domain, &w.env, event(), &mut out);
        assert!(out.is_empty());
        assert_eq!(domain.task(PREPARING_TASK), Some(&preparing));
    }
}
