//! Configuration and retained-value bounds at load (jig/docs/design/domain/engine.md, 6).
use crate::{
    Amendment, Authority, Contract, Domain, Ending, Executor, HoldSource, Last, Limits, MessageKind, New, Numbers,
    Parameter, PersonAction, PersonProposal, Phase, ProposalAction, RestoreBound, RestoreCause, RestoreLlmWhy,
    ResultField, ResultKind, Spec, TaskRecord, TaskResult, Was,
};

pub(crate) fn bound(which: RestoreBound, held: u64, limit: u64) -> Result<(), RestoreCause> {
    if held > limit { Err(RestoreCause::Bound { which, held, limit }) } else { Ok(()) }
}

fn count(which: RestoreBound, len: usize, limit: u32) -> Result<(), RestoreCause> {
    bound(which, u64::try_from(len).or(Err(RestoreCause::Accounting))?, u64::from(limit))
}

pub(crate) fn room(which: RestoreBound, len: u32, limit: u32) -> Result<(), RestoreCause> {
    bound(which, u64::from(len).checked_add(1).ok_or(RestoreCause::Accounting)?, u64::from(limit))
}

pub(crate) fn numbers(value: Numbers) -> Result<(), RestoreCause> {
    let spent = crate::funders::total(value).ok_or(RestoreCause::Accounting)?;
    spent.checked_add(value.reserved).ok_or(RestoreCause::Accounting)?;
    Ok(())
}

fn spec(value: &Spec, limits: &Limits) -> Result<(), RestoreCause> {
    count(RestoreBound::Parameters, value.parameters.len(), limits.parameters)?;
    count(RestoreBound::Inputs, value.inputs.len(), limits.inputs)?;
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
        Contract::Report { words } | Contract::Change { words, .. } => {
            bound(RestoreBound::ResultBytes, u64::from(*words), u64::from(limits.result_bytes))
        }
        Contract::Verdict { choices } => {
            count(RestoreBound::ContractChoices, choices.len(), limits.contract_choices)?;
            for choice in choices {
                count(RestoreBound::ResultFields, choice.fields.len(), limits.result_bytes)?;
                bound(RestoreBound::Batch, u64::from(choice.followups), u64::from(limits.batch))?;
                let mut bytes = usize::try_from(choice.words).expect("u32 fits usize");
                for field in &choice.fields {
                    bytes = bytes.checked_add(field.name.len()).ok_or(RestoreCause::Accounting)?;
                    bytes = bytes
                        .checked_add(usize::try_from(field.max).expect("u32 fits usize"))
                        .ok_or(RestoreCause::Accounting)?;
                }
                count(RestoreBound::ResultBytes, bytes, limits.result_bytes)?;
            }
            Ok(())
        }
    }
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
        TaskResult::Report { words } | TaskResult::Change { words, .. } => words.len(),
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
            HoldSource::Account { words, .. } => {
                count(RestoreBound::AccountOriginBytes, words.len(), crate::hold_origin::ACCOUNT_BYTES)?;
            }
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
    count(RestoreBound::Subagents, choice.subagents.len(), crate::MODELS)
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

