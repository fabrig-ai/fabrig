//! Bounded typed amendment fields; no authority judgment (view.md, 18.5).
use crate::state::clock::deadline;
use crate::tree::cards::proposal;
use crate::tree::fragments::proposed_task;
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::{JigBindings, JigMeaning};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{
    Borrowed,
    state::{
        action::Act,
        confirm::Confirmation,
        objects::{Body, Object},
    },
};
use alloc::boxed::Box;
use jig_client::contract::{
    AddressChange, Amendment, Executor, PersonAddress, Recipient, ResultsWake, Revision, Task, Text, WakeRule,
};
/// The one property replaced by an input's whole value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Words,
    Instructions,
    Budget,
    Deadline,
    Party,
    Role,
}
/// Decode one field against the exact amendment snapshot that named its token.
#[must_use]
pub fn edit(before: &Amendment, selected: &Amendment, field: Field, value: Box<[u8]>, text_limit: u32) -> Option<Act> {
    let mut change = selected.clone();
    match field {
        Field::Words => {
            let mut spec = selected.spec.clone().or(before.spec.clone())?;
            spec.words = Text::new(value, text_limit).ok()?;
            change.spec = Some(spec);
        }
        Field::Instructions => change.instructions = Some(Text::new(value, text_limit).ok()?),
        Field::Budget => {
            let mut authority = selected.authority.clone().or(before.authority.clone())?;
            authority.spend = skein_ui::number(&value)?;
            change.authority = Some(authority);
        }
        Field::Deadline => {
            let mut authority = selected.authority.clone().or(before.authority.clone())?;
            authority.deadline = Some(deadline(&value)?);
            change.authority = Some(authority);
        }
        Field::Party => {
            let mut addressed = selected.addressed.or(before.addressed)?;
            addressed.address = PersonAddress::Party(skein_ui::number(&value)?);
            change.addressed = Some(addressed);
        }
        Field::Role => {
            let mut addressed = selected.addressed.or(before.addressed)?;
            addressed.address = PersonAddress::Role(u32::try_from(skein_ui::number(&value)?).ok()?);
            change.addressed = Some(addressed);
        }
    }
    Some(Act::Change { change: Box::new(change) })
}
/// Show all before facts and form fields; each input preserves the other selected changes.
pub fn form(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: &Object,
    open: &Confirmation,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let text_limit = bindings.text_limit();
    let Some(before) = before(object) else {
        return;
    };
    words.text(b, JigPhrase::Before, &[]);
    proposal::amendment(b, &before, words, bindings, client.limits.rows);
    loss(b, object, words);
    task_loss(b, object, words);
    authority_loss(b, object, words, text_limit, client.limits.rows);
    let empty = empty();
    let change = open.choices.change.as_ref().unwrap_or(&empty);
    let value = values(&before, change);
    words.text(b, JigPhrase::After, &[]);
    proposal::amendment(b, change, words, bindings, client.limits.rows);
    if value.spec.is_some() {
        input(b, &value, open, Field::Words, JigPhrase::NewSpec, words, bindings);
    }
    input(b, &value, open, Field::Instructions, JigPhrase::NewInstructions, words, bindings);
    if value.authority.is_some() {
        input(b, &value, open, Field::Budget, JigPhrase::NewBudget, words, bindings);
        input(b, &value, open, Field::Deadline, JigPhrase::NewDeadline, words, bindings);
    }
    if let Some(to) = &value.addressed {
        proposal::address_change(b, to, words);
        input(b, &value, open, Field::Party, JigPhrase::AddressParty, words, bindings);
        input(b, &value, open, Field::Role, JigPhrase::AddressRole, words, bindings);
    }
    if let Some(dependencies) = &value.dependencies {
        for task in dependencies {
            let mut without = skein_lib::List::with_capacity(dependencies.capacity());
            for other in dependencies {
                if other != task {
                    without.push(*other).expect("filtered dependencies fit");
                }
            }
            let mut next = change.clone();
            next.dependencies = Some(without);
            crate::tree::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::DropDependency,
                &[Hole::Number(*task)],
                Act::Change { change: Box::new(next) },
            );
        }
    }
    if let Some(wake) = &value.wake {
        let mut next = change.clone();
        let mut policy = *wake;
        policy.words = match wake.words {
            WakeRule::Never => WakeRule::Immediate,
            WakeRule::Immediate | WakeRule::Batch { .. } => WakeRule::Never,
        };
        next.wake = Some(policy);
        crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::WakeInbox,
            &[],
            Act::Change { change: Box::new(next) },
        );
        let mut next = change.clone();
        let mut policy = *wake;
        policy.results = match wake.results {
            ResultsWake::Never => ResultsWake::Each,
            ResultsWake::Each | ResultsWake::LastOrFailure => ResultsWake::Never,
        };
        next.wake = Some(policy);
        crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::WakeResult,
            &[],
            Act::Change { change: Box::new(next) },
        );
    }
    // Funding is kept with the document and may change independently of its spec revision.
    match &object.body {
        Some(Body::Task(doc)) => proposed_task::funder(b, doc.budget.funder, words),
        Some(Body::OwnerEscalation(doc)) => proposed_task::funder(b, doc.task.budget.funder, words),

        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => {}
    }
}
fn authority_loss(b: &mut Builder, object: &Object, words: &Words, text_limit: u32, rows: u32) {
    let document = match &object.body {
        Some(Body::Task(doc)) => Some(&doc.authority),
        Some(Body::OwnerEscalation(doc)) => Some(&doc.task.authority),

        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    };
    if let Some(document) = document
        && document.cut
    {
        crate::tree::fragments::authority::document_words(b, document, words, text_limit, rows);
    }
}
fn task_loss(b: &mut Builder, object: &Object, words: &Words) {
    match &object.body {
        Some(Body::Task(doc)) if doc.cut => words.text(b, JigPhrase::TaskListsCut, &[]),
        Some(Body::OwnerEscalation(doc)) if doc.task.cut => words.text(b, JigPhrase::TaskListsCut, &[]),
        Some(
            Body::Task(_)
            | Body::Historical(_)
            | Body::Chip(_)
            | Body::OwnerProposal(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => {}
    }
}
fn loss(b: &mut Builder, object: &Object, words: &Words) {
    match &object.body {
        Some(Body::Task(doc)) => crate::tree::fragments::instructions::before(b, doc.instructions.as_ref(), words),
        Some(Body::OwnerEscalation(doc)) => {
            crate::tree::fragments::instructions::before(b, doc.task.instructions.as_ref(), words);
        }

        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => {}
    }
    let doc = match &object.body {
        Some(Body::Task(doc)) => Some(&doc.spec),
        Some(Body::OwnerEscalation(doc)) => Some(&doc.task.spec),

        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    };
    let Some(doc) = doc else {
        return;
    };
    if !doc.has_loss() {
        return;
    }
    crate::tree::fragments::specification::document(b, doc, words);
    if !doc.cut {
        words.text(b, JigPhrase::SpecificationCut, &[]);
    }
}
fn numbers(chips: &skein_lib::List<jig_client::contract::OwnerTaskLink>) -> skein_lib::List<u64> {
    let mut numbers = skein_lib::List::with_capacity(chips.capacity());
    for chip in chips {
        numbers.push(chip.number()).expect("one number per supplied reference fits");
    }
    numbers
}
fn empty() -> Amendment {
    Amendment {
        addressed: None,
        spec: None,
        instructions: None,
        wake: None,
        dependencies: None,
        authority: None,
        tracked: None,
        llm: None,
    }
}
// Display/edit values come from the source unless the person selected a whole replacement.
// The replacement request remains the separate selected delta.
fn values(before: &Amendment, change: &Amendment) -> Amendment {
    Amendment {
        addressed: change.addressed.or(before.addressed),
        spec: change.spec.clone().or(before.spec.clone()),
        instructions: change.instructions.clone().or(before.instructions.clone()),
        wake: change.wake.or(before.wake),
        dependencies: change.dependencies.clone().or(before.dependencies.clone()),
        authority: change.authority.clone().or(before.authority.clone()),
        tracked: change.tracked.or(before.tracked),
        llm: change.llm.clone().or(before.llm.clone()),
    }
}
fn input(
    b: &mut Builder,
    value: &Amendment,
    open: &Confirmation,
    field: Field,
    phrase: JigPhrase,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let empty = empty();
    let change = open.choices.change.as_ref().unwrap_or(&empty);
    b.open(match field {
        Field::Words | Field::Instructions => Role::TextArea,
        Field::Budget | Field::Deadline | Field::Party | Field::Role => Role::TextField,
    });
    b.name(&words.say(phrase, &[]));
    match field {
        Field::Words => {
            if let Some(value) = &value.spec {
                b.value(value.words.bytes(), open.writes);
            }
        }
        Field::Instructions => {
            if let Some(value) = &value.instructions {
                b.value(value.bytes(), open.writes);
            }
        }
        Field::Budget => {
            if let Some(authority) = &value.authority {
                b.value(&words.say(JigPhrase::Amount, &[Hole::Amount(authority.spend)]), open.writes);
            }
        }
        Field::Deadline | Field::Party | Field::Role => b.value(&[], open.writes),
    }
    b.bind(bindings.insert(JigMeaning::Amendment {
        before: Box::new(value.clone()),
        change: Box::new(change.clone()),
        field,
    }));
    b.close();
}

fn before(object: &Object) -> Option<Amendment> {
    Some(match &object.body {
        Some(Body::Task(doc)) => task_before(doc),
        Some(Body::OwnerEscalation(doc)) => task_before(&doc.task),

        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::OwnerProposal(_)
            | Body::Question(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => return None,
    })
}

// A held person task keeps its concrete executor; lifecycle alone is not its address.
fn task_address(doc: &Task) -> Option<AddressChange> {
    let revision = doc.person_revision?;
    let address = match doc.chip.executor {
        Executor::Party { party } => PersonAddress::Party(party),
        Executor::Role { project, role } if project == doc.project => PersonAddress::Role(role),
        Executor::Role { .. } | Executor::Agent { .. } | Executor::Procedure { .. } => return None,
    };
    Some(AddressChange { revision, address })
}
fn recipient_address(
    revision: Option<Revision>,
    to: Option<&Recipient>,
    project: Option<u32>,
) -> Option<AddressChange> {
    let revision = revision?;
    let address = match to? {
        Recipient::Party(party) => PersonAddress::Party(*party),
        Recipient::Role { project: scope, role } if Some(*scope) == project => PersonAddress::Role(*role),
        Recipient::Role { .. } | Recipient::Task(_) | Recipient::Roles { .. } => return None,
    };
    Some(AddressChange { revision, address })
}

fn task_before(doc: &Task) -> Amendment {
    Amendment {
        addressed: task_address(doc),
        spec: doc.spec.whole(),
        instructions: crate::tree::fragments::instructions::whole(doc.instructions.as_ref()),
        wake: Some(doc.wake),
        dependencies: if doc.cut { None } else { Some(numbers(&doc.dependencies)) },
        authority: doc.authority.whole(),
        tracked: None,
        llm: doc.llm.clone(),
    }
}
