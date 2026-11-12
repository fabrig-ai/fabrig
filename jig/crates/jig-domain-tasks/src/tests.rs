use crate::{Retry, failures::backoff};
use skein_lib::{Duration, Rng};

const KEPT: crate::Kept = crate::Kept { revisions: 4, versions: 8 };

#[test]
fn verdict_field_bytes_refuse_more_than_the_schema_count() {
    let mut fields = skein_lib::List::with_capacity(crate::VERDICT_FIELDS + 1);
    for name in 0..=crate::VERDICT_FIELDS {
        fields
            .push(crate::ResultField { name: Box::new(name.to_be_bytes()), value: Box::new([]) })
            .expect("one past the schema field count");
    }
    assert_eq!(crate::verdict::fields_bytes(fields.as_slice()), None);
}

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
    Box::new([crate::CharterModels { resumes: false, number: 1, revision: 1, models: Some(Box::new([0])) }])
}

mod batch_bytes;
mod decision_reads;
mod hold_origin;
mod kept;
mod keys;
mod refusals;
pub(crate) mod revisions;
mod waiting;

mod close_project;
mod periods;
mod sessions;

mod last_batch;

mod changed;
mod control;

mod address;
mod instructions;

mod deadlines;

mod index;

mod payer;

mod account;

mod room;
mod waits;

mod replaced;
mod settings_proposals;

mod startup;

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    let limits = revisions::LIMITS;
    for (name, limits, kept, accepted) in [
        ("fixture", limits, KEPT, true),
        ("placed_holds zero", crate::Limits { placed_holds: 0, ..limits }, KEPT, false),
        ("project_placed_holds zero", crate::Limits { project_placed_holds: 0, ..limits }, KEPT, false),
        ("party_placed_holds zero", crate::Limits { party_placed_holds: 0, ..limits }, KEPT, false),
        ("exemptions zero", crate::Limits { exemptions: 0, ..limits }, KEPT, false),
        ("circle zero", crate::Limits { circle: 0, ..limits }, KEPT, false),
        ("tasks zero", crate::Limits { tasks: 0, ..limits }, KEPT, false),
        ("batch zero", crate::Limits { batch: 0, ..limits }, KEPT, false),
        ("batch_bytes zero", crate::Limits { batch_bytes: 0, ..limits }, KEPT, false),
        ("tree_tasks zero", crate::Limits { tree_tasks: 0, ..limits }, KEPT, false),
        ("run_inbox zero", crate::Limits { run_inbox: 0, ..limits }, KEPT, false),
        ("inbox_messages zero", crate::Limits { inbox_messages: 0, ..limits }, KEPT, false),
        ("inbox_bytes zero", crate::Limits { inbox_bytes: 0, ..limits }, KEPT, false),
        ("message_bytes zero", crate::Limits { message_bytes: 0, ..limits }, KEPT, false),
        ("exemption range", crate::Limits { exemptions: limits.tasks + 1, ..limits }, KEPT, false),
        ("run inbox range", crate::Limits { run_inbox: limits.inbox_messages + 1, ..limits }, KEPT, false),
        ("message range", crate::Limits { message_bytes: limits.inbox_bytes + 1, ..limits }, KEPT, false),
        ("grace", crate::Limits { grace: Duration::ZERO, ..limits }, KEPT, false),
        ("hold_wait", crate::Limits { hold_wait: Duration::ZERO, ..limits }, KEPT, false),
        ("person_stall", crate::Limits { person_stall: Duration::ZERO, ..limits }, KEPT, false),
        ("proposal_stall", crate::Limits { proposal_stall: Duration::ZERO, ..limits }, KEPT, false),
        ("escalation_stall", crate::Limits { escalation_stall: Duration::ZERO, ..limits }, KEPT, false),
        ("batch names", crate::Limits { batch: u32::MAX, ..limits }, KEPT, false),
        ("timer place boundary", crate::Limits { subscriptions: u32::MAX / limits.tasks, ..limits }, KEPT, true),
        ("timer places", crate::Limits { subscriptions: u32::MAX, ..limits }, KEPT, false),
        ("revision holders", limits, crate::Kept { revisions: u32::MAX, ..KEPT }, false),
        // No retained dependency/input names means a large batch needs no stub multiplication.
        ("batch without names", crate::Limits { batch: u32::MAX, dependencies: 0, inputs: 0, ..limits }, KEPT, true),
        ("revision holder boundary", limits, crate::Kept { revisions: u32::MAX - 1, ..KEPT }, true),
        ("stub capacity", crate::Limits { dependencies: u32::MAX, ..limits }, KEPT, false),
        ("wait resources", crate::Limits { tasks: 65_536, holdings: 1, subscriptions: 0, ..limits }, KEPT, false),
        ("charter doubling", crate::Limits { charters: u32::MAX, ..limits }, KEPT, false),
        (
            "charter addition",
            crate::Limits { charters: 1, ..limits },
            crate::Kept { versions: u32::MAX, ..KEPT },
            false,
        ),
    ] {
        assert_eq!(crate::valid(&limits, kept), accepted, "{name}");
    }
}

