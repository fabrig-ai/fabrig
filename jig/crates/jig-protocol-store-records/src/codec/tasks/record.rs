//! Durable task values translated exhaustively (protocol/store/records.md, 1).
use super::Context;
use super::{phase, state, value};
use crate::generated::tasks as mapped;
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::{List, Wall};

fn optional_grace(source: Option<&domain::Grace>) -> Option<wire::Grace> {
    match source {
        Some(source) => Some(mapped::grace_to_record(source)),
        None => None,
    }
}
fn read_optional_grace(source: Option<wire::Grace>) -> Option<domain::Grace> {
    match source {
        Some(source) => Some(mapped::grace_to_domain(source)),
        None => None,
    }
}

fn optional_retry_notice(source: Option<&domain::RetryNotice>) -> Option<wire::RetryNotice> {
    match source {
        Some(source) => Some(mapped::retry_notice_to_record(source)),
        None => None,
    }
}
fn read_optional_retry_notice(source: Option<wire::RetryNotice>) -> Option<domain::RetryNotice> {
    match source {
        Some(source) => Some(mapped::retry_notice_to_domain(source)),
        None => None,
    }
}

fn optional_proposal(
    source: Option<&domain::Proposal>,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<Option<wire::Proposal>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(mapped::proposal_to_record(source, &Context::new(limits, moved))?)),
        None => Ok(None),
    }
}
fn read_optional_proposal(
    source: Option<wire::Proposal>,
    limits: &wire::Limits,
) -> Result<Option<Box<domain::Proposal>>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(Box::new(mapped::proposal_to_domain(source, &Context::new(limits, None))?))),
        None => Ok(None),
    }
}

fn optional_person_answer(source: Option<&domain::PersonAnswer>) -> Option<wire::PersonAnswer> {
    match source {
        Some(source) => Some(mapped::person_answer_to_record(source)),
        None => None,
    }
}
fn read_optional_person_answer(source: Option<wire::PersonAnswer>) -> Option<domain::PersonAnswer> {
    match source {
        Some(source) => Some(mapped::person_answer_to_domain(source)),
        None => None,
    }
}

fn optional_person_wait(source: Option<&domain::PersonWait>) -> Option<wire::PersonWait> {
    match source {
        Some(source) => Some(mapped::person_wait_to_record(source)),
        None => None,
    }
}
fn read_optional_person_wait(source: Option<wire::PersonWait>) -> Option<domain::PersonWait> {
    match source {
        Some(source) => Some(mapped::person_wait_to_domain(source)),
        None => None,
    }
}

fn optional_recurring(
    source: Option<&domain::RecurringState>,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<Option<wire::RecurringState>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(mapped::recurring_state_to_record(source, &Context::new(limits, moved))?)),
        None => Ok(None),
    }
}
fn read_optional_recurring(
    source: Option<wire::RecurringState>,
    limits: &wire::Limits,
) -> Result<Option<Box<domain::RecurringState>>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(Box::new(mapped::recurring_state_to_domain(source, &Context::new(limits, None))?))),
        None => Ok(None),
    }
}

fn optional_invalid(source: Option<domain::InvalidResult>) -> Option<wire::InvalidResult> {
    match source {
        Some(value) => Some(mapped::invalid_result_to_record(&value)),
        None => None,
    }
}
fn read_optional_invalid(source: Option<wire::InvalidResult>) -> Option<domain::InvalidResult> {
    match source {
        Some(value) => Some(mapped::invalid_result_to_domain(value)),
        None => None,
    }
}

fn question_lines(
    source: &[domain::QuestionLine],
    limits: &wire::Limits,
) -> Result<List<wire::QuestionLine>, wire::Problem> {
    let mut members = List::with_capacity(limits.task_history_question_lines);
    for member in source {
        let member = mapped::question_line_to_record(member);
        if members.push(member).is_err() {
            return Err(problem(wire::paths::TASK_HISTORY_QUESTION_LINES));
        }
    }
    Ok(members)
}
fn read_question_lines(source: List<wire::QuestionLine>) -> Box<[domain::QuestionLine]> {
    let mut members = List::with_capacity(source.len());
    for member in source.into_boxed() {
        members.push(mapped::question_line_to_domain(member)).expect("same number of members");
    }
    members.into_boxed()
}

fn proposal_rows(
    source: &[domain::ProposalRow],
    limits: &wire::Limits,
) -> Result<List<wire::ProposalRow>, wire::Problem> {
    let mut members = List::with_capacity(limits.task_history_proposal_rows);
    for member in source {
        let member = mapped::proposal_row_to_record(member);
        if members.push(member).is_err() {
            return Err(problem(wire::paths::TASK_HISTORY_PROPOSAL_ROWS));
        }
    }
    Ok(members)
}
fn read_proposal_rows(source: List<wire::ProposalRow>) -> Box<[domain::ProposalRow]> {
    let mut members = List::with_capacity(source.len());
    for member in source.into_boxed() {
        members.push(mapped::proposal_row_to_domain(member)).expect("same number of members");
    }
    members.into_boxed()
}

