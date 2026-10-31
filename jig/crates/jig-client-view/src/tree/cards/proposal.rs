//! Proposal action, funding and history in words (client/domain/objects.md, section 4).
use crate::JigBindings;
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{Borrowed, state::objects::Object};
use jig_client::contract::{Amendment, ProposedTask};
use skein_lib::Id;

pub(crate) fn proposed_task(
    b: &mut Builder,
    task: &ProposedTask,
    words: &Words,
    bindings: &mut JigBindings,
    rows: u32,
) {
    let text_limit = bindings.text_limit();
    b.open(Role::Item);
    words.text(b, JigPhrase::BatchTask, &[Hole::Number(task.number)]);
    crate::tree::fragments::specification::document(b, &task.spec, words);
    crate::tree::fragments::proposed_task::executor(b, task.executor, words);
    crate::tree::fragments::proposed_task::contract(b, &task.contract, words, bindings);
    crate::tree::fragments::proposed_task::authority(b, task, words, text_limit, rows);
    words.text(b, JigPhrase::Budget, &[Hole::Amount(task.budget)]);
    crate::tree::fragments::proposed_task::funder(b, task.funder, words);
    crate::tree::fragments::wake::policy(b, &task.wake, words.book(), words.language());
    if let Some(priority) = task.priority {
        words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(priority))]);
    }
    for dependency in &task.dependencies {
        words.text(b, JigPhrase::AfterTask, &[Hole::Number(*dependency)]);
    }
    for resource in &task.resources {
        crate::tree::fragments::proposed_task::part(b, resource, words, bindings);
    }
    if task.cut {
        crate::tree::fragments::proposed_task::text(
            b,
            words,
            crate::words::proposed_task_words::ProposedPhrase::TaskCut,
            &[],
        );
    }
    b.close();
}
/// Changes retain the before/after facts instead of interpreting instructions.
pub fn amendment(b: &mut Builder, change: &Amendment, words: &Words, bindings: &mut JigBindings, rows: u32) {
    let text_limit = bindings.text_limit();
    if let Some(to) = &change.addressed {
        address_change(b, to, words);
    }
    if let Some(spec) = &change.spec {
        crate::tree::fragments::specification::specification(b, spec, words);
    }
    if let Some(text) = &change.instructions {
        skein_ui::markdown::read(text.bytes(), 2, b);
    }
    if let Some(wake) = &change.wake {
        crate::tree::fragments::wake::policy(b, wake, words.book(), words.language());
    }
    if let Some(dependencies) = &change.dependencies {
        for task in dependencies {
            words.text(b, JigPhrase::ContextTask, &[Hole::Number(*task)]);
        }
    }
    if let Some(authority) = &change.authority {
        crate::tree::fragments::authority::render_words(b, authority, words, text_limit, rows);
    }
    b.open(Role::Separator);
    b.close();
}

/// A named person revision and exact closed destination; the ask owns its project scope.
pub(crate) fn address_change(b: &mut Builder, change: &jig_client::contract::AddressChange, words: &Words) {
    words.text(b, JigPhrase::PersonAddressRevision, &[Hole::Number(change.revision.value)]);
    crate::tree::fragments::proposed_task::executor(
        b,
        jig_client::contract::OwnerExecutor::Person(change.address),
        words,
    );
}

/// The same keyed card for the independently supplied pending owner document.
pub fn owner_proposal(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<Object>,
    place: &super::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    let text_limit = bindings.text_limit();
    let Some(facts) = client.objects().get(object) else {
        return false;
    };
    let document = match &facts.body {
        Some(crate::state::objects::Body::OwnerProposal(document)) => document,
        Some(
            crate::state::objects::Body::Historical(_)
            | crate::state::objects::Body::Chip(_)
            | crate::state::objects::Body::Task(_)
            | crate::state::objects::Body::OwnerEscalation(_)
            | crate::state::objects::Body::Question(_)
            | crate::state::objects::Body::OwnerPersonTask(_)
            | crate::state::objects::Body::Result(_)
            | crate::state::objects::Body::Note { .. },
        )
        | None => return false,
    };
    let title = words.say(JigPhrase::Proposal, &[]);
    if !super::open(b, client, object, place, &title, words, bindings) {
        return false;
    }
    crate::tree::fragments::proposal::render(b, client, document, words, bindings);
    super::common::answer(b, &facts, words, text_limit);
    super::common::acts(b, &facts.card, &document.acts, object, words, bindings);
    super::close(b);
    true
}
