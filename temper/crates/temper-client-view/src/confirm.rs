//! The page's deciding controls, their supplied facts, and its one confirmation.
use crate::cards::P;
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{
    Answered, Outcome,
    contract::{ActKind, Admission, Answer, Effect, ReadValue, ReleaseEffect},
};
use skein_ui::{Builder, Role, Run};
use temper_client_domain::{
    Domain, acts,
    pages::{Act, inbox::Card},
};
const ACTIONS: [(ActKind, Word); 7] = [
    (ActKind::Accept, Word::Accept),
    (ActKind::Reject, Word::Reject),
    (ActKind::Answer, Word::Answer),
    (ActKind::Release, Word::Release),
    (ActKind::LeaveHeld, Word::LeaveHeld),
    (ActKind::Take, Word::Take),
    (ActKind::HandBack, Word::HandBack),
];
fn button(b: &mut Builder, bindings: &mut Bindings, words: &Words, word: Word, act: Act) {
    b.open(Role::Button);
    b.name(&words.say(word, &[]));
    b.bind(bindings.insert(act));
    b.close();
}
pub(crate) fn card(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings, rows: u32) {
    let Some(page) = &card.page else { return };
    crate::cards::proposal_reason(b, page);
    facts(b, page, words, rows);
    let conversation = root.pages().chat().is_some() && words.shell == crate::words::Shell::Browser;
    if conversation {
        b.open(Role::Slot);
    }
    controls(b, root, card, words, bindings);
    if conversation {
        b.close();
    }
}
pub(crate) fn facts(b: &mut Builder, page: &ReadValue, words: &Words, rows: u32) {
    facts_at(b, page, words, rows, crate::cards::Place::Chat);
}
pub(crate) fn facts_at(b: &mut Builder, page: &ReadValue, words: &Words, rows: u32, place: crate::cards::Place) {
    crate::cards::proposal_facts(b, page, words, rows, place);
    if !crate::cards::proposal_has_rows(page) {
        crate::cards::decision(b, words, Some(page), ActKind::Accept);
    }
}
pub(crate) fn controls(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings) {
    card_controls(b, root, card, words, bindings);
    if proposal_object(card.object) {
        let name = words.say(Word::ProposalInbox, &[]);
        crate::link(
            b,
            &name,
            bindings.insert(Act::Go {
                address: temper_client_domain::Address::Inbox {
                    filter: temper_client_domain::address::Filter {
                        kind: Some(temper_client_domain::address::Kind::Proposal),
                        ..temper_client_domain::address::Filter::ALL
                    },
                },
            }),
        );
        b.text(Run::Plain, &name);
        b.close();
    }
}
pub(crate) fn card_controls(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings) {
    controls_at(b, root, card, words, bindings, false);
}