#[test]
fn revision_holders_are_sized_by_the_core_retention() {
    // Four retained revisions plus the newest (domain/engine/settings.md, 8).
    const HOLDERS: u32 = 5;
    let domain = crate::Domain::new(&revisions::LIMITS, KEPT, 91, 1, charters(), Box::new([0]));
    assert_eq!(domain.holders.capacity(), HOLDERS);
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
        crate::Class::Infrastructure,
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
                crate::Class::Infrastructure => limits.retries.infrastructure = retry,
            }
            assert!(!crate::valid(&limits, KEPT), "{class:?}: {retry:?}");
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

/// Job allocations count toward admission bytes (jig/docs/design/domain/tasks/README.md, 3, and jig/docs/design/domain/tasks/lifecycle.md, 3.1).
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

mod classifiers {
    use super::revisions::{Harness, task};
    use crate::{
        Hint, Hold, MessageFamily, MessageKind, NewsClass, NoticeState, Party, Phase, ResultKind, Separate, Slot,
        Spoken, Stored, Word,
    };
    use skein_lib::{Queue, Wall};

    const TASK: u64 = 1;
    const SOURCE: u64 = 2;
    const MESSAGE: u64 = 11;
    const SUBSCRIPTION: u64 = 7;
    const HOLD: u64 = 13;
    const QUESTION: u64 = 17;

    fn message_word(kind: MessageKind) -> Word {
        Word {
            number: MESSAGE,
            from: Party::Task(SOURCE),
            kind,
            words: Box::new([1]),
            at: Wall::EPOCH,
            hits: 1,
            eligible: false,
        }
    }

