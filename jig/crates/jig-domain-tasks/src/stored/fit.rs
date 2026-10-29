//! Configuration and retained-value bounds at load (jig/docs/design/domain/engine.md, 6).
use crate::{
    Amendment, Authority, Contract, Domain, Ending, Executor, HoldSource, Last, Limits, MessageKind, New, Numbers,
    Parameter, PersonAction, PersonProposal, Phase, ProposalAction, ResultField, ResultKind, Spec, TaskRecord,
    TaskResult, Was,
};

fn count(len: usize, limit: u32) -> bool {
    len <= usize::try_from(limit).expect("u32 fits usize")
}

pub(crate) fn numbers(value: Numbers) -> bool {
    let Some(spent) = crate::funders::total(value) else { return false };
    spent.checked_add(value.reserved).is_some()
}

fn spec(value: &Spec, limits: &Limits) -> bool {
    if !count(value.parameters.len(), limits.parameters) || !count(value.inputs.len(), limits.inputs) {
        return false;
    }
    let mut bytes = value.words.len();
    for parameter in &value.parameters {
        match parameter {
            Parameter::Bytes { value, .. } => {
                let Some(total) = bytes.checked_add(value.len()) else { return false };
                bytes = total;
            }
            Parameter::Number { .. } | Parameter::Resource { .. } => {}
        }
    }
    count(bytes, limits.spec_bytes) && (!value.asks || count(crate::questions::OWED_WORDS.len(), limits.inbox_bytes))
}

fn authority(value: &Authority, limits: &Limits) -> bool {
    if !count(value.grants.len(), limits.authority_grants)
        || !count(value.note_resources.len(), limits.authority_grants)
        || !count(value.delegation.kinds.len(), limits.executor_kinds)
    {
        return false;
    }
    let mut bytes = 0_usize;
    for grant in &value.grants {
        let Some(size) = pattern(&grant.pattern, limits) else { return false };
        let Some(total) = bytes.checked_add(size) else { return false };
        bytes = total;
    }
    for scope in &value.note_resources {
        let Some(size) = pattern(&scope.pattern, limits) else { return false };
        let Some(total) = bytes.checked_add(size) else { return false };
        bytes = total;
    }
    count(bytes, limits.authority_bytes)
}

fn pattern(value: &crate::Pattern, limits: &Limits) -> Option<usize> {
    if !count(value.segments.len(), limits.authority_segments) {
        return None;
    }
    let mut bytes = 0_usize;
    for segment in &value.segments {
        bytes = bytes.checked_add(segment.len())?;
    }
    match &value.last {
        Last::Exact(last) | Last::Open(last) => bytes.checked_add(last.len()),
    }
}

fn contract(value: &Contract, limits: &Limits) -> bool {
    match value {
        Contract::Approval { state, part } => {
            count(state.len(), limits.result_bytes)
                && match &part.key {
                    crate::PartKey::Number(_) => true,
                    crate::PartKey::Path(path) => {
                        if path.len() > limits.hold_segments || path.capacity() > limits.hold_segments {
                            return false;
                        }
                        let mut bytes = 0_usize;
                        for segment in path {
                            let Some(total) = bytes.checked_add(segment.len()) else { return false };
                            bytes = total;
                        }
                        count(bytes, limits.hold_bytes)
                    }
                }
        }
        Contract::Report { words } | Contract::Change { words, .. } => *words <= limits.result_bytes,
        Contract::Verdict { choices } => {
            if !count(choices.len(), limits.contract_choices) {
                return false;
            }
            for choice in choices {
                if !count(choice.fields.len(), limits.result_bytes) || choice.followups > limits.batch {
                    return false;
                }
                let mut bytes = usize::try_from(choice.words).expect("u32 fits usize");
                for field in &choice.fields {
                    let Some(total) = bytes.checked_add(field.name.len()) else { return false };
                    let Some(total) = total.checked_add(usize::try_from(field.max).expect("u32 fits usize")) else {
                        return false;
                    };
                    bytes = total;
                }
                if !count(bytes, limits.result_bytes) {
                    return false;
                }
            }
            true
        }
    }
}

