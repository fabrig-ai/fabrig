//! An agent's card reads the reader's offer in the charter's order
//! (temper/docs/design/domain/forge/change/branch-path.md).
use crate::{Domain, ways};
use alloc::boxed::Box;
use jig_domain_core::{
    Charter,
    documents::Offered,
    settings::{CatalogueModel, Setting, Value},
};
use skein_lib::List;

pub(super) fn agent(domain: &Domain, project: u32, party: u64, number: u32) -> Option<ways::Agent> {
    let charter = domain.core.current.charter(number)?;
    let policy = &charter.policy;
    let offer =
        domain.core.offer(jig_domain_tasks::Payer::Party(party), project, allowed_models(domain, project, party));
    let order = order(domain, charter);
    let (selected, usable) = select(order.as_slice(), offer.as_slice());
    let model = match selected {
        Some(index) => domain.core.settings.models.get(usize::from(index)),
        None => None,
    };
    let name = match selected {
        Some(index) => {
            let model = domain.core.settings.models.get(usize::from(index)).expect("selected configured model");
            let number = u32::from(index).checked_add(1).expect("model number");
            Some(match row(domain, number) {
                Some(row) => match &row.shown {
                    Some(shown) => shown.clone(),
                    None => model.name.clone(),
                },
                None => model.name.clone(),
            })
        }
        None => None,
    };
    let mut effort = match model {
        Some(model) => match model.default_effort() {
            Some(effort) => Some(Box::from(effort)),
            None => None,
        },
        None => None,
    };
    if let Some(model) = model {
        for named in core::iter::once(&policy.model).chain(policy.alternatives.iter()) {
            if named.name == model.name {
                if let Some(chosen) = &named.effort
                    && model.efforts.contains(chosen)
                {
                    effort = Some(chosen.clone());
                }
                break;
            }
        }
    }
    Some(ways::Agent {
        charter: number,
        model: name,
        effort,
        usable,
        can: ways::can(
            policy.inspect,
            policy.modify,
            policy.shell,
            policy.agents,
            policy.tools.0 & jig_domain_core::ToolFamilies::JIG.settings != 0,
        )
        .into(),
        asked: ways::first_sentence(&policy.instructions, ways::ASKED.checked_sub("“”".len()).expect("quote room")),
    })
}

fn allowed_models(domain: &Domain, project: u32, party: u64) -> jig_domain_tasks::Models {
    if let Some(role) = domain.core.people.role(party, project)
        && let Some(policy) = domain.core.authority.policy(project)
    {
        for held in &policy.roles {
            if held.number == role.number() {
                return jig_domain_tasks::Models::Named(held.authority.models.bits());
            }
        }
    }
    jig_domain_tasks::Models::NONE
}

fn row(domain: &Domain, number: u32) -> Option<&CatalogueModel> {
    match domain.core.current.value(Setting::Model(number)) {
        Some(Value::Model(model)) => Some(model),
        Some(
            Value::Overlay(_)
            | Value::Charter(_)
            | Value::Template(_)
            | Value::Policy(_)
            | Value::Rules(_)
            | Value::Application(_),
        )
        | None => None,
    }
}

fn order(domain: &Domain, charter: &Charter) -> List<u8> {
    let mut order = List::with_capacity(jig_domain_tasks::MODELS);
    for named in core::iter::once(&charter.policy.model).chain(charter.policy.alternatives.iter()) {
        for (at, model) in domain.core.settings.models.iter().enumerate() {
            let index = u8::try_from(at).expect("configured model index");
            if model.name == named.name && available(domain, index) && !order.as_slice().contains(&index) {
                order.push(index).expect("unique configured models");
            }
        }
    }
    if charter.rest {
        for (at, _) in domain.core.settings.models.iter().enumerate() {
            let index = u8::try_from(at).expect("configured model index");
            if available(domain, index) && !order.as_slice().contains(&index) {
                order.push(index).expect("unique configured models");
            }
        }
    }
    order
}

fn available(domain: &Domain, index: u8) -> bool {
    let model = domain.core.settings.models.get(usize::from(index)).expect("configured model");
    if model.is_hole() {
        return false;
    }
    let number = u32::from(index).checked_add(1).expect("model number");
    match row(domain, number) {
        Some(row) => !row.withdrawn,
        None => true,
    }
}

pub(super) fn select(order: &[u8], offer: &[Offered]) -> (Option<u8>, bool) {
    for &model in order {
        for held in offer {
            if model == held.model {
                return (Some(model), true);
            }
        }
    }
    (order.first().copied(), false)
}
