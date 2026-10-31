//! Pending person work, supplied contracts and shown results
//! (client/domain/objects.md, 7; view.md, 7 and 10).
use crate::words::Words;
use crate::words::person_task_words::PersonPhrase as P;
use crate::words::phrases::jig::JigPhrase;
use crate::{JigBindings, JigMeaning};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run, State};

use crate::{
    Borrowed, Limits,
    state::{action::Act, fields::Purpose, objects::Object},
};
use alloc::boxed::Box;
use jig_client::contract::{ChipPhase, OwnerPersonTask, PersonAddress, TaskContractDocument};
use skein_lib::Id;
/// Every shown link and optional whole result, specification and contract.
#[must_use]
pub fn max_nodes(l: &Limits) -> Option<u32> {
    // The reused card/link chip renderer shows the brief phase, not the
    // standalone full metadata/result renderer. All metadata remains stored.
    let chip = crate::chip_nodes(l)?;
    crate::sizes::sizes::result_facts(l)?
        .checked_add(chip)?
        .checked_add(5)?
        .checked_mul(l.rows)?
        .checked_add(chip)?
        .checked_add(crate::tree::fragments::specification::nodes(l)?)?
        .checked_add(crate::tree::fragments::proposed_task::contract_nodes(l)?)?
        .checked_add(20)
}
/// Choices, entered fields and Cannot alongside the full contract.
#[must_use]
pub fn form_nodes(l: &Limits) -> Option<u32> {
    crate::tree::fragments::proposed_task::contract_nodes(l)?.checked_add(controls_nodes(l)?)
}
/// The single confirmation form's controls, alongside its contract.
pub(crate) fn controls_nodes(l: &Limits) -> Option<u32> {
    l.rows.checked_mul(3)?.checked_add(5)
}
fn saying(words: &Words, phrase: P, holes: &[Hole]) -> Box<[u8]> {
    words.book().say(words.language(), phrase.id(), holes).expect("checked person words and typed holes").text
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    b.text(Run::Plain, &saying(words, phrase, holes));
}
/// Render only the supplied facts. Missing results stay beside their whole links.
pub fn render(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &OwnerPersonTask,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::Region);
    b.name(&saying(words, P::PersonTask, &[]));
    crate::tree::cards::chip::chip(b, client, &value.task, words, bindings);
    text(b, words, P::Revision, &[Hole::Number(value.revision.value)]);
    crate::tree::fragments::result::requester(b, value.requester, words);
    crate::tree::fragments::specification::document(b, &value.spec, words);
    match value.address {
        PersonAddress::Party(number) => text(b, words, P::Party, &[Hole::Number(number)]),
        PersonAddress::Role(number) => text(b, words, P::Role, &[Hole::Number(u64::from(number))]),
    }
    match value.taken_by {
        Some(number) => text(b, words, P::Taker, &[Hole::Number(number)]),
        None => text(b, words, P::TakerAbsent, &[]),
    }
    match value.since {
        Some(at) => text(b, words, P::Since, &[Hole::Wall(words.adjusted_wall(at))]),
        None => text(b, words, P::SinceAbsent, &[]),
    }
    match value.stall {
        Some(span) => text(b, words, P::Stall, &[Hole::Span(span)]),
        None => text(b, words, P::StallAbsent, &[]),
    }
    match value.deadline {
        Some(at) => text(b, words, P::Deadline, &[Hole::Wall(words.adjusted_wall(at))]),
        None => text(b, words, P::DeadlineAbsent, &[]),
    }
    b.open(Role::List);
    b.name(&saying(words, P::Results, &[]));
    for shown in &value.results {
        b.open(Role::Item);
        crate::tree::fragments::result::link(b, client, &shown.task, words, bindings);
        match &shown.result {
            Some(result) => crate::tree::cards::result::facts(b, client, result, words, bindings),
            None => text(b, words, P::MissingResult, &[]),
        }
        b.close();
    }
    b.close();
    if value.cut {
        text(b, words, P::Cut, &[]);
    }
    crate::tree::fragments::proposed_task::contract(b, &value.contract, words, bindings);
    b.close();
}
fn controls(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    contract: &TaskContractDocument,
    words: &Words,
    bindings: &mut JigBindings,
) {
    if let Some(fact) = client.objects().get(object)
        && let Some(value) = crate::state::objects::person_task::owner_facts(&fact)
    {
        for dependency in &value.task.details.details.dependencies {
            if dependency.keeps && dependency.pending {
                words.text(b, JigPhrase::KeptAnswer, &[]);
                return;
            }
        }
    }
    let answering = crate::tree::cards::common::answering(client, object);
    match contract {
        TaskContractDocument::Verdict { choices, .. } => {
            b.open(Role::ChoiceGroup);
            b.name(&words.say(JigPhrase::Options, &[]));
            for choice in choices {
                b.open(Role::Choice);
                let caption = saying(words, P::Choice, &[Hole::Number(u64::from(choice.code))]);
                b.name(&caption);
                b.text(Run::Plain, &caption);
                if answering && choice.fields.is_empty() {
                    if let Some(open) = client.confirmation()
                        && open.choices.option == Some(choice.code)
                    {
                        b.state(State::Selected);
                    }
                    b.bind(bindings.insert(JigMeaning::Action(Act::Choose { option: choice.code })));
                } else {
                    b.state(State::Disabled);
                }
                b.close();
                if !choice.fields.is_empty() {
                    text(b, words, P::FieldsUnavailable, &[]);
                }
            }
            b.close();
        }
        TaskContractDocument::Report { .. } => {
            if answering {
                crate::tree::cards::common::field(b, client, Purpose::Answer, JigPhrase::AnswerField, words, bindings);
            }
        }
        TaskContractDocument::Change { .. } => text(b, words, P::ChangeUnavailable, &[]),
        TaskContractDocument::Approval { .. } => {}
    }
    if answering {
        crate::tree::frame::control(b, words, bindings, Role::Button, JigPhrase::CannotDo, &[], Act::CannotDo);
        crate::tree::cards::common::field(b, client, Purpose::Reason, JigPhrase::ReasonField, words, bindings);
    }
}
/// The confirmation reuses the same full contract and supported form controls.
pub fn form(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    contract: &TaskContractDocument,
    words: &Words,
    bindings: &mut JigBindings,
) {
    crate::tree::fragments::proposed_task::contract(b, contract, words, bindings);
    controls(b, client, object, contract, words, bindings);
}
/// One shared native card and original keyed answer, without a pending ending.
pub fn card(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &crate::tree::cards::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let text_limit = bindings.text_limit();
    let Some(fact) = client.objects().get(object) else {
        return false;
    };
    let Some(value) = crate::state::objects::person_task::owner_facts(&fact) else {
        return false;
    };
    if !crate::tree::cards::open(b, client, object, place, &saying(words, P::PersonTask, &[]), words, bindings) {
        return false;
    }
    render(b, client, value, words, bindings);
    // The card keeps the whole contract; editable drafts live only in its confirmation.
    crate::tree::cards::common::answer(b, &fact, words, text_limit);
    let mut acts = skein_lib::List::with_capacity(value.acts.capacity());
    let kept = {
        let ChipPhase { details, .. } = value.task.details.as_ref();

        let mut kept = false;
        for dependency in &details.dependencies {
            kept |= dependency.keeps && dependency.pending;
        }
        kept
    };
    for act in &value.acts {
        if kept && act.act == jig_client::contract::ActKind::Answer {
            continue;
        }
        acts.push(act.clone()).expect("same bounded offers");
    }
    crate::tree::cards::common::acts(b, &fact.card, &acts, object, words, bindings);
    crate::tree::cards::close(b);
    true
}
