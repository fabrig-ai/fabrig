use super::revisions::{Harness, task};
use crate::{Event, MessageKind, Party, Request};
use alloc::boxed::Box;
use skein_lib::Wall;

fn refused_news(number: u64) -> crate::Word {
    crate::Word {
        number,
        from: Party::Task(1),
        kind: MessageKind::News { subscription: 10, class: crate::NewsClass::Wakes },
        words: Box::new([]),
        at: Wall::EPOCH,
        hits: 1,
        eligible: false,
    }
}

#[test]
fn merging_refused_news_retires_its_refusal_and_relays_the_new_hint() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    let row = &mut crate::domain::task_mut(&mut h.domain, 1).expect("task").record;
    row.attempt = 1;
    row.phase = crate::Phase::Active(crate::Active::Claimed { attempt: 1 });
    row.last_message = 1;
    row.inbox = Box::new([refused_news(1)]);
    row.refused = Box::new([1]);
    row.subscriptions = Box::new([crate::Subscription {
        number: 10,
        kind: crate::SubscriptionKind::Topic { connector: 0, topic: 10 },
    }]);
    let out = h.send(Event::Notice { task: 1, word: refused_news(2) });
    assert!(h.domain.task(1).expect("task").refused.is_empty());
    let mut relayed = false;
    for request in &out {
        if let Request::Relay { word, .. } = request {
            relayed |= word.number == 2;
        }
    }
    assert!(relayed);
}

#[test]
fn an_answers_refusals_are_kept_without_a_turn_or_charge() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    let row = &mut crate::domain::task_mut(&mut h.domain, 1).expect("task").record;
    row.attempt = 1;
    row.phase = crate::Phase::Active(crate::Active::Claimed { attempt: 1 });
    row.last_message = 1;
    row.inbox = Box::new([refused_news(1)]);
    let out = h.send(Event::Refusals { task: 1, attempt: 1, refused: Box::new([1]), offered: Some(1) });
    let row = h.domain.task(1).expect("task");
    assert_eq!(row.refused.as_ref(), &[1]);
    assert_eq!(row.inbox.len(), 1);
    assert_eq!((row.turn, row.run_spent), (0, 0));
    let mut kept = false;
    for request in &out {
        if let Request::Save { record: crate::Stored::Live(row) } = request {
            kept |= row.refused.as_ref() == [1];
        }
    }
    assert!(kept, "answer refusal row is durable");
}

fn job(number: u64) -> crate::New {
    let mut member = task(number, &[]);
    member.executor = crate::Executor::Job { kind: 7 };
    member.spec.job =
        Some(Box::new(crate::JobSpec { bytes: Box::new([0xff]), deadline: skein_lib::Duration::from_secs(7) }));
    member.spec.words = Box::new([]);
    member.contract = crate::Contract::Job { bytes: 16 };
    member.numbers = crate::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    member.authority.budget.spend = 0;
    member.funder = crate::Funder::Task(1);
    member
}

fn procedure() -> Harness {
    let mut h = Harness::new();
    let mut parent = task(1, &[]);
    parent.executor = crate::Executor::Procedure { connector: 1, code: 1 };
    h.make([parent].into());
    h
}

fn admission(h: &mut Harness, creator: Party, member: crate::New) -> Option<crate::Refusal> {
    let out = h.send(Event::Make { reply_to: super::revisions::to(), creator, batch: Box::new([member]) });
    for request in out {
        if let Request::Refused { problem, .. } = request {
            return Some(problem.why);
        }
    }
    None
}

/// A procedure makes its job with bounded bytes (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_procedures_batch_admits_a_job_with_its_spec_and_contract() {
    let mut h = procedure();
    let member = job(2);
    assert_eq!(admission(&mut h, Party::Task(1), member.clone()), None);
    let row = h.domain.task(2).unwrap();
    assert_eq!(row.executor, member.executor);
    assert_eq!(row.spec, member.spec);
    assert_eq!(row.contract, member.contract);
}

