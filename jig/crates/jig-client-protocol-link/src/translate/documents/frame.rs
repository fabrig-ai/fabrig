//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{Limits, Wall, c, too_large, w};

pub(crate) fn frame_party_fields(value: &w::MeParty, limits: &Limits) -> Result<c::MeParty, w::Problem> {
    Ok(c::MeParty {
        number: value.number(),
        kind: crate::translate::answers::party_kind(value.kind()),
        login: crate::translate::facts::document_text(value.login(), limits)?,
        name: crate::translate::facts::document_text(value.name(), limits)?,
    })
}

pub(crate) fn pool_ledger_fields(value: &w::PoolLedger) -> c::PoolLedger {
    let opt0 = match value.parent() {
        Some(item) => Some(crate::translate::facts::funder(item)),
        None => None,
    };
    let opt1 = match value.began() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match value.next() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    c::PoolLedger {
        source: crate::translate::facts::funder(value.source()),
        parent: opt0,
        numbers: crate::translate::facts::numbers(value.numbers()),
        left: value.left(),
        closed: value.closed(),
        began: opt1,
        next: opt2,
    }
}

pub(crate) fn project_role_fields(value: &w::MeProject, limits: &Limits) -> Result<c::MeProject, w::Problem> {
    let opt1 = match value.pool() {
        Some(item) => Some(pool_ledger_fields(item)),
        None => None,
    };
    Ok(c::MeProject {
        public: *value.public(),
        project: value.project(),
        name: crate::translate::facts::document_text(value.name(), limits)?,
        role: crate::translate::answers::held_role(value.role()),
        pool: opt1,
        templates: super::me::templates(value.templates(), limits)?,
        creating: match value.creating() {
            Some(value) => Some(c::Creating {
                within: match super::bounds_fields(&w::Bounds::Create(value.within().clone()), limits)? {
                    c::Bounds::Create(value) => value,
                    c::Bounds::Amend { .. } => unreachable!("wire creating bounds"),
                },
                beyond: super::beyond_fields(value.beyond(), limits)?,
            }),
            None => None,
        },
        models: crate::translate::llm::models(value.models(), limits)?,
    })
}

pub(crate) fn needs_fields(value: &w::Needs) -> c::Needs {
    c::Needs {
        questions: value.questions(),
        proposals: value.proposals(),
        escalations: value.escalations(),
        person_tasks: value.person_tasks(),
    }
}

pub(crate) fn count_fields(value: &w::Count) -> c::Count {
    c::Count { value: value.value(), limited: value.limited() }
}

pub(crate) fn frame_fields(value: &w::Me, limits: &Limits) -> Result<c::Me, w::Problem> {
    let list0 = super::me::projects(value.projects(), limits)?;
    let opt1 = match value.deployment_role() {
        Some(item) => Some(crate::translate::answers::held_role(item)),
        None => None,
    };
    if value.key_bytes() > limits.receiving().keyed_value_some
        || value.redirect_bytes() > limits.receiving().keyed_value_some
    {
        return Err(too_large());
    }
    Ok(c::Me {
        providers: super::catalogue::offers(value.providers(), limits, &mut super::document_room(limits))?,
        providers_cut: value.providers_cut(),
        key_bytes: value.key_bytes(),
        redirect_bytes: value.redirect_bytes(),
        words_bytes: value.words_bytes(),
        party: frame_party_fields(value.party(), limits)?,
        projects: list0,
        needs: needs_fields(value.needs()),
        needs_total: value.needs_total(),
        updates: count_fields(value.updates()),
        results_seen: value.results_seen(),
        cut: value.cut(),
        retention: value.retention(),
        deployment_role: opt1,
    })
}
