//! Room for landed commits (temper/docs/design/domain/forge/change/README.md, 7).
#[cfg(test)]
mod tests;
use super::{Domain, Env, Key, Limits, Queue, Request, Stored, emit};
use skein_forge::api::Commit;

pub(super) fn landed_room(d: &mut Domain, env: &Env<Limits>, commit: Commit, task: u64, out: &mut Queue<Request>) {
    if d.landed.len() >= env.limits.landings.max(env.limits.changes) && !d.landed.contains_key(&commit) {
        let mut victim = None;
        for (kept, owner) in &d.landed {
            let priority: u8 = if !d.changes.contains_key(owner) {
                0
            } else if d.droppable(*owner) {
                1
            } else {
                2
            };
            let rank = (*owner == task, priority, *owner);
            if match victim {
                Some((_, best)) => rank < best,
                None => true,
            } {
                victim = Some((*kept, rank));
            }
        }
        if let Some((victim, _)) = victim {
            d.landed.remove(&victim);
            emit(out, Request::Erase { key: Key::Landed(victim) });
        }
    }
    d.landed.insert(commit, task).expect("preflighted landed capacity");
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Landed { commit, task } });
}