/// Browser card acts share the terminal's admissions, reasons and outcomes (client/README.md, 3.5).
pub(crate) fn needs_controls(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings) {
    controls_at(b, root, card, words, bindings, true);
}
fn controls_at(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings, needs: bool) {
    let Some(page) = &card.page else { return };
    let sheet = showing(root, card.object);
    if !sheet && let Some(outcome) = &card.outcome {
        outcome_words(b, words, outcome);
        return;
    }
    if !sheet && let Some(progress) = acts::pending(card, root.client()) {
        progress_text(b, words, progress);
        return;
    }
    if card.key.is_some() {
        return;
    }
    let mut first = true;
    for (act, word) in ACTIONS {
        if root.pages().task().is_some() && root.pages().inbox().is_none() && act != ActKind::Release {
            continue;
        }
        let Some(admission) = jig_client::facts::offer(acts::offers(page), act) else { continue };
        match admission {
            Admission::Allowed | Admission::Bounded { .. } => {
                let rejecting = act == ActKind::Reject && proposal_object(card.object);
                let answering = act == ActKind::Answer;
                let disclosure = rejecting || answering || (needs && act == ActKind::LeaveHeld);
                if disclosure {
                    b.open(Role::Disclosure);
                    b.name(&words.say(
                        if answering {
                            Word::AnswerInWords
                        } else if needs && act == ActKind::LeaveHeld {
                            Word::LeaveHeld
                        } else if needs {
                            Word::RejectProposal
                        } else {
                            Word::Reject
                        },
                        &[],
                    ));
                    if let Some(open) = root.pages().confirmation()
                        && open.object == card.object
                        && open.act == act
                    {
                        b.state(skein_ui::State::Expanded);
                    }
                }
                if act == ActKind::Answer || act == ActKind::Reject || act == ActKind::LeaveHeld {
                    b.open(Role::TextArea);
                    b.name(&words.say(if act == ActKind::Answer { Word::AnswerWords } else { Word::Reason }, &[]));
                    b.value(
                        match &card.words {
                            Some(words) => words.bytes(),
                            None => b"",
                        },
                        temper_client_domain::pages::shown(root.pages()),
                    );
                    b.bind(bindings.insert(Act::CardWords { object: card.object, words: Box::default() }));
                    b.close();
                }
                let word = if needs && act == ActKind::Release {
                    Word::NeedsRelease
                } else if rejecting {
                    Word::RejectProposal
                } else {
                    word
                };
                b.open(Role::Button);
                b.name(&words.say(word, &[]));
                b.bind(bindings.insert(Act::Decide { object: card.object, revision: acts::revision(page), act }));
                if (needs && (act == ActKind::Accept || act == ActKind::Release))
                    || (root.pages().chat().is_some() && first)
                {
                    b.style(crate::look::RECOMMENDED);
                }
                first = false;
                b.close();
                if disclosure {
                    b.close();
                }
            }
            Admission::Proposed { to } => {
                words.card_text(b, P::ProposeInstead, &[]);
                crate::cards::recipient(b, words, to);
            }
            Admission::Refused(refusal) => words.refusal(b, refusal, 0),
        }
    }
}