pub(super) fn task(limits: &Limits, value: &TaskRecord) -> Result<(), RestoreCause> {
    numbers(value.numbers)?;
    spec(&value.spec, limits)?;
    contract(&value.contract, limits)?;
    authority(&value.authority, limits)?;
    phase(&value.phase, limits)?;
    count(RestoreBound::Delegates, value.delegates.len(), limits.delegates)?;
    count(RestoreBound::Batch, value.result_followups.len(), limits.batch)?;
    count(RestoreBound::References, value.references.len(), limits.references)?;
    count(RestoreBound::Subscriptions, value.subscriptions.len(), limits.subscriptions)?;
    count(RestoreBound::Dependencies, value.dependencies.len(), limits.dependencies)?;
    count(RestoreBound::Dependencies, value.waiting_on.len(), limits.dependencies)?;
    count(RestoreBound::Dependencies, value.added.len(), limits.dependencies)?;
    count(RestoreBound::InboxMessages, value.questions.len(), limits.inbox_messages)?;
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
    bound(RestoreBound::RefusedRows, u64::try_from(value.refused.len()).or(Err(RestoreCause::Accounting))?, refused)?;
    let (which, holding_bound) = if value.root == value.number {
        (
            RestoreBound::RootHoldings,
            u64::from(limits.tasks).checked_mul(u64::from(limits.holdings)).ok_or(RestoreCause::Accounting)?,
        )
    } else {
        (RestoreBound::Holdings, u64::from(limits.holdings))
    };
    bound(which, u64::try_from(value.holdings.len()).or(Err(RestoreCause::Accounting))?, holding_bound)?;
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
    match &word.kind {
        MessageKind::Result(ResultKind::Approved { state, .. }) => {
            state.len().checked_add(word.words.len()).ok_or(RestoreCause::Accounting)
        }
        MessageKind::Result(ResultKind::Verdict { fields: values, .. }) => {
            count(RestoreBound::ResultFields, values.len(), limits.result_bytes)?;
            fields(values)?.checked_add(word.words.len()).ok_or(RestoreCause::Accounting)
        }
        MessageKind::Result(
            ResultKind::Report | ResultKind::Change { .. } | ResultKind::Failed | ResultKind::Cancelled,
        )
        | MessageKind::Words
        | MessageKind::Question
        | MessageKind::Answer { .. }
        | MessageKind::EffectNotMade { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::Escalation { .. }
        | MessageKind::SettingsDecision { .. }
        | MessageKind::ProposalDecision { .. }
        | MessageKind::Amendment { .. }
        | MessageKind::FinishRequested { .. }
        | MessageKind::Met { .. }
        | MessageKind::QuestionAnswered { .. }
        | MessageKind::QuestionOwed
        | MessageKind::QuestionEnded { .. }
        | MessageKind::Notice { .. }
        | MessageKind::Timer { .. }
        | MessageKind::News { .. }
        | MessageKind::PersonStalled { .. } => Ok(word.words.len()),
    }
}

fn ordinary(kind: &MessageKind) -> bool {
    match kind {
        MessageKind::Amendment { .. } | MessageKind::FinishRequested { .. } | MessageKind::Met { .. } => false,
        MessageKind::Words
        | MessageKind::Question
        | MessageKind::Answer { .. }
        | MessageKind::EffectNotMade { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::Escalation { .. }
        | MessageKind::SettingsDecision { .. }
        | MessageKind::ProposalDecision { .. }
        | MessageKind::QuestionAnswered { .. }
        | MessageKind::QuestionOwed
        | MessageKind::QuestionEnded { .. }
        | MessageKind::Notice { .. }
        | MessageKind::Timer { .. }
        | MessageKind::News { .. }
        | MessageKind::PersonStalled { .. }
        | MessageKind::Result(_) => true,
    }
}

fn payload_bound(value: &TaskRecord, word: &crate::Word, limits: &Limits) -> (RestoreBound, u32) {
    if crate::inbox::finish_word(value, word.number) {
        return (RestoreBound::ResultBytes, limits.result_bytes);
    }
    match word.kind {
        MessageKind::QuestionOwed => {
            (RestoreBound::ReminderBytes, u32::try_from(crate::questions::OWED_WORDS.len()).expect("fixed reminder"))
        }
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
        | MessageKind::Escalation { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::PersonStalled { .. }
        | MessageKind::Met { .. }
        | MessageKind::QuestionAnswered { .. }
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
        .checked_add(crate::inbox::received_credits(
            value.requester,
            &value.question_lines,
            &value.questions,
            &value.inbox,
        ))
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
    for word in &value.inbox {
        let size = payload(word, limits)?;
        let (which, limit) = payload_bound(value, word, limits);
        count(which, size, limit)?;
        if ordinary(&word.kind) {
            ordinary_count = ordinary_count.checked_add(1).ok_or(RestoreCause::Accounting)?;
            bytes = bytes.checked_add(size).ok_or(RestoreCause::Accounting)?;
        } else {
            match word.kind {
                MessageKind::Met { .. } => met = met.checked_add(1).ok_or(RestoreCause::Accounting)?,
                MessageKind::Amendment { .. } | MessageKind::FinishRequested { .. } => {
                    amendments = amendments.checked_add(1).ok_or(RestoreCause::Accounting)?;
                }
                MessageKind::Words
                | MessageKind::Question
                | MessageKind::Answer { .. }
                | MessageKind::EffectNotMade { .. }
                | MessageKind::Proposal { .. }
                | MessageKind::Escalation { .. }
                | MessageKind::SettingsDecision { .. }
                | MessageKind::ProposalDecision { .. }
                | MessageKind::QuestionAnswered { .. }
                | MessageKind::QuestionOwed
                | MessageKind::QuestionEnded { .. }
                | MessageKind::Notice { .. }
                | MessageKind::Timer { .. }
                | MessageKind::News { .. }
                | MessageKind::PersonStalled { .. }
                | MessageKind::Result(_) => {}
            }
        }
    }
    count(RestoreBound::AmendmentSlots, amendments, 2)?;
    count(RestoreBound::MetSlots, met, 1)?;
    count(RestoreBound::InboxMessages, messages(value, ordinary_count, subscriptions)?, limits.inbox_messages)?;
    let reserved = value
        .delegates
        .len()
        .checked_mul(usize::try_from(limits.result_bytes).expect("byte bound"))
        .ok_or(RestoreCause::Accounting)?;
    let answers = value
        .questions
        .len()
        .checked_mul(usize::try_from(limits.message_bytes).expect("byte bound"))
        .ok_or(RestoreCause::Accounting)?;
    let proposal =
        if value.proposal.is_some() { usize::try_from(limits.message_bytes).expect("byte bound") } else { 0 };
    let total = bytes
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
    for word in &value.inbox {
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
        Executor::Procedure { .. } | Executor::Person(_) => return Ok(()),
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
        Executor::Procedure { .. } | Executor::Person(_) => return Ok(()),
    };
    let models = domain.charter_models(charter, value.settings.task).expect("charter linked");
    model(domain, models, choice.main)?;
    for chosen in &choice.subagents {
        model(domain, models, *chosen)?;
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
