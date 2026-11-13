//! Home: needs first, updates afterwards, and deciding controls (06.3).
use crate::{
    bindings::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::{EndedValue, InboxGroup, ObjectKey, Proposer, ReadValue, ResultValue};
use skein_lib::Decimal;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    address::{Filter, Kind},
    pages::{Act, inbox::Card},
};
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn control(b: &mut Builder, bindings: &mut Bindings, role: Role, name: &[u8], act: Act) {
    if role == Role::Link {
        crate::bindings::link(b, name, bindings.insert(act));
        b.text(Run::Plain, name);
    } else {
        b.open(role);
        b.name(name);
        b.bind(bindings.insert(act));
    }
    b.close();
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    times: &mut Times,
    sign_in: &crate::forge_sign_in::Forges,
) {
    if words.shell == crate::words::Shell::Browser {
        super::needs::main(b, root, words, bindings, rows, times, sign_in);
        return;
    }
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::PAGE);
    b.open(Role::Slot);
    b.style(look::PAGEBAR);
    text(b, Role::Heading { level: 1 }, &words.say(Word::InboxTitle, &[]));
    b.close();
    let Some(inbox) = root.pages().inbox() else {
        b.close();
        return;
    };
    for (heading, groups) in [
        (
            Word::NeedsYou,
            [
                Some(InboxGroup::Proposals),
                Some(InboxGroup::Questions),
                Some(InboxGroup::Escalations),
                Some(InboxGroup::PersonTasks),
            ],
        ),
        (Word::InboxUpdates, [Some(InboxGroup::Results), Some(InboxGroup::Replies), None, None]),
    ] {
        b.open(Role::Slot);
        b.style(look::HEADING);
        text(b, Role::Heading { level: 2 }, &words.say(heading, &[]));
        let updates = heading == Word::InboxUpdates;
        let mut count = 0_u64;
        for card in &inbox.cards {
            if groups.contains(&Some(card.group)) {
                count = count.checked_add(1).expect("listed cards fit");
            }
        }
        let limited = *inbox.cut.get(usize::from(updates)).expect("two sections");
        let answered = inbox.answered(if updates {
            jig_client::contract::InboxReadGroup::Updates
        } else {
            jig_client::contract::InboxReadGroup::Needs
        });
        if answered || count != 0 {
            b.open(Role::Paragraph);
            b.style(look::SUBTLE_COUNT);
            let said = count_words(words, count, limited, updates && count != 0);
            b.name(&said);
            b.text(Run::Plain, &said);
            b.close();
        }
        if heading == Word::NeedsYou {
            filters(b, root, inbox.filter, words, bindings);
        }
        b.close();
        let mut opened = false;
        for group in groups.into_iter().flatten() {
            for card in &inbox.cards {
                if card.group == group {
                    if !opened {
                        b.open(Role::List);
                        b.name(&words.say(heading, &[]));
                        b.style(look::CARDS);
                        if heading == Word::InboxUpdates {
                            b.style(look::UPDATES);
                        }
                        opened = true;
                    }
                    entry(b, root, card, words, bindings, rows, times);
                }
            }
        }
        if opened {
            b.close();
        } else if answered {
            b.open(Role::Paragraph);
            b.style(look::EMPTY);
            let said = words.say(if updates { Word::InboxUpdatesEmpty } else { Word::InboxNeedsEmpty }, &[]);
            b.name(&said);
            b.text(Run::Plain, &said);
            b.close();
        }
    }
    super::needs::decided(b, root, words, bindings, times);
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}
fn filter_control(b: &mut Builder, bindings: &mut Bindings, name: &[u8], filter: Filter, pressed: bool) {
    b.open(Role::Button);
    b.name(name);
    if pressed {
        b.state(State::Pressed);
    }
    b.bind(bindings.insert(Act::Filter { filter }));
    b.close();
}
fn filters(b: &mut Builder, root: &Domain, filter: Filter, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::FILTERS);
    // The new chat's link leads the filters' row, as the mockup's button does.
    control(
        b,
        bindings,
        Role::Link,
        &words.say(Word::NewChatLink, &[]),
        Act::Go { address: Address::NewChat { draft: false } },
    );
    project_filter(b, root, filter, words, bindings);
    b.open(Role::Disclosure);
    b.style(look::KIND_MENU);
    let chosen = match filter.goal {
        Some(goal) => words.say(
            Word::ModelEffort,
            &[Hole::Name(words.say(Word::Goal, &[])), Hole::Name(words.say(Word::TaskWho, &[Hole::Number(goal)]))],
        ),
        None => words.say(Word::AllGoals, &[]),
    };
    b.name(&chosen);
    b.open(Role::Slot);
    filter_control(
        b,
        bindings,
        &words.say(Word::AllGoals, &[]),
        Filter { goal: None, ..filter },
        filter.goal.is_none(),
    );
    b.open(Role::Slot);
    b.open(Role::TextField);
    b.name(&words.say(Word::Goal, &[]));
    let value = match filter.goal {
        Some(goal) => Some(Decimal::of(goal)),
        None => None,
    };
    b.value(
        match &value {
            Some(value) => value.as_bytes(),
            None => b"",
        },
        temper_client_domain::pages::shown(root.pages()),
    );
    b.bind(bindings.insert(Act::Filter { filter }));
    b.close();
    b.close();
    b.close();
    b.close();
    b.open(Role::Disclosure);
    b.style(look::KIND_MENU);
    let chosen = match filter.kind {
        None => words.say(Word::Kind, &[]),
        Some(kind) => words.say(
            Word::KindChosen,
            &[Hole::Name(words.say(
                match kind {
                    Kind::Proposal => Word::Proposal,
                    Kind::Question => Word::Question,
                    Kind::Held => Word::Held,
                    Kind::PersonTask => Word::PersonTask,
                    Kind::Finding => Word::Finding,
                },
                &[],
            ))],
        ),
    };
    b.name(&chosen);
    b.open(Role::Slot);
    for (kind, word) in [
        (None, Word::AllKinds),
        (Some(Kind::Proposal), Word::Proposal),
        (Some(Kind::Question), Word::Question),
        (Some(Kind::Held), Word::Held),
        (Some(Kind::PersonTask), Word::PersonTask),
        (Some(Kind::Finding), Word::Finding),
    ] {
        filter_control(b, bindings, &words.say(word, &[]), Filter { kind, ..filter }, filter.kind == kind);
    }
    b.close();
    b.close();
    b.close();
}
fn project_filter(b: &mut Builder, root: &Domain, filter: Filter, words: &Words, bindings: &mut Bindings) {
    if let Some(me) = root.pages().me(root.client())
        && me.projects.len() >= 2
    {
        b.open(Role::Disclosure);
        b.style(look::KIND_MENU);
        let chosen = match filter.project {
            Some(project) => super::project_name(Some(me), project)
                .unwrap_or(words.say(Word::Project, &[Hole::Number(u64::from(project))])),
            None => words.say(Word::AllProjects, &[]),
        };
        b.name(&chosen);
        b.open(Role::Slot);
        filter_control(
            b,
            bindings,
            &words.say(Word::AllProjects, &[]),
            Filter { project: None, ..filter },
            filter.project.is_none(),
        );
        for project in &me.projects {
            let name = project.name.bytes();
            filter_control(
                b,
                bindings,
                name,
                Filter { project: Some(project.project), ..filter },
                filter.project == Some(project.project),
            );
        }
        b.close();
        b.close();
    }
}
#[expect(clippy::too_many_lines, reason = "the inbox card keeps its facts, controls and shared reason together")]
fn entry(
    b: &mut Builder,
    root: &Domain,
    card: &Card,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    times: &mut Times,
) {
    b.open(Role::Item);
    b.key(card_key(card.object));
    b.style(look::CARD);
    let (kind, style) = match card.group {
        InboxGroup::Proposals => (Word::Proposal, look::KIND_PROPOSAL),
        InboxGroup::Questions => (Word::Question, look::KIND_QUESTION),
        InboxGroup::Escalations => (Word::Held, look::KIND_HELD),
        InboxGroup::PersonTasks => (Word::PersonTask, look::KIND_PERSON_TASK),
        InboxGroup::Results => (Word::InboxUpdates, look::KIND_RESULT),
        InboxGroup::Replies => (Word::Reply, look::KIND_REPLY),
    };
    b.name(&card_name(root, card.object, kind, words));
    b.style(style);
    let update = card.group == InboxGroup::Results || card.group == InboxGroup::Replies;
    if !update {
        b.open(Role::Slot);
        b.style(look::META);
        b.open(Role::Paragraph);
        b.style(look::INBOX_TAG);
        let said = words.say(kind, &[]);
        b.name(&said);
        b.text(Run::Plain, &said);
        b.close();
        b.open(Role::Slot);
        b.style(look::META);
        if let Some(project) = card.project {
            let name = super::project_name(root.pages().me(root.client()), project)
                .unwrap_or(words.say(Word::Project, &[Hole::Number(u64::from(project))]));
            b.open(Role::Text);
            b.run(Run::Plain, &name);
            b.run(Run::Plain, " › ".as_bytes());
            b.close();
        }
        if let Some(task) = source_task(card.object) {
            control(
                b,
                bindings,
                Role::Link,
                &words.say(Word::TaskWho, &[Hole::Number(task)]),
                Act::Go { address: Address::Task { task } },
            );
        }
        if let Some(page) = &card.page
            && let Some(title) = task_title(page)
        {
            b.open(Role::Text);
            b.run(Run::Plain, b" ");
            b.run(Run::Plain, title);
            b.close();
        }
        b.close();
        card_time(b, card, words, times);
        b.close();
    }
    if let Some(page) = &card.page {
        if update {
            update_body(b, card, page, words, bindings);
            card_time(b, card, words, times);
            b.close();
            return;
        }
        if crate::cards::proposal::settings(b, root, card, words, bindings, false) {
            super::needs::why_line(
                b,
                root,
                words,
                bindings,
                temper_client_domain::pages::needs::Need::Card(card),
                super::needs::WhyForm::Short,
            );
            b.close();
            return;
        }
        body(b, page, words);
        crate::cards::proposal_reason(b, page);
        super::needs::why_line(
            b,
            root,
            words,
            bindings,
            temper_client_domain::pages::needs::Need::Card(card),
            super::needs::WhyForm::Short,
        );
        if has_facts(page) {
            let proposal = crate::cards::proposal_has_rows(page);
            if !proposal {
                b.open(Role::Slot);
                b.style(look::DETAILS);
            }
            crate::confirm::facts_at(b, page, words, rows, crate::cards::Place::Inbox);
            if !proposal {
                b.close();
            }
        }
        crate::confirm::card_controls(b, root, card, words, bindings);
        if (card.group == InboxGroup::Proposals || card.group == InboxGroup::Questions)
            && let Some(task) = source_task(card.object)
        {
            let name = open_task_name(words, task);
            control(b, bindings, Role::Link, &name, Act::Go { address: Address::Task { task } });
        }
    } else if !update {
        super::needs::why_line(
            b,
            root,
            words,
            bindings,
            temper_client_domain::pages::needs::Need::Card(card),
            super::needs::WhyForm::Short,
        );
    } else if update && let Some(task) = source_task(card.object) {
        b.open(Role::Heading { level: 3 });
        b.style(look::TITLE);
        let name = words.say(Word::Task, &[Hole::Number(task)]);
        b.name(&name);
        control(b, bindings, Role::Link, &name, update_act(card, task));
        b.close();
    }
    b.close();
}
fn update_act(card: &Card, task: u64) -> Act {
    if let ObjectKey::Result(_) = card.object
        && let Some(through) = card.position
    {
        Act::OpenResult { task, through }
    } else {
        Act::Go { address: Address::Task { task } }
    }
}
fn open_task_name(words: &Words, task: u64) -> Box<[u8]> {
    let task = words.say(Word::TaskWho, &[Hole::Number(task)]);
    words.say(Word::SetupOpen, &[Hole::Name(crate::accounts::words::join(&[&task, " ↗".as_bytes()]))])
}
pub(crate) fn task_title(page: &ReadValue) -> Option<&[u8]> {
    if let ReadValue::Question(question) = page {
        return Some(question.task.title.bytes());
    }

    if let ReadValue::Escalation(held) = page {
        return Some(held.task.chip.title.bytes());
    }

    if let ReadValue::PersonTask(task) = page {
        return Some(task.task.title.bytes());
    }
    if let ReadValue::Task(task) = page {
        return Some(task.chip.title.bytes());
    }
    None
}
pub(crate) fn source_task(object: ObjectKey) -> Option<u64> {
    match object {
        ObjectKey::Task(task)
        | ObjectKey::Escalation(task)
        | ObjectKey::PersonTask(task)
        | ObjectKey::Result(task)
        | ObjectKey::Question { task, .. } => Some(task),
        ObjectKey::Proposal(proposal) => match proposal.proposer {
            Proposer::Task(task) => Some(task),
            Proposer::Party(_) => None,
        },
        ObjectKey::Note { .. } => None,
    }
}
pub(crate) fn card_name(root: &Domain, object: ObjectKey, kind: Word, words: &Words) -> Box<[u8]> {
    let kind = words.say(kind, &[]);
    let source = if let Some(task) = source_task(object) {
        let digits = Decimal::of(task);
        let mut name = skein_lib::Writer::new(digits.as_bytes().len().checked_add(1).expect("task name fits"));
        name.put(b"T").expect("task prefix fits");
        name.put(digits.as_bytes()).expect("task digits fit");
        Some(name.finish())
    } else if let ObjectKey::Proposal(proposal) = object
        && let Proposer::Party(party) = proposal.proposer
        && let Some(me) = root.pages().me(root.client())
        && me.party.number == party
    {
        Some(skein_lib::bytes::copy_of(me.party.name.bytes()))
    } else {
        None
    };
    match source {
        Some(source) => words.say(Word::KindFrom, &[Hole::Name(kind), Hole::Name(source)]),
        None => kind,
    }
}
fn has_facts(page: &ReadValue) -> bool {
    let proposal = match page {
        ReadValue::Proposal(_) => true,
        ReadValue::Task(task) => task.hold.is_some(),
        ReadValue::Escalation(held) => held.task.hold.is_some(),
        ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Ended(_)
        | ReadValue::Me(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => false,
    };
    let decision = jig_client::facts::decision(Some(page), jig_client::contract::ActKind::Accept);
    proposal || decision.dangerous || decision.effect.is_some() || !decision.funding.is_empty()
}
fn title(b: &mut Builder, value: &[u8]) {
    b.open(Role::Heading { level: 3 });
    b.style(look::TITLE);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
/// The card's prose without its controls or decision facts (client/pages/README.md).
pub(crate) fn card_words(page: &ReadValue) -> Box<[u8]> {
    let (title, spec) = match page {
        ReadValue::Question(question) => return question.words.bytes().into(),
        ReadValue::Escalation(held) => (held.task.chip.title.bytes(), &held.task.spec),
        ReadValue::PersonTask(task) => (task.task.title.bytes(), &task.spec),
        ReadValue::Task(task) => (task.chip.title.bytes(), &task.spec),
        ReadValue::Ended(_)
        | ReadValue::Result(_)
        | ReadValue::Proposal(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Me(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => return Box::default(),
    };
    let words = spec.specification.words.bytes();
    crate::accounts::words::join(&[title, if title.is_empty() || words.is_empty() { b"" } else { b" " }, words])
}

fn body(b: &mut Builder, page: &ReadValue, words: &Words) {
    match page {
        ReadValue::Question(question) => {
            text(b, Role::Paragraph, &card_words(page));
            if question.words.cut {
                crate::cards::status(b, words, crate::cards::P::Cut, &[]);
            }
        }

        ReadValue::Escalation(held) => {
            title(b, held.task.chip.title.bytes());
            crate::cards::specification(b, &held.task.spec, words);
        }

        ReadValue::PersonTask(task) => {
            title(b, task.task.title.bytes());
            crate::cards::specification(b, &task.spec, words);
        }
        ReadValue::Task(task) => {
            title(b, task.chip.title.bytes());
            crate::cards::specification(b, &task.spec, words);
        }
        ReadValue::Ended(_)
        | ReadValue::Result(_)
        | ReadValue::Proposal(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Me(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => {}
    }
}

pub(crate) fn card_key(object: ObjectKey) -> skein_ui::Key {
    let (kind, a, b) = match object {
        ObjectKey::Task(task) => (0, task, 0),
        ObjectKey::Proposal(key) => match key.proposer {
            Proposer::Task(task) => (1, task, key.number),
            Proposer::Party(party) => (29, party, key.number),
        },
        ObjectKey::Escalation(task) => (2, task, 0),
        ObjectKey::Question { task, message } => (3, task, message),
        ObjectKey::PersonTask(task) => (4, task, 0),
        ObjectKey::Result(task) => (5, task, 0),
        ObjectKey::Note { project, note } => (6, u64::from(project), note),
    };
    skein_ui::Key { owner: look::owner(), kind, a, b }
}

/// The card's wait; task-page decision facts carry no wait.
pub(crate) fn waiting_since(page: &ReadValue) -> Option<skein_lib::Wall> {
    match page {
        ReadValue::Proposal(proposal) => Some(proposal.waiting.since),

        ReadValue::Question(question) => Some(question.since),

        ReadValue::Escalation(escalation) => match escalation.task.hold.as_ref() {
            Some(hold) => hold.hold_wait_since,
            None => None,
        },

        ReadValue::PersonTask(task) => task.since,
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Ended(_)
        | ReadValue::Me(_)
        | ReadValue::Turn(_)
        | ReadValue::Task(_)
        | ReadValue::Result(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}
fn card_time(b: &mut Builder, card: &Card, words: &Words, times: &mut Times) {
    let Some(page) = &card.page else { return };
    let (at, wait) = match card.group {
        InboxGroup::Proposals | InboxGroup::Questions | InboxGroup::Escalations | InboxGroup::PersonTasks => {
            (waiting_since(page), true)
        }
        InboxGroup::Results | InboxGroup::Replies => (arrival(page, card.group == InboxGroup::Replies), false),
    };
    let Some(at) = at else { return };
    let said = times.say(words, if wait { TimeForm::Span } else { TimeForm::Ago }, at);
    let said = if wait {
        words.say(Word::TimeWaiting, &[Hole::Name(said)])
    } else if let Some(count) = &card.unread
        && count.value != 0
    {
        words.say(
            Word::UpdateTime,
            &[Hole::Name(count_words(words, u64::from(count.value), count.limited, true)), Hole::Name(said)],
        )
    } else {
        said
    };
    b.open(Role::Paragraph);
    b.style(if wait { look::WAIT } else { look::UPDATE_TIME });
    b.name(&said);
    b.text(Run::Plain, &said);
    b.close();
}

fn arrival(page: &ReadValue, reply: bool) -> Option<skein_lib::Wall> {
    match page {
        ReadValue::Result(result) => {
            if reply {
                None
            } else {
                result.at
            }
        }
        ReadValue::Task(task) => {
            if reply {
                Some(task.last_activity)
            } else {
                None
            }
        }
        ReadValue::Ended(history) => match &history.value {
            EndedValue::Task { document, result } => {
                if reply {
                    Some(document.last_activity)
                } else {
                    result.at
                }
            }
            EndedValue::PersonTask { result, .. } => {
                if reply {
                    None
                } else {
                    result.at
                }
            }
            EndedValue::Proposal(_)
            | EndedValue::PersonProposal { .. }
            | EndedValue::Escalation { .. }
            | EndedValue::Question { .. } => None,
        },
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Me(_)
        | ReadValue::Turn(_)
        | ReadValue::Proposal(_)
        | ReadValue::Escalation(_)
        | ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}

fn count_words(words: &Words, count: u64, limited: bool, new: bool) -> Box<[u8]> {
    words.say(
        match (limited, new) {
            (false, false) => Word::Count,
            (true, false) => Word::CountLimited,
            (false, true) => Word::NewCount,
            (true, true) => Word::NewCountLimited,
        },
        &[Hole::Number(count)],
    )
}
fn update_body(b: &mut Builder, card: &Card, page: &ReadValue, words: &Words, bindings: &mut Bindings) {
    let reply = card.group == InboxGroup::Replies;
    let (chip, result) = match page {
        ReadValue::Task(task) => (&task.chip, None),
        ReadValue::Result(result) => (&result.task, Some(result.as_ref())),
        ReadValue::Ended(history) => match &history.value {
            EndedValue::Task { document, result } => (&document.chip, Some(result.as_ref())),
            EndedValue::PersonTask { result, .. } => (&result.task, Some(result.as_ref())),
            EndedValue::Proposal(_)
            | EndedValue::PersonProposal { .. }
            | EndedValue::Escalation { .. }
            | EndedValue::Question { .. } => return,
        },
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Me(_)
        | ReadValue::Turn(_)
        | ReadValue::Proposal(_)
        | ReadValue::Escalation(_)
        | ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => return,
    };
    let word = if reply {
        Word::UpdateReplied
    } else if match result {
        Some(result) => is_done(&result.ending),
        None => false,
    } {
        Word::UpdateDone
    } else {
        Word::UpdateEnded
    };
    let task = words.say(Word::TaskWho, &[Hole::Number(chip.task)]);
    b.open(Role::Heading { level: 3 });
    b.style(look::TITLE);
    let name = words.say(word, &[Hole::Name(task), Hole::Name(skein_lib::bytes::copy_of(chip.title.bytes()))]);
    b.name(&name);
    control(b, bindings, Role::Link, &name, update_act(card, chip.task));
    b.close();
    if reply {
        return;
    }
    let Some(result) = result else {
        return;
    };
    let embedded = match &result.ending {
        jig_client::contract::Ending::Task(ending) => match ending.as_ref() {
            jig_client::contract::ChipEnding::Done(value) => {
                words.status(b, crate::cards::P::Done, &[]);
                Some(value)
            }
            jig_client::contract::ChipEnding::Failed(_) => {
                words.status(b, crate::cards::P::Failed, &[]);
                None
            }
            jig_client::contract::ChipEnding::Cancelled { result, .. } => {
                words.status(b, crate::cards::P::Cancelled, &[]);
                result.as_ref()
            }
        },
        jig_client::contract::Ending::Done
        | jig_client::contract::Ending::Failed { .. }
        | jig_client::contract::Ending::Cancelled { .. }
        | jig_client::contract::Ending::Withdrawn { .. }
        | jig_client::contract::Ending::Decided { .. }
        | jig_client::contract::Ending::Deleted => {
            words.ending(b, &result.ending);
            None
        }
    };
    result_excerpt(b, result.value.as_ref().or(embedded));
}
fn result_excerpt(b: &mut Builder, value: Option<&ResultValue>) {
    let value = match value {
        Some(
            ResultValue::Report(text)
            | ResultValue::Failure(text)
            | ResultValue::Words(text)
            | ResultValue::OwnerVerdict { words: text, .. }
            | ResultValue::Verdict { words: text, .. }
            | ResultValue::Connector { summary: text, .. }
            | ResultValue::Change { words: text, .. }
            | ResultValue::Choice { reason: text, .. },
        ) => Some(text),
        Some(ResultValue::Approved { words, .. }) => words.as_ref(),
        None => None,
    };
    if let Some(value) = value {
        let mut start = 0;
        for end in 0..=value.bytes().len() {
            if end == value.bytes().len() || value.bytes().get(end) == Some(&b'\n') {
                let line = value.bytes().get(start..end).expect("line bounds").trim_ascii();
                if !line.is_empty() {
                    text(b, Role::Paragraph, line);
                    break;
                }
                start = end.checked_add(1).expect("line bounds");
            }
        }
    }
}

fn is_done(ending: &jig_client::contract::Ending) -> bool {
    match ending {
        jig_client::contract::Ending::Done => true,
        jig_client::contract::Ending::Task(ending) => match ending.as_ref() {
            jig_client::contract::ChipEnding::Done(_) => true,
            jig_client::contract::ChipEnding::Failed(_) | jig_client::contract::ChipEnding::Cancelled { .. } => false,
        },
        jig_client::contract::Ending::Failed { .. }
        | jig_client::contract::Ending::Cancelled { .. }
        | jig_client::contract::Ending::Withdrawn { .. }
        | jig_client::contract::Ending::Decided { .. }
        | jig_client::contract::Ending::Deleted => false,
    }
}
