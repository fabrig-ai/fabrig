//! Person task facts are carried whole; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 7.
use super::{Body, Object};
use alloc::boxed::Box;

/// Names only the independently decidable facts on this card.
#[must_use]
#[cfg(test)]
pub fn revision(object: &Object) -> Option<jig_client::contract::Revision> {
    Some(owner_facts(object)?.revision)
}

/// The separate pending owner envelope in this working-set object.
#[must_use]
pub fn owner_facts(object: &Object) -> Option<&jig_client::contract::PersonTask> {
    match &object.body {
        Some(Body::PersonTask(document)) => Some(document),
        Some(
            Body::Escalation(_)
            | Body::Ended(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::Question(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
/// Positive forms the current person answer schema can express.
#[must_use]
pub fn answer(
    document: &jig_client::contract::PersonTask,
    option: Option<u32>,
    fields: &crate::state::fields::Fields,
) -> Option<jig_client::contract::PersonAnswer> {
    use crate::state::fields::Purpose;
    use jig_client::contract::{PersonAnswer, TaskContractDocument, Text};
    match &document.contract {
        TaskContractDocument::Approval { state, .. } => Some(PersonAnswer::Approved {
            state: state.clone(),
            words: match fields.get(Purpose::Reason) {
                Some(field) => Some(field.text.clone()),
                None => None,
            },
        }),
        TaskContractDocument::Report { words } => {
            let field = fields.get(Purpose::Answer)?;
            if !field.text.fits(*words) {
                return None;
            }
            Some(PersonAnswer::Words(field.text.clone()))
        }
        TaskContractDocument::Verdict { choices, .. } => {
            let option = option?;
            for choice in choices {
                if choice.code == option && choice.fields.is_empty() {
                    let reason = match fields.get(Purpose::Reason) {
                        Some(field) => field.text.clone(),
                        None => Text::new(Box::from([]), 0).expect("empty text fits"),
                    };
                    if !reason.fits(choice.words) {
                        return None;
                    }
                    return Some(PersonAnswer::Choice { option, reason });
                }
            }
            None
        }
        TaskContractDocument::Change { .. } => None,
    }
}
