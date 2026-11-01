//! Forge creation (temper/docs/design/draft/playground-reset.md, 3;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, 2).
use crate::{Configured, Domain, Forge, Limits, Request, Stored, record::SetupMode};
use alloc::boxed::Box;
use skein_lib::{Env, Queue};

#[derive(Debug)]
pub(crate) struct State;

impl State {
    pub(crate) const fn new(_l: &Limits) -> Self {
        Self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Refusal {
    Full,
    Busy,
    Invalid,
    Exhausted,
}

#[cfg_attr(not(test), expect(dead_code, reason = "creation callers belong to fr-setup, fr-connect and pg-start"))]
pub(crate) fn make(
    d: &mut Domain,
    env: &Env<Limits>,
    mut forge: Forge,
    out: &mut Queue<Request>,
) -> Result<u16, Refusal> {
    if d.forge_records.len() >= env.limits.forges {
        return Err(Refusal::Full);
    }
    if kind_busy(d, &forge) {
        return Err(Refusal::Busy);
    }
    if !forge.fits() {
        return Err(Refusal::Invalid);
    }
    let number = d.next_forge();
    let next = number.checked_add(1).ok_or(Refusal::Exhausted)?;
    let mut identity = 0;
    for (_, row) in &d.forge_records {
        identity = identity.max(row.identity.number);
    }
    forge.number = number;
    forge.identity.number = identity.checked_add(1).ok_or(Refusal::Exhausted)?;

    out.push(Request::Save { record: Stored::Forge(forge.clone()), release: None, read_afresh: None });
    out.push(Request::Save { record: Stored::Forges { next }, release: None, read_afresh: None });
    d.forge_records.insert(number, forge.clone()).expect("forge admission checked");
    d.configured.insert(number, Configured::from_record(&forge)).expect("forge admission checked");
    d.identities.insert(number, forge.identity.number).expect("forge admission checked");
    d.next_forge = Some(next);
    out.push(Request::MarkedForges);
    out.push(Request::ForgeKept { forge: Box::new(forge) });
    Ok(number)
}

pub(crate) fn kind_busy(d: &Domain, forge: &Forge) -> bool {
    let owned = is_owned(forge);
    for (_, row) in &d.forge_records {
        if !row.removing && ((forge.playground && row.playground) || (owned && is_owned(row))) {
            return true;
        }
    }
    false
}

fn is_owned(forge: &Forge) -> bool {
    if forge.kind != crate::Kind::Forgejo {
        return false;
    }
    match &forge.setup {
        Some(setup) => match &setup.mode {
            SetupMode::Owned(_) => true,
            SetupMode::Managed { .. } | SetupMode::Fake(_) => false,
        },
        None => false,
    }
}