/// Only procedures create jobs; jobs delegate nothing (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_is_refused_unless_a_procedure_makes_it() {
    for creator in [Party::Person(9), Party::Deployment { project: 1 }, Party::Task(1), Party::Task(2)] {
        let mut h = procedure();
        assert_eq!(admission(&mut h, Party::Task(1), job(2)), None);
        crate::domain::task_mut(&mut h.domain, 1).unwrap().record.executor = crate::Executor::Agent { charter: 1 };
        assert_eq!(admission(&mut h, creator, job(3)), Some(crate::Refusal::Executor));
        assert!(h.domain.task(3).is_none());
    }
    let mut h = procedure();
    crate::domain::task_mut(&mut h.domain, 1).unwrap().record.executor =
        crate::Executor::Person(crate::PersonAddress::Person(9));
    assert_eq!(admission(&mut h, Party::Task(1), job(2)), Some(crate::Refusal::Executor));
    let mut h = procedure();
    assert_eq!(admission(&mut h, Party::Task(1), job(2)), None);
    assert_eq!(admission(&mut h, Party::Task(2), task(3, &[])), Some(crate::Refusal::Executor));
}

/// A job spec is only nonzero-deadline bounded bytes (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_without_its_spec_or_with_words_is_refused() {
    for fault in 0_u8..7 {
        let mut h = procedure();
        let mut member = job(2);
        match fault {
            0 => member.spec.job = None,
            1 => member.spec.words = Box::new([1]),
            2 => member.spec.inputs = Box::new([1]),
            3 => member.spec.asks = true,
            4 => member.spec.job.as_mut().unwrap().deadline = skein_lib::Duration::ZERO,
            5 => member.spec.job.as_mut().unwrap().bytes = Box::new([0; 65]),
            6 => member.spec.parameters = Box::new([crate::Parameter::Number { name: 1, value: 1 }]),
            _ => unreachable!(),
        }
        assert_eq!(admission(&mut h, Party::Task(1), member), Some(crate::Refusal::Spec));
        assert!(h.domain.task(2).is_none());
    }
}

/// Job contracts belong only to jobs within the bound (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_contract_must_be_a_job_and_within_the_result_bound() {
    for fault in 0_u8..3 {
        let mut h = procedure();
        let mut member = job(2);
        match fault {
            0 => member.contract = crate::Contract::Report { words: 16 },
            1 => {
                member.executor = crate::Executor::Agent { charter: 1 };
                member.spec.job = None;
            }
            2 => member.contract = crate::Contract::Job { bytes: 33 },
            _ => unreachable!(),
        }
        assert_eq!(admission(&mut h, Party::Task(1), member), Some(crate::Refusal::Contract));
    }
}

/// A job is result-ended and never a conversation (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_is_never_a_conversation() {
    let mut h = procedure();
    let mut member = job(2);
    member.flags.conversation = true;
    assert_eq!(admission(&mut h, Party::Task(1), member), Some(crate::Refusal::Contract));
    let mut member = job(2);
    member.flags.ended_by = crate::EndedBy::Requester;
    assert_eq!(admission(&mut h, Party::Task(1), member), Some(crate::Refusal::Contract));
}

/// A spec's job bytes belong only to jobs (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn other_executors_refuse_a_job_spec() {
    for executor in [
        crate::Executor::Agent { charter: 1 },
        crate::Executor::Procedure { connector: 1, code: 1 },
        crate::Executor::Person(crate::PersonAddress::Person(1)),
    ] {
        let mut h = procedure();
        let mut member = job(2);
        member.executor = executor;
        assert_eq!(admission(&mut h, Party::Task(1), member), Some(crate::Refusal::Executor));
    }
}

