//! Configuration and retained-value bounds at load (jig/docs/design/domain/engine/restart.md).
use crate::{
    Amendment, Authority, Contract, Domain, Ending, Executor, HoldSource, Last, Limits, MessageFamily, MessageKind,
    New, Numbers, Parameter, PersonAction, PersonProposal, Phase, ProposalAction, RestoreBound, RestoreCause,
    RestoreLlmWhy, ResultField, ResultKind, Separate, Spec, TaskRecord, TaskResult, Was,
};
use skein_lib::length::len_u64;

pub(crate) fn bound(which: RestoreBound, held: u64, limit: u64) -> Result<(), RestoreCause> {
    if held > limit { Err(RestoreCause::Bound { which, held, limit }) } else { Ok(()) }
}

fn count(which: RestoreBound, len: usize, limit: u32) -> Result<(), RestoreCause> {
    bound(which, u64::try_from(len).or(Err(RestoreCause::Accounting))?, u64::from(limit))
}

pub(crate) fn room(which: RestoreBound, len: u32, limit: u32) -> Result<(), RestoreCause> {
    bound(which, u64::from(len).checked_add(1).ok_or(RestoreCause::Accounting)?, u64::from(limit))
}

pub(super) fn timers(domain: &Domain, task: &TaskRecord) -> Result<u32, RestoreCause> {
    let mut held = u64::from(domain.timer_subscriptions);
    for subscription in &task.subscriptions {
        match subscription.kind {
            crate::SubscriptionKind::Timer { .. } => {
                held = held.checked_add(1).ok_or(RestoreCause::Accounting)?;
            }
            crate::SubscriptionKind::Task { .. } | crate::SubscriptionKind::Topic { .. } => {}
        }
    }
    bound(RestoreBound::Timers, held, u64::from(domain.timer_capacity))?;
    u32::try_from(held).or(Err(RestoreCause::Accounting))
}

pub(crate) fn numbers(value: Numbers) -> Result<(), RestoreCause> {
    let spent = crate::funders::total(value).ok_or(RestoreCause::Accounting)?;
    spent.checked_add(value.reserved).ok_or(RestoreCause::Accounting)?;
    Ok(())
}

fn spec(value: &Spec, limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::Parameters, value.parameters.len(), limits.parameters)?;
    count(RestoreBound::Inputs, value.inputs.len(), limits.inputs)?;
    if let Some(job) = &value.job {
        count(RestoreBound::SpecBytes, job.bytes.len(), limits.spec_bytes)?;
    }
    let mut bytes = value.words.len();
    for parameter in &value.parameters {
        match parameter {
            Parameter::Bytes { value, .. } => bytes = bytes.checked_add(value.len()).ok_or(RestoreCause::Accounting)?,
            Parameter::Number { .. } | Parameter::Resource { .. } => {}
        }
    }
    count(RestoreBound::SpecBytes, bytes, limits.spec_bytes)?;
    if value.asks {
        count(RestoreBound::InboxBytes, crate::questions::OWED_WORDS.len(), limits.inbox_bytes)?;
        count(RestoreBound::InboxBytes, crate::questions::ANSWER_NOTICE_BYTES, limits.inbox_bytes)?;
    }
    Ok(())
}

fn authority(value: &Authority, limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::AuthorityGrants, value.grants.len(), limits.authority_grants)?;
    count(RestoreBound::AuthorityGrants, value.note_resources.len(), limits.authority_grants)?;
    count(RestoreBound::ExecutorKinds, value.delegation.kinds.len(), limits.executor_kinds)?;
    let mut bytes = 0_usize;
    for grant in &value.grants {
        let size = pattern(&grant.pattern, limits)?;
        bytes = bytes.checked_add(size).ok_or(RestoreCause::Accounting)?;
    }
    for scope in &value.note_resources {
        let size = pattern(&scope.pattern, limits)?;
        bytes = bytes.checked_add(size).ok_or(RestoreCause::Accounting)?;
    }
    count(RestoreBound::AuthorityBytes, bytes, limits.authority_bytes)
}