fn fields(value: &[ResultField]) -> Option<usize> {
    let mut bytes = 0_usize;
    for field in value {
        bytes = bytes.checked_add(field.name.len())?.checked_add(field.value.len())?;
    }
    Some(bytes)
}

fn result(value: &TaskResult, limits: &Limits) -> bool {
    let bytes = match value {
        TaskResult::Report { words } | TaskResult::Change { words, .. } => Some(words.len()),
        TaskResult::Failure { reason } => Some(reason.len()),
        TaskResult::Approved { state, words } => state.len().checked_add(words.as_deref().unwrap_or(&[]).len()),
        TaskResult::Verdict { fields: values, words, .. } => {
            if !count(values.len(), limits.result_bytes) {
                return false;
            }
            match fields(values) {
                Some(size) => size.checked_add(words.len()),
                None => None,
            }
        }
    };
    match bytes {
        Some(bytes) => count(bytes, limits.result_bytes),
        None => false,
    }
}

fn ending(value: &Ending, limits: &Limits) -> bool {
    match value {
        Ending::Done(value) => result(value, limits),
        Ending::Failed { reason } => count(reason.len(), limits.result_bytes),
        Ending::Cancelled { reason, result: value, .. } => {
            count(reason.len(), limits.result_bytes)
                && match value {
                    Some(value) => result(value, limits),
                    None => true,
                }
        }
    }
}

fn phase(value: &Phase, limits: &Limits) -> bool {
    let source = match value {
        Phase::Held { source, .. } => source.as_deref(),
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => None,
    };
    if let Some(source) = source {
        let fits = match source {
            HoldSource::Account { words, .. } => count(words.len(), crate::hold_origin::ACCOUNT_BYTES),
            HoldSource::Circle(circle) => count(circle.tasks.len(), limits.circle),
            HoldSource::Resource(name) => crate::holds::valid_name(limits, name),
            HoldSource::Effect { .. } => true,
        };
        if !fits {
            return false;
        }
    }
    match value {
        Phase::Closing(closing) | Phase::Held { was: Was::Closing(closing), .. } => ending(&closing.ending, limits),
        Phase::Ended(ended) => ending(ended, limits),
        Phase::Waiting | Phase::Active(_) | Phase::Held { .. } => true,
    }
}

fn member(value: &New, limits: &Limits) -> bool {
    spec(&value.spec, limits)
        && contract(&value.contract, limits)
        && authority(&value.authority, limits)
        && numbers(value.numbers)
        && count(value.dependencies.len(), limits.dependencies)
        && count(value.holdings.len(), limits.holdings)
        && holdings(&value.holdings, limits)
        && match &value.llm {
            Some(choice) => crate::llm::valid_shape(choice),
            None => true,
        }
}

fn batch(value: &[New], limits: &Limits) -> bool {
    if !count(value.len(), limits.batch) {
        return false;
    }
    for value in value {
        if !member(value, limits) {
            return false;
        }
    }
    true
}

fn amendment(value: &Amendment, limits: &Limits) -> bool {
    if !count(value.reason.len(), limits.message_bytes) {
        return false;
    }
    if let Some(value) = &value.instructions
        && !count(value.len(), limits.message_bytes)
    {
        return false;
    }
    if let Some(value) = &value.spec
        && !spec(value, limits)
    {
        return false;
    }
    if let Some(value) = &value.authority
        && !authority(value, limits)
    {
        return false;
    }
    if let Some(value) = &value.dependencies
        && !count(value.len(), limits.dependencies)
    {
        return false;
    }
    if let Some(choice) = &value.llm
        && !crate::llm::valid_shape(choice)
    {
        return false;
    }
    true
}