/// Restoration checks job shape and its linked procedure (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn restoring_a_job_checks_its_spec_contract_and_creator() {
    let mut h = procedure();
    assert_eq!(admission(&mut h, Party::Task(1), job(2)), None);
    let row = h.domain.task(2).unwrap().clone();
    let mut cold = Harness::new();
    cold.domain = crate::Domain::new(&h.env.limits, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));
    cold.send(Event::Restore { record: crate::Stored::Live(Box::new(row.clone())) });
    assert!(cold.domain.restore_failure().is_none());
    for fault in 0_u8..7 {
        let mut bad = row.clone();
        match fault {
            0 => bad.spec.job = None,
            1 => bad.spec.words = Box::new([1]),
            2 => bad.contract = crate::Contract::Report { words: 16 },
            3 => bad.spec.job.as_mut().unwrap().deadline = skein_lib::Duration::ZERO,
            4 => bad.flags.conversation = true,
            5 => bad.executor = crate::Executor::Agent { charter: 1 },
            6 => bad.contract = crate::Contract::Job { bytes: 33 },
            _ => unreachable!(),
        }
        cold.domain =
            crate::Domain::new(&h.env.limits, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));
        cold.send(Event::Restore { record: crate::Stored::Live(Box::new(bad)) });
        assert!(cold.domain.restore_failure().is_some());
    }
    let mut cold =
        crate::Domain::new(&h.env.limits, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));
    let mut out = skein_lib::Queue::uncapped();
    for stored in h.stored.values() {
        if !stored.key().live() {
            continue;
        }
        let mut stored = stored.clone();
        if let crate::Stored::Live(row) = &mut stored
            && row.number == 1
        {
            row.executor = crate::Executor::Agent { charter: 1 };
        }
        crate::step(&mut cold, &h.env, Event::Restore { record: stored }, &mut out);
    }
    crate::step(&mut cold, &h.env, Event::Check, &mut out);
    assert!(cold.restore_failure().is_some());
}

fn assert_refusal(out: &[Request], why: crate::Refusal) {
    let mut refused = false;
    for request in out {
        if let Request::Refused { problem, .. } = request {
            refused |= problem.why == why;
        }
    }
    assert!(refused, "{out:?}");
}

fn running_job() -> Harness {
    let mut h = procedure();
    assert_eq!(admission(&mut h, Party::Task(1), job(2)), None);
    assert!(h.domain.run_context(2).is_some());
    h.send(Event::Prepare { reply_to: super::revisions::to(), task: 2 });
    assert_eq!(h.domain.task(2).unwrap().phase, crate::Phase::Active(crate::Active::Preparing));
    h.send(Event::Claim {
        session_end: None,
        offered: crate::Reasons::ALL,
        reasons: crate::Reasons::FIRST,
        reply_to: super::revisions::to(),
        task: 2,
        attempt: 1,
        resumed: false,
        budget: 0,
        account: None,
        writes: Box::new([]),
    });
    h.send(Event::Started { task: 2, attempt: 1 });
    assert_eq!(h.domain.task(2).unwrap().phase, crate::Phase::Active(crate::Active::Running { attempt: 1 }));
    h
}

/// Forbidden job terminals leave even a priced attempt live (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_refuses_parks_followups_and_exceeded_without_changing_its_attempt() {
    let mut h = running_job();
    let before = h.domain.task(2).unwrap().clone();
    for end in [
        crate::End::Parked,
        crate::End::Exceeded(crate::Exceeded { bound: crate::Bound::Items, value: 1 }),
        crate::End::FinishedWithFollowups {
            result: crate::TaskResult::Job { bytes: Box::new([]) },
            followups: crate::ResultFollowups::Delegates(Box::new([])),
        },
    ] {
        for cause in [crate::Cause::Unpriced, crate::Cause::Priced { cumulative: 0 }] {
            let out = h.send(Event::Activation {
                reply_to: super::revisions::to(),
                task: 2,
                attempt: 1,
                end: end.clone(),
                saved: None,
                cause,
            });
            assert_refusal(&out, crate::Refusal::Executor);
            assert_eq!(h.domain.task(2), Some(&before));
        }
    }
}

/// A job claim reserves zero and has no LLM account (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_claim_refuses_an_account_or_a_nonzero_budget() {
    let mut h = procedure();
    assert_eq!(admission(&mut h, Party::Task(1), job(2)), None);
    h.send(Event::Prepare { reply_to: super::revisions::to(), task: 2 });
    let before = h.domain.task(2).unwrap().clone();
    for (budget, account, why) in [(1, None, crate::Refusal::Funding), (0, Some(1), crate::Refusal::Executor)] {
        let out = h.send(Event::Claim {
            session_end: None,
            offered: crate::Reasons::ALL,
            reasons: crate::Reasons::FIRST,
            reply_to: super::revisions::to(),
            task: 2,
            attempt: 1,
            resumed: false,
            budget,
            account,
            writes: Box::new([]),
        });
        assert_refusal(&out, why);
        assert_eq!(h.domain.task(2), Some(&before));
    }
}

