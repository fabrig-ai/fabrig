//! Standalone amendment settings (client/domain/objects.md, 4 and 11;
//! view.md, 7 and 10). Context comes from the enclosing owner document.
use crate::owner_amendment_words::AmendmentPhrase as P;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::Limits;
use jig_client::contract::{OwnerAmendmentState, PersonAddress};

/// Finite whole-node envelope shared by standalone and enclosing page renderers.
#[must_use]
pub fn max_nodes(limits: &Limits) -> Option<u32> {
    20_u32
        .checked_add(skein_ui::markdown::max_nodes(limits.text)?)?
        .checked_add(crate::specification::nodes(limits)?)?
        .checked_add(crate::owner_authority::document_nodes(limits.rows, limits.segments)?)?
        .checked_add(limits.rows)
}
fn saying(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let value =
        words.book().say(words.language(), phrase.id(), holes).expect("checked amendment sayings and typed holes");
    b.text(Run::Plain, &value.text);
}
/// Render only supplied owner facts, borrowing the normal checked language book.
/// No request, revision, actor, project, clock, origin or subscription is inferred.
pub fn render(b: &mut Builder, value: &OwnerAmendmentState, words: &Words, text_limit: u32, rows: u32) {
    b.open(Role::Region);
    let name = words.book().say(words.language(), P::State.id(), &[]).expect("checked amendment region name");
    b.name(&name.text);
    match value.addressed {
        Some(PersonAddress::Party(party)) => saying(b, words, P::Party, &[Hole::Number(party)]),
        Some(PersonAddress::Role(role)) => saying(b, words, P::Role, &[Hole::Number(u64::from(role))]),
        None => saying(b, words, P::AddressAbsent, &[]),
    }
    match value.taken_by {
        Some(party) => saying(b, words, P::TakenBy, &[Hole::Number(party)]),
        None => saying(b, words, P::TakerAbsent, &[]),
    }
    saying(b, words, P::Instructions, &[]);
    crate::instructions::document(b, value.instructions.as_ref(), words);
    saying(b, words, P::Specification, &[]);
    crate::specification::document(b, &value.spec, words);
    saying(b, words, P::Authority, &[]);
    crate::owner_authority::document_words(b, &value.authority, words, text_limit, rows);
    crate::owner_wake::policy(b, &value.wake, words.book(), words.language());
    for task in &value.dependencies {
        saying(b, words, P::Dependencies, &[Hole::Number(*task)]);
    }
    if value.cut {
        saying(b, words, P::DependenciesCut, &[]);
    }
    b.close();
}
