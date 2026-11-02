use crate::{Retry, failures::backoff};
use skein_lib::{Duration, Rng};

#[test]
fn exponential_equal_jitter_saturates_without_overflow_and_replays() {
    let retry = Retry { retries: 4, base: Duration::from_millis(10), max: Duration::from_secs(1) };
    let mut first = Rng::new(42);
    let mut second = Rng::new(42);
    for times in [1, 2, 3, 4, 7, 32, 64, u32::MAX] {
        let factor = 1_u64.checked_shl(times.saturating_sub(1)).unwrap_or(u64::MAX);
        let ceiling = retry.base.saturating_mul(factor).min(retry.max);
        let pause = backoff(times, retry, &mut first);
        assert!(
            pause.as_nanos() >= ceiling.as_nanos().div_euclid(2) && pause <= ceiling,
            "pause stays in equal-jitter half-to-ceiling range"
        );
        assert_eq!(pause, backoff(times, retry, &mut second), "same seed replays backoff");
    }
}

/// One configured charter, number 1, with the one configured model, which declares no effort.
pub(crate) fn charters() -> Box<[crate::CharterModels]> {
    Box::new([crate::CharterModels { number: 1, revision: 1, models: Some(Box::new([0])) }])
}

mod decision_reads;
mod hold_origin;
mod kept;
mod keys;
mod refusals;
mod revisions;
mod waiting;

mod close_project;
mod periods;

mod last_batch;

mod changed;
mod control;

mod address;
mod instructions;

mod deadlines;

mod index;

mod payer;

mod account;

mod waits;

mod settings_proposals;

mod startup;

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    let limits = revisions::LIMITS;
    for (name, limits, accepted) in [
        ("fixture", limits, true),
        ("placed_holds zero", crate::Limits { placed_holds: 0, ..limits }, false),
        ("project_placed_holds zero", crate::Limits { project_placed_holds: 0, ..limits }, false),
        ("party_placed_holds zero", crate::Limits { party_placed_holds: 0, ..limits }, false),
        ("exemptions zero", crate::Limits { exemptions: 0, ..limits }, false),
        ("circle zero", crate::Limits { circle: 0, ..limits }, false),
        ("tasks zero", crate::Limits { tasks: 0, ..limits }, false),
        ("batch zero", crate::Limits { batch: 0, ..limits }, false),
        ("tree_tasks zero", crate::Limits { tree_tasks: 0, ..limits }, false),
        ("run_inbox zero", crate::Limits { run_inbox: 0, ..limits }, false),
        ("inbox_messages zero", crate::Limits { inbox_messages: 0, ..limits }, false),
        ("inbox_bytes zero", crate::Limits { inbox_bytes: 0, ..limits }, false),
        ("message_bytes zero", crate::Limits { message_bytes: 0, ..limits }, false),
        ("exemption range", crate::Limits { exemptions: limits.tasks + 1, ..limits }, false),
        ("run inbox range", crate::Limits { run_inbox: limits.inbox_messages + 1, ..limits }, false),
        ("message range", crate::Limits { message_bytes: limits.inbox_bytes + 1, ..limits }, false),
        ("grace", crate::Limits { grace: Duration::ZERO, ..limits }, false),
        ("hold_wait", crate::Limits { hold_wait: Duration::ZERO, ..limits }, false),
        ("person_stall", crate::Limits { person_stall: Duration::ZERO, ..limits }, false),
        ("proposal_stall", crate::Limits { proposal_stall: Duration::ZERO, ..limits }, false),
        ("escalation_stall", crate::Limits { escalation_stall: Duration::ZERO, ..limits }, false),
        ("batch names", crate::Limits { batch: u32::MAX, ..limits }, false),
        ("timer place boundary", crate::Limits { subscriptions: u32::MAX / limits.tasks, ..limits }, true),
        ("timer places", crate::Limits { subscriptions: u32::MAX, ..limits }, false),
        ("revision holders", crate::Limits { revisions: u32::MAX, ..limits }, false),
        // No retained dependency/input names means a large batch needs no stub multiplication.
        ("batch without names", crate::Limits { batch: u32::MAX, dependencies: 0, inputs: 0, ..limits }, true),
        ("revision holder boundary", crate::Limits { revisions: u32::MAX - 1, ..limits }, true),
        ("stub capacity", crate::Limits { dependencies: u32::MAX, ..limits }, false),
        ("wait resources", crate::Limits { tasks: 65_536, holdings: 1, subscriptions: 0, ..limits }, false),
        ("charter doubling", crate::Limits { charters: u32::MAX, ..limits }, false),
        ("charter addition", crate::Limits { charters: 1, versions: u32::MAX, ..limits }, false),
    ] {
        assert_eq!(crate::valid(&limits), accepted, "{name}");
    }
}