fn pattern(value: &crate::Pattern, limits: &Limits) -> Result<usize, RestoreCause> {
    count(RestoreBound::AuthoritySegments, value.segments.len(), limits.authority_segments)?;
    let mut bytes = 0_usize;
    for segment in &value.segments {
        bytes = bytes.checked_add(segment.len()).ok_or(RestoreCause::Accounting)?;
    }
    match &value.last {
        Last::Exact(last) | Last::Open(last) => bytes.checked_add(last.len()).ok_or(RestoreCause::Accounting),
    }
}

pub(crate) fn name(value: &crate::Name, limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::HoldSegments, value.path.len(), limits.hold_segments)?;
    let mut bytes = 0_usize;
    for segment in &value.path {
        bytes = bytes.checked_add(segment.len()).ok_or(RestoreCause::Accounting)?;
    }
    count(RestoreBound::HoldBytes, bytes, limits.hold_bytes)
}

fn contract(value: &Contract, limits: &Limits) -> Result<(), RestoreCause> {
    match value {
        Contract::Job { bytes } => bound(RestoreBound::ResultBytes, u64::from(*bytes), u64::from(limits.result_bytes)),
        Contract::Approval { state, part } => {
            count(RestoreBound::ResultBytes, state.len(), limits.result_bytes)?;
            match &part.key {
                crate::PartKey::Number(_) => Ok(()),
                crate::PartKey::Path(path) => {
                    bound(RestoreBound::HoldSegments, u64::from(path.len()), u64::from(limits.hold_segments))?;
                    bound(RestoreBound::HoldPathCapacity, u64::from(path.capacity()), u64::from(limits.hold_segments))?;
                    let mut bytes = 0_usize;
                    for segment in path {
                        bytes = bytes.checked_add(segment.len()).ok_or(RestoreCause::Accounting)?;
                    }
                    count(RestoreBound::HoldBytes, bytes, limits.hold_bytes)
                }
            }
        }
        Contract::Report { words } => {
            bound(RestoreBound::ResultBytes, u64::from(*words), u64::from(limits.result_bytes))
        }
        Contract::Change { words, choices, .. } => {
            bound(RestoreBound::ResultBytes, u64::from(*words), u64::from(limits.result_bytes))?;
            contract_choices(choices, limits)
        }
        Contract::Verdict { choices } => contract_choices(choices, limits),
    }
}

fn contract_choices(choices: &[crate::Verdict], limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::ContractChoices, choices.len(), limits.contract_choices)?;
    for choice in choices {
        count(RestoreBound::ResultFields, choice.fields.len(), limits.result_bytes)?;
        bound(RestoreBound::Batch, u64::from(choice.followups), u64::from(limits.batch))?;
        let mut bytes = skein_lib::length::usize_of(choice.words);
        for field in &choice.fields {
            bytes = bytes.checked_add(field.name.len()).ok_or(RestoreCause::Accounting)?;
            bytes = bytes.checked_add(skein_lib::length::usize_of(field.max)).ok_or(RestoreCause::Accounting)?;
        }
        count(RestoreBound::ResultBytes, bytes, limits.result_bytes)?;
    }
    Ok(())
}

fn fields(value: &[ResultField]) -> Result<usize, RestoreCause> {
    let mut bytes = 0_usize;
    for field in value {
        bytes = bytes.checked_add(field.name.len()).ok_or(RestoreCause::Accounting)?;
        bytes = bytes.checked_add(field.value.len()).ok_or(RestoreCause::Accounting)?;
    }
    Ok(bytes)
}

fn result(value: &TaskResult, limits: &Limits) -> Result<(), RestoreCause> {
    let bytes = match value {
        TaskResult::Job { bytes } => bytes.len(),
        TaskResult::Report { words } | TaskResult::NoChange { words } => words.len(),
        TaskResult::Change { title, words, .. } => {
            count(RestoreBound::ResultBytes, title.words.len(), limits.result_bytes)?;
            words.len()
        }
        TaskResult::Failure { reason } => reason.len(),
        TaskResult::Approved { state, words } => {
            state.len().checked_add(words.as_deref().unwrap_or(&[]).len()).ok_or(RestoreCause::Accounting)?
        }
        TaskResult::Verdict { fields: values, words, .. } => {
            count(RestoreBound::ResultFields, values.len(), limits.result_bytes)?;
            fields(values)?.checked_add(words.len()).ok_or(RestoreCause::Accounting)?
        }
    };
    count(RestoreBound::ResultBytes, bytes, limits.result_bytes)
}