fn replies(source: &[domain::ReplyHint], limits: &wire::Limits) -> Result<List<wire::ReplyHint>, wire::Problem> {
    let mut members = List::with_capacity(limits.task_run_replies);
    for member in source {
        let member = mapped::reply_hint_to_record(member);
        if members.push(member).is_err() {
            return Err(problem(wire::paths::TASK_RUN_REPLIES));
        }
    }
    Ok(members)
}
fn read_replies(source: List<wire::ReplyHint>) -> Box<[domain::ReplyHint]> {
    let mut members = List::with_capacity(source.len());
    for member in source.into_boxed() {
        members.push(mapped::reply_hint_to_domain(member)).expect("same number of members");
    }
    members.into_boxed()
}

fn holdings(source: &[domain::Holding], limits: &wire::Limits) -> Result<List<wire::Holding>, wire::Problem> {
    let mut members = List::with_capacity(limits.task_interests_holdings);
    for member in source {
        let member = mapped::holding_to_record(member, &Context::new(limits, None))?;
        if members.push(member).is_err() {
            return Err(problem(wire::paths::TASK_INTERESTS_HOLDINGS));
        }
    }
    Ok(members)
}
fn read_holdings(source: List<wire::Holding>, limits: &wire::Limits) -> Result<Box<[domain::Holding]>, wire::Problem> {
    let mut members = List::with_capacity(source.len());
    for member in source.into_boxed() {
        members.push(mapped::holding_to_domain(member, &Context::new(limits, None))?).expect("same number of members");
    }
    Ok(members.into_boxed())
}

fn subscriptions(
    source: &[domain::Subscription],
    limits: &wire::Limits,
) -> Result<List<wire::Subscription>, wire::Problem> {
    let mut members = List::with_capacity(limits.task_interests_subscriptions);
    for member in source {
        let member = mapped::subscription_to_record(member);
        if members.push(member).is_err() {
            return Err(problem(wire::paths::TASK_INTERESTS_SUBSCRIPTIONS));
        }
    }
    Ok(members)
}
fn read_subscriptions(source: List<wire::Subscription>) -> Box<[domain::Subscription]> {
    let mut members = List::with_capacity(source.len());
    for member in source.into_boxed() {
        members.push(mapped::subscription_to_domain(member)).expect("same number of members");
    }
    members.into_boxed()
}

// Answer credits and waiting words share the same admitted inbox slots.
fn inbox(
    source: &[domain::Word],
    unrecorded: &[domain::Word],
    credits: &[domain::QuestionCredit],
    limits: &wire::Limits,
) -> Result<List<wire::InboxItem>, wire::Problem> {
    let context = &Context::new(limits, None);
    let mut members = List::with_capacity(limits.task_interests_inbox);
    for credit in credits {
        if members.push(wire::InboxItem::Credit(mapped::question_credit_to_record(credit))).is_err() {
            return Err(problem(wire::paths::TASK_INTERESTS_INBOX));
        }
    }
    for word in source {
        if members.push(wire::InboxItem::Word(mapped::word_to_record(word, context)?)).is_err() {
            return Err(problem(wire::paths::TASK_INTERESTS_INBOX));
        }
    }
    for word in unrecorded {
        if members.push(wire::InboxItem::Unrecorded(mapped::word_to_record(word, context)?)).is_err() {
            return Err(problem(wire::paths::TASK_INTERESTS_INBOX));
        }
    }
    Ok(members)
}
struct Inbox {
    words: Box<[domain::Word]>,
    unrecorded: Box<[domain::Word]>,
    credits: Box<[domain::QuestionCredit]>,
}

