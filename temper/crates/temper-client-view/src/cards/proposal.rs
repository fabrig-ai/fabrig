//! The proposal's task specifications, executors and authority are delivered facts.
use super::{P, phrase, specification, status, text};
use crate::words::{Word, Words};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, phrases::Hole};
fn batch(page: &c::ReadValue) -> Option<&skein_lib::List<c::ProposedTask>> {
    match page {
        c::ReadValue::Proposal(proposal) => match &proposal.action {
            c::ProposalAction::Batch(tasks) => Some(tasks),
            c::ProposalAction::Settings(_)
            | c::ProposalAction::Effect { .. }
            | c::ProposalAction::ProcedureEffect { .. }
            | c::ProposalAction::Amend { .. }
            | c::ProposalAction::Widen { .. }
            | c::ProposalAction::Release { .. } => None,
        },

        c::ReadValue::Task(_)
        | c::ReadValue::Settings(_)
        | c::ReadValue::Setting(_)
        | c::ReadValue::Revision(_)
        | c::ReadValue::TaskText(_)
        | c::ReadValue::Revisions(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::Escalation(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::EndedPage(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_) => None,
    }
}
pub(super) fn reason(b: &mut Builder, page: &c::ReadValue) {
    // Batch headings need the phrasebook; facts draws them before its rows.
    if batch(page).is_none() {
        why(b, page);
    }
}
/// The place is the caller's: it sets the title's level and whether a kicker says the kind.
fn title(b: &mut Builder, tasks: &skein_lib::List<c::ProposedTask>, words: &Words, place: super::Place) {
    let level = place.level();
    if place.kicker() {
        b.open(Role::Paragraph);
        b.style(crate::look::PROPOSAL_KICKER);
        let kicker = words.say(Word::Proposal, &[]);
        b.name(&kicker);
        b.text(skein_ui::Run::Plain, &kicker);
        b.close();
    }
    for task in tasks {
        let written = task.spec.specification.words.bytes();
        let end = skein_lib::bytes::find(written, b"\n").unwrap_or(written.len());
        text(b, Role::Heading { level }, written.get(..end).expect("title prefix"));
        if end < written.len() {
            skein_ui::markdown::read(
                written.get(end..).expect("spec body"),
                level.checked_add(1).expect("proposal heading fits"),
                b,
            );
        }
    }
}
fn why(b: &mut Builder, page: &c::ReadValue) {
    match page {
        c::ReadValue::Proposal(proposal) => super::document(b, &proposal.reason),

        c::ReadValue::Task(_)
        | c::ReadValue::Settings(_)
        | c::ReadValue::Setting(_)
        | c::ReadValue::Revision(_)
        | c::ReadValue::TaskText(_)
        | c::ReadValue::Revisions(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::Escalation(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::EndedPage(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_) => {}
    }
}
pub(super) fn facts(b: &mut Builder, page: &c::ReadValue, words: &Words, rows: u32, place: super::Place) {
    if let Some(tasks) = batch(page) {
        title(b, tasks, words, place);
        batch_facts(b, page, tasks, words, rows);
        return;
    }
    match page {
        c::ReadValue::Proposal(proposal) => match &proposal.action {
            c::ProposalAction::Batch(tasks) => proposed(b, tasks, words, rows, false),
            c::ProposalAction::Settings(_) => {}
            c::ProposalAction::Effect { part, authority: value, .. }
            | c::ProposalAction::ProcedureEffect { part, authority: value, .. } => {
                crate::forge::owner_part(b, part, words);
                authority(b, value, words, rows);
            }
            c::ProposalAction::Amend { task, before, after, .. } => {
                phrase(b, words, P::ProposalAmend, &[Hole::Number(task.number())]);
                specification(b, &after.spec, words);
                authority_change(b, &before.authority, &after.authority, words, rows);
            }
            c::ProposalAction::Widen { task, before, after } => {
                phrase(b, words, P::ProposalTask, &[Hole::Number(task.number())]);
                authority_change(b, before, after, words, rows);
            }
            c::ProposalAction::Release { task, hold } => {
                phrase(b, words, P::ProposalTask, &[Hole::Number(task.number())]);
                if let Some(hold) = hold {
                    held(b, task.number(), hold, words, rows);
                }
            }
        },

        c::ReadValue::Task(task) => task_hold(b, task, words, rows),
        c::ReadValue::Escalation(escalation) => task_hold(b, &escalation.task, words, rows),

        c::ReadValue::Settings(_)
        | c::ReadValue::Setting(_)
        | c::ReadValue::Revision(_)
        | c::ReadValue::TaskText(_)
        | c::ReadValue::Revisions(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::EndedPage(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_) => {}
    }
}
fn batch_facts(
    b: &mut Builder,
    page: &c::ReadValue,
    tasks: &skein_lib::List<c::ProposedTask>,
    words: &Words,
    rows: u32,
) {
    b.open(Role::Slot);
    b.style(crate::look::PROPOSAL_FACTS);
    text(b, Role::Term, &words.say(Word::ProposalWhy, &[]));
    b.open(Role::Description);
    b.open(Role::Paragraph);
    b.text(skein_ui::Run::Plain, "“".as_bytes());
    if let c::ReadValue::Proposal(proposal) = page {
        b.text(skein_ui::Run::Plain, proposal.reason.bytes());
    }

    b.text(skein_ui::Run::Plain, "”".as_bytes());
    b.close();
    b.close();
    proposed(b, tasks, words, rows, owns(page));
    text(b, Role::Term, &words.say(Word::ProposalBudget, &[]));
    b.open(Role::Description);
    b.open(Role::Paragraph);
    let facts = jig_client::facts::decision(Some(page), c::ActKind::Accept);
    let mut first = true;
    for fact in &jig_client::words::decision(&facts, &words.book, words.language) {
        if !first {
            separator(b);
        }
        b.text(skein_ui::Run::Plain, fact);
        first = false;
    }
    b.close();
    b.close();
    b.close();
}

fn landing_pattern(b: &mut Builder, pattern: &c::Pattern, words: &Words) {
    let mut path = skein_lib::List::with_capacity(pattern.segments.len().checked_add(1).expect("bounded pattern"));
    for segment in &pattern.segments {
        path.push(segment.clone()).expect("pattern segment");
    }
    let (last, open) = match &pattern.last {
        c::PatternEnd::Exact(last) => (last, false),
        c::PatternEnd::Open(last) => (last, true),
    };
    path.push(last.clone()).expect("pattern end");
    crate::forge::drawn(b, &c::ApplicationPart::Resource { connector: words.connector, path }, words);
    if open {
        b.text(skein_ui::Run::Plain, "…".as_bytes());
    }
}
fn owns(page: &c::ReadValue) -> bool {
    if let c::ReadValue::Proposal(proposal) = page
        && let c::HistoryHolder::Person(holder) = proposal.waiting.holder
    {
        for offer in &proposal.funding {
            if let c::Funder::Pool { person, .. } = offer.source
                && holder == person
            {
                return true;
            }
        }
    }
    false
}
fn task_hold(b: &mut Builder, task: &c::Task, words: &Words, rows: u32) {
    if let Some(hold) = &task.hold {
        held(b, task.chip.task, hold, words, rows);
    }
}
fn held(b: &mut Builder, task: u64, hold: &c::ChipHold, words: &Words, _rows: u32) {
    super::hold(b, task, &hold.reason, words);
    if let Some(account) = &hold.words {
        text(b, Role::Paragraph, account.bytes());
        if account.cut {
            phrase(b, words, P::Cut, &[]);
        }
    }
    phrase(b, words, P::HoldRevision, &[Hole::Number(hold.revision)]);
}
fn proposed(b: &mut Builder, tasks: &skein_lib::List<c::ProposedTask>, words: &Words, rows: u32, own: bool) {
    for task in window(b, words, tasks.as_slice(), rows) {
        text(b, Role::Term, &words.say(Word::Goal, &[]));
        b.open(Role::Description);
        b.open(Role::Paragraph);
        if own {
            words.text(b, Word::ProposalOwn, &[]);
        }
        match task.executor {
            c::Executor::Job { kind } => words.text(b, Word::Job, &[Hole::Number(u64::from(kind))]),
            c::Executor::Agent { charter } => {
                b.text(skein_ui::Run::Plain, &words.say_card(P::Charter, &[Hole::Number(u64::from(charter))]));
            }
            c::Executor::Procedure { code, .. } => {
                b.text(skein_ui::Run::Plain, &words.say_card(P::ProcedureCode, &[Hole::Number(u64::from(code))]));
            }
            c::Executor::Person(_) => {
                b.text(skein_ui::Run::Plain, &words.say(Word::PersonTask, &[]));
            }
        }

        specification_facts(b, &task.spec, words);
        for dependency in &task.dependencies {
            fact(b, words, P::Input, &[Hole::Number(*dependency)], true);
        }
        if task.cut {
            fact(b, words, P::Cut, &[], true);
        }
        b.close();
        b.close();
        text(b, Role::Term, &words.say(Word::ProposalLanding, &[]));
        b.open(Role::Description);
        b.open(Role::Paragraph);
        for (at, resource) in window_facts(b, words, task.resources.as_slice(), rows, true).iter().enumerate() {
            if at != 0 {
                separator(b);
            }
            crate::forge::owner_part(b, resource, words);
        }
        for (at, grant) in
            window_facts(b, words, task.authority.authority.grants.as_slice(), rows, true).iter().enumerate()
        {
            if !task.resources.is_empty() || at > 0 {
                separator(b);
            }
            crate::forge::drawn(b, &c::ApplicationPart::Kind { connector: grant.connector, kind: grant.kind }, words);
            b.text(skein_ui::Run::Plain, b" ");
            landing_pattern(b, &grant.pattern, words);
        }
        if let Some(priority) = task.priority {
            if !task.resources.is_empty() || !task.authority.authority.grants.is_empty() {
                separator(b);
            }
            b.text(skein_ui::Run::Plain, &words.say(Word::Priority, &[Hole::Number(u64::from(priority))]));
        }
        b.close();
        b.close();
    }
}
pub(super) fn has_rows(page: &c::ReadValue) -> bool {
    batch(page).is_some()
}
fn authority_change(
    b: &mut Builder,
    before: &c::AuthorityDocument,
    after: &c::AuthorityDocument,
    words: &Words,
    rows: u32,
) {
    phrase(b, words, P::AuthorityBefore, &[]);
    authority(b, before, words, rows);
    phrase(b, words, P::AuthorityAfter, &[]);
    authority(b, after, words, rows);
}
fn authority(b: &mut Builder, document: &c::AuthorityDocument, words: &Words, rows: u32) {
    authority_facts(b, document, words, rows, false);
}
fn authority_facts(b: &mut Builder, document: &c::AuthorityDocument, words: &Words, rows: u32, inline: bool) {
    let value = &document.authority;
    fact(b, words, P::Spend, &[Hole::Number(value.spend)], inline);
    fact(b, words, P::Tools, &[Hole::Number(value.tools)], inline);
    fact(b, words, P::NoteBits, &[Hole::Number(u64::from(value.notes))], inline);
    fact(
        b,
        words,
        P::Delegation,
        &[Hole::Number(u64::from(value.delegation.tasks)), Hole::Number(u64::from(value.delegation.depth))],
        inline,
    );
    if let Some(deadline) = value.deadline {
        if inline {
            separator(b);
        }
        b.text(skein_ui::Run::Plain, &words.deadline(deadline));
    }
    for kind in window_facts(b, words, value.delegation.kinds.as_slice(), rows, inline) {
        let (word, number) = match kind {
            c::AuthorityExecutor::Job(number) => {
                if inline {
                    separator(b);
                }
                words.text(b, Word::Job, &[Hole::Number(u64::from(*number))]);
                continue;
            }
            c::AuthorityExecutor::Charter(number) => (P::Charter, *number),
            c::AuthorityExecutor::Procedure(number) => (P::ProcedureCode, *number),
            c::AuthorityExecutor::Role(number) => (P::PersonRole, *number),
        };
        fact(b, words, word, &[Hole::Number(u64::from(number))], inline);
    }
    for grant in window_facts(b, words, value.grants.as_slice(), rows, inline) {
        if inline {
            separator(b);
        }
        crate::forge::drawn(b, &c::ApplicationPart::Kind { connector: grant.connector, kind: grant.kind }, words);
        pattern_facts(b, &grant.pattern, words, rows, inline);
    }
    for scope in window_facts(b, words, value.note_resources.as_slice(), rows, inline) {
        pattern_facts(b, &scope.pattern, words, rows, inline);
    }
    if document.cut {
        if inline {
            fact(b, words, P::Cut, &[], true);
        } else {
            status(b, words, P::Cut, &[]);
        }
    }
}

fn pattern_facts(b: &mut Builder, pattern: &c::Pattern, words: &Words, _rows: u32, inline: bool) {
    for segment in &pattern.segments {
        fact(b, words, P::PatternSegment, &[super::name(segment)], inline);
    }
    let (word, bytes) = match &pattern.last {
        c::PatternEnd::Exact(bytes) => (P::PatternExact, bytes),
        c::PatternEnd::Open(bytes) => (P::PatternOpen, bytes),
    };
    fact(b, words, word, &[super::name(bytes)], inline);
}

fn window<'a, T>(b: &mut Builder, words: &Words, values: &'a [T], rows: u32) -> &'a [T] {
    window_facts(b, words, values, rows, false)
}
fn window_facts<'a, T>(b: &mut Builder, words: &Words, values: &'a [T], rows: u32, inline: bool) -> &'a [T] {
    let count = values.len().min(usize::try_from(rows).expect("row bound fits"));
    if count < values.len() {
        if inline {
            fact(b, words, P::Cut, &[], true);
        } else {
            status(b, words, P::Cut, &[]);
        }
    }
    values.get(..count).expect("bounded prefix")
}

fn separator(b: &mut Builder) {
    b.text(skein_ui::Run::Plain, " · ".as_bytes());
}
fn fact(b: &mut Builder, words: &Words, word: P, holes: &[Hole], inline: bool) {
    if inline {
        separator(b);
        b.text(skein_ui::Run::Plain, &words.say_card(word, holes));
    } else {
        phrase(b, words, word, holes);
    }
}
fn specification_facts(b: &mut Builder, document: &c::SpecificationDocument, words: &Words) {
    for parameter in &document.specification.parameters {
        match parameter {
            c::ParameterDocument::Number { name, value } => {
                fact(b, words, P::ParameterNumber, &[Hole::Number(u64::from(*name)), Hole::Number(*value)], true);
            }
            c::ParameterDocument::Bytes { name, value } => {
                fact(b, words, P::ParameterBytes, &[Hole::Number(u64::from(*name)), super::name(value.bytes())], true);
            }
            c::ParameterDocument::Resource { name, connector, resource } => fact(
                b,
                words,
                P::ParameterResource,
                &[Hole::Number(u64::from(*name)), Hole::Number(*resource), Hole::Number(u64::from(*connector))],
                true,
            ),
        }
    }
    for task in &document.specification.inputs {
        fact(b, words, P::Input, &[Hole::Number(*task)], true);
    }
    if document.has_loss() {
        fact(b, words, P::Cut, &[], true);
    }
}

/// One settings card in a conversation or Needs you (client/pages/settings.md, 2).
/// Ended cards retain their original proposal's words and read their recorded decision.
pub(crate) fn settings(
    b: &mut Builder,
    root: &temper_client_domain::Domain,
    card: &temper_client_domain::pages::inbox::Card,
    words: &Words,
    bindings: &mut crate::frame::Bindings,
    replaced: bool,
) -> bool {
    let Some(facts) = settings_facts(card) else { return false };
    let terminal = crate::pages::terminal::shown(root, words);
    let state = settings_state(card);
    let accepted = state == SettingsState::Accepted || facts.made.is_some();
    let superseded = replaced && state == SettingsState::Withdrawn;
    b.open(Role::Region);
    b.style(crate::look::SETTINGS_PROPOSAL);
    if terminal {
        b.style(crate::look::TERMINAL_CARD);
    }
    if superseded {
        b.style(crate::look::SETTINGS_PROPOSAL_SUPERSEDED);
    }
    b.name(&words.say(Word::Proposal, &[]));
    if terminal {
        b.open(Role::Slot);
        b.style(crate::look::TERMINAL_CARD_LINE);
        crate::pages::terminal::kicker(b, words);
    }
    b.open(Role::Paragraph);
    b.style(crate::look::SETTINGS_PROPOSAL_TITLE);
    let title = settings_title(root, &facts, words);
    b.name(&title);
    b.text(skein_ui::Run::Plain, &title);
    b.close();
    if !terminal {
        settings_lines(b, root, &facts, words);
    }
    let ending = if accepted {
        Some(Word::SettingsProposalAccepted)
    } else {
        match state {
            SettingsState::Rejected => Some(Word::SettingsProposalRejected),
            SettingsState::Withdrawn => {
                Some(if superseded { Word::SettingsProposalReplaced } else { Word::SettingsProposalWithdrawn })
            }
            SettingsState::Reading | SettingsState::Pending | SettingsState::Accepted | SettingsState::Other => None,
        }
    };
    if let Some(ending) = ending {
        b.open(Role::Paragraph);
        b.style(crate::look::SETTINGS_PROPOSAL_STATE);
        if accepted {
            b.style(crate::look::SETTINGS_PROPOSAL_OK);
        }
        let said = words.say(ending, &[]);
        b.name(&said);
        b.text(skein_ui::Run::Plain, &said);
        b.close();
    } else if state == SettingsState::Pending
        && let Some(proposal) = facts.live
    {
        let offered = jig_client::facts::offer(proposal.acts.as_slice(), c::ActKind::Accept);
        if let Some(c::Admission::Allowed | c::Admission::Bounded { .. }) = offered {
            b.open(Role::Slot);
            b.style(crate::look::SETTINGS_PROPOSAL_ACTS);
            settings_button(
                b,
                words,
                bindings,
                Word::Accept,
                temper_client_domain::pages::Act::Decide {
                    object: card.object,
                    revision: None,
                    act: c::ActKind::Accept,
                },
                true,
                card.key.is_some(),
            );
            if root.pages().chat().is_some() {
                settings_button(
                    b,
                    words,
                    bindings,
                    Word::SettingsProposalChange,
                    temper_client_domain::pages::Act::ChangeProposal { proposal: proposal.key },
                    false,
                    false,
                );
            }
            b.close();
            if let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal, ..
            }))) = &card.outcome
            {
                words.refusal(b, refusal, 0);
            }
        } else {
            text(b, Role::Paragraph, &words.say(Word::SettingsProposalWaiting, &[]));
            waiting(b, root, proposal.waiting.holder, words);
        }
    }
    if terminal {
        b.close();
        crate::pages::terminal::details(b, words);
        settings_lines(b, root, &facts, words);
        b.close();
    }
    b.close();
    true
}