    #[test]
    #[expect(clippy::too_many_lines, reason = "one table pins every message family and its caller behaviour")]
    fn message_families_keep_their_inbox_and_activity_behaviour() {
        use crate::SettingsOutcome::Rejected;
        use Hint as H;
        use MessageFamily as F;
        use MessageKind as M;
        use NewsClass::{Dropped, Kept, Wakes};
        use NoticeState::{Cancelled, Done, Failed};
        let mut h = Harness::new();
        h.make([task(TASK, &[])].into());
        let base = h.domain.task(TASK).expect("fixture task").clone();
        let subscription = SUBSCRIPTION;
        let target = SOURCE;
        let held = NoticeState::Held { revision: 3 };
        let notice = |state| F::Hint(H::Notice { subscription, target, state });
        for (kind, family, category) in [
            (M::Escalation { task: SOURCE, revision: 3 }, F::Virtual, 7),
            (M::Proposal { proposer: SOURCE, proposal: 5, kind: crate::ProposalKind::Batch }, F::Virtual, 7),
            (M::PersonStalled { task: SOURCE }, F::Slot(Slot::PersonStalled), 6),
            (M::ProposalDecision { proposal: 5, accepted: true }, F::Slot(Slot::ProposalDecision), 7),
            (M::SettingsDecision { proposal: 5, outcome: Rejected }, F::Slot(Slot::SettingsDecision), 7),
            (M::Words, F::Spoken(Spoken::Words), 0),
            (M::EffectNotMade { entry: 3 }, F::Slot(Slot::EffectNotMade), 7),
            (M::Amendment { revision: 3 }, F::Separate(Separate::Amendment), 7),
            (M::FinishRequested { revision: 3 }, F::Separate(Separate::FinishRequested), 7),
            (M::Question, F::Spoken(Spoken::Question), 1),
            (M::Answer { question: QUESTION }, F::Spoken(Spoken::Answer { question: QUESTION }), 2),
            (M::Notice { subscription, target, state: held }, notice(held), 4),
            (M::Notice { subscription, target, state: Done }, notice(Done), 4),
            (M::Notice { subscription, target, state: Failed }, notice(Failed), 4),
            (M::Notice { subscription, target, state: Cancelled }, notice(Cancelled), 4),
            (M::Timer { subscription }, F::Hint(H::Timer { subscription }), 5),
            (M::News { subscription, class: Wakes }, F::Hint(H::News { subscription, class: Wakes }), 6),
            (M::News { subscription, class: Kept }, F::Hint(H::News { subscription, class: Kept }), 6),
            (M::News { subscription, class: Dropped }, F::Hint(H::News { subscription, class: Dropped }), 6),
            (M::Result(ResultKind::Report), F::Result, 3),
            (M::Met { dependency: SOURCE, removed: true }, F::Separate(Separate::Met), 7),
            (M::QuestionEnded { question: QUESTION }, F::Slot(Slot::QuestionEnded), 7),
            (M::QuestionAnswered { question: QUESTION }, F::Slot(Slot::QuestionAnswered), 7),
            (M::QuestionOwed, F::Slot(Slot::QuestionOwed), 7),
            (M::NothingAwaited, F::Separate(Separate::NothingAwaited), 7),
        ] {
            #[expect(clippy::match_same_arms, reason = "one arm per family keeps its caller behaviour explicit")]
            let (spoken, hint, separate, finish) = match family {
                F::Spoken(_) => (true, false, false, false),
                F::Hint(_) => (false, true, false, false),
                F::Slot(_) => (false, false, false, false),
                F::Separate(slot) => (false, false, true, slot == Separate::FinishRequested),
                F::Result => (false, false, false, false),
                F::Virtual => (false, false, false, false),
            };
            assert_eq!(kind.family(), family, "{kind:?}: family");
            assert_eq!(crate::wake::category(&kind), category, "{kind:?}: wake category");
            assert_eq!(kind.result().is_some(), family == MessageFamily::Result, "{kind:?}: result");
            let word = message_word(kind);
            let mut row = base.clone();
            let mut out = Queue::uncapped();
            crate::activity::message(&mut row, &word, h.env.limits.inbox_messages, &mut out);
            let expected_line = if spoken {
                crate::Line::Words {
                    message: MESSAGE,
                    by: word.from,
                    kind: word.kind.clone(),
                    words: word.words.clone(),
                }
            } else {
                crate::Line::Heard(word.clone())
            };
            assert_eq!(
                out.pop(),
                Some(crate::Request::Save {
                    record: Stored::Activity(crate::ActivityLine {
                        task: TASK,
                        position: base.activity + 1,
                        at: word.at,
                        line: expected_line,
                    })
                }),
                "{:?}: activity",
                word.kind
            );
            assert_eq!(word.finish_requested(), finish, "{:?}: finish", word.kind);

            row.inbox = Box::new([word.clone()]);
            row.subscriptions = Box::new([crate::Subscription {
                number: SUBSCRIPTION,
                kind: crate::SubscriptionKind::Topic { connector: 1, topic: 1 },
            }]);
            let credits = match crate::subscriptions::credit(&row, &h.env.limits) {
                Some((count, _)) => Some(count),
                None => None,
            };
            assert_eq!(credits, Some(usize::from(!hint)), "{:?}: hint credit", word.kind);
            row.subscriptions = Box::new([]);
            crate::domain::task_mut(&mut h.domain, TASK).expect("fixture task").record = row;
            let limits = crate::Limits { inbox_messages: 0, ..h.env.limits };
            assert_eq!(crate::inbox::room(&h.domain, &limits, TASK, 0, 0), separate, "{:?}: separate slot", word.kind);

            let batch = message_word(M::News { subscription: SUBSCRIPTION + 1, class: Wakes });
            let row = &mut crate::domain::task_mut(&mut h.domain, TASK).expect("fixture task").record;
            row.inbox = Box::new([word.clone(), batch.clone()]);
            // Two single-hit messages reach the threshold only in the same category.
            row.wake.news = crate::WakeRule::Batch { count: 2, age: skein_lib::Duration::from_secs(1) };
            crate::wake::after_message(&mut h.domain, &h.env, TASK, None, batch, &mut out);
            assert_eq!(
                h.domain.task(TASK).expect("fixture task").inbox.first().expect("first word").eligible,
                category == 6,
                "{:?}: news batch",
                word.kind
            );
        }
    }

