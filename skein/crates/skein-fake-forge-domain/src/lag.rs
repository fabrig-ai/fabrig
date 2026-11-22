//! Transient read projections, never write state (testing/fakes/forge.md, 2.3–2.4).

use skein_lib::{Env, Id, Time};

use crate::api::State;
use crate::domain::{Config, Domain};
use crate::fell::{Fault, Fell};
use crate::store::{Item, Pull, Repository};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Snapshot {
    pub(crate) state: State,
    pub(crate) commit: u64,
    pub(crate) base_commit: Option<u64>,
    pub(crate) merged: Option<u64>,
    pub(crate) checking: bool,
    pub(crate) mergeable: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct View {
    pub(crate) until: Time,
    pub(crate) snapshot: Snapshot,
}

pub(crate) fn active(pull: &Pull, now: Time) -> Option<Snapshot> {
    match pull.lagging {
        Some(view) if now < view.until => Some(view.snapshot),
        Some(_) | None => None,
    }
}

pub(crate) fn state(item: &Item, now: Time) -> State {
    match &item.pull {
        Some(pull) => match active(pull, now) {
            Some(snapshot) => snapshot.state,
            None => item.state,
        },
        None => item.state,
    }
}

/// Called before changing the pull; a moved base already names its new tip,
/// so the caller supplies the former base explicitly.
pub(crate) fn start(
    domain: &mut Domain,
    env: &Env<Config>,
    id: Id<Repository>,
    number: u64,
    base_commit: Option<u64>,
    commit: u64,
    fault: Fault,
) {
    let Some(lag) = domain.lag else { return };
    let repository = domain.repositories.get(id).expect("a repository of the forge");
    let item = repository.items.get(&number).expect("an affected pull request");
    let pull = item.pull.as_ref().expect("a pull request");
    if item.state != State::Open || active(pull, env.now).is_some() {
        return;
    }
    if !domain.rng.chance(lag.chance) {
        return;
    }
    let span = crate::faults::draw(domain, lag.min, lag.max);
    let until = env.now.saturating_add(span);
    let repository = domain.repositories.get(id).expect("a repository of the forge");
    let item = repository.items.get(&number).expect("an affected pull request");
    let snapshot = crate::pulls::snapshot(domain, &env.limits.limits, item, base_commit);
    let repository = domain.repositories.get_mut(id).expect("a repository of the forge");
    let pull =
        repository.items.get_mut(&number).expect("an affected pull request").pull.as_mut().expect("a pull request");
    pull.lagging = Some(View { until, snapshot });
    let mut fell = Fell::repository(domain, env.now, fault, id);
    fell.until = Some(until);
    fell.pull = Some(number);
    fell.commit = Some(commit);
    domain.record_fell(fell);
}