pub(crate) fn showing(root: &Domain, object: jig_client::contract::ObjectKey) -> bool {
    match root.pages().confirmation() {
        Some(open) => open.object == object,
        None => false,
    }
}
/// Each confirmation's sheet (client/README.md, 3.6, The sheet).
struct Sheet {
    title: Word,
    keep: Word,
    does: Word,
}
fn sheet_open(b: &mut Builder, _words: &Words, title: &[u8]) {
    b.open(Role::Dialog);
    b.style(crate::look::CONFIRM);
    b.name(title);
    b.open(Role::Heading { level: 2 });
    b.name(title);
    b.text(Run::Plain, title);
    b.close();
}
fn sheet_why(b: &mut Builder, root: &Domain, words: &Words) {
    let Some(open) = root.pages().confirmation() else { return };
    if let Some(key) = open.sent
        && let Some(pending) = root.client().pending(key)
    {
        progress_text(b, words, pending.progress);
    }
    if let Some(outcome) = &open.failed {
        outcome_words(b, words, outcome);
    }
}
fn sheet_acts(b: &mut Builder, bindings: &mut Bindings, words: &Words, sheet: &Sheet) {
    b.open(Role::Slot);
    b.key(skein_ui::Key { owner: crate::frame::owner(), kind: 60, a: 0, b: 0 });
    b.style(crate::look::CONFIRM_ACTS);
    button(b, bindings, words, sheet.keep, Act::Cancel);
    b.open(Role::Button);
    b.style(crate::look::CONFIRM_DO);
    b.name(&words.say(sheet.does, &[]));
    b.bind(bindings.insert(Act::Confirm));
    b.close();
    b.close();
}
fn lead(b: &mut Builder, words: &Words, word: Word) {
    b.open(Role::Paragraph);
    words.text(b, word, &[]);
    b.close();
}
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, rows: u32) {
    let Some(open) = root.pages().confirmation() else { return };
    if open.act == ActKind::Cancel && root.pages().chat().is_some() {
        close(b, root, words, bindings);
        return;
    }
    if let Some(page) = root.pages().task()
        && open.object == jig_client::contract::ObjectKey::Task(page.number)
        && (open.act == ActKind::Stop || open.act == ActKind::Cancel)
        && let Some(task) = page.document(root.client())
    {
        let goal = page.shape == temper_client_domain::pages::task::Shape::Tracked && open.act == ActKind::Cancel;
        let sheet = if goal {
            Sheet { title: Word::GoalStopTitle, keep: Word::KeepGoing, does: Word::StopGoal }
        } else if open.act == ActKind::Stop {
            Sheet { title: Word::StopTaskTitle, keep: Word::KeepGoing, does: Word::StopTask }
        } else {
            Sheet { title: Word::CancelTaskTitle, keep: Word::KeepIt, does: Word::CancelTask }
        };
        sheet_open(b, words, &words.say(sheet.title, &[]));
        if open.act == ActKind::Stop {
            lead(b, words, Word::StopHolds);
        } else if goal {
            lead(b, words, Word::GoalStopStays);
        } else {
            fates(b, task, &[], Word::WhatCancellingDoes, words, bindings);
        }
        sheet_why(b, root, words);
        sheet_acts(b, bindings, words, &sheet);
        b.close();
        return;
    }
    let sheet = match open.act {
        ActKind::Accept => Sheet { title: Word::AcceptTitle, keep: Word::NotYet, does: Word::Accept },
        ActKind::Reject => Sheet { title: Word::RejectTitle, keep: Word::KeepIt, does: Word::RejectProposal },
        ActKind::Release => Sheet { title: Word::ReleaseTitle, keep: Word::KeepHeld, does: Word::NeedsRelease },
        ActKind::Answer
        | ActKind::LeaveHeld
        | ActKind::Take
        | ActKind::HandBack
        | ActKind::Message
        | ActKind::Amend
        | ActKind::Cancel
        | ActKind::Stop
        | ActKind::Move
        | ActKind::PassUp
        | ActKind::Reorder
        | ActKind::Seen
        | ActKind::EditNote
        | ActKind::DeleteNote
        | ActKind::Adopt
        | ActKind::ChangePolicy
        | ActKind::Rename
        | ActKind::RemoveProject
        | ActKind::SetRoles
        | ActKind::MakeService
        | ActKind::SetPool
        | ActKind::Withdraw => return,
    };
    let Some(cards) = root.pages().cards() else { return };
    for card in cards {
        if card.object != open.object {
            continue;
        }
        let Some(page) = &card.page else { return };
        sheet_open(b, words, &words.say(sheet.title, &[]));
        crate::cards::proposal_at(b, page, words, rows, crate::cards::Place::Confirmation);
        if open.act != ActKind::Accept || !crate::cards::proposal_has_rows(page) {
            crate::cards::decision(b, words, Some(page), open.act);
        }
        if let Some(reason) = &card.words {
            b.open(Role::Paragraph);
            b.text(Run::Plain, reason.bytes());
            b.close();
        }
        sheet_why(b, root, words);
        if let Some(Effect::Release(effect)) = jig_client::facts::effect(acts::offers(page), open.act)
            && *effect == ReleaseEffect::Uncertain
        {
            b.open(Role::Slot);
            b.key(skein_ui::Key { owner: crate::frame::owner(), kind: 61, a: 0, b: 0 });
            button(b, bindings, words, Word::Again, Act::Again { again: true });
            button(b, bindings, words, Word::DoNotRepeat, Act::Again { again: false });
            b.close();
        }
        sheet_acts(b, bindings, words, &sheet);
        b.close();
        return;
    }
}

pub(crate) fn progress_text(b: &mut Builder, words: &Words, progress: jig_client::Progress) {
    crate::cards::progress(b, words, progress);
}