/// What a settings card draws, from a live proposal or its retained record (client/pages/settings.md, 2).
pub(crate) struct SettingsFacts<'a> {
    pub(crate) key: c::ProposalKey,
    pub(crate) projects: &'a [u32],
    pub(crate) reason: &'a c::DocumentText,
    pub(crate) items: &'a [c::RevisionItemHead],
    pub(crate) review: &'a [c::ReviewItem],
    pub(crate) made: Option<u32>,
    /// The live proposal, for its acts and who it waits for; none once it has ended.
    pub(crate) live: Option<&'a c::Proposal>,
}
pub(crate) fn settings_facts(card: &temper_client_domain::pages::inbox::Card) -> Option<SettingsFacts<'_>> {
    let c::ObjectKey::Proposal(key) = card.object else { return None };
    if let Some(c::ReadValue::Proposal(proposal)) = &card.page
        && proposal.key == key
        && let c::ProposalAction::Settings(settings) = &proposal.action
    {
        return Some(SettingsFacts {
            key,
            projects: proposal.projects.as_slice(),
            reason: &proposal.reason,
            items: settings.items.as_slice(),
            review: match &settings.review {
                Some(review) => review.items.as_slice(),
                None => &[],
            },
            made: settings.made,
            live: Some(proposal),
        });
    }
    if let Some(c::ReadValue::Ended(ended)) = &card.page
        && ended.matches(card.object)
        && let c::EndedValue::Proposal(proposal) = &ended.value
        && let c::RetainedAction::Settings(settings) = &proposal.action
    {
        return Some(SettingsFacts {
            key,
            projects: settings.projects.as_slice(),
            reason: &proposal.reason,
            items: settings.items.as_slice(),
            review: &[],
            made: settings.made,
            live: None,
        });
    }
    None
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsState {
    Reading,
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
    Other,
}
fn settings_state(card: &temper_client_domain::pages::inbox::Card) -> SettingsState {
    if let Some(c::ReadValue::Ended(ended)) = &card.page
        && let c::EndedValue::Proposal(proposal) = &ended.value
    {
        return match proposal.state {
            c::ProposalHistoryState::Accepted { .. } => SettingsState::Accepted,
            c::ProposalHistoryState::Rejected { .. } => SettingsState::Rejected,
            c::ProposalHistoryState::Withdrawn => SettingsState::Withdrawn,
            c::ProposalHistoryState::Pending { .. } => SettingsState::Reading,
        };
    }
    if let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
        c::Outcome::Account(_) | c::Outcome::SettingsChanged(_),
    )))) = &card.outcome
    {
        return SettingsState::Accepted;
    }
    if let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
        c::DocumentOutcome::ProposalDecided { choice, .. },
    ))))) = &card.outcome
    {
        return match choice {
            c::ProposalChoice::Accepted => SettingsState::Accepted,
            c::ProposalChoice::Rejected => SettingsState::Rejected,
            c::ProposalChoice::Withdrawn => SettingsState::Withdrawn,
            c::ProposalChoice::Stale => SettingsState::Other,
            c::ProposalChoice::Passed => settings_read_state(card),
        };
    }
    if let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Unknown,
        ..
    }))) = &card.outcome
    {
        return SettingsState::Withdrawn;
    }
    settings_read_state(card)
}
fn settings_read_state(card: &temper_client_domain::pages::inbox::Card) -> SettingsState {
    if let Some(c::ReadValue::Proposal(_)) = &card.page { SettingsState::Pending } else { SettingsState::Reading }
}
fn settings_button(
    b: &mut Builder,
    words: &Words,
    bindings: &mut crate::frame::Bindings,
    word: Word,
    act: temper_client_domain::pages::Act,
    primary: bool,
    disabled: bool,
) {
    b.open(Role::Button);
    b.style(crate::look::SETTINGS_BTN);
    b.style(if primary { crate::look::SETTINGS_PRIMARY } else { crate::look::SETTINGS_SMALL });
    if disabled {
        b.state(skein_ui::State::Disabled);
    }
    b.name(&words.say(word, &[]));
    b.bind(bindings.insert(act));
    b.close();
}
fn waiting(b: &mut Builder, root: &temper_client_domain::Domain, holder: c::HistoryHolder, words: &Words) {
    match holder {
        c::HistoryHolder::Person(person) => {
            if let Some(me) = root.pages().me(root.client())
                && me.party.number == person
            {
                text(b, Role::Paragraph, me.party.name.bytes());
                return;
            }
            text(b, Role::Paragraph, &words.say(Word::WaitingForPerson, &[Hole::Number(person)]));
        }
        c::HistoryHolder::Task(task) => text(b, Role::Paragraph, &words.say(Word::TaskWho, &[Hole::Number(task)])),
        c::HistoryHolder::Policy { project: 0, .. } => {
            text(b, Role::Paragraph, &words.say(Word::SettingsProposalDeploymentOwners, &[]));
        }
        c::HistoryHolder::Policy { project, .. } => {
            let project = project_label(root, project);
            text(b, Role::Paragraph, &words.say(Word::SettingsProposalOwners, &[super::name(&project)]));
        }
    }
}