fn ending(value: &Ending, limits: &Limits) -> Result<(), RestoreCause> {
    match value {
        Ending::Done(value) => result(value, limits),
        Ending::Failed { reason } => count(RestoreBound::ResultBytes, reason.len(), limits.result_bytes),
        Ending::Cancelled { reason, result: value, .. } => {
            count(RestoreBound::ResultBytes, reason.len(), limits.result_bytes)?;
            match value {
                Some(value) => result(value, limits),
                None => Ok(()),
            }
        }
    }
}

fn phase(value: &Phase, limits: &Limits) -> Result<(), RestoreCause> {
    let source = match value {
        Phase::Held { source, .. } => source.as_deref(),
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => None,
    };
    if let Some(source) = source {
        match source {
            HoldSource::Preparation { words } => {
                count(RestoreBound::PreparationReasonBytes, words.len(), crate::REASON_BYTES)?;
            }
            HoldSource::Account { words, .. } => {
                count(RestoreBound::AccountOriginBytes, words.len(), crate::hold_origin::ACCOUNT_BYTES)?;
            }
            HoldSource::Wait { words } => count(RestoreBound::WaitOriginBytes, words.len(), crate::WAIT_WORDS)?,
            HoldSource::Circle(circle) => count(RestoreBound::Circle, circle.tasks.len(), limits.circle)?,
            HoldSource::Resource(resource) => name(resource, limits)?,
            HoldSource::Effect { .. } => {}
        }
    }
    match value {
        Phase::Closing(closing) | Phase::Held { was: Was::Closing(closing), .. } => ending(&closing.ending, limits),
        Phase::Ended(ended) => ending(ended, limits),
        Phase::Waiting | Phase::Active(_) | Phase::Held { .. } => Ok(()),
    }
}

fn shape(choice: &crate::LlmChoice) -> Result<(), RestoreCause> {
    count(RestoreBound::Subagents, choice.subagents.len(), crate::MODELS)?;
    if let Some(later) = &choice.later {
        count(RestoreBound::Subagents, later.subagents.len(), crate::MODELS)?;
    }
    Ok(())
}

fn member(value: &New, limits: &Limits) -> Result<(), RestoreCause> {
    spec(&value.spec, limits)?;
    contract(&value.contract, limits)?;
    authority(&value.authority, limits)?;
    numbers(value.numbers)?;
    count(RestoreBound::Dependencies, value.dependencies.len(), limits.dependencies)?;
    count(RestoreBound::Holdings, value.holdings.len(), limits.holdings)?;
    holdings(&value.holdings, limits)?;
    if let Some(choice) = &value.llm {
        shape(choice)?;
    }
    Ok(())
}

fn batch(value: &[New], limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::Batch, value.len(), limits.batch)?;
    bound(RestoreBound::Batch, crate::batch_bytes(value).unwrap_or(u64::MAX), u64::from(limits.batch_bytes))?;
    for value in value {
        member(value, limits)?;
    }
    Ok(())
}

fn amendment(value: &Amendment, limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::MessageBytes, value.reason.len(), limits.message_bytes)?;
    if let Some(value) = &value.instructions {
        count(RestoreBound::MessageBytes, value.len(), limits.message_bytes)?;
    }
    if let Some(value) = &value.spec {
        spec(value, limits)?;
    }
    if let Some(value) = &value.authority {
        authority(value, limits)?;
    }
    if let Some(value) = &value.dependencies {
        count(RestoreBound::Dependencies, value.len(), limits.dependencies)?;
    }
    if let Some(choice) = &value.llm {
        shape(choice)?;
    }
    Ok(())
}

fn action(value: &ProposalAction, limits: &Limits) -> Result<(), RestoreCause> {
    match value {
        ProposalAction::Batch(value) => batch(value, limits),
        ProposalAction::Amend { amendment: value, .. } => amendment(value, limits),
        ProposalAction::Widen { authority: value, .. }
        | ProposalAction::Effect { authority: value, .. }
        | ProposalAction::ProcedureEffect { authority: value, .. } => authority(value, limits),
        ProposalAction::Settings { items, .. } => {
            count(RestoreBound::SettingsItems, items.len(), crate::SETTINGS_ITEMS)
        }
        ProposalAction::Release { .. } | ProposalAction::ReleasePlaced { .. } => Ok(()),
    }
}