fn close(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings) {
    use jig_client::contract::Ask;
    let Some(chat) = root.pages().chat() else { return };
    let Some(Ask::Compound { operations, .. }) = &chat.closing else { return };
    let sheet = if chat.restarting {
        Sheet { title: Word::StartOverTitle, keep: Word::KeepGoing, does: Word::StartOver }
    } else {
        Sheet { title: Word::CloseTitle, keep: Word::KeepIt, does: Word::CloseChat }
    };
    sheet_open(b, words, &words.say(sheet.title, &[]));
    lead(b, words, if chat.restarting { Word::StartOverQuestion } else { Word::CloseCannotReopen });
    if let Some(task) = chat.task(root.client()) {
        fates(b, task, operations.as_slice(), Word::WhatClosingDoes, words, bindings);
    }
    sheet_why(b, root, words);
    sheet_acts(b, bindings, words, &sheet);
    b.close();
}
fn run_live(task: &jig_client::contract::Task) -> bool {
    use jig_client::contract::{ChipPhase, ChipPhaseFacts, RunStage};
    let stage = match task.chip.details.as_ref() {
        ChipPhaseFacts { phase: ChipPhase::Active(stage), .. } => Some(stage),

        ChipPhaseFacts { .. } => None,
    };
    match stage {
        Some(RunStage::Claimed { .. } | RunStage::Running { .. }) => true,
        Some(RunStage::Idle | RunStage::Due | RunStage::Preparing | RunStage::BackingOff { .. }) | None => false,
    }
}
pub(crate) fn fates(
    b: &mut Builder,
    task: &jig_client::contract::Task,
    operations: &[jig_client::contract::Operation],
    name: Word,
    words: &Words,
    bindings: &mut Bindings,
) {
    use jig_client::contract::Operation;
    use skein_ui::phrases::Hole;
    let mut delegates = 0_u32;
    for operation in operations {
        match operation {
            Operation::Cancel { task: number, .. } | Operation::Move { task: number, .. }
                if *number != task.chip.task =>
            {
                delegates = delegates.saturating_add(1);
            }
            Operation::Cancel { .. }
            | Operation::Move { .. }
            | Operation::Message { .. }
            | Operation::Amend { .. }
            | Operation::Release { .. }
            | Operation::Stop { .. } => {}
        }
    }
    let impact = match jig_client::facts::effect(task.acts.as_slice(), ActKind::Cancel) {
        Some(Effect::Cancel(impact)) => Some(impact),
        Some(Effect::Release(_) | Effect::Move { .. }) | None => None,
    };
    let below = match impact {
        Some(impact) => impact
            .tasks
            .waiting
            .saturating_add(impact.tasks.running)
            .saturating_add(impact.tasks.held)
            .saturating_add(impact.tasks.closing)
            .saturating_sub(delegates),
        None => 0,
    };
    let proposals = match impact {
        Some(impact) => impact.proposals,
        None => 0,
    };
    let run = run_live(task);
    if !run && delegates == 0 && below == 0 && proposals == 0 {
        return;
    }
    b.open(Role::List);
    b.style(crate::look::CONFIRM_FATES);
    b.name(&words.say(name, &[]));
    if run {
        let mut grace = None;
        for operation in operations {
            match operation {
                Operation::Cancel { task: number, grace: value, .. } if *number == task.chip.task => grace = *value,
                Operation::Cancel { .. }
                | Operation::Move { .. }
                | Operation::Message { .. }
                | Operation::Amend { .. }
                | Operation::Release { .. }
                | Operation::Stop { .. } => {}
            }
        }
        b.open(Role::Item);
        match grace {
            Some(grace) if grace.as_nanos() % 1_000_000_000 == 0 => {
                words.text(b, Word::CloseGrace, &[Hole::Span(grace)]);
            }
            Some(grace) => words.text(b, Word::CloseGraceNanos, &[Hole::Number(grace.as_nanos())]),
            None => words.text(b, Word::CloseRunNow, &[]),
        }
        b.close();
    }
    for operation in operations {
        match operation {
            Operation::Cancel { task: number, .. } if *number != task.chip.task => {
                delegate(b, task, *number, true, words, bindings);
            }
            Operation::Move { task: number, .. } => delegate(b, task, *number, false, words, bindings),
            Operation::Cancel { .. }
            | Operation::Message { .. }
            | Operation::Amend { .. }
            | Operation::Release { .. }
            | Operation::Stop { .. } => {}
        }
    }
    for (count, one, many) in
        [(below, Word::TasksBelowOne, Word::TasksBelow), (proposals, Word::ProposalWithdrawn, Word::ProposalsWithdrawn)]
    {
        if count != 0 {
            b.open(Role::Item);
            if count == 1 {
                words.text(b, one, &[]);
            } else {
                words.text(b, many, &[Hole::Number(u64::from(count))]);
            }
            b.close();
        }
    }
    b.open(Role::Item);
    words.text(b, Word::EffectsWithdrawn, &[]);
    b.close();
    b.close();
}
fn delegate(
    b: &mut Builder,
    parent: &jig_client::contract::Task,
    task: u64,
    cancel: bool,
    words: &Words,
    bindings: &mut Bindings,
) {
    use jig_client::shapes::Fate;
    use skein_ui::phrases::Hole;
    let mut title = b"".as_slice();
    for delegate in &parent.delegates {
        if delegate.chip.task == task {
            title = delegate.chip.title.bytes();
        }
    }
    b.open(Role::Item);
    b.key(crate::pages::inbox::card_key(jig_client::contract::ObjectKey::Task(task)));
    b.open(Role::Text);
    b.run(Run::Strong, &words.say(Word::DelegateNumber, &[Hole::Number(task)]));
    let fate = words.say(
        if cancel { Word::DelegateCancel } else { Word::DelegateMove },
        &[Hole::Name(skein_lib::bytes::copy_of(title))],
    );
    b.run(Run::Plain, if title.is_empty() { fate.get(1..).expect("fate starts with a space") } else { &fate });
    b.close();
    button(
        b,
        bindings,
        words,
        if cancel { Word::TakeInstead } else { Word::CancelInstead },
        Act::DelegateFate { task, fate: if cancel { Fate::Move } else { Fate::Cancel } },
    );
    b.close();
}

