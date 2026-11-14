//! A person's hold leaves a run in flight alone (temper/docs/design/domain/after-landing.md, 3).
use super::{configured_all, empty, save, start_due};
use crate::tiers::{TierAct, TierActRequest, TierRefusal};
use crate::{Answered, Domain, Limits, Request, TierHold};
use skein_forge::api::Repository;
use skein_lib::{Env, Queue, Token};

fn repository(d: &Domain, call: &TierActRequest) -> Option<Repository> {
    let Some(named) = &call.repository else { return Some(d.home(call.project)?.provider) };
    for (_, row) in &d.repositories {
        if row.project == call.project
            && row.owner == named.owner
            && row.name == named.name
            && !d.forge_ended(row.provider.forge)
        {
            return Some(row.provider);
        }
    }
    None
}

fn apply(
    d: &mut Domain,
    env: &Env<Limits>,
    party: u64,
    call: TierActRequest,
    out: &mut Queue<Request>,
) -> Result<(), TierRefusal> {
    let repository = repository(d, &call).ok_or(TierRefusal::NoRepository)?;
    let key = (repository, call.tier.clone());
    if !d.tiers.contains_key(&key) {
        let mut configured = false;
        for tier in configured_all(d, repository) {
            configured |= tier.name == call.tier;
        }
        if !configured {
            return Err(TierRefusal::NoTier);
        }
    }
    match &call.act {
        TierAct::Hold { reason } => {
            if reason.is_empty() || reason.len() > 1024 || !skein_lib::bytes::is_utf8(reason) {
                return Err(TierRefusal::Words);
            }
        }
        TierAct::Release => {}
    }
    let mut row = d.tiers.get(&key).cloned().unwrap_or(empty(repository, call.tier));
    match call.act {
        TierAct::Hold { reason } => {
            if let Some(held) = &row.hold {
                return Err(TierRefusal::Held { party: held.party, reason: held.reason.clone() });
            }
            row.hold = Some(TierHold { party, at: env.wall, reason });
        }
        TierAct::Release => {
            if row.hold.take().is_none() {
                return Err(TierRefusal::NotHeld);
            }
        }
    }
    let released = row.hold.is_none();
    save(&row, out);
    d.tiers.insert(key, row).expect("configured or recorded tier");
    if released {
        start_due(d, env, repository, out);
    }
    Ok(())
}

pub(crate) fn asked(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    party: u64,
    call: TierActRequest,
    out: &mut Queue<Request>,
) {
    let answer = apply(d, env, party, call, out);
    d.connect.answer(request, token, Answered::Tier(answer), out);
}