/// A Job contract checks result form and bytes (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_result_is_bounded_bytes_or_a_failure() {
    let contract = crate::Contract::Job { bytes: 16 };
    let limits = super::revisions::LIMITS;
    assert_eq!(
        crate::run::invalid_result(&contract, &crate::TaskResult::Job { bytes: Box::new([0xff; 16]) }, &limits),
        None
    );
    assert_eq!(
        crate::run::invalid_result(&contract, &crate::TaskResult::Job { bytes: Box::new([0xff; 17]) }, &limits),
        Some(crate::InvalidResult::Words)
    );
    for result in
        [crate::TaskResult::Report { words: Box::new([]) }, crate::TaskResult::NoChange { words: Box::new([]) }]
    {
        assert_eq!(crate::run::invalid_result(&contract, &result, &limits), Some(crate::InvalidResult::Form));
    }
    assert_eq!(
        crate::run::invalid_result(&contract, &crate::TaskResult::Failure { reason: Box::new([1]) }, &limits),
        None
    );
    assert_eq!(
        crate::run::invalid_result(
            &crate::Contract::Report { words: 16 },
            &crate::TaskResult::Job { bytes: Box::new([]) },
            &limits
        ),
        Some(crate::InvalidResult::Form)
    );
}

/// A job refusal pauses without consuming a try (jig/docs/design/domain/tasks/lifecycle.md, 3.1 and 6).
#[test]
fn a_job_refusal_pauses_without_a_try() {
    let mut h = running_job();
    h.send(Event::Activation {
        reply_to: super::revisions::to(),
        task: 2,
        attempt: 1,
        end: crate::End::Refused,
        saved: None,
        cause: crate::Cause::Unpriced,
    });
    let row = h.domain.task(2).unwrap();
    let crate::Phase::Active(crate::Active::BackingOff { .. }) = row.phase else { panic!("job backs off") };
    assert_eq!(row.tries, crate::Tries::NONE);
    assert_eq!(row.refusals, 1);
    assert_eq!(row.last_answer, Some(1));
}

/// Jobs share procedures' LLM and recurring refusals (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[test]
fn a_job_refuses_an_llm_choice_and_recurrence() {
    let mut h = procedure();
    let mut member = job(2);
    member.llm = Some(crate::LlmChoice {
        later: None,
        account: None,
        main: crate::ModelEffort { model: 0, effort: None },
        subagents: Box::new([]),
    });
    assert_eq!(
        admission(&mut h, Party::Task(1), member),
        Some(crate::Refusal::Llm { model: 0, why: crate::LlmFault::Procedure })
    );
    let mut member = job(2);
    member.recurring = Some(Box::new(crate::RecurringTemplate {
        key: 1,
        batch: Box::new([]),
        overlap: crate::RecurringOverlap::Skip,
    }));
    assert_eq!(admission(&mut h, Party::Task(1), member), Some(crate::Refusal::Spec));
}