fn proposal_object(object: jig_client::contract::ObjectKey) -> bool {
    use jig_client::contract::ObjectKey;
    match object {
        ObjectKey::Proposal(_) => true,
        ObjectKey::Task(_)
        | ObjectKey::Question { .. }
        | ObjectKey::Escalation(_)
        | ObjectKey::PersonTask(_)
        | ObjectKey::Result(_)
        | ObjectKey::Note { .. } => false,
    }
}

/// An answer returned to a card (temper/docs/design/client/README.md, 3.5).
pub(crate) fn outcome_words(b: &mut Builder, words: &Words, outcome: &Outcome) {
    match outcome {
        Outcome::Answered(Answered::Jig(Answer::Done(outcome))) => words.outcome(b, outcome),
        Outcome::Answered(Answered::Jig(Answer::Refused { refusal, .. })) => {
            words.refusal(b, refusal, 0);
            if let jig_client::contract::Refusal::Moved { .. } = refusal {
                words.card_text(b, P::LookAgain, &[]);
            }
        }
        Outcome::Full { held, room_at } => crate::cards::answers_full(b, words, *held, *room_at),
        Outcome::NotKept | Outcome::Held => words.status(b, P::RequestNotKept, &[]),
        Outcome::Unknown => words.status(b, P::Unknown, &[]),
        Outcome::Limit => words.status(b, P::RequestsInFlight, &[]),
        Outcome::Unconfirmed => words.status(b, P::RequestUnconfirmed, &[]),
        Outcome::Answered(Answered::Family(_)) => {}
        Outcome::Unsent | Outcome::Lost => unreachable!("temper's requests carry no value"),
    }
}