fn action(value: &ProposalAction, limits: &Limits) -> bool {
    match value {
        ProposalAction::Batch(value) => batch(value, limits),
        ProposalAction::Amend { amendment: value, .. } => amendment(value, limits),
        ProposalAction::Widen { authority: value, .. }
        | ProposalAction::Effect { authority: value, .. }
        | ProposalAction::ProcedureEffect { authority: value, .. } => authority(value, limits),
        ProposalAction::Settings { items, .. } => count(items.len(), crate::SETTINGS_ITEMS),
        ProposalAction::Release { .. } | ProposalAction::ReleasePlaced { .. } => true,
    }
}

pub(super) fn person_proposal(value: &PersonProposal, limits: &Limits) -> bool {
    if !count(value.reason.len(), limits.message_bytes) {
        return false;
    }
    match &value.state {
        crate::PersonProposalState::Rejected { reason, .. } if !count(reason.len(), limits.message_bytes) => {
            return false;
        }
        crate::PersonProposalState::Rejected { .. }
        | crate::PersonProposalState::Pending { .. }
        | crate::PersonProposalState::SettingsAccepted { .. }
        | crate::PersonProposalState::Accepted { .. }
        | crate::PersonProposalState::Withdrawn { .. } => {}
    }
    match &value.action {
        PersonAction::Batch(value) => batch(value, limits),
        PersonAction::Amend { amendment: value, .. } => amendment(value, limits),
        PersonAction::Settings { items, .. } => count(items.len(), crate::SETTINGS_ITEMS),
    }
}

pub(super) fn placed(value: &crate::PlacedHold, limits: &Limits) -> bool {
    count(value.reason.len(), limits.message_bytes) && count(value.exempt.len(), limits.exemptions)
}

fn holdings(value: &[crate::Holding], limits: &Limits) -> bool {
    for holding in value {
        if !crate::holds::valid_name(limits, crate::holds::name(holding)) {
            return false;
        }
    }
    true
}

pub(super) fn task(limits: &Limits, value: &TaskRecord) -> bool {
    if !numbers(value.numbers)
        || !spec(&value.spec, limits)
        || !contract(&value.contract, limits)
        || !authority(&value.authority, limits)
        || !phase(&value.phase, limits)
        || !count(value.delegates.len(), limits.delegates)
        || !count(value.result_followups.len(), limits.batch)
        || !count(value.references.len(), limits.references)
        || !count(value.subscriptions.len(), limits.subscriptions)
        || !count(value.dependencies.len(), limits.dependencies)
        || !count(value.waiting_on.len(), limits.dependencies)
        || !count(value.added.len(), limits.dependencies)
        || !count(value.questions.len(), limits.inbox_messages)
        || !count(value.question_lines.len(), limits.inbox_messages)
        || !count(value.proposal_rows.len(), limits.inbox_messages)
        || !count(value.replies.len(), limits.inbox_messages)
        || !count(value.saved.len(), limits.saved_resources)
        || value.refused.len()
            > usize::try_from(limits.inbox_messages)
                .expect("message count")
                .saturating_add(usize::try_from(limits.tasks).expect("task count").saturating_mul(2))
                .saturating_add(3)
    {
        return false;
    }
    let holding_bound = if value.root == value.number {
        limits.tasks.checked_mul(limits.holdings).expect("live holdings bound")
    } else {
        limits.holdings
    };
    if !count(value.holdings.len(), holding_bound) || !holdings(&value.holdings, limits) {
        return false;
    }
    for saved in &value.saved {
        if !count(saved.path.len(), limits.authority_segments) {
            return false;
        }
        let mut bytes = 0_usize;
        for segment in &saved.path {
            let Some(total) = bytes.checked_add(segment.len()) else { return false };
            bytes = total;
        }
        if !count(bytes, limits.authority_bytes) {
            return false;
        }
    }
    if let Some(instructions) = &value.instructions
        && !count(instructions.len(), limits.message_bytes)
    {
        return false;
    }
    if let Some(grace) = &value.grace
        && !count(grace.reason.len(), limits.result_bytes)
    {
        return false;
    }
    match &value.escalation {
        crate::Escalation::Rejected { reason, .. } if !count(reason.len(), limits.result_bytes) => return false,
        crate::Escalation::Rejected { .. }
        | crate::Escalation::Waiting { .. }
        | crate::Escalation::Unheld { .. }
        | crate::Escalation::Routing { .. } => {}
    }
    if let Some(proposal) = &value.proposal
        && (!count(proposal.reason.len(), limits.message_bytes) || !action(&proposal.action, limits))
    {
        return false;
    }
    if let Some(proposal) = &value.proposal {
        match &proposal.state {
            crate::ProposalState::Rejected { reason, .. } if !count(reason.len(), limits.message_bytes) => {
                return false;
            }
            crate::ProposalState::Rejected { .. }
            | crate::ProposalState::Pending { .. }
            | crate::ProposalState::Accepted { .. }
            | crate::ProposalState::Withdrawn => {}
        }
    }
    if let Some(state) = &value.recurring
        && (!count(state.last_batch.len(), limits.batch) || !batch(&state.template.batch, limits))
    {
        return false;
    }
    if let Some(choice) = &value.llm
        && !crate::llm::valid_shape(choice)
    {
        return false;
    }
    inbox(value, limits)
}