fn read_inbox(source: List<wire::InboxItem>, limits: &wire::Limits) -> Result<Inbox, wire::Problem> {
    let context = &Context::new(limits, None);
    let mut words = List::with_capacity(source.len());
    let mut credits = List::with_capacity(source.len());
    let mut unrecorded = List::with_capacity(source.len());
    for member in source.into_boxed() {
        match member {
            wire::InboxItem::Word(word) => {
                words.push(mapped::word_to_domain(word, context)?).expect("same number of items");
            }
            wire::InboxItem::Unrecorded(word) => {
                unrecorded.push(mapped::word_to_domain(word, context)?).expect("same number of items");
            }
            wire::InboxItem::Credit(credit) => {
                credits.push(mapped::question_credit_to_domain(credit)).expect("same number of items");
            }
        }
    }
    Ok(Inbox { words: words.into_boxed(), unrecorded: unrecorded.into_boxed(), credits: credits.into_boxed() })
}
fn saved(source: &[domain::SavedResource], limits: &wire::Limits) -> Result<List<wire::SavedResource>, wire::Problem> {
    let mut members = List::with_capacity(limits.task_interests_saved);
    for member in source {
        let member = state::saved_resource(member, limits)?;
        if members.push(member).is_err() {
            return Err(problem(wire::paths::TASK_INTERESTS_SAVED));
        }
    }
    Ok(members)
}
fn read_saved(
    source: List<wire::SavedResource>,
    limits: &wire::Limits,
) -> Result<Box<[domain::SavedResource]>, wire::Problem> {
    let mut members = List::with_capacity(source.len());
    for member in source.into_boxed() {
        members.push(state::read_saved_resource(member, limits)?).expect("same number of members");
    }
    Ok(members.into_boxed())
}