pub(super) fn person_proposal(value: &PersonProposal, limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::MessageBytes, value.reason.len(), limits.message_bytes)?;
    match &value.state {
        crate::PersonProposalState::Rejected { reason, .. } => {
            count(RestoreBound::MessageBytes, reason.len(), limits.message_bytes)?;
        }
        crate::PersonProposalState::Pending { .. }
        | crate::PersonProposalState::SettingsAccepted { .. }
        | crate::PersonProposalState::Accepted { .. }
        | crate::PersonProposalState::Withdrawn { .. } => {}
    }
    match &value.action {
        PersonAction::Batch(value) => batch(value, limits),
        PersonAction::Amend { amendment: value, .. } => amendment(value, limits),
        PersonAction::Settings { items, .. } => count(RestoreBound::SettingsItems, items.len(), crate::SETTINGS_ITEMS),
    }
}

pub(super) fn placed(value: &crate::PlacedHold, limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::MessageBytes, value.reason.len(), limits.message_bytes)?;
    count(RestoreBound::Exemptions, value.exempt.len(), limits.exemptions)
}

fn holdings(value: &[crate::Holding], limits: &Limits) -> Result<(), RestoreCause> {
    for holding in value {
        name(crate::holds::name(holding), limits)?;
    }
    Ok(())
}