fn settings_lines(b: &mut Builder, root: &temper_client_domain::Domain, facts: &SettingsFacts<'_>, words: &Words) {
    use temper_client_domain::forge::ways::Proposed;
    b.open(Role::Slot);
    b.style(crate::look::SETTINGS_PROPOSAL_REASON);
    super::document(b, facts.reason);
    b.close();
    let mut rows = skein_lib::List::uncapped();
    for item in facts.items {
        rows.push(item.clone()).expect("proposal heads fit");
    }
    for item in facts.review {
        match item.setting {
            c::Setting::Application { .. } | c::Setting::ProjectPolicy { .. } => {
                rows.push(c::RevisionItemHead { setting: item.setting, edit: item.edit.clone() })
                    .expect("review rows fit");
            }
            c::Setting::Charter(_)
            | c::Setting::Template(_)
            | c::Setting::Policy(_)
            | c::Setting::Model(_)
            | c::Setting::Rules => {}
        }
    }
    if facts.live.is_none()
        && let Some(values) = root.forge().proposed(facts.key.number)
    {
        for value in values {
            let (kind, project) = match value {
                Proposed::Ways { project, .. } => (7, *project),
                Proposed::Questions { project, .. } => (6, *project),
                Proposed::Classes { project, .. } => (8, *project),
            };
            rows.push(c::RevisionItemHead {
                setting: c::Setting::Application { kind, scope: project, number: 1 },
                edit: c::RevisionItemEdit::Set,
            })
            .expect("retained values fit");
        }
    }
    if rows.is_empty() {
        return;
    }
    b.open(Role::List);
    b.style(crate::look::SETTINGS_PROPOSAL_ITEMS);
    for item in &rows {
        if let Some(review) = model_review(facts.review, item.setting)
            && let Some(summary) = model_item(review, words)
        {
            setting_line(b, &summary);
            if let Some(previous) = review_model(&review.now) {
                setting_line(b, &model_previous(previous, words));
            }
            continue;
        }
        let mut rendered = false;
        if let Some(values) = root.forge().proposed(facts.key.number) {
            for value in values {
                match value {
                    Proposed::Ways { project, ways }
                        if item.setting == (c::Setting::Application { kind: 7, scope: *project, number: 1 }) =>
                    {
                        proposed_ways(b, root, *project, *ways, words);
                        rendered = true;
                    }
                    Proposed::Questions { project, route }
                        if item.setting == (c::Setting::Application { kind: 6, scope: *project, number: 1 }) =>
                    {
                        use temper_client_domain::forge::ways::Route;
                        setting_line(
                            b,
                            &words.say(
                                match route {
                                    Route::None => Word::SettingsProposalNone,
                                    Route::Up => Word::SettingsProposalUp,
                                },
                                &[],
                            ),
                        );
                        rendered = true;
                    }
                    Proposed::Classes { project, classes }
                        if item.setting == (c::Setting::Application { kind: 8, scope: *project, number: 1 }) =>
                    {
                        proposed_classes(b, classes, words);
                        rendered = true;
                    }
                    Proposed::Ways { .. } | Proposed::Questions { .. } | Proposed::Classes { .. } => {}
                }
            }
        }
        if !rendered {
            setting_line(b, &setting_name(root, item, words));
        }
    }
    b.close();
    owned_approval(b, root, facts.key.number, words);
}
/// The boundary of an owned forge's approval (client/pages/settings.md, 2).
pub(crate) fn owned_approval(b: &mut Builder, root: &temper_client_domain::Domain, number: u64, words: &Words) {
    if root.forge().owned_approval(number) {
        b.open(Role::Paragraph);
        b.style(crate::look::SETTINGS_PROPOSAL_ITEMS);
        b.open(Role::Text);
        b.style(crate::look::SETTINGS_DETAIL);
        b.run(skein_ui::Run::Plain, &words.say(Word::ProposalOwnedApproval, &[]));
        b.close();
        b.close();
    }
}
#[expect(clippy::manual_find, reason = "step code searches without closures")]
fn model_review(items: &[c::ReviewItem], setting: c::Setting) -> Option<&c::ReviewItem> {
    for item in items {
        if item.setting == setting {
            return Some(item);
        }
    }
    None
}
/// A model proposal names the change in the person's terms (domain/agent/agents.md, 1).
fn model_item(item: &c::ReviewItem, words: &Words) -> Option<Box<[u8]>> {
    let c::Setting::Model(_) = item.setting else { return None };
    let before = review_model(&item.now);
    let after = review_model(&item.would_be);
    let model = after.or(before)?;
    let change = if item.edit == c::RevisionItemEdit::Removed || model.withdrawn {
        b"withdrawn".as_slice()
    } else if let Some(after) = after {
        match before {
            None => b"added",
            Some(before) if before.withdrawn => b"added",
            Some(before) => model_change(before, after),
        }
    } else {
        b"new settings"
    };
    Some(joined(&[&model_name(model, words), b": ", change]))
}
fn review_model(value: &c::ReviewValue) -> Option<&c::CatalogueEntry> {
    match value {
        c::ReviewValue::Value(c::SettingSet { value: c::SettingValue::Model(model) }) => Some(model),
        c::ReviewValue::Value(_) | c::ReviewValue::TooLarge(_) | c::ReviewValue::None => None,
    }
}
fn model_name(model: &c::CatalogueEntry, words: &Words) -> Box<[u8]> {
    match &model.shown {
        Some(shown) => skein_lib::bytes::copy_of(shown.bytes()),
        None => words.model_name(model.model.name.bytes()),
    }
}
fn model_previous(model: &c::CatalogueEntry, words: &Words) -> Box<[u8]> {
    let value = &model.model;
    let write = match value.cache_write_price {
        Some(price) => joined(&[b", cache write ", skein_lib::Decimal::of(price).as_bytes()]),
        None => skein_lib::bytes::copy_of(b""),
    };
    joined(&[
        b"Previously: ",
        &model_name(model, words),
        b": ",
        if model.withdrawn { b"withdrawn" } else { picker_words(model.picker) },
        b"; prices: input ",
        skein_lib::Decimal::of(value.input_price).as_bytes(),
        b", cached ",
        skein_lib::Decimal::of(value.cached_price).as_bytes(),
        &write,
        b", output ",
        skein_lib::Decimal::of(value.output_price).as_bytes(),
        b" per ",
        skein_lib::Decimal::of(u64::from(value.price_unit)).as_bytes(),
        b" tokens",
    ])
}
fn picker_words(on: bool) -> &'static [u8] {
    if on { b"on the picker" } else { b"off the picker" }
}
fn model_change(before: &c::CatalogueEntry, after: &c::CatalogueEntry) -> &'static [u8] {
    let mut other = before.clone();
    other.picker = after.picker;
    if before.picker != after.picker && &other == after {
        return picker_words(after.picker);
    }
    other = before.clone();
    other.model.input_price = after.model.input_price;
    other.model.cached_price = after.model.cached_price;
    other.model.cache_write_price = after.model.cache_write_price;
    other.model.output_price = after.model.output_price;
    other.model.price_unit = after.model.price_unit;
    if before.model != after.model && &other == after { b"new prices" } else { b"new settings" }
}
fn setting_line(b: &mut Builder, value: &[u8]) {
    b.open(Role::Item);
    b.open(Role::Text);
    b.style(crate::look::SETTINGS_DETAIL);
    b.run(skein_ui::Run::Plain, value);
    b.close();
    b.close();
}
fn proposed_ways(
    b: &mut Builder,
    root: &temper_client_domain::Domain,
    project: u32,
    ways: temper_client_domain::forge::ways::Ways,
    words: &Words,
) {
    for (word, value) in [
        (Word::SettingsProposalDrafts, if ways.drafts { Word::SettingsProposalOn } else { Word::SettingsProposalOff }),
        (
            Word::SettingsProposalGoals,
            if ways.goals { Word::SettingsProposalShown } else { Word::SettingsProposalHidden },
        ),
        (
            Word::SettingsProposalChanges,
            if ways.changes { Word::SettingsProposalShown } else { Word::SettingsProposalHidden },
        ),
    ] {
        setting_line(b, &words.say(word, &[Hole::Name(words.say(value, &[]))]));
    }
    if let Some(me) = root.pages().me(root.client()) {
        let selected = temper_client_domain::forge::ways::default_way(me, project, &ways);
        for way in &crate::pages::goals::ways(me, project, words) {
            if Some(way.number) == selected {
                setting_line(b, &words.say(Word::SettingsProposalWay, &[super::name(&way.name)]));
                return;
            }
        }
    }
    setting_line(
        b,
        &words.say(Word::SettingsProposalWay, &[super::name(skein_lib::Decimal::of(u64::from(ways.way)).as_bytes())]),
    );
}
/// The table's subjects in the settings order (client/pages/settings.md, 2, The card in a chat).
pub(crate) fn class_objects(
    words: &Words,
    classes: &temper_protocol_reaches::Classes,
    class: temper_protocol_reaches::Class,
) -> Box<[u8]> {
    use temper_protocol_reaches::{Kind, Subject};
    let mut objects = skein_lib::List::uncapped();
    for kind in Kind::ALL {
        if kind.fixed() {
            continue;
        }
        let subject = Subject::Kind(kind);
        if classes.class_of(subject) == class {
            objects.push(crate::shared::object(words, subject, 0, Some(classes))).expect("bounded subjects");
        }
    }
    for (index, item) in classes.items.iter().enumerate() {
        let own_kind = match item.subject {
            Subject::Kind(_) => item.words.is_empty(),
            Subject::Set(_) => false,
        };
        if !own_kind && item.class == class {
            let under = u8::try_from(index).expect("bounded classes").checked_add(1).expect("bounded classes");
            objects.push(crate::shared::object(words, item.subject, under, Some(classes))).expect("bounded subjects");
        }
    }
    for set in &classes.sets {
        let subject = Subject::Set(set.number);
        let mut own = false;
        for item in &classes.items {
            if item.subject == subject && item.words.is_empty() {
                own = true;
                break;
            }
        }
        if !own && classes.class_of(subject) == class {
            objects.push(crate::shared::object(words, subject, 0, Some(classes))).expect("bounded subjects");
        }
    }
    comma_join(objects.as_slice())
}
pub(crate) fn comma_join(objects: &[Box<[u8]>]) -> Box<[u8]> {
    let mut length = 0_usize;
    for (index, object) in objects.iter().enumerate() {
        if index != 0 {
            length = length.checked_add(2).expect("bounded subjects");
        }
        length = length.checked_add(object.len()).expect("bounded subjects");
    }
    let mut writer = skein_lib::Writer::new(length);
    for (index, object) in objects.iter().enumerate() {
        if index != 0 {
            writer.put(b", ").expect("bounded subjects");
        }
        writer.put(object).expect("bounded subjects");
    }
    writer.finish()
}
fn proposed_classes(b: &mut Builder, classes: &temper_protocol_reaches::Classes, words: &Words) {
    use temper_protocol_reaches::Class;
    setting_line(b, &words.say(Word::ClassesTitle, &[]));
    for (class, word) in
        [(Class::Decide, Word::ClassesDecide), (Class::Ask, Word::ClassesAsk), (Class::AskAway, Word::ClassesAskAway)]
    {
        let objects = class_objects(words, classes, class);
        if !objects.is_empty() {
            setting_line(b, &words.say(word, &[Hole::Name(objects)]));
        }
    }
    for set in &classes.sets {
        setting_line(
            b,
            &words.say(Word::ClassesSet, &[Hole::Name(set.name.clone()), Hole::Name(comma_join(&set.patterns))]),
        );
    }
}
fn setting_name(root: &temper_client_domain::Domain, item: &c::RevisionItemHead, words: &Words) -> Box<[u8]> {
    let base = match template_name(root, item.setting, words) {
        Some(name) => name,
        None => setting_base(item.setting),
    };

    let suffix = match item.edit {
        c::RevisionItemEdit::Set => b"".as_slice(),
        c::RevisionItemEdit::Removed => b" removed",
        c::RevisionItemEdit::Reset => b" reset",
    };
    joined(&[&base, suffix])
}
fn template_name(root: &temper_client_domain::Domain, setting: c::Setting, words: &Words) -> Option<Box<[u8]>> {
    let c::Setting::Template(number) = setting else { return None };
    for project in &root.pages().me(root.client())?.projects {
        for template in &project.templates {
            if template.number == number.number {
                return Some(crate::pages::goals::way_name(template.name.bytes(), words));
            }
        }
    }
    None
}
fn setting_base(setting: c::Setting) -> Box<[u8]> {
    let (label, number) = match setting {
        c::Setting::Charter(n) => (b"charter".as_slice(), Some(n.number)),
        c::Setting::Template(n) => (b"template".as_slice(), Some(n.number)),
        c::Setting::Policy(n) => (b"policy".as_slice(), Some(n.number)),
        c::Setting::Model(n) => (b"model".as_slice(), Some(n.number)),
        c::Setting::Rules => (b"rules".as_slice(), None),
        c::Setting::ProjectPolicy { project } => (b"project policy".as_slice(), Some(project)),
        c::Setting::Application { kind: 1 | 2, number, .. } => (b"landing rule".as_slice(), Some(number)),
        c::Setting::Application { kind: 3, number, .. } => (b"forge charter".as_slice(), Some(number)),
        c::Setting::Application { kind: 11, number, .. } => (b"template-goal".as_slice(), Some(number)),
        c::Setting::Application { kind: 12, number, .. } => (b"follow".as_slice(), Some(number)),
        c::Setting::Application { kind: 13, number, .. } => (b"dependency".as_slice(), Some(number)),
        c::Setting::Application { kind: 6, scope, .. } => (b"project-questions".as_slice(), Some(scope)),
        c::Setting::Application { kind: 7, scope, .. } => (b"project-ways".as_slice(), Some(scope)),
        c::Setting::Application { kind: 8, scope, .. } => (b"project-classes".as_slice(), Some(scope)),
        c::Setting::Application { kind: 9, number, .. } => (b"charter-words".as_slice(), Some(number)),
        c::Setting::Application { kind: 10, scope, .. } => (b"project-words".as_slice(), Some(scope)),
        c::Setting::Application { kind, scope, number } => {
            return joined(&[
                b"application kind ",
                skein_lib::Decimal::of(u64::from(kind)).as_bytes(),
                b", ",
                skein_lib::Decimal::of(u64::from(scope)).as_bytes(),
                b", ",
                skein_lib::Decimal::of(u64::from(number)).as_bytes(),
            ]);
        }
    };
    match number {
        Some(number) => joined(&[label, b" ", skein_lib::Decimal::of(u64::from(number)).as_bytes()]),
        None => skein_lib::bytes::copy_of(label),
    }
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded setting text");
    }
    let mut out = skein_lib::Writer::new(length);
    for part in parts {
        out.put(part).expect("counted setting text");
    }
    out.finish()
}
fn project_label(root: &temper_client_domain::Domain, project: u32) -> Box<[u8]> {
    match crate::pages::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => skein_lib::bytes::copy_of(skein_lib::Decimal::of(u64::from(project)).as_bytes()),
    }
}

