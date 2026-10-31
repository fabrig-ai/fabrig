//! Exact batch member contracts, funding and connector references (objects.md, 4).
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::words::proposed_task_words::ProposedPhrase;
use crate::{JigBindings, JigMeaning, Owner};
use jig_client::words::DecisionWord as D;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, SlotRef};

use crate::Config;
use jig_client::contract::{Executor, Funder, Part, PartKey, PartKind, PersonAddress, TaskContractDocument};

pub fn text(b: &mut Builder, words: &Words, phrase: ProposedPhrase, holes: &[Hole]) {
    let saying = words.book().say(words.language(), phrase.id(), holes).expect("exact member words were checked");
    b.text(skein_ui::Run::Plain, &saying.text);
}
pub fn executor(b: &mut Builder, value: Executor, words: &Words) {
    match value {
        Executor::Agent { charter } => {
            text(b, words, ProposedPhrase::ExecutorCharter, &[Hole::Number(u64::from(charter))]);
        }
        Executor::Procedure { connector, code } => text(
            b,
            words,
            ProposedPhrase::ExecutorProcedure,
            &[Hole::Number(u64::from(connector)), Hole::Number(u64::from(code))],
        ),
        Executor::Person(address) => match address {
            PersonAddress::Party(person) => text(b, words, ProposedPhrase::ExecutorParty, &[Hole::Number(person)]),
            PersonAddress::Role(role) => text(b, words, ProposedPhrase::ExecutorRole, &[Hole::Number(u64::from(role))]),
        },
    }
}
pub fn funder(b: &mut Builder, value: Funder, words: &Words) {
    match value {
        Funder::Task(task) => words.decision_fact(b, D::FunderTask, &[Hole::Number(task)]),
        Funder::Pool { project, person, period } => words.decision_fact(
            b,
            D::FunderPool,
            &[Hole::Number(u64::from(project)), Hole::Number(person), Hole::Number(period)],
        ),
        Funder::Recurring { project, task, period } => words.decision_fact(
            b,
            D::FunderRecurring,
            &[Hole::Number(u64::from(project)), Hole::Number(task), Hole::Number(period)],
        ),
        Funder::Period { project, period } => {
            words.decision_fact(b, D::FunderPeriod, &[Hole::Number(u64::from(project)), Hole::Number(period)]);
        }
    }
}
/// Every actual funder identity is one checked saying leaf.
#[must_use]
#[cfg(test)]
pub const fn funder_nodes() -> u32 {
    1
}
pub fn part(b: &mut Builder, value: &Part, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Region);
    text(b, words, ProposedPhrase::PartConnector, &[Hole::Number(u64::from(value.connector))]);
    let kind = match value.kind {
        PartKind::Resource => ProposedPhrase::PartResource,
        PartKind::Pool => ProposedPhrase::PartPool,
        PartKind::Result => ProposedPhrase::PartResult,
        PartKind::Procedure => ProposedPhrase::PartProcedure,
        PartKind::Effect => ProposedPhrase::PartEffect,
        PartKind::Outbox => ProposedPhrase::PartOutbox,
        PartKind::Hold => ProposedPhrase::PartHold,
    };
    let name = words.book().say(words.language(), kind.id(), &[]).expect("exact part kind words were checked");
    b.name(&name.text);
    text(b, words, kind, &[]);
    match &value.key {
        PartKey::Number(number) => text(b, words, ProposedPhrase::PartNumber, &[Hole::Number(*number)]),
        PartKey::Path(segments) => {
            let reserved = u32::try_from(name.text.len()).expect("bounded resource words");
            crate::words::literal_segment::path(
                b,
                segments.as_slice(),
                words,
                crate::words::literal_segment::PathDisplay { text: bindings.text_limit(), reserved, keep_last: true },
            );
        }
    }
    let token = bindings.insert(JigMeaning::Part(value.clone())).token;
    b.slot(
        SlotRef { owner: (Owner::Application { number: value.connector }).opaque(), token },
        &words.say(JigPhrase::LoadingPart, &[]),
    );
    b.close();
}
pub fn contract(b: &mut Builder, value: &TaskContractDocument, words: &Words, bindings: &mut JigBindings) {
    match value {
        TaskContractDocument::Approval { state, part: reference } => {
            text(b, words, ProposedPhrase::ContractApproval, &[]);
            crate::words::literal_segment::render(b, state);
            part(b, reference, words, bindings);
        }
        TaskContractDocument::Report { words: cap } => {
            text(b, words, ProposedPhrase::ContractReport, &[Hole::Number(u64::from(*cap))]);
        }
        TaskContractDocument::Verdict { choices, cut } => {
            text(b, words, ProposedPhrase::ContractVerdict, &[]);
            for choice in choices {
                text(
                    b,
                    words,
                    ProposedPhrase::VerdictChoice,
                    &[
                        Hole::Number(u64::from(choice.code)),
                        Hole::Number(u64::from(choice.words)),
                        Hole::Number(u64::from(choice.followups)),
                    ],
                );
                for field in &choice.fields {
                    crate::words::literal_segment::render(b, &field.name);
                    text(b, words, ProposedPhrase::FieldCap, &[Hole::Number(u64::from(field.max))]);
                }
            }
            if *cut {
                text(b, words, ProposedPhrase::ContractCut, &[]);
            }
        }
        TaskContractDocument::Change { connector, kind, words: cap } => text(
            b,
            words,
            ProposedPhrase::ContractChange,
            &[Hole::Number(u64::from(*connector)), Hole::Number(u64::from(*kind)), Hole::Number(u64::from(*cap))],
        ),
    }
}
#[must_use]
pub fn part_nodes(_limits: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    client.segments.max(1).checked_add(6)
}
#[must_use]
pub fn contract_nodes(limits: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    let approval = part_nodes(limits, client)?.checked_add(2)?;
    let verdict = client.rows.checked_mul(2)?.checked_add(1)?.checked_mul(client.rows)?.checked_add(2)?;
    Some(approval.max(verdict).max(1))
}

/// A pending span supersedes the optional authority wall in the retained facts.
pub fn authority(
    b: &mut Builder,
    task: &jig_client::contract::ProposedTask,
    words: &Words,
    text_limit: u32,
    rows: u32,
) {
    match task.deadline {
        jig_client::contract::NewDeadline::Named => {
            crate::tree::fragments::authority::document_words(b, &task.authority, words, text_limit, rows);
        }
        jig_client::contract::NewDeadline::Span(span) => {
            let mut given = task.authority.clone();
            given.authority.deadline = None;
            crate::tree::fragments::authority::document_words(b, &given, words, text_limit, rows);
            text(b, words, ProposedPhrase::DeadlineSpan, &[Hole::Span(span)]);
        }
    }
}