#[test]
fn validity_checks_each_retry_class() {
    for class in [
        crate::Class::Transient,
        crate::Class::Permanent,
        crate::Class::Run,
        crate::Class::Agent,
        crate::Class::Lost,
        crate::Class::Invalid,
    ] {
        for retry in [
            Retry { retries: u32::MAX, base: Duration::from_nanos(1), max: Duration::from_nanos(2) },
            Retry { retries: 1, base: Duration::ZERO, max: Duration::from_nanos(2) },
            Retry { retries: 1, base: Duration::from_nanos(2), max: Duration::from_nanos(1) },
        ] {
            let mut limits = revisions::LIMITS;
            match class {
                crate::Class::Transient => limits.retries.transient = retry,
                crate::Class::Permanent => limits.retries.permanent = retry,
                crate::Class::Run => limits.retries.run = retry,
                crate::Class::Agent => limits.retries.agent = retry,
                crate::Class::Lost => limits.retries.lost = retry,
                crate::Class::Invalid => limits.retries.invalid = retry,
            }
            assert!(!crate::valid(&limits), "{class:?}: {retry:?}");
        }
    }
}

mod wait_validity {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let limits = super::revisions::LIMITS;
        for (name, limits, accepted) in [
            ("fixture", limits, true),
            ("squared tasks", crate::Limits { tasks: 65_536, holdings: 0, ..limits }, false),
            ("resource product", crate::Limits { tasks: 256, holdings: 65_536, ..limits }, false),
            ("dependencies", crate::Limits { tasks: 2, dependencies: u32::MAX, ..limits }, false),
        ] {
            assert_eq!(crate::waits::valid(&limits), accepted, "{name}");
        }
    }
}

#[test]
fn procedure_inputs_select_news_and_results_without_text_inference() {
    use crate::{MessageKind, NewsClass, ProcedureInput, ResultKind};
    assert_eq!(
        MessageKind::News { subscription: 7, class: NewsClass::Wakes }.procedure_input(),
        Some(ProcedureInput::News)
    );
    assert_eq!(
        MessageKind::Result(ResultKind::Report).procedure_input(),
        Some(ProcedureInput::Result(ResultKind::Report))
    );
    for kind in [
        MessageKind::Words,
        MessageKind::Question,
        MessageKind::Timer { subscription: 7 },
        MessageKind::FinishRequested { revision: 1 },
    ] {
        assert_eq!(kind.procedure_input(), None);
    }
}

#[test]
fn change_titles_and_words_have_independent_contract_bounds() {
    let limits = revisions::LIMITS;
    let contract = crate::Contract::Change { connector: 1, kind: 2, words: 4 };
    for (title, words, accepted) in [(4, 4, true), (5, 4, false), (4, 5, false), (4, 0, true)] {
        let result = crate::TaskResult::Change {
            connector: 1,
            kind: 2,
            resource: 3,
            title: crate::ChangeTitle::new(skein_lib::bytes::zeroed(title)),
            words: skein_lib::bytes::zeroed(words),
        };
        assert_eq!(crate::run::valid_result(&contract, &result, &limits), accepted);
        let tight = crate::Limits { result_bytes: 3, ..limits };
        assert!(!crate::run::valid_result(&contract, &result, &tight));
    }
}

#[test]
fn no_change_requires_a_change_contract_and_both_word_bounds() {
    use crate::{Contract, TaskResult};
    let limits = revisions::LIMITS;
    let contract = Contract::Change { connector: 1, kind: 2, words: 4 };
    for length in [0, 4, 5] {
        let result = TaskResult::NoChange { words: skein_lib::bytes::zeroed(length) };
        assert_eq!(crate::run::valid_result(&contract, &result, &limits), length <= 4);
        assert_eq!(
            crate::run::valid_result(&contract, &result, &crate::Limits { result_bytes: 3, ..limits }),
            length <= 3
        );
        assert!(!crate::run::valid_result(&Contract::Report { words: 4 }, &result, &limits));
        assert!(!crate::run::valid_result(&Contract::Verdict { choices: Box::new([]) }, &result, &limits));
        assert!(!crate::run::valid_result(
            &Contract::Approval {
                state: Box::new([]),
                part: crate::Part { connector: 1, kind: crate::PartKind::Result, key: crate::PartKey::Number(1) }
            },
            &result,
            &limits
        ));
    }
}

/// Job allocations count toward admission bytes (jig/docs/design/domain/tasks.md, 3 and 5.3.1).
#[test]
fn job_spec_and_result_heap_bytes_include_their_allocations() {
    use super::{End, JobSpec, TaskResult, task_bytes, terminal_bytes};
    use revisions::{Harness, task};
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    let row = h.domain.task(1).unwrap();
    let before = task_bytes(row).unwrap();
    let mut row = row.clone();
    row.spec.job = Some(Box::new(JobSpec { bytes: Box::new([0xff; 7]), deadline: Duration::from_secs(1) }));
    assert_eq!(task_bytes(&row), Some(before + u64::try_from(size_of::<JobSpec>()).unwrap() + 7));
    let end = End::Finished { result: TaskResult::Job { bytes: Box::new([0xff; 11]) }, cancel_delegates: false };
    assert_eq!(terminal_bytes(&end), Some(11));
}