/// Translate every retained task field.
// The live and ended tags without a talk keep this payload.
fn before_talk(
    source: &domain::TaskRecord,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::BeforeTalk, wire::Problem> {
    Ok(wire::BeforeTalk {
        activation_open: source.activation_open,
        session: match source.session {
            Some(session) => Some(mapped::session_to_record(&session)),
            None => None,
        },
        closer: source.closer,
        asked_requester: source.asked_requester,
        words_woken: source.words_woken,
        held_since: value::wall(source.held_since),
        history: task_history(source, limits)?,
        run: task_run(source, limits, moved)?,
        definition: task_definition(source, limits, moved)?,
        started_from: phase::references(
            &source.started_from,
            limits.before_talk_started_from,
            wire::paths::BEFORE_TALK_STARTED_FROM,
        )?,
        interests: task_interests(source, limits)?,
        outcome: task_outcome(source, limits)?,
    })
}

fn task_history(source: &domain::TaskRecord, limits: &wire::Limits) -> Result<wire::TaskHistory, wire::Problem> {
    Ok(wire::TaskHistory {
        last_activity: source.last_activity.as_nanos(),
        ended_below: mapped::endings_to_record(&source.ended_below),
        ended_delegates: mapped::endings_to_record(&source.ended_delegates),
        question_lines: question_lines(&source.question_lines, limits)?,
        hold: source.hold,
        placed: source.placed,
        proposal_rows: proposal_rows(&source.proposal_rows, limits)?,
        instructions: source.instructions.clone(),
        refused: refused(&source.refused, limits)?,
        grace: optional_grace(source.grace.as_ref()),
        settings: mapped::settings_revision_to_record(&source.settings),
        charters: source.charters,
        revisions: mapped::revisions_to_record(&source.revisions),
        created_at: source.created_at.as_nanos(),
        changed: source.changed.as_nanos(),
        phase_since: source.phase_since.as_nanos(),
        run_since: value::wall(source.run_since),
        retry_notice: optional_retry_notice(source.retry_notice.as_ref()),
        ended_at: value::wall(source.ended_at),
        revision: source.revision,
    })
}

fn task_run(
    source: &domain::TaskRecord,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::TaskRun, wire::Problem> {
    Ok(wire::TaskRun {
        milestone: source.milestone,
        activity: source.activity,
        due_reason: value::why(source.due_reason),
        reasons: source.reasons.0,
        offered: source.offered.0,
        narrowing: source.narrowing,
        llm_changed: source.llm_changed,
        result_position: source.result_position,
        last_reply: source.last_reply,
        replies: replies(&source.replies, limits)?,
        escalation: state::escalation(&source.escalation),
        proposal: optional_proposal(source.proposal.as_deref(), limits, moved)?,
        result_proposal: source.result_proposal,
        number: source.number,
        project: source.project,
        requester: mapped::party_to_record(&source.requester),
        run_spent: source.run_spent,
        run_reserved: source.run_reserved,
        open_attempt: super::lifecycle::open(source.open_attempt.as_ref())?,
        suspension: super::lifecycle::suspension(source.suspension),
        suspended: source.suspended,
        restarted: source.restarted,
    })
}

fn task_definition(
    source: &domain::TaskRecord,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::TaskDefinition, wire::Problem> {
    Ok(wire::TaskDefinition {
        label: source.label,
        flags: mapped::flags_to_record(&source.flags),
        root: source.root,
        depth: source.depth,
        executor: mapped::executor_to_record(&source.executor),
        llm: value::llm(source.llm.as_ref())?,
        kept_account: source.kept_account,
        taken_by: source.taken_by,
        person_answer: optional_person_answer(source.person_answer.as_ref()),
        person_wait: optional_person_wait(source.person_wait.as_ref()),
        recurring: optional_recurring(source.recurring.as_deref(), limits, moved)?,
        tracked: source.tracked,
        spec: value::spec(&source.spec, limits)?,
        contract: value::contract(&source.contract, limits)?,
        authority: value::authority(&source.authority, limits, moved)?,
        numbers: mapped::numbers_to_record(&source.numbers),
        funder: mapped::funder_to_record(&source.funder),
        dependencies: dependencies(&source.dependencies, limits)?,
        waiting_on: waiting_on(&source.waiting_on, limits)?,
        added: added(&source.added, limits)?,
    })
}

fn notice_credits(
    source: &[domain::NoticeCredit],
    limits: &wire::Limits,
) -> Result<List<wire::NoticeCredit>, wire::Problem> {
    let mut members = List::with_capacity(limits.task_interests_notice_credits);
    for credit in source {
        if members.push(mapped::notice_credit_to_record(credit)).is_err() {
            return Err(problem(wire::paths::TASK_INTERESTS_NOTICE_CREDITS));
        }
    }
    Ok(members)
}

fn read_notice_credits(source: List<wire::NoticeCredit>) -> Box<[domain::NoticeCredit]> {
    let mut members = List::with_capacity(source.len());
    for credit in source.into_boxed() {
        members.push(mapped::notice_credit_to_domain(credit)).expect("same number of credits");
    }
    members.into_boxed()
}

fn task_interests(source: &domain::TaskRecord, limits: &wire::Limits) -> Result<wire::TaskInterests, wire::Problem> {
    Ok(wire::TaskInterests {
        holdings: holdings(&source.holdings, limits)?,
        holds_taken: source.holds_taken,
        hold_wait_since: value::wall(source.hold_wait_since),
        hold_bound_since: value::wall(source.hold_bound_since),
        delegates: delegates(&source.delegates, limits)?,
        result_followups: result_followups(&source.result_followups, limits)?,
        references: references(&source.references, limits)?,
        subscriptions: subscriptions(&source.subscriptions, limits)?,
        wake: mapped::wake_policy_to_record(&source.wake),
        turn: source.turn,
        last_message: source.last_message,
        inbox: inbox(&source.inbox, &source.unrecorded, &source.questions, limits)?,
        saved: saved(&source.saved, limits)?,
        ever_turned: wire::TranscriptState { ever_turned: source.ever_turned },
        made: source.made,
        notice_credits: notice_credits(&source.notice_credits, limits)?,
    })
}

fn task_outcome(source: &domain::TaskRecord, limits: &wire::Limits) -> Result<wire::TaskOutcome, wire::Problem> {
    let context = &Context::new(limits, None);
    Ok(wire::TaskOutcome {
        attempt: source.attempt,
        last_answer: source.last_answer,
        tries: mapped::tries_to_record(&source.tries),
        invalid_result: optional_invalid(source.invalid_result),
        refusals: source.refusals,
        phase: mapped::phase_to_record(&source.phase, context)?,
        room_wait: room_wait(source.room_wait),
    })
}
/// Read every retained task field.
#[expect(clippy::too_many_lines, reason = "one translation carries every retained task field")]
pub fn read_before_talk(source: wire::BeforeTalk, limits: &wire::Limits) -> Result<domain::TaskRecord, wire::Problem> {
    let context = &Context::new(limits, None);
    let wire::BeforeTalk {
        activation_open,
        session,
        closer,
        history,
        run,
        definition,
        started_from,
        interests,
        outcome,
        asked_requester,
        words_woken,
        held_since,
    } = source;

    let reasons = match super::reasons_to_domain(run.reasons) {
        Ok(value) => value,
        Err(reason) => return Err(wire::paths::TASK_RUN_REASONS.problem(reason)),
    };
    let offered = match super::reasons_to_domain(run.offered) {
        Ok(value) => value,
        Err(reason) => return Err(wire::paths::TASK_RUN_OFFERED.problem(reason)),
    };

    let Inbox { words: inbox, unrecorded, credits: questions } = read_inbox(interests.inbox, limits)?;
    Ok(domain::TaskRecord {
        concluded: None,
        talk: None,
        activation_open,
        session: match session {
            Some(session) => Some(mapped::session_to_domain(session)?),
            None => None,
        },
        closer,
        parked_result: None,
        offered,
        reasons,
        asked_requester,
        words_woken,
        unrecorded,
        label: definition.label,
        flags: mapped::flags_to_domain(definition.flags),
        last_activity: Wall::from_nanos(history.last_activity),
        ended_below: mapped::endings_to_domain(history.ended_below),
        ended_delegates: mapped::endings_to_domain(history.ended_delegates),
        question_lines: read_question_lines(history.question_lines),
        hold: history.hold,
        placed: history.placed,
        proposal_rows: read_proposal_rows(history.proposal_rows),
        instructions: history.instructions,
        refused: phase::read_references(history.refused),
        grace: read_optional_grace(history.grace),
        settings: mapped::settings_revision_to_domain(history.settings),
        charters: history.charters,
        revisions: mapped::revisions_to_domain(history.revisions),
        created_at: Wall::from_nanos(history.created_at),
        changed: Wall::from_nanos(history.changed),
        phase_since: Wall::from_nanos(history.phase_since),
        held_since: value::read_wall(held_since),
        run_since: value::read_wall(history.run_since),
        retry_notice: read_optional_retry_notice(history.retry_notice),
        ended_at: value::read_wall(history.ended_at),
        revision: history.revision,
        milestone: run.milestone,
        open_attempt: super::lifecycle::read_open(run.open_attempt)?,
        suspension: super::lifecycle::read_suspension(run.suspension),
        suspended: run.suspended,
        restarted: run.restarted,
        activity: run.activity,
        due_reason: value::read_why(run.due_reason),
        narrowing: run.narrowing,
        llm_changed: run.llm_changed,
        result_position: run.result_position,
        last_reply: run.last_reply,
        replies: read_replies(run.replies),
        escalation: state::read_escalation(run.escalation),
        proposal: read_optional_proposal(run.proposal, limits)?,
        result_proposal: run.result_proposal,
        number: run.number,
        project: run.project,
        requester: mapped::party_to_domain(run.requester),
        run_spent: run.run_spent,
        run_reserved: run.run_reserved,
        root: definition.root,
        depth: definition.depth,
        executor: mapped::executor_to_domain(definition.executor),
        llm: value::read_llm(definition.llm)?,
        kept_account: definition.kept_account,
        taken_by: definition.taken_by,
        person_answer: read_optional_person_answer(definition.person_answer),
        person_wait: read_optional_person_wait(definition.person_wait),
        recurring: read_optional_recurring(definition.recurring, limits)?,
        tracked: definition.tracked,
        spec: value::read_spec(definition.spec, limits)?,
        contract: value::read_contract(definition.contract, limits)?,
        authority: value::read_authority(definition.authority, limits)?,
        numbers: mapped::numbers_to_domain(definition.numbers),
        funder: mapped::funder_to_domain(definition.funder),
        dependencies: phase::read_references(definition.dependencies),
        started_from: phase::read_references(started_from),
        waiting_on: phase::read_references(definition.waiting_on),
        added: read_added(definition.added),
        holdings: read_holdings(interests.holdings, limits)?,
        holds_taken: interests.holds_taken,
        hold_wait_since: value::read_wall(interests.hold_wait_since),
        hold_bound_since: value::read_wall(interests.hold_bound_since),
        delegates: phase::read_references(interests.delegates),
        result_followups: phase::read_references(interests.result_followups),
        references: phase::read_references(interests.references),
        questions,
        notice_credits: read_notice_credits(interests.notice_credits),
        subscriptions: read_subscriptions(interests.subscriptions),
        wake: mapped::wake_policy_to_domain(interests.wake),
        turn: interests.turn,
        last_message: interests.last_message,
        inbox,
        saved: read_saved(interests.saved, limits)?,
        ever_turned: interests.ever_turned.ever_turned,
        made: interests.made,
        attempt: outcome.attempt,
        last_answer: outcome.last_answer,
        tries: mapped::tries_to_domain(outcome.tries),
        invalid_result: read_optional_invalid(outcome.invalid_result),
        refusals: outcome.refusals,
        room_wait: read_room_wait(outcome.room_wait),
        phase: mapped::phase_to_domain(outcome.phase, context)?,
    })
}
fn problem(path: wire::Path) -> wire::Problem {
    wire::Problem { path, reason: skein_codec::Reason::Bound }
}

fn refused(source: &[u64], limits: &wire::Limits) -> Result<List<wire::Number>, wire::Problem> {
    phase::references(source, limits.task_history_refused, wire::paths::TASK_HISTORY_REFUSED)
}

fn dependencies(source: &[u64], limits: &wire::Limits) -> Result<List<wire::Number>, wire::Problem> {
    phase::references(source, limits.task_definition_dependencies, wire::paths::TASK_DEFINITION_DEPENDENCIES)
}

fn waiting_on(source: &[u64], limits: &wire::Limits) -> Result<List<wire::Number>, wire::Problem> {
    phase::references(source, limits.task_definition_waiting_on, wire::paths::TASK_DEFINITION_WAITING_ON)
}

fn delegates(source: &[u64], limits: &wire::Limits) -> Result<List<wire::Number>, wire::Problem> {
    phase::references(source, limits.task_interests_delegates, wire::paths::TASK_INTERESTS_DELEGATES)
}

fn result_followups(source: &[u64], limits: &wire::Limits) -> Result<List<wire::Number>, wire::Problem> {
    phase::references(source, limits.task_interests_result_followups, wire::paths::TASK_INTERESTS_RESULT_FOLLOWUPS)
}

fn references(source: &[u64], limits: &wire::Limits) -> Result<List<wire::Number>, wire::Problem> {
    phase::references(source, limits.task_interests_references, wire::paths::TASK_INTERESTS_REFERENCES)
}

fn added(source: &[domain::Added], limits: &wire::Limits) -> Result<List<wire::Added>, wire::Problem> {
    let mut rows = List::with_capacity(limits.task_definition_added);
    for row in source {
        let row = mapped::added_to_record(row);
        if rows.push(row).is_err() {
            return Err(problem(wire::paths::TASK_DEFINITION_ADDED));
        }
    }
    Ok(rows)
}
fn read_added(source: List<wire::Added>) -> Box<[domain::Added]> {
    let mut rows = List::with_capacity(source.len());
    for row in source.into_boxed() {
        let row = mapped::added_to_domain(row);
        rows.push(row).expect("same number of additions");
    }
    rows.into_boxed()
}

fn room_wait(source: Option<domain::RoomWait>) -> Option<wire::RoomWait> {
    match source {
        Some(value) => Some(mapped::room_wait_to_record(&value)),
        None => None,
    }
}

fn read_room_wait(source: Option<wire::RoomWait>) -> Option<domain::RoomWait> {
    match source {
        Some(value) => Some(mapped::room_wait_to_domain(value)),
        None => None,
    }
}

pub fn task_record(
    source: &domain::TaskRecord,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::TaskRecord, wire::Problem> {
    let talk = match source.talk {
        Some(talk) => Some(mapped::talk_to_record(&talk)),
        None => None,
    };
    Ok(wire::TaskRecord { record: before_talk(source, limits, moved)?, talk, concluded: concluded(source.concluded) })
}

pub fn read_task_record(source: wire::TaskRecord, limits: &wire::Limits) -> Result<domain::TaskRecord, wire::Problem> {
    let mut record = read_before_talk(source.record, limits)?;
    record.concluded = read_concluded(source.concluded);
    record.talk = match source.talk {
        Some(talk) => Some(mapped::talk_to_domain(talk)),
        None => None,
    };
    Ok(record)
}

// Live parts (jig/docs/design/protocol/store/parts.md, 2).
pub fn live_cold(
    spec: &domain::Spec,
    contract: &domain::Contract,
    authority: &domain::Authority,
    instructions: &Option<Box<[u8]>>,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::LiveCold, wire::Problem> {
    Ok(wire::LiveCold {
        spec: value::spec(spec, limits)?,
        contract: value::contract(contract, limits)?,
        authority: value::authority(authority, limits, moved)?,
        instructions: instructions.clone(),
    })
}

#[expect(clippy::too_many_lines, reason = "exhaustive placement of every task field before encoding its hot part")]
pub fn live_hot(
    source: &domain::TaskRecord,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::LiveHot, wire::Problem> {
    let context = &Context::new(limits, None);
    let domain::TaskRecord {
        concluded: concluding,
        talk,
        session,
        activation_open,
        closer,
        parked_result,
        reasons,
        offered,
        placed,
        settings,
        charters,
        label,
        flags,
        last_activity,
        ended_below,
        ended_delegates,
        question_lines,
        hold,
        proposal_rows,
        instructions: _,
        llm,
        kept_account,
        refused,
        grace,
        revisions,
        created_at,
        changed,
        phase_since,
        held_since,
        run_since,
        retry_notice,
        ended_at,
        revision,
        milestone,
        activity,
        open_attempt,
        suspension,
        suspended,
        restarted,
        due_reason,
        narrowing,
        llm_changed,
        result_position,
        last_reply,
        replies,
        escalation,
        proposal,
        result_proposal,
        number,
        project,
        requester,
        run_spent,
        run_reserved,
        root,
        depth,
        executor,
        taken_by,
        person_answer,
        person_wait,
        recurring,
        tracked,
        spec: _,
        contract: _,
        authority: _,
        numbers,
        funder,
        dependencies,
        started_from,
        added,
        waiting_on,
        holdings,
        holds_taken,
        hold_wait_since,
        hold_bound_since,
        delegates,
        result_followups,
        references,
        questions,
        notice_credits,
        subscriptions,
        wake,
        turn,
        last_message,
        inbox,
        unrecorded,
        saved,
        ever_turned,
        made,
        attempt,
        last_answer,
        tries,
        invalid_result,
        refusals,
        phase,
        room_wait,
        asked_requester,
        words_woken,
    } = source;
    let history = wire::HotHistory {
        last_activity: last_activity.as_nanos(),
        ended_below: mapped::endings_to_record(ended_below),
        ended_delegates: mapped::endings_to_record(ended_delegates),
        question_lines: hot_question_lines(question_lines, limits)?,
        hold: *hold,
        placed: *placed,
        proposal_rows: hot_proposal_rows(proposal_rows, limits)?,
        refused: phase::references(refused, limits.hot_history_refused, wire::paths::HOT_HISTORY_REFUSED)?,
        grace: optional_grace(grace.as_ref()),
        settings: mapped::settings_revision_to_record(settings),
        charters: *charters,
        revisions: mapped::revisions_to_record(revisions),
        created_at: created_at.as_nanos(),
        changed: changed.as_nanos(),
        phase_since: phase_since.as_nanos(),
        run_since: value::wall(*run_since),
        retry_notice: optional_retry_notice(retry_notice.as_ref()),
        ended_at: value::wall(*ended_at),
        revision: *revision,
    };
    let run = wire::TaskRun {
        milestone: *milestone,
        activity: *activity,
        open_attempt: super::lifecycle::open(open_attempt.as_ref())?,
        suspension: super::lifecycle::suspension(*suspension),
        suspended: *suspended,
        restarted: *restarted,
        due_reason: value::why(*due_reason),
        reasons: reasons.0,
        offered: offered.0,
        narrowing: *narrowing,
        llm_changed: *llm_changed,
        result_position: *result_position,
        last_reply: *last_reply,
        replies: self::replies(replies, limits)?,
        escalation: state::escalation(escalation),
        proposal: optional_proposal(proposal.as_deref(), limits, moved)?,
        result_proposal: *result_proposal,
        number: *number,
        project: *project,
        requester: mapped::party_to_record(requester),
        run_spent: *run_spent,
        run_reserved: *run_reserved,
    };
    let definition = wire::HotDefinition {
        label: *label,
        flags: mapped::flags_to_record(flags),
        root: *root,
        depth: *depth,
        executor: mapped::executor_to_record(executor),
        llm: value::llm(llm.as_ref())?,
        kept_account: *kept_account,
        taken_by: *taken_by,
        person_answer: optional_person_answer(person_answer.as_ref()),
        person_wait: optional_person_wait(person_wait.as_ref()),
        recurring: optional_recurring(recurring.as_deref(), limits, moved)?,
        tracked: *tracked,
        numbers: mapped::numbers_to_record(numbers),
        funder: mapped::funder_to_record(funder),
        dependencies: phase::references(
            dependencies,
            limits.hot_definition_dependencies,
            wire::paths::HOT_DEFINITION_DEPENDENCIES,
        )?,
        waiting_on: phase::references(
            waiting_on,
            limits.hot_definition_waiting_on,
            wire::paths::HOT_DEFINITION_WAITING_ON,
        )?,
        added: hot_added(added, limits)?,
    };
    let interests = wire::TaskInterests {
        holdings: self::holdings(holdings, limits)?,
        holds_taken: *holds_taken,
        hold_wait_since: value::wall(*hold_wait_since),
        hold_bound_since: value::wall(*hold_bound_since),
        delegates: self::delegates(delegates, limits)?,
        result_followups: self::result_followups(result_followups, limits)?,
        references: self::references(references, limits)?,
        subscriptions: self::subscriptions(subscriptions, limits)?,
        wake: mapped::wake_policy_to_record(wake),
        turn: *turn,
        last_message: *last_message,
        inbox: self::inbox(inbox, unrecorded, questions, limits)?,
        saved: self::saved(saved, limits)?,
        ever_turned: wire::TranscriptState { ever_turned: *ever_turned },
        made: *made,
        notice_credits: self::notice_credits(notice_credits, limits)?,
    };
    let outcome = wire::TaskOutcome {
        attempt: *attempt,
        last_answer: *last_answer,
        tries: mapped::tries_to_record(tries),
        invalid_result: optional_invalid(*invalid_result),
        refusals: *refusals,
        phase: mapped::phase_to_record(phase, context)?,
        room_wait: self::room_wait(*room_wait),
    };
    Ok(wire::LiveHot {
        concluded: concluded(*concluding),
        talk: match *talk {
            Some(talk) => Some(mapped::talk_to_record(&talk)),
            None => None,
        },
        activation_open: *activation_open,
        session: match *session {
            Some(session) => Some(mapped::session_to_record(&session)),
            None => None,
        },
        closer: *closer,
        parked_result: match parked_result {
            Some(result) => Some(super::task_result_to_record(result, context)?),
            None => None,
        },
        asked_requester: *asked_requester,
        words_woken: *words_woken,
        held_since: value::wall(*held_since),
        history,
        run,
        definition,
        started_from: phase::references(
            started_from,
            limits.live_hot_started_from,
            wire::paths::LIVE_HOT_STARTED_FROM,
        )?,
        interests,
        outcome,
    })
}

fn hot_question_lines(
    source: &[domain::QuestionLine],
    limits: &wire::Limits,
) -> Result<List<wire::QuestionLine>, wire::Problem> {
    let mut members = List::with_capacity(limits.hot_history_question_lines);
    for member in source {
        let member = mapped::question_line_to_record(member);
        if members.push(member).is_err() {
            return Err(problem(wire::paths::HOT_HISTORY_QUESTION_LINES));
        }
    }
    Ok(members)
}
fn hot_proposal_rows(
    source: &[domain::ProposalRow],
    limits: &wire::Limits,
) -> Result<List<wire::ProposalRow>, wire::Problem> {
    let mut members = List::with_capacity(limits.hot_history_proposal_rows);
    for member in source {
        let member = mapped::proposal_row_to_record(member);
        if members.push(member).is_err() {
            return Err(problem(wire::paths::HOT_HISTORY_PROPOSAL_ROWS));
        }
    }
    Ok(members)
}
fn hot_added(source: &[domain::Added], limits: &wire::Limits) -> Result<List<wire::Added>, wire::Problem> {
    let mut rows = List::with_capacity(limits.hot_definition_added);
    for row in source {
        let row = mapped::added_to_record(row);
        if rows.push(row).is_err() {
            return Err(problem(wire::paths::HOT_DEFINITION_ADDED));
        }
    }
    Ok(rows)
}

/// Join the hot and cold fields, then translate the live record (protocol/store/parts.md, 2).
#[expect(clippy::too_many_lines, reason = "every hot and cold field is named in the whole record join")]
pub fn read_live(
    hot: wire::LiveHot,
    cold: wire::LiveCold,
    limits: &wire::Limits,
) -> Result<domain::TaskRecord, wire::Problem> {
    let wire::LiveHot {
        concluded,
        activation_open,
        session,
        closer,
        history,
        run,
        definition,
        started_from,
        interests,
        outcome,
        asked_requester,
        words_woken,
        held_since,
        talk,
        parked_result,
    } = hot;
    let wire::LiveCold { spec, contract, authority, instructions } = cold;
    let wire::HotHistory {
        last_activity,
        ended_below,
        ended_delegates,
        question_lines,
        hold,
        placed,
        proposal_rows,
        refused,
        grace,
        settings,
        charters,
        revisions,
        created_at,
        changed,
        phase_since,
        run_since,
        retry_notice,
        ended_at,
        revision,
    } = history;
    let history = wire::TaskHistory {
        last_activity,
        ended_below,
        ended_delegates,
        question_lines,
        hold,
        placed,
        proposal_rows,
        instructions,
        refused,
        grace,
        settings,
        charters,
        revisions,
        created_at,
        changed,
        phase_since,
        run_since,
        retry_notice,
        ended_at,
        revision,
    };
    let wire::HotDefinition {
        label,
        flags,
        root,
        depth,
        executor,
        llm,
        kept_account,
        taken_by,
        person_answer,
        person_wait,
        recurring,
        tracked,
        numbers,
        funder,
        dependencies,
        waiting_on,
        added,
    } = definition;
    let definition = wire::TaskDefinition {
        label,
        flags,
        root,
        depth,
        executor,
        llm,
        kept_account,
        taken_by,
        person_answer,
        person_wait,
        recurring,
        tracked,
        spec,
        contract,
        authority,
        numbers,
        funder,
        dependencies,
        waiting_on,
        added,
    };
    let record = wire::BeforeTalk {
        activation_open,
        session,
        closer,
        history,
        run,
        definition,
        started_from,
        interests,
        outcome,
        asked_requester,
        words_woken,
        held_since,
    };
    let mut row = read_task_record(wire::TaskRecord { record, talk, concluded }, limits)?;
    row.parked_result = match parked_result {
        Some(result) => Some(super::task_result_to_domain(result, &Context::new(limits, None))?),
        None => None,
    };
    Ok(row)
}

fn concluded(source: Option<domain::Concluded>) -> Option<wire::Concluded> {
    match source {
        Some(value) => Some(mapped::concluded_to_record(&value)),
        None => None,
    }
}
fn read_concluded(source: Option<wire::Concluded>) -> Option<domain::Concluded> {
    match source {
        Some(value) => Some(mapped::concluded_to_domain(value)),
        None => None,
    }
}