    #[test]
    fn result_shapes_keep_payload_and_owned_byte_accounting() {
        let fields = Box::new([crate::ResultField { name: Box::new([1]), value: Box::new([2]) }]);
        let field_storage = u64::try_from(size_of::<crate::ResultField>()).expect("field size fits");
        for (result, payload, owned) in [
            (None, Some(1), Some(1)),
            (Some(ResultKind::Job), Some(1), Some(1)),
            (Some(ResultKind::Approved { state: Box::new([2, 3, 4]), words_present: true }), Some(4), Some(4)),
            (Some(ResultKind::Approved { state: Box::new([]), words_present: false }), None, None),
            (Some(ResultKind::NoChange), Some(1), Some(1)),
            (Some(ResultKind::Report), Some(1), Some(1)),
            (Some(ResultKind::Verdict { code: 1, fields }), Some(3), Some(3 + field_storage)),
            (Some(ResultKind::Change { connector: 1, kind: 2, resource: 3 }), Some(1), Some(1)),
            (Some(ResultKind::Failed), Some(1), Some(1)),
            (Some(ResultKind::Cancelled), Some(1), Some(1)),
        ] {
            let kind = match &result {
                Some(result) => MessageKind::Result(result.clone()),
                None => MessageKind::Words,
            };
            let word = message_word(kind);
            assert_eq!(word.kind.result(), result.as_ref(), "{:?}: result", word.kind);
            assert_eq!(word.payload_bytes(), payload, "{:?}: payload", word.kind);
            let record = Stored::Activity(crate::ActivityLine {
                task: TASK,
                position: 1,
                at: word.at,
                line: crate::Line::Heard(word.clone()),
            });
            assert_eq!(crate::stored_bytes(&record), owned, "{:?}: owned", word.kind);
        }
    }

    #[test]
    fn placed_holds_keep_their_identity() {
        for (why, placed) in [
            (Hold::Exceeded(crate::Exceeded { bound: crate::Bound::Answer, value: 1 }), None),
            (Hold::Failures(crate::Class::Run), None),
            (Hold::Stalled, None),
            (Hold::Dependency(SOURCE), None),
            (Hold::Stopped, None),
            (Hold::Placed { hold: HOLD }, Some(HOLD)),
            (Hold::Drift, None),
            (Hold::Effects, None),
            (Hold::EffectFailed, None),
            (Hold::Uncertain { entry: 3 }, None),
            (Hold::Budget, None),
            (Hold::Deadline, None),
            (Hold::HoldsWaited, None),
            (Hold::ResourceUnavailable, None),
            (Hold::ResourceAccess, None),
            (Hold::PoolLost { pool: 3 }, None),
            (Hold::Procedure, None),
            (Hold::Circle, None),
        ] {
            assert_eq!(why.placed(), placed, "{why:?}: placed identity");
            let phase = Phase::Held { was: crate::Was::Waiting, why, source: None };
            assert_eq!(crate::placed::placed_reason(&phase), placed, "{why:?}: identity");
            assert_eq!(crate::placed::is_placed(why), placed.is_some(), "{why:?}: placed");
        }
    }
}

mod closer_funds;
mod offering;
mod reasons;