fn settings_title(root: &temper_client_domain::Domain, facts: &SettingsFacts<'_>, words: &Words) -> Box<[u8]> {
    if facts.projects.is_empty() {
        return words.say(Word::SettingsProposalTitleAll, &[]);
    }
    let mut names = skein_lib::bytes::copy_of(b"");
    for (index, project) in facts.projects.iter().enumerate() {
        let separator: &[u8] = if index == 0 {
            b"".as_slice()
        } else if index.checked_add(1).expect("bounded projects") == facts.projects.len() {
            b" and "
        } else {
            b", "
        };
        let name = project_label(root, *project);
        names = joined(&[&names, separator, &name]);
    }
    words.say(Word::SettingsProposalTitle, &[super::name(&names)])
}

/// A proposal's line keeps its actions outside its folded facts (client/pages/README.md).
pub(crate) fn terminal(
    b: &mut Builder,
    root: &temper_client_domain::Domain,
    card: &temper_client_domain::pages::inbox::Card,
    words: &Words,
    bindings: &mut crate::frame::Bindings,
    rows: u32,
) {
    let Some(page) = &card.page else { return };
    b.style(crate::look::TERMINAL_CARD);
    b.open(Role::Slot);
    b.style(crate::look::TERMINAL_CARD_LINE);
    crate::pages::terminal::kicker(b, words);
    let title = crate::shared::proposal_line(card, words).expect("terminal proposal");
    text(b, Role::Heading { level: 3 }, &title);
    crate::confirm::card_controls(b, root, card, words, bindings);
    b.close();
    crate::pages::terminal::details(b, words);
    reason(b, page);
    crate::confirm::facts(b, page, words, rows);
    b.close();
}

/// A proposal's terminal heading (temper/docs/design/client/pages/README.md).
pub(crate) fn line(card: &temper_client_domain::pages::inbox::Card, _words: &Words) -> Option<Box<[u8]>> {
    let page = card.page.as_ref()?;
    let c::ReadValue::Proposal(_) = page else { return None };
    let title = if let Some(tasks) = batch(page)
        && let Some(task) = tasks.as_slice().first()
    {
        task.spec.specification.words.bytes()
    } else if let c::ReadValue::Proposal(proposal) = page {
        proposal.reason.bytes()
    } else {
        b""
    };
    let end = skein_lib::bytes::find(title, b"\n").unwrap_or(title.len());
    Some(title.get(..end).expect("proposal first line").into())
}