#[expect(clippy::too_many_lines, reason = "one task restore checks every durable field bound")]
pub(super) fn task(limits: &Limits, value: &TaskRecord) -> Result<(), RestoreCause> {
    if value.flags.ended_by == crate::EndedBy::Standing
        && (!crate::flags::valid(value.flags, value.executor, value.requester) || value.tracked.is_some())
    {
        return Err(RestoreCause::Accounting);
    }
    if crate::batch::matches_job(value.executor) {
        crate::batch::job_member(limits, &value.spec, &value.contract).or(Err(RestoreCause::Accounting))?;
        if !crate::flags::valid(value.flags, value.executor, value.requester)
            || value.llm.is_some()
            || value.recurring.is_some()
        {
            return Err(RestoreCause::Accounting);
        }
    } else if value.spec.job.is_some() || crate::batch::job_contract(&value.contract) {
        return Err(RestoreCause::Accounting);
    }
    if let Some(wait) = value.person_wait
        && let Some(offer) = wait.offer
        && (wait.notice.is_none()
            || !crate::control::valid_address(offer.holder)
            || match value.executor {
                Executor::Person(address) => address == offer.holder,
                Executor::Agent { .. } | Executor::Procedure { .. } | Executor::Job { .. } => true,
            })
    {
        return Err(RestoreCause::Accounting);
    }
    numbers(value.numbers)?;
    spec(&value.spec, limits)?;
    contract(&value.contract, limits)?;
    authority(&value.authority, limits)?;
    phase(&value.phase, limits)?;
    let reason = match &value.retry_notice {
        Some(crate::RetryNotice::Failed { reason, .. } | crate::RetryNotice::Refused { reason, .. }) => {
            reason.as_deref()
        }
        None => None,
    };
    if let Some(reason) = reason {
        count(RestoreBound::PreparationReasonBytes, reason.len(), crate::REASON_BYTES)?;
    }
    count(RestoreBound::Delegates, value.delegates.len(), limits.delegates)?;
    count(RestoreBound::Batch, value.result_followups.len(), limits.batch)?;
    count(RestoreBound::References, value.references.len(), limits.references)?;
    count(RestoreBound::Dependencies, value.dependencies.len(), limits.dependencies)?;
    count(RestoreBound::Dependencies, value.waiting_on.len(), limits.dependencies)?;
    count(RestoreBound::Dependencies, value.added.len(), limits.dependencies)?;
    count(RestoreBound::Dependencies, value.started_from.len(), limits.dependencies)?;
    count(RestoreBound::InboxMessages, value.questions.len(), limits.inbox_messages)?;
    for credit in &value.questions {
        if let Some(offer) = &credit.offer {
            match (&offer.holder, &offer.relay) {
                (crate::EscalationHolder::Task(holder), Some(relay)) if *holder != 0 && relay.entry > credit.number => {
                    count(RestoreBound::MessageBytes, relay.words.len(), limits.message_bytes)?;
                }
                (crate::EscalationHolder::Person(person), None) if *person != 0 => {}
                (crate::EscalationHolder::Role { project, .. }, None) if *project == value.project => {}
                _ => return Err(RestoreCause::Accounting),
            }
        }
    }
    for line in &value.question_lines {
        if let Some(resolution) = &line.resolution {
            if line.answer.is_none() {
                return Err(RestoreCause::Accounting);
            }
            if let Some(notice) = &resolution.notice {
                bound(
                    RestoreBound::MessageBytes,
                    len_u64(notice.words.len()),
                    u64::try_from(crate::questions::ANSWER_NOTICE_BYTES).expect("question notice bound fits"),
                )?;
            }
        }
    }
    count(RestoreBound::InboxMessages, value.question_lines.len(), limits.inbox_messages)?;
    count(RestoreBound::InboxMessages, value.proposal_rows.len(), limits.inbox_messages)?;
    count(RestoreBound::InboxMessages, value.replies.len(), limits.inbox_messages)?;
    count(RestoreBound::SavedResources, value.saved.len(), limits.saved_resources)?;
    let refused = u64::from(limits.tasks)
        .checked_mul(2)
        .ok_or(RestoreCause::Accounting)?
        .checked_add(u64::from(limits.inbox_messages))
        .ok_or(RestoreCause::Accounting)?
        .checked_add(3)
        .ok_or(RestoreCause::Accounting)?;
    bound(RestoreBound::RefusedRows, len_u64(value.refused.len()), refused)?;
    let (which, holding_bound) = if value.root == value.number {
        (
            RestoreBound::RootHoldings,
            u64::from(limits.tasks).checked_mul(u64::from(limits.holdings)).ok_or(RestoreCause::Accounting)?,
        )
    } else {
        (RestoreBound::Holdings, u64::from(limits.holdings))
    };
    bound(which, len_u64(value.holdings.len()), holding_bound)?;
    holdings(&value.holdings, limits)?;
    for saved in &value.saved {
        count(RestoreBound::AuthoritySegments, saved.path.len(), limits.authority_segments)?;
        let mut bytes = 0_usize;
        for segment in &saved.path {
            bytes = bytes.checked_add(segment.len()).ok_or(RestoreCause::Accounting)?;
        }
        count(RestoreBound::AuthorityBytes, bytes, limits.authority_bytes)?;
    }
    if let Some(instructions) = &value.instructions {
        count(RestoreBound::MessageBytes, instructions.len(), limits.message_bytes)?;
    }
    if let Some(grace) = &value.grace {
        count(RestoreBound::ResultBytes, grace.reason.len(), limits.result_bytes)?;
    }
    match &value.escalation {
        crate::Escalation::Rejected { reason, .. } => {
            count(RestoreBound::ResultBytes, reason.len(), limits.result_bytes)?;
        }
        crate::Escalation::Waiting { .. } | crate::Escalation::Unheld { .. } | crate::Escalation::Routing { .. } => {}
    }
    if let Some(proposal) = &value.proposal {
        count(RestoreBound::MessageBytes, proposal.reason.len(), limits.message_bytes)?;
        action(&proposal.action, limits)?;
        match &proposal.state {
            crate::ProposalState::Rejected { reason, .. } => {
                count(RestoreBound::MessageBytes, reason.len(), limits.message_bytes)?;
            }
            crate::ProposalState::Pending { .. }
            | crate::ProposalState::Accepted { .. }
            | crate::ProposalState::Withdrawn => {}
        }
    }
    if let Some(state) = &value.recurring {
        count(RestoreBound::Batch, state.last_batch.len(), limits.batch)?;
        batch(&state.template.batch, limits)?;
    }
    if let Some(choice) = &value.llm {
        shape(choice)?;
    }
    inbox(value, limits)
}

fn payload(word: &crate::Word, limits: &Limits) -> Result<usize, RestoreCause> {
    match word.kind.result() {
        Some(ResultKind::Approved { state, .. }) => {
            state.len().checked_add(word.words.len()).ok_or(RestoreCause::Accounting)
        }
        Some(ResultKind::Verdict { fields: values, .. }) => {
            count(RestoreBound::ResultFields, values.len(), limits.result_bytes)?;
            fields(values)?.checked_add(word.words.len()).ok_or(RestoreCause::Accounting)
        }
        Some(
            ResultKind::Job
            | ResultKind::NoChange
            | ResultKind::Report
            | ResultKind::Change { .. }
            | ResultKind::Failed
            | ResultKind::Cancelled,
        )
        | None => Ok(word.words.len()),
    }
}