fn payload(word: &crate::Word, limits: &Limits) -> Option<usize> {
    match &word.kind {
        MessageKind::Result(ResultKind::Approved { state, .. }) => state.len().checked_add(word.words.len()),
        MessageKind::Result(ResultKind::Verdict { fields: values, .. }) => {
            if !count(values.len(), limits.result_bytes) {
                return None;
            }
            fields(values)?.checked_add(word.words.len())
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
        | MessageKind::PersonStalled { .. } => Some(word.words.len()),
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

fn inbox(value: &TaskRecord, limits: &Limits) -> bool {
    let Some((subscriptions, subscription_bytes)) = crate::subscriptions::credit(value, limits) else { return false };
    let mut bytes = 0_usize;
    let mut amendments = 0_usize;
    let mut met = 0_usize;
    for word in &value.inbox {
        let Some(size) = payload(word, limits) else { return false };
        let bound = if crate::inbox::finish_word(value, word.number) {
            limits.result_bytes
        } else {
            match word.kind {
                MessageKind::QuestionOwed => u32::try_from(crate::questions::OWED_WORDS.len()).expect("fixed reminder"),
                MessageKind::Words
                | MessageKind::Question
                | MessageKind::Answer { .. }
                | MessageKind::Amendment { .. }
                | MessageKind::FinishRequested { .. }
                | MessageKind::SettingsDecision { .. }
                | MessageKind::ProposalDecision { .. } => limits.message_bytes,
                MessageKind::Result(_) => limits.result_bytes,
                MessageKind::Notice { .. } | MessageKind::News { .. } => limits.result_bytes.max(limits.message_bytes),
                MessageKind::EffectNotMade { .. }
                | MessageKind::Escalation { .. }
                | MessageKind::Proposal { .. }
                | MessageKind::PersonStalled { .. }
                | MessageKind::Met { .. }
                | MessageKind::QuestionAnswered { .. }
                | MessageKind::QuestionEnded { .. }
                | MessageKind::Timer { .. } => 0,
            }
        };
        if !count(size, bound) {
            return false;
        }
        if ordinary(&word.kind) {
            let Some(total) = bytes.checked_add(size) else { return false };
            bytes = total;
        } else {
            match word.kind {
                MessageKind::Met { .. } => met = met.checked_add(1).expect("bounded inbox"),
                MessageKind::Amendment { .. } | MessageKind::FinishRequested { .. } => {
                    amendments = amendments.checked_add(1).expect("bounded inbox");
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
    // Amendment and met slots are retained beside the ordinary inbox budget.
    if amendments > 2 || met > 1 {
        return false;
    }
    let messages = value
        .inbox
        .len()
        .saturating_sub(amendments)
        .saturating_sub(met)
        .saturating_add(value.delegates.len())
        .saturating_add(value.questions.len())
        .saturating_add(crate::inbox::received_credits(
            value.requester,
            &value.question_lines,
            &value.questions,
            &value.inbox,
        ))
        .saturating_add(usize::from(value.proposal.is_some()))
        .saturating_add(subscriptions);
    if !count(messages, limits.inbox_messages) {
        return false;
    }
    let Some(reserved) = value.delegates.len().checked_mul(usize::try_from(limits.result_bytes).expect("byte bound"))
    else {
        return false;
    };
    let Some(total) = bytes.checked_add(reserved) else { return false };
    let Some(answers) = value.questions.len().checked_mul(usize::try_from(limits.message_bytes).expect("byte bound"))
    else {
        return false;
    };
    let Some(total) = total.checked_add(answers) else { return false };
    let proposal =
        if value.proposal.is_some() { usize::try_from(limits.message_bytes).expect("byte bound") } else { 0 };
    let Some(total) = total.checked_add(proposal) else { return false };
    let Some(total) = total.checked_add(subscription_bytes) else { return false };
    count(total, limits.inbox_bytes)
}

pub(super) fn linked_inbox(domain: &Domain, limits: &Limits, value: &TaskRecord) -> bool {
    let mut stalls = 0_usize;
    for child in &value.delegates {
        if crate::person::credit(domain.task(*child).expect("delegate linked")) {
            stalls = stalls.checked_add(1).expect("bounded delegates");
        }
    }
    let mut messages = 0_usize;
    for word in &value.inbox {
        if ordinary(&word.kind) {
            messages = messages.checked_add(1).expect("bounded inbox");
        }
    }
    let Some((subscriptions, _)) = crate::subscriptions::credit(value, limits) else { return false };
    count(
        messages
            .saturating_add(value.delegates.len())
            .saturating_add(value.questions.len())
            .saturating_add(crate::inbox::received_credits(
                value.requester,
                &value.question_lines,
                &value.questions,
                &value.inbox,
            ))
            .saturating_add(usize::from(value.proposal.is_some()))
            .saturating_add(subscriptions)
            .saturating_add(stalls),
        limits.inbox_messages,
    )
}

pub(super) fn settings(domain: &Domain, value: &TaskRecord) -> bool {
    match value.executor {
        Executor::Agent { .. } => {}
        Executor::Procedure { .. } | Executor::Person(_) => return true,
    }
    let mut extra = u32::from(!domain.holders.contains_key(&value.settings.task));
    if let Some(run) = value.settings.run
        && run != value.settings.task
        && !domain.holders.contains_key(&run)
    {
        extra = extra.checked_add(1).expect("two held revisions");
    }
    domain.holders.len().saturating_add(extra) <= domain.holders.capacity()
}

pub(super) fn choice(domain: &Domain, value: &TaskRecord) -> bool {
    let Some(choice) = &value.llm else { return true };
    let charter = match value.executor {
        Executor::Agent { charter } => charter,
        Executor::Procedure { .. } | Executor::Person(_) => return false,
    };
    let models = domain.charter_models(charter, value.settings.task).expect("charter linked");
    if !model(domain, models, choice.main) {
        return false;
    }
    for chosen in &choice.subagents {
        if !model(domain, models, *chosen) {
            return false;
        }
    }
    true
}

fn model(domain: &Domain, models: &[u8], chosen: crate::ModelEffort) -> bool {
    let Some(efforts) = domain.efforts.get(usize::from(chosen.model)) else { return false };
    models.contains(&chosen.model)
        && match chosen.effort {
            Some(effort) => effort < *efforts,
            None => *efforts == 0,
        }
}