/// jig/docs/design/domain/tasks/lifecycle.md, 6: a job's cause survives retry exhaustion and storage.
#[test]
fn a_jobs_failure_reason_reaches_its_retry_notice_and_hold() {
    let mut h = running_job();
    let reason: Box<[u8]> = b"push: Unreachable: early EOF".as_slice().into();
    for attempt in 1..=3 {
        let row = &mut crate::domain::task_mut(&mut h.domain, 2).unwrap().record;
        row.attempt = attempt;
        row.phase = crate::Phase::Active(crate::Active::Claimed { attempt });
        h.send(Event::Activation {
            reply_to: super::revisions::to(),
            task: 2,
            attempt,
            cause: crate::Cause::Unpriced,
            saved: None,
            end: crate::End::FailedWithReason { class: crate::Class::Transient, reason: reason.clone() },
        });
        let row = h.domain.task(2).unwrap();
        assert_eq!(
            row.retry_notice,
            Some(crate::RetryNotice::Failed {
                class: crate::Class::Transient,
                count: u32::try_from(attempt).unwrap(),
                allowance: 2,
                reason: Some(reason.clone()),
            })
        );
    }
    let crate::Phase::Held { why, source: Some(source), .. } = &h.domain.task(2).unwrap().phase else {
        panic!("exhausted job has a hold source");
    };
    assert_eq!(*why, crate::Hold::Failures(crate::Class::Transient));
    assert_eq!(**source, crate::HoldSource::Preparation { words: reason.clone() });
    let held = h.domain.task(2).unwrap().phase.clone();
    h.restart();
    assert_eq!(h.domain.task(2).unwrap().phase, held);
}

/// A shutdown interruption releases a job without backoff or a try (lifecycle.md, 2).
#[test]
fn a_job_recheck_releases_its_attempt_due_without_spending_a_try() {
    for cause in [crate::Cause::Unpriced, crate::Cause::Priced { cumulative: 0 }] {
        let mut h = running_job();
        let tries = crate::Tries { lost: 1, ..crate::Tries::NONE };
        crate::domain::task_mut(&mut h.domain, 2).unwrap().record.tries = tries;
        h.send(Event::Activation {
            reply_to: super::revisions::to(),
            task: 2,
            attempt: 1,
            end: crate::End::Recheck,
            saved: None,
            cause,
        });
        let row = h.domain.task(2).unwrap();
        assert_eq!(row.phase, crate::Phase::Active(crate::Active::Due));
        assert_eq!(row.tries, tries);
        assert_eq!(row.last_answer, Some(1));
        assert_eq!(row.run_reserved, 0);
        assert_eq!(row.refusals, 0);
    }
}

/// Infrastructure owns its allowance, delay and reason (domain/tasks/lifecycle.md, 6).
#[test]
fn infrastructure_backs_off_twelve_times_then_holds_with_its_reason() {
    let mut h = running_job();
    h.env.limits.retries.infrastructure = crate::Retry {
        retries: 12,
        base: skein_lib::Duration::from_secs(10),
        max: skein_lib::Duration::from_secs(300),
    };
    let reason: Box<[u8]> = b"fetch: TimedOut".as_slice().into();
    for (index, ceiling) in [10, 20, 40, 80, 160, 300, 300, 300, 300, 300, 300, 300, 0].into_iter().enumerate() {
        let attempt = u64::try_from(index).unwrap() + 1;
        let row = &mut crate::domain::task_mut(&mut h.domain, 2).unwrap().record;
        row.attempt = attempt;
        row.phase = crate::Phase::Active(crate::Active::Claimed { attempt });
        h.send(Event::Activation {
            reply_to: super::revisions::to(),
            task: 2,
            attempt,
            cause: crate::Cause::Unpriced,
            saved: None,
            end: crate::End::FailedWithReason { class: crate::Class::Infrastructure, reason: reason.clone() },
        });
        let row = h.domain.task(2).unwrap();
        assert_eq!(row.tries.infrastructure, u32::try_from(attempt).unwrap());
        assert_eq!(row.tries.transient, 0);
        if ceiling > 0 {
            let crate::Phase::Active(crate::Active::BackingOff { until }) = row.phase else {
                panic!("infrastructure retries");
            };
            let delay = until.as_nanos() - h.env.wall.as_nanos();
            let ceiling = skein_lib::Duration::from_secs(ceiling).as_nanos();
            assert!((ceiling / 2..=ceiling).contains(&delay));
        } else {
            assert_eq!(
                row.phase,
                crate::Phase::Held {
                    was: crate::Was::Active(crate::Active::Due),
                    why: crate::Hold::Failures(crate::Class::Infrastructure),
                    source: Some(Box::new(crate::HoldSource::Preparation { words: reason.clone() })),
                }
            );
        }
    }
    let held = h.domain.task(2).unwrap().clone();
    h.restart();
    assert_eq!(h.domain.task(2).unwrap(), &held);
}