fn ordinary(kind: &MessageKind) -> bool {
    match kind.family() {
        MessageFamily::Separate(_) => false,
        MessageFamily::Slot(_)
        | MessageFamily::Spoken(_)
        | MessageFamily::Hint(_)
        | MessageFamily::Result
        | MessageFamily::Virtual => true,
    }
}

fn payload_bound(value: &TaskRecord, word: &crate::Word, limits: &Limits) -> (RestoreBound, u32) {
    if crate::inbox::finish_word(value, word.number) {
        return (RestoreBound::ResultBytes, limits.result_bytes);
    }
    match word.kind {
        MessageKind::QuestionAnswered { .. } => (
            RestoreBound::MessageBytes,
            u32::try_from(crate::questions::ANSWER_NOTICE_BYTES).expect("fixed answer notice"),
        ),
        MessageKind::QuestionOwed => {
            (RestoreBound::ReminderBytes, u32::try_from(crate::questions::OWED_WORDS.len()).expect("fixed reminder"))
        }
        MessageKind::NothingAwaited => (
            RestoreBound::ReminderBytes,
            u32::try_from(crate::run::NOTHING_AWAITED_WORDS.len()).expect("fixed reminder"),
        ),
        MessageKind::Words
        | MessageKind::Question
        | MessageKind::Answer { .. }
        | MessageKind::Amendment { .. }
        | MessageKind::FinishRequested { .. }
        | MessageKind::SettingsDecision { .. }
        | MessageKind::ProposalDecision { .. } => (RestoreBound::MessageBytes, limits.message_bytes),
        MessageKind::Result(_) => (RestoreBound::ResultBytes, limits.result_bytes),
        MessageKind::Notice { .. } | MessageKind::News { .. } => {
            if limits.result_bytes >= limits.message_bytes {
                (RestoreBound::ResultBytes, limits.result_bytes)
            } else {
                (RestoreBound::MessageBytes, limits.message_bytes)
            }
        }
        MessageKind::EffectNotMade { .. }
        | MessageKind::PassedQuestion { .. }
        | MessageKind::PassedAnswer { .. }
        | MessageKind::Escalation { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::PersonStalled { .. }
        | MessageKind::Met { .. }
        | MessageKind::QuestionEnded { .. }
        | MessageKind::Timer { .. } => (RestoreBound::ZeroBytes, 0),
    }
}

fn messages(value: &TaskRecord, ordinary: usize, subscriptions: usize) -> Result<usize, RestoreCause> {
    ordinary
        .checked_add(value.delegates.len())
        .ok_or(RestoreCause::Accounting)?
        .checked_add(value.questions.len())
        .ok_or(RestoreCause::Accounting)?
        .checked_add(crate::inbox::received_credits(value.requester, &value.question_lines, &value.questions))
        .ok_or(RestoreCause::Accounting)?
        .checked_add(usize::from(value.proposal.is_some()))
        .ok_or(RestoreCause::Accounting)?
        .checked_add(subscriptions)
        .ok_or(RestoreCause::Accounting)
}

fn inbox(value: &TaskRecord, limits: &Limits) -> Result<(), RestoreCause> {
    let (subscriptions, subscription_bytes) =
        crate::subscriptions::credit(value, limits).ok_or(RestoreCause::Accounting)?;
    let mut bytes = 0_usize;
    let mut amendments = 0_usize;
    let mut met = 0_usize;
    let mut ordinary_count = 0_usize;
    for word in value.inbox.iter().chain(value.unrecorded.iter()) {
        let size = payload(word, limits)?;
        let (which, limit) = payload_bound(value, word, limits);
        count(which, size, limit)?;
        if ordinary(&word.kind) {
            ordinary_count = ordinary_count.checked_add(1).ok_or(RestoreCause::Accounting)?;
            bytes = bytes.checked_add(size).ok_or(RestoreCause::Accounting)?;
        } else {
            match word.kind.family() {
                MessageFamily::Separate(Separate::Met | Separate::NothingAwaited) => {
                    met = met.checked_add(1).ok_or(RestoreCause::Accounting)?;
                }
                MessageFamily::Separate(Separate::Amendment | Separate::FinishRequested) => {
                    amendments = amendments.checked_add(1).ok_or(RestoreCause::Accounting)?;
                }
                MessageFamily::Slot(_)
                | MessageFamily::Spoken(_)
                | MessageFamily::Hint(_)
                | MessageFamily::Result
                | MessageFamily::Virtual => {}
            }
        }
    }
    count(RestoreBound::AmendmentSlots, amendments, 2)?;
    count(RestoreBound::MetSlots, met, 1)?;
    count(RestoreBound::InboxMessages, messages(value, ordinary_count, subscriptions)?, limits.inbox_messages)?;
    let reserved = value
        .delegates
        .len()
        .checked_mul(skein_lib::length::usize_of(limits.result_bytes))
        .ok_or(RestoreCause::Accounting)?;
    let answers = value
        .questions
        .len()
        .checked_mul(skein_lib::length::usize_of(limits.message_bytes))
        .ok_or(RestoreCause::Accounting)?;
    let proposal = if value.proposal.is_some() { skein_lib::length::usize_of(limits.message_bytes) } else { 0 };
    let notices = crate::inbox::received_credits(value.requester, &value.question_lines, &value.questions)
        .checked_mul(crate::questions::ANSWER_NOTICE_BYTES)
        .ok_or(RestoreCause::Accounting)?;
    let total = bytes
        .checked_add(notices)
        .ok_or(RestoreCause::Accounting)?
        .checked_add(reserved)
        .ok_or(RestoreCause::Accounting)?
        .checked_add(answers)
        .ok_or(RestoreCause::Accounting)?
        .checked_add(proposal)
        .ok_or(RestoreCause::Accounting)?
        .checked_add(subscription_bytes)
        .ok_or(RestoreCause::Accounting)?;
    count(RestoreBound::InboxBytes, total, limits.inbox_bytes)
}

pub(super) fn linked_inbox(domain: &Domain, limits: &Limits, value: &TaskRecord) -> Result<(), RestoreCause> {
    let mut stalls = 0_usize;
    for child in &value.delegates {
        if crate::person::credit(domain.task(*child).expect("delegate linked")) {
            stalls = stalls.checked_add(1).ok_or(RestoreCause::Accounting)?;
        }
    }
    let mut ordinary_count = 0_usize;
    for word in value.inbox.iter().chain(value.unrecorded.iter()) {
        if ordinary(&word.kind) {
            ordinary_count = ordinary_count.checked_add(1).ok_or(RestoreCause::Accounting)?;
        }
    }
    let (subscriptions, _) = crate::subscriptions::credit(value, limits).ok_or(RestoreCause::Accounting)?;
    let total = messages(value, ordinary_count, subscriptions)?.checked_add(stalls).ok_or(RestoreCause::Accounting)?;
    count(RestoreBound::InboxMessages, total, limits.inbox_messages)
}

pub(super) fn settings(domain: &Domain, value: &TaskRecord) -> Result<(), RestoreCause> {
    match value.executor {
        Executor::Agent { .. } => {}
        Executor::Procedure { .. } | Executor::Person(_) | Executor::Job { .. } => return Ok(()),
    }
    let mut extra = u64::from(!domain.holders.contains_key(&value.settings.task));
    if let Some(run) = value.settings.run
        && run != value.settings.task
        && !domain.holders.contains_key(&run)
    {
        extra = extra.checked_add(1).ok_or(RestoreCause::Accounting)?;
    }
    let held = u64::from(domain.holders.len()).checked_add(extra).ok_or(RestoreCause::Accounting)?;
    bound(RestoreBound::HeldRevisions, held, u64::from(domain.holders.capacity()))
}

pub(super) fn choice(domain: &Domain, value: &TaskRecord) -> Result<(), RestoreCause> {
    let Some(choice) = &value.llm else { return Ok(()) };
    let charter = match value.executor {
        Executor::Agent { charter } => charter,
        Executor::Procedure { .. } | Executor::Person(_) | Executor::Job { .. } => return Ok(()),
    };
    let models = domain.charter_models(charter, value.settings.task).expect("charter linked");
    model(domain, models, choice.main)?;
    for chosen in &choice.subagents {
        model(domain, models, *chosen)?;
    }
    if let Some(later) = &choice.later {
        model(domain, models, later.main)?;
        for chosen in &later.subagents {
            model(domain, models, *chosen)?;
        }
    }
    Ok(())
}

fn model(domain: &Domain, models: &[u8], chosen: crate::ModelEffort) -> Result<(), RestoreCause> {
    let Some(efforts) = domain.efforts.get(usize::from(chosen.model)) else {
        return Err(RestoreCause::LlmChoice { chosen, why: RestoreLlmWhy::Unconfigured });
    };
    if !models.contains(&chosen.model) {
        return Err(RestoreCause::LlmChoice { chosen, why: RestoreLlmWhy::Charter });
    }
    let fits = match chosen.effort {
        Some(effort) => effort < *efforts,
        None => *efforts == 0,
    };
    if !fits {
        return Err(RestoreCause::LlmChoice { chosen, why: RestoreLlmWhy::Effort });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::phase;
    use crate::{Ending, HoldSource, Phase, TaskResult, Was};
    use skein_lib::{List, Wall, bytes::zeroed, length::usize_of};

    #[test]
    fn the_phase_bound_is_attained_by_admitted_closing_holds() {
        let base = crate::tests::revisions::LIMITS;
        for limits in [
            crate::Limits { result_bytes: 0, ..base },
            crate::Limits { result_bytes: 64, ..base },
            crate::Limits { result_bytes: 1024, hold_bytes: 4096, ..base },
            crate::Limits { result_bytes: 64, circle: 1024, ..base },
        ] {
            let mut fields = List::with_capacity(limits.result_bytes.min(crate::VERDICT_FIELDS));
            for _ in 0..fields.capacity() {
                fields.push(crate::ResultField { name: Box::new(*b"x"), value: Box::new([]) }).expect("field room");
            }
            let remaining = limits.result_bytes.checked_sub(fields.len()).expect("field names fit result");
            let results = [
                TaskResult::Verdict { code: 1, fields: fields.into_boxed(), words: zeroed(usize_of(remaining)) },
                TaskResult::Change {
                    connector: 1,
                    kind: 1,
                    resource: 1,
                    title: crate::ChangeTitle::new(zeroed(usize_of(limits.result_bytes))),
                    words: zeroed(usize_of(limits.result_bytes)),
                },
            ];
            let mut path = List::with_capacity(limits.hold_segments);
            for at in 0..limits.hold_segments {
                path.push(zeroed(if at == 0 { usize_of(limits.hold_bytes) } else { 0 })).expect("path room");
            }
            let mut circle = List::with_capacity(limits.circle);
            for number in 0..u64::from(limits.circle) {
                circle.push(number).expect("circle room");
            }
            let mut largest = 0;
            for result in results {
                for source in [
                    HoldSource::Preparation { words: zeroed(usize_of(crate::REASON_BYTES)) },
                    HoldSource::Account {
                        connector: 1,
                        entry: None,
                        words: zeroed(usize_of(crate::HOLD_ACCOUNT_BYTES)),
                    },
                    HoldSource::Resource(crate::Name { connector: 1, path: path.as_slice().into() }),
                    HoldSource::Circle(crate::Circle { tasks: circle.as_slice().into(), cut: false }),
                    HoldSource::Effect { connector: 1, entry: 1 },
                ] {
                    let value = Phase::Held {
                        was: Was::Closing(crate::Closing {
                            stage: crate::Stage::Effects,
                            ending: Ending::Cancelled {
                                by: crate::Party::Person(1),
                                at: Wall::EPOCH,
                                reason: zeroed(usize_of(limits.result_bytes)),
                                result: Some(result.clone()),
                            },
                        }),
                        why: crate::Hold::Stopped,
                        source: Some(Box::new(source)),
                    };
                    assert_eq!(phase(&value, &limits), Ok(()));
                    largest = largest.max(crate::phase_bytes(&value).expect("admitted phase accounted"));
                    assert!(largest <= crate::phase_bound(&limits).expect("bounded limits"));
                }
            }
            assert_eq!(Some(largest), crate::phase_bound(&limits));
        }
    }
}
