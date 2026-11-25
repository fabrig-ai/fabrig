//! Complete durable tasks and all remaining row kinds (protocol/store/records.md, 1).
use super::{bound_problem, fixtures, saved_tasks, tasks_limits};
use crate::{codec::tasks, tasks_v13 as wire};
use jig_domain_tasks as domain;
use skein_lib::{Duration, Token, Wall};

const PROJECT_INDEX_TASK: u64 = 17;
const PROPOSAL_UPDATED_AT: Wall = Wall::from_nanos(41);

fn activity(line: domain::Line) -> domain::Stored {
    domain::Stored::Activity(domain::ActivityLine { task: 17, position: 29, at: Wall::from_nanos(31), line })
}
fn live_record(record: domain::TaskRecord) -> domain::Stored {
    domain::Stored::Live(Box::new(record))
}
fn message_row(kind: domain::MessageKind) -> domain::Stored {
    let mut word = fixtures::task_word(5);
    word.kind = kind;
    activity(domain::Line::Heard(word))
}
fn current_record() -> domain::TaskRecord {
    let mut record = fixtures::task_record();
    record.proposal = None;
    record.recurring = None;
    record
}

fn a_task_records_label_flags_and_last_activity_read_back() {
    for (ended_by, conversation) in [(domain::EndedBy::Result, false), (domain::EndedBy::Requester, true)] {
        let mut record = current_record();
        record.label = 0x0102_0304;
        record.flags =
            domain::Flags { ended_by, conversation, listed: true, retention: domain::Retention::ForgottenOnceRead };
        record.last_activity = Wall::from_nanos(9_000_000_000);
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
// TaskInterests.subscriptions in tasks-v13.schema.
const SUBSCRIPTIONS_CEILING: u32 = 64;

fn record_with_subscriptions(count: u32) -> domain::Stored {
    let mut record = current_record();
    let mut subscriptions = skein_lib::List::with_capacity(count);
    for number in 1..=u64::from(count) {
        subscriptions
            .push(domain::Subscription {
                number,
                kind: domain::SubscriptionKind::Topic { connector: 1, topic: number },
            })
            .expect("fixture interest");
    }
    record.subscriptions = subscriptions.into_boxed();
    live_record(record)
}

#[test]
fn task_subscriptions_at_the_schema_ceiling_read_back() {
    let row = record_with_subscriptions(SUBSCRIPTIONS_CEILING);
    assert_eq!(saved_tasks(&row), Ok(row));
}

#[test]
fn task_subscriptions_past_the_schema_ceiling_are_refused() {
    let row = record_with_subscriptions(SUBSCRIPTIONS_CEILING + 1);
    let names = fixtures::configured_model_names(&[b"first", b"second", b"third"]);
    assert_eq!(
        tasks::encode(&row, &tasks_limits(), &names),
        Err(bound_problem(wire::paths::TASK_INTERESTS_SUBSCRIPTIONS))
    );
}

fn a_live_task_record_reads_back_whole() {
    for (reasons, offered) in [
        (domain::Reasons::QUESTION.union(domain::Reasons::TIMER), domain::Reasons::WORDS),
        (domain::Reasons::EMPTY, domain::Reasons::EMPTY),
    ] {
        let mut record = fixtures::task_record();
        record.reasons = reasons;
        record.offered = offered;
        record.llm.as_mut().expect("fixture choice").later = Some(domain::LlmLater {
            main: domain::ModelEffort { model: 7, effort: Some(1) },
            subagents: Box::new([domain::ModelEffort { model: 0, effort: None }]),
        });
        let row = domain::Stored::Live(Box::new(record));
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
/// Suspensions keep their cause, window and clock accounting in all task rows (executors.md, 6.2).
#[test]
fn suspended_ends_and_task_gaps_read_back() {
    for why in [domain::SuspendWhy::Stopped, domain::SuspendWhy::Restarted] {
        let end = domain::EndKind::Suspended { worker: 2, why };
        let mut record = current_record();
        record.suspension =
            Some(domain::Suspension { worker: 2, since: Wall::from_nanos(100), until: Wall::from_nanos(130) });
        record.suspended = 17;
        record.restarted = why == domain::SuspendWhy::Restarted;
        record.open_attempt = Some(domain::OpenAttempt {
            attempt: 3,
            claimed: Some(Wall::from_nanos(80)),
            started: Some(Wall::from_nanos(90)),
            from: 1,
            end: Some(end),
        });
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
        let row = activity(domain::Line::RunEnded { attempt: 3, end, spent: 7 });
        assert_eq!(saved_tasks(&row), Ok(row));
        let row = domain::Stored::Attempt(domain::AttemptRow {
            task: 17,
            attempt: 3,
            claimed: Some(Wall::from_nanos(80)),
            started: Some(Wall::from_nanos(90)),
            ended: Some(Wall::from_nanos(100)),
            end: Some(end),
            phases: Some(2),
        });
        assert_eq!(saved_tasks(&row), Ok(row));
        super::end_round_trip(domain::End::Suspended(domain::Suspended {
            worker: 2,
            why,
            window: Duration::from_secs(30),
        }));
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the layout plan is tested against the tasks family's schema text")]
fn task_parts_read_a_nonzero_legacy_question_watermark_without_keeping_it() {
    use skein_codec::{
        layout::{Mode, Plan},
        schema::parse,
    };

    const TEXT: &str = include_str!("../../../../jig-protocol-store/schema/tasks-v13.schema");
    const PREVIOUS: &str = include_str!("../../../../jig-protocol-store/src/tests/fixtures/dev-tasks-v13.schema");
    const DROPPED: &str = "dropped question_woken: u64 = 0";
    let current = parse(TEXT.as_bytes()).expect("current tasks schema");
    let legacy = parse(PREVIOUS.as_bytes()).expect("previous layout");
    let nonzero = parse(TEXT.replace(DROPPED, "dropped question_woken: u64 = 37").as_bytes())
        .expect("fixture writer supplies the previous field at a nonzero value");
    let names = fixtures::configured_model_names(&[b"first", b"second", b"third"]);
    let limits = tasks_limits();
    for (variant, row) in [
        (b"live_hot".as_slice(), live_record(current_record())),
        (b"ended_talk".as_slice(), domain::Stored::Ended(Box::new(current_record()))),
    ] {
        let current_bytes = tasks::encode(&row, &limits, &names).expect("current record fits");
        let old_root = legacy.root(b"Row", Some(variant)).expect("old variant");
        let new_root = current.root(b"Row", Some(variant)).expect("new variant");
        let back = Plan::between(Mode::Keep, &current, new_root, &[], &legacy, old_root, &[])
            .expect("the tombstone supplies a downgrade default");
        let write_nonzero = Plan::between(
            Mode::Keep,
            &nonzero,
            nonzero.root(b"Row", Some(variant)).expect("fixture variant"),
            &[],
            &legacy,
            old_root,
            &[],
        )
        .expect("encode an old value with the field present and nonzero");
        let zero_bytes = convert_question_layout(&back, &current_bytes);
        let legacy_bytes = convert_question_layout(&write_nonzero, &current_bytes);
        assert_ne!(legacy_bytes, zero_bytes, "the fixture really stores 37 rather than 0");
        assert_eq!(legacy_bytes.len(), zero_bytes.len());
        let upgrade = Plan::derive(&legacy, old_root, &[], &current, new_root, &[])
            .expect("the store can read the previous layout");
        assert_eq!(convert_question_layout(&upgrade, &legacy_bytes), current_bytes);
    }
}

fn convert_question_layout(plan: &skein_codec::layout::Plan, bytes: &[u8]) -> Box<[u8]> {
    const ROW_BYTES: u32 = 2048;
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(plan));
    let mut out = skein_lib::Buffer::with_capacity(ROW_BYTES);
    plan.convert(bytes, &mut out, &mut work).expect("convert layout");
    out.as_slice().into()
}

#[test]
fn a_task_record_with_started_from_dependencies_reads_back() {
    let mut record = current_record();
    record.dependencies = Box::new([19, 23]);
    record.waiting_on = Box::new([19]);
    record.started_from = Box::new([23, 19]);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn an_ended_task_with_its_result_reads_back() {
    let mut record = fixtures::task_record();
    record.settings = domain::SettingsRevision { task: 5, run: None };
    record.phase = domain::Phase::Ended(domain::Ending::Done(domain::TaskResult::Report {
        words: Box::from(b"result".as_slice()),
    }));
    let row = domain::Stored::Ended(Box::new(record));
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn an_activity_line_of_each_kind_reads_back() {
    for fresh in [
        None,
        Some(domain::Fresh::Moved),
        Some(domain::Fresh::Closed),
        Some(domain::Fresh::Cut),
        Some(domain::Fresh::Limit),
        Some(domain::Fresh::Session),
    ] {
        let row = activity(domain::Line::RunBegan { attempt: 3, why: domain::Why::Ready, resumed: false, fresh });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
    for line in [
        domain::Line::Words {
            message: 5,
            by: domain::Party::Person(31),
            kind: domain::MessageKind::Question,
            words: Box::from(b"question".as_slice()),
        },
        domain::Line::Heard(fixtures::task_word(5)),
        domain::Line::Notified { recipient: domain::Party::Task(19), word: fixtures::task_word(5) },
        domain::Line::RunBegan {
            fresh: None,
            attempt: 3,
            why: domain::Why::Retry { tries: fixtures::task_record().tries },
            resumed: true,
        },
        domain::Line::RunEnded {
            attempt: 3,
            end: domain::EndKind::Failed { class: domain::Class::Lost, invalid_result: None },
            spent: 7,
        },
        domain::Line::Turn { reply: 37, attempt: 3, turn: 5, read: Some(5) },
        domain::Line::Decided { revision: 3, proposal: Some(domain::DecidedProposal { number: 7, made: true }) },
        domain::Line::Milestone { position: 13 },
    ] {
        let row = activity(line);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_terminal_activity_kind_reads_back() {
    for end in
        [domain::EndKind::Parked, domain::EndKind::Finished, domain::EndKind::Cancelled, domain::EndKind::Refused]
    {
        let row = activity(domain::Line::RunEnded { attempt: 3, end, spent: 7 });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_index_row_reads_back() {
    for row in [
        domain::Stored::Update(domain::Update {
            party: 31,
            at: Wall::from_nanos(41),
            task: 17,
            object: domain::UpdateObject::Result,
        }),
        domain::Stored::Update(domain::Update {
            party: 31,
            at: Wall::from_nanos(41),
            task: 17,
            object: domain::UpdateObject::Reply { message: domain::Newest::of(37), activity: domain::Newest::of(29) },
        }),
        domain::Stored::Update(domain::Update {
            party: 31,
            at: PROPOSAL_UPDATED_AT,
            task: 17,
            object: domain::UpdateObject::Proposal { proposal: domain::Newest::of(43) },
        }),
        domain::Stored::Result(domain::ResultRow { party: 31, position: 43, task: 17 }),
        domain::Stored::EndedDelegate(domain::Edge { requester: 17, task: 19 }),
        domain::Stored::Delegate(domain::Edge { requester: 17, task: 19 }),
        domain::Stored::EndedByParty(domain::EndedByParty {
            party: 31,
            label: 3,
            ended_at: Wall::from_nanos(41),
            task: 17,
        }),
        domain::Stored::EndedByProject(domain::EndedByProject {
            project: 31,
            label: 3,
            ended_at: Wall::from_nanos(41),
            task: PROJECT_INDEX_TASK,
        }),
    ] {
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_stub_status_reads_back() {
    for phase in [domain::Status::Done, domain::Status::Failed, domain::Status::Cancelled] {
        let row = domain::Stored::Stub(domain::Stub { task: 17, phase, result: Token::new(19) });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_message_kind_reads_back() {
    for kind in [
        domain::MessageKind::Escalation { task: 17, revision: 3 },
        domain::MessageKind::Proposal { proposer: 17, proposal: 43, kind: domain::ProposalKind::Widen },
        domain::MessageKind::PersonStalled { task: 19 },
        domain::MessageKind::ProposalDecision { proposal: 43, accepted: false },
        domain::MessageKind::SettingsDecision {
            proposal: 43,
            outcome: domain::SettingsOutcome::Accepted { revision: 2 },
        },
        domain::MessageKind::SettingsDecision { proposal: 43, outcome: domain::SettingsOutcome::Rejected },
        domain::MessageKind::SettingsDecision { proposal: 43, outcome: domain::SettingsOutcome::Stale { newest: 3 } },
        domain::MessageKind::Words,
        domain::MessageKind::EffectNotMade { entry: 11 },
        domain::MessageKind::FinishRequested { revision: 1 },
        domain::MessageKind::Amendment { revision: 3 },
        domain::MessageKind::Question,
        domain::MessageKind::Answer { question: 5 },
        domain::MessageKind::Notice { subscription: 109, target: 19, state: domain::NoticeState::Held { revision: 1 } },
        domain::MessageKind::Timer { subscription: 109 },
        domain::MessageKind::News { subscription: 109, class: domain::NewsClass::Wakes },
        domain::MessageKind::Result(domain::ResultKind::Report),
    ] {
        let row = message_row(kind);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_result_message_kind_reads_back() {
    for kind in [
        domain::ResultKind::Approved { state: Box::from(b"state".as_slice()), words_present: true },
        domain::ResultKind::Report,
        domain::ResultKind::Verdict {
            code: 7,
            fields: Box::new([domain::ResultField {
                name: Box::from(b"field".as_slice()),
                value: Box::from(b"value".as_slice()),
            }]),
        },
        domain::ResultKind::Change { connector: 2, kind: 3, resource: 11 },
        domain::ResultKind::Failed,
        domain::ResultKind::Cancelled,
    ] {
        let row = message_row(domain::MessageKind::Result(kind));
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_notice_state_reads_back() {
    for state in [
        domain::NoticeState::Held { revision: 1 },
        domain::NoticeState::Done,
        domain::NoticeState::Failed,
        domain::NoticeState::Cancelled,
        domain::NoticeState::Unreadable,
    ] {
        let row = message_row(domain::MessageKind::Notice { subscription: 109, target: 19, state });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_news_class_reads_back() {
    for class in [domain::NewsClass::Wakes, domain::NewsClass::Kept, domain::NewsClass::Dropped] {
        let row = message_row(domain::MessageKind::News { subscription: 109, class });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_escalation_state_reads_back() {
    for escalation in [
        domain::Escalation::Unheld { revision: 3 },
        domain::Escalation::Routing { revision: 5 },
        domain::Escalation::Rejected {
            revision: 7,
            by: domain::EscalationDecider::Person(31),
            reason: Box::from(b"rejected".as_slice()),
        },
    ] {
        let mut record = current_record();
        record.escalation = escalation;
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_escalation_holder_reads_back() {
    for holder in [
        domain::EscalationHolder::Task(19),
        domain::EscalationHolder::Person(31),
        domain::EscalationHolder::Role { project: 3, role: 7 },
    ] {
        let mut record = current_record();
        record.escalation = domain::Escalation::Waiting { revision: 3, holder, entry: 43, since: Wall::from_nanos(47) };
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_invalid_result_reason_reads_back() {
    for reason in [
        domain::InvalidResult::Form,
        domain::InvalidResult::Verdict,
        domain::InvalidResult::Words,
        domain::InvalidResult::Change,
        domain::InvalidResult::Followups,
    ] {
        let mut record = current_record();
        record.invalid_result = Some(reason);
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn each_subscription_kind_reads_back() {
    for kind in [
        domain::SubscriptionKind::Task { target: 19, held: true, result: false },
        domain::SubscriptionKind::Timer { at: Wall::from_nanos(59), period: Some(Duration::from_nanos(3)) },
        domain::SubscriptionKind::Timer { at: Wall::from_nanos(59), period: None },
        domain::SubscriptionKind::Topic { connector: 2, topic: 113 },
    ] {
        let mut record = current_record();
        record.subscriptions = Box::new([domain::Subscription { number: 109, kind }]);
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
fn a_task_record_without_optional_values_reads_back() {
    let mut record = fixtures::task_record();
    record.instructions = None;
    record.llm = None;
    record.grace = None;
    record.run_since = None;
    record.retry_notice = None;
    record.ended_at = None;
    record.proposal = None;
    record.taken_by = None;
    record.person_answer = None;
    record.person_wait = None;
    record.recurring = None;
    record.tracked = None;
    record.hold_wait_since = None;
    record.last_answer = None;
    record.invalid_result = None;
    record.question_lines = Box::new([domain::QuestionLine { message: 5, line: 7, answer: None, resolution: None }]);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
fn a_task_record_at_its_inbox_bound_reads_back() {
    let mut record = fixtures::task_record();
    let mut first = fixtures::task_word(5);
    first.words = skein_lib::bytes::zeroed(16);
    let mut second = fixtures::task_word(7);
    second.words = skein_lib::bytes::zeroed(16);
    record.inbox = Box::new([first, second]);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
#[test]
fn a_task_record_past_its_inbox_count_is_refused() {
    let mut record = fixtures::task_record();
    record.inbox = Box::new([
        fixtures::task_word(5),
        fixtures::task_word(7),
        fixtures::task_word(11),
        fixtures::task_word(13),
        fixtures::task_word(17),
    ]);
    assert_eq!(
        tasks::encode(
            &live_record(record),
            &tasks_limits(),
            &fixtures::configured_model_names(&[b"first", b"second", b"third"])
        ),
        Err(bound_problem(wire::paths::TASK_INTERESTS_INBOX))
    );
}
#[test]
fn a_task_record_inbox_decoded_above_its_count_is_refused() {
    let limits = tasks_limits();
    let mut wider = limits;
    // Four words plus the fixture credit exceed the ordinary four-item bound.
    wider.task_interests_inbox = 5;
    let mut record = fixtures::task_record();
    record.inbox =
        Box::new([fixtures::task_word(5), fixtures::task_word(7), fixtures::task_word(11), fixtures::task_word(13)]);
    let bytes = tasks::encode(
        &live_record(record),
        &wider,
        &fixtures::configured_model_names(&[b"first", b"second", b"third"]),
    )
    .expect("within wider count");
    assert_eq!(
        tasks::decode(&bytes, &limits, &fixtures::configured_model_names(&[b"first", b"second", b"third"])),
        Err(bound_problem(wire::paths::TASK_INTERESTS_INBOX))
    );
}
#[test]
fn record_count_limits_past_the_schema_ceilings_are_refused() {
    for field in 0_u8..5 {
        let mut limits = fixtures::TASKS_LIMITS;
        match field {
            0 => limits.inbox_messages = u32::MAX,
            1 => limits.delegates = u32::MAX,
            2 => limits.references = u32::MAX,
            3 => limits.saved_resources = u32::MAX,
            _ => limits.authority_segments = u32::MAX,
        }
        assert_eq!(tasks::limits(&limits), None);
    }
}

#[test]
fn the_configured_subscription_bound_leaves_the_stored_ceiling() {
    for subscriptions in [0, u32::MAX] {
        let limits = domain::Limits { subscriptions, ..fixtures::TASKS_LIMITS };
        let stored = tasks::limits(&limits).expect("stored limits independent of subscription admission");
        assert_eq!(
            stored.task_interests_subscriptions,
            wire::CEILINGS.task_interests_subscriptions,
            "tasks.subscriptions = {subscriptions}"
        );
    }
}

fn a_result_word_uses_the_larger_shared_word_bound() {
    let mut record = fixtures::task_record();
    let mut word = fixtures::task_word(5);
    word.kind = domain::MessageKind::Result(domain::ResultKind::Report);
    word.words = skein_lib::bytes::zeroed(32);
    record.inbox = Box::new([word]);
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}
#[test]
fn an_activity_words_line_past_its_own_word_bound_is_refused() {
    let row = activity(domain::Line::Words {
        message: 5,
        by: domain::Party::Person(31),
        kind: domain::MessageKind::Words,
        words: skein_lib::bytes::zeroed(17),
    });
    assert_eq!(
        tasks::encode(&row, &tasks_limits(), &fixtures::configured_model_names(&[b"first", b"second", b"third"])),
        Err(bound_problem(wire::paths::ACTIVITY_WORDS_WORDS))
    );
}
fn a_person_wait_without_a_stall_notice_reads_back() {
    let mut record = fixtures::task_record();
    record.person_wait.as_mut().expect("fixture wait").notice = None;
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}

fn live_task_records_read_back_whole() {
    table([
        ("a_live_task_record_reads_back_whole", a_live_task_record_reads_back_whole),
        ("a_task_record_without_optional_values_reads_back", a_task_record_without_optional_values_reads_back),
        ("a_task_record_at_its_inbox_bound_reads_back", a_task_record_at_its_inbox_bound_reads_back),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    skein_rows::check(cases, |case| case());
}

#[test]
fn task_records_read_back_as_written() {
    table([
        ("run_reason_bits_refuse_unknown_kinds", run_reason_bits_refuse_unknown_kinds),
        (
            "a_task_records_label_flags_and_last_activity_read_back",
            a_task_records_label_flags_and_last_activity_read_back,
        ),
        ("an_ended_task_with_its_result_reads_back", an_ended_task_with_its_result_reads_back),
        ("an_activity_line_of_each_kind_reads_back", an_activity_line_of_each_kind_reads_back),
        ("each_terminal_activity_kind_reads_back", each_terminal_activity_kind_reads_back),
        ("each_index_row_reads_back", each_index_row_reads_back),
        ("each_stub_status_reads_back", each_stub_status_reads_back),
        ("each_message_kind_reads_back", each_message_kind_reads_back),
        ("each_result_message_kind_reads_back", each_result_message_kind_reads_back),
        ("each_notice_state_reads_back", each_notice_state_reads_back),
        ("each_news_class_reads_back", each_news_class_reads_back),
        ("each_escalation_state_reads_back", each_escalation_state_reads_back),
        ("each_escalation_holder_reads_back", each_escalation_holder_reads_back),
        ("each_invalid_result_reason_reads_back", each_invalid_result_reason_reads_back),
        ("each_subscription_kind_reads_back", each_subscription_kind_reads_back),
        ("a_person_wait_without_a_stall_notice_reads_back", a_person_wait_without_a_stall_notice_reads_back),
        ("live_task_records_read_back_whole", live_task_records_read_back_whole),
        ("a_result_word_uses_the_larger_shared_word_bound", a_result_word_uses_the_larger_shared_word_bound),
    ]);
}

#[test]
fn a_saved_named_deadline_keeps_its_optional_wall_in_both_batch_asks() {
    use jig_domain_people as people;
    for (selector, wall) in [
        (jig_domain_people::Deadline::Named, None),
        (jig_domain_people::Deadline::Named, Some(fixtures::DEADLINE)),
        (jig_domain_people::Deadline::Span(Duration::from_secs(5)), Some(fixtures::DEADLINE)),
    ] {
        let mut member = fixtures::people_member();
        member.authority.deadline = wall;
        member.deadline = selector;
        for ask in [
            people::Ask::Create { project: 1, batch: Box::new([member.clone()]) },
            people::Ask::Propose {
                project: 1,
                action: people::Action::Batch(Box::new([member])),
                reason: Box::from(b"please".as_slice()),
            },
        ] {
            super::ask_round_trip(ask);
        }
    }
}

#[test]
fn added_dependencies_circle_and_restarted_wait_bound_read_back() {
    let mut record = current_record();
    record.dependencies = Box::new([7, 9]);
    record.waiting_on = Box::new([9]);
    record.added = Box::new([
        domain::Added { task: 7, at: Wall::from_nanos(11), by: domain::Party::Person(13) },
        domain::Added { task: 9, at: Wall::from_nanos(17), by: domain::Party::Task(19) },
    ]);
    record.hold_wait_since = Some(Wall::from_nanos(23));
    record.hold_bound_since = Some(Wall::from_nanos(29));
    record.phase = domain::Phase::Held {
        was: domain::Was::Waiting,
        why: domain::Hold::Circle,
        source: Some(Box::new(domain::HoldSource::Circle(domain::Circle { tasks: Box::new([9, 7]), cut: true }))),
    };
    let row = live_record(record);
    assert_eq!(saved_tasks(&row), Ok(row));
}

#[test]
fn kept_lines_and_met_notices_read_back_and_pending_is_bounded() {
    let limits = tasks_limits();
    let pending = task_numbers(limits.kept_line_pending);
    let row = activity(domain::Line::Kept { attempt: 3, pending, question: None });
    assert_eq!(saved_tasks(&row), Ok(row));
    for removed in [false, true] {
        let row = message_row(domain::MessageKind::Met { dependency: 7, removed });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
    let pending = task_numbers(limits.kept_line_pending + 1);
    let row = activity(domain::Line::Kept { attempt: 3, pending, question: None });
    assert_eq!(tasks::encode(&row, &limits, &[]), Err(bound_problem(wire::paths::KEPT_LINE_PENDING)));
    let mut wider = limits;
    wider.kept_line_pending += 1;
    let bytes = tasks::encode(&row, &wider, &[]).unwrap();
    assert_eq!(tasks::decode(&bytes, &limits, &[]), Err(bound_problem(wire::paths::KEPT_LINE_PENDING)));
}

#[test]
fn question_keeps_notices_and_refusals_read_back() {
    let row = activity(domain::Line::Kept { attempt: 3, pending: Box::new([]), question: Some(u64::MAX) });
    assert_eq!(saved_tasks(&row), Ok(row));
    let row = message_row(domain::MessageKind::QuestionEnded { question: u64::MAX });
    assert_eq!(saved_tasks(&row), Ok(row));
    let problem = domain::Problem::new(Some(3), domain::Refusal::KeptQuestion { question: u64::MAX });
    let bytes = tasks::carried::problem(&problem, &tasks_limits()).unwrap();
    assert_eq!(tasks::carried::read_problem(&bytes, &tasks_limits()), Ok(problem));
}

#[test]
fn circle_and_added_records_refuse_one_past_their_bounds() {
    let limits = tasks_limits();
    let mut record = current_record();
    let mut added = skein_lib::List::with_capacity(limits.task_definition_added + 1);
    for _ in 0..=limits.task_definition_added {
        added.push(domain::Added { task: 7, at: Wall::EPOCH, by: domain::Party::Task(9) }).unwrap();
    }
    record.added = added.into_boxed();
    let row = live_record(record);
    assert_eq!(tasks::encode(&row, &limits, &[]), Err(bound_problem(wire::paths::HOT_DEFINITION_ADDED)));
    let mut record = current_record();
    record.phase = domain::Phase::Held {
        was: domain::Was::Waiting,
        why: domain::Hold::Circle,
        source: Some(Box::new(domain::HoldSource::Circle(domain::Circle {
            tasks: task_numbers(limits.circle_tasks + 1),
            cut: true,
        }))),
    };
    let row = live_record(record);
    assert_eq!(tasks::encode(&row, &limits, &[]), Err(bound_problem(wire::paths::CIRCLE_TASKS)));
    let mut wider = limits;
    wider.circle_tasks += 1;
    let bytes = tasks::encode(&row, &wider, &[]).unwrap();
    assert_eq!(tasks::decode(&bytes, &limits, &[]), Err(bound_problem(wire::paths::CIRCLE_TASKS)));
}

fn task_numbers(count: u32) -> Box<[u64]> {
    let mut numbers = skein_lib::List::with_capacity(count);
    for _ in 0..count {
        numbers.push(7).unwrap();
    }
    numbers.into_boxed()
}

#[test]
fn a_task_outcome_with_or_without_a_room_wait_reads_back() {
    for room_wait in [
        None,
        Some(domain::RoomWait { room: domain::LiveRoom::Deployment, order: 1 }),
        Some(domain::RoomWait { room: domain::LiveRoom::Project(u32::MAX), order: u64::MAX }),
    ] {
        let mut record = current_record();
        record.executor = domain::Executor::Procedure { connector: 1, code: 1 };
        record.phase = domain::Phase::Active(domain::Active::Idle);
        record.room_wait = room_wait;
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}

fn run_reason_bits_refuse_unknown_kinds() {
    // Bit 14 is the first bit outside the fourteen stored reasons.
    const UNKNOWN: domain::Reasons = domain::Reasons(1 << 14);
    for (reasons, offered, path) in [
        (UNKNOWN, domain::Reasons::ALL, wire::paths::TASK_RUN_REASONS),
        (domain::Reasons::FIRST, UNKNOWN, wire::paths::TASK_RUN_OFFERED),
    ] {
        let mut record = current_record();
        record.reasons = reasons;
        record.offered = offered;
        assert_eq!(saved_tasks(&live_record(record)), Err(bound_problem(path)));
    }
}

#[test]
fn invalid_run_ends_keep_their_contract_kind_in_store_bytes() {
    for reason in [
        domain::InvalidResult::Form,
        domain::InvalidResult::Verdict,
        domain::InvalidResult::Words,
        domain::InvalidResult::Change,
        domain::InvalidResult::Followups,
    ] {
        let row = activity(domain::Line::RunEnded {
            attempt: 3,
            end: domain::EndKind::Failed { class: domain::Class::Invalid, invalid_result: Some(reason) },
            spent: 7,
        });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}

#[test]
fn a_closer_ended_task_record_round_trips() {
    for closer in [Some(41), None] {
        let mut record = current_record();
        record.flags.conversation = false;
        record.flags.ended_by = domain::EndedBy::Closer(domain::Closer { connector: 1, code: 6 });
        record.closer = closer;
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}

#[test]
fn standing_sessions_and_activity_round_trip_at_the_words_bound() {
    for session in [
        None,
        Some(domain::Session {
            first: 3,
            reasons: domain::Reasons::FIRST.union(domain::Reasons::WORDS),
            last: Wall::from_nanos(7),
        }),
    ] {
        let mut record = current_record();
        record.flags.conversation = false;
        record.flags.ended_by = domain::EndedBy::Standing;
        record.tracked = None;
        record.session = session;
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
    let words = Box::from([b'w'; 512]);
    let row = activity(domain::Line::Stood { attempt: 3, words });
    assert_eq!(saved_tasks(&row), Ok(row));
    for end in [
        domain::SessionEnd::Quiet,
        domain::SessionEnd::Bound,
        domain::SessionEnd::Unoffered,
        domain::SessionEnd::Fresh,
        domain::SessionEnd::Closed,
        domain::SessionEnd::Rules,
    ] {
        let row = activity(domain::Line::SessionEnded { attempt: 3, end });
        assert_eq!(saved_tasks(&row), Ok(row));
    }
    let limits = tasks_limits();
    let words = Box::from([b'w'; 513]);
    let row = activity(domain::Line::Stood { attempt: 3, words });
    assert_eq!(tasks::encode(&row, &limits, &[]), Err(bound_problem(wire::paths::STOOD_LINE_WORDS)));
    let row = activity(domain::Line::Stood { attempt: 3, words: Box::from([b'w'; 512]) });
    let bytes = tasks::encode(&row, &limits, &[]).unwrap();
    let narrower = wire::Limits { stood_line_words: domain::STOOD_WORDS - 1, ..limits };
    assert_eq!(tasks::decode(&bytes, &narrower, &[]), Err(bound_problem(wire::paths::STOOD_LINE_WORDS)));
}

#[test]
fn now_written_round_trips_and_enforces_the_line_bound() {
    let row = activity(domain::Line::NowWritten { attempt: 7, line: Box::from([b'n'; 512]) });
    assert_eq!(saved_tasks(&row), Ok(row.clone()));
    let limits = tasks_limits();
    let bytes = tasks::encode(&row, &limits, &[]).unwrap();
    let narrower = wire::Limits { now_written_line_line: domain::NOW_LINE - 1, ..limits };
    assert_eq!(tasks::decode(&bytes, &narrower, &[]), Err(bound_problem(wire::paths::NOW_WRITTEN_LINE_LINE)));
    let row = activity(domain::Line::NowWritten { attempt: 7, line: Box::from([b'n'; 513]) });
    assert_eq!(tasks::encode(&row, &limits, &[]), Err(bound_problem(wire::paths::NOW_WRITTEN_LINE_LINE)));
}

#[test]
fn rejection_deciders_keep_their_stored_kind() {
    for (by, kind) in [
        (domain::EscalationDecider::Task(31), wire::DeciderKind::Task),
        (domain::EscalationDecider::Person(31), wire::DeciderKind::Person),
    ] {
        let rejection = domain::Escalation::Rejected { revision: 7, by, reason: Box::from(b"leave held".as_slice()) };
        let stored = tasks::state::escalation(&rejection);
        let wire::Escalation::Rejected(record) = &stored else {
            panic!("stored rejection");
        };
        assert_eq!(record.by, 31);
        assert_eq!(record.by_kind, kind);
        assert_eq!(tasks::state::read_escalation(stored), rejection.clone());
        let mut record = current_record();
        record.escalation = rejection;
        let row = live_record(record);
        assert_eq!(saved_tasks(&row), Ok(row));
    }
}
