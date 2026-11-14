//! A person's hold leaves a run in flight alone (temper/docs/design/domain/after-landing.md, 3).
use super::{configured_all, empty, save, start_due};
use crate::{Answered, Domain, Limits, Request, TierHold};
use crate::tiers::{TierAct, TierActRequest, TierRefusal};
use skein_forge::api::Repository;
use skein_lib::{Env, Queue, Token};

#[expect(clippy::manual_find, reason = "step code resolves adopted names without callbacks")]
fn repository(d: &Domain, call: &TierActRequest) -> Option<Repository> {
    let Some(named) = &call.repository else { return Some(d.home(call.project)?.provider) };
    for (_, row) in &d.repositories {
        if row.project == call.project && row.owner == named.owner && row.name == named.name
            && !d.forge_ended(row.provider.forge)
        {
            return Some(row.provider);
        }
    }
    None
}

fn apply(d: &mut Domain, env: &Env<Limits>, party: u64, call: TierActRequest, out: &mut Queue<Request>) -> Result<(), TierRefusal> {
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
    if let TierAct::Hold { reason } = &call.act {
        if reason.is_empty() || reason.len() > 1024 || core::str::from_utf8(reason).is_err() {
            return Err(TierRefusal::Words);
        }
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
    save(&row, out);
    d.tiers.insert(key, row).expect("configured or recorded tier");
    start_due(d, env, repository, out);
    Ok(())
}

#[expect(clippy::too_many_arguments, reason = "the admitted act carries its key and authenticated party")]
pub(crate) fn asked(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, party: u64, call: TierActRequest, out: &mut Queue<Request>) {
    let answer = apply(d, env, party, call, out);
    d.connect.answer(request, token, Answered::Tier(answer), out);
}
