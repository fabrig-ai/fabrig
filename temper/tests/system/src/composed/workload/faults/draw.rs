//! The composed fault and party draw, without a world (temper/docs/design/testing/workloads.md, 5).
use super::{DISK_FULL, Plan, bounds, forge_faults, parks, restarts, stale, worker};
use skein_lib::Duration;
use skein_world::{
    faults::Kind,
    streams::{Stream, Streams},
};
use std::collections::BTreeSet;
use temper_engine_domain_world::acts;
use temper_faults::people;
use temper_workload::{
    draw::{COMPOSED, Workload},
    parties::{self, Act, Action, When},
};

/// The composed world's plans and acts for one seed and its configured kinds.
pub struct FaultDraw {
    pub(super) plans: Vec<Plan>,
    pub(super) actions: Vec<Action>,
    people: Vec<(Kind, String)>,
    pub(super) ineligible: BTreeSet<Kind>,
    pub(super) bounds: Vec<Kind>,
    pub(super) disk: worker::Disk,
    pub(super) read_latency: Duration,
    pub(super) release_configured: bool,
}

impl FaultDraw {
    #[must_use]
    pub fn new(seed: u64, kinds: &[Kind]) -> Self {
        Self::for_workload(seed, &Workload::draw(&Streams::new(seed), &COMPOSED), kinds)
    }

    pub(in crate::composed::workload) fn for_workload(seed: u64, workload: &Workload, kinds: &[Kind]) -> Self {
        let streams = Streams::new(seed);
        let mut forge_rng = streams.rng(Stream::Faults, "forge");
        let mut llm_rng = streams.rng(Stream::Faults, "llm");
        let mut restart_rng = streams.rng(Stream::Faults, restarts::PART);
        let mut worker_rng = streams.rng(Stream::Faults, "worker");
        // Draw the fixed positions even for unconfigured kinds. Removing a
        // family cannot shift another family's choice within the forge part.
        let supported = [
            forge_faults::TIMED_OUT,
            forge_faults::HEAD_BEHIND,
            forge_faults::JOB_LOG,
            forge_faults::CLOSED_BY_ANOTHER,
            temper_blame::LOST,
            stale::MOVED_BASE,
            skein_llm_kit::faults::RATE_WINDOW,
            restarts::ENGINE_CRASH_MID_TURN,
            DISK_FULL,
            parks::DELIVERED_THEN_PARKED,
        ];
        let mut allowed = supported.to_vec();
        allowed.extend([people::HOLDS_A_MEMBER, people::RELEASES_A_MEMBER, bounds::FORGE_READ_ADMISSIONS]);
        assert!(kinds.iter().all(|kind| allowed.contains(kind)), "supported composed faults");
        let mut ineligible = BTreeSet::new();
        let plans = supported
            .iter()
            .filter_map(|&kind| {
                let rng = match kind.family {
                    "provider" => &mut llm_rng,
                    "restarts" => &mut restart_rng,
                    "worker" | "runs that park, resume and move" => &mut worker_rng,
                    _ => &mut forge_rng,
                };
                let eligible: Vec<_> = workload
                    .goals
                    .iter()
                    .filter(|goal| {
                        if kind == stale::MOVED_BASE {
                            goal.train
                        } else if kind == forge_faults::HEAD_BEHIND || kind == forge_faults::JOB_LOG {
                            goal.changes.iter().any(|change| change.red_once)
                        } else if kind == parks::DELIVERED_THEN_PARKED {
                            goal.changes.iter().any(|change| !change.asks)
                        } else {
                            true
                        }
                    })
                    .collect();
                // below(0) still spends the target draw.
                let index = usize::try_from(rng.below(u64::try_from(eligible.len()).expect("bounded goals")))
                    .expect("goal index");
                let after = rng.below(16);
                let query = if kind == restarts::ENGINE_CRASH_MID_TURN {
                    1 + usize::try_from(rng.below(2)).expect("two query points")
                } else {
                    1
                };
                if !kinds.contains(&kind) {
                    return None;
                }
                if let Some(goal) = eligible.get(index) {
                    Some(Plan { kind, goal: goal.name.clone(), after, fired: false, query })
                } else {
                    ineligible.insert(kind);
                    None
                }
            })
            .collect();
        let disk = worker::Disk::draw(&mut worker_rng);
        let (actions, people) = party_draw(streams, workload, kinds, &mut ineligible);
        Self {
            plans,
            actions,
            people,
            ineligible,
            bounds: kinds.iter().copied().filter(|kind| kind.family == bounds::FAMILY).collect(),
            disk,
            read_latency: Duration::from_millis(streams.rng(Stream::Scheduling, "system").between(1000, 1500)),
            release_configured: kinds.contains(&people::RELEASES_A_MEMBER),
        }
    }

    /// Each planned kind and the logical goal it strikes, in plan order.
    #[must_use]
    pub fn planned(&self) -> Vec<(Kind, &str)> {
        self.plans
            .iter()
            .map(|plan| (plan.kind, plan.goal.as_str()))
            .chain(self.people.iter().map(|(kind, goal)| (*kind, goal.as_str())))
            .chain(self.bounds.iter().map(|&kind| (kind, "")))
            .collect()
    }

    /// Configured kinds the draw could not place, in kind order.
    #[must_use]
    pub fn ineligible(&self) -> Vec<Kind> {
        self.ineligible.iter().copied().collect()
    }

    /// The parties' acts, each hold followed by its release.
    #[must_use]
    pub fn acts(&self) -> &[Action] {
        &self.actions
    }
}

fn party_draw(
    streams: Streams,
    workload: &Workload,
    kinds: &[Kind],
    ineligible: &mut BTreeSet<Kind>,
) -> (Vec<Action>, Vec<(Kind, String)>) {
    // Even a hold-only sweep must release its member to let the train end.
    let mut actions = if kinds.contains(&people::HOLDS_A_MEMBER) {
        parties::draw(&streams, workload, &[Act::HoldMember, Act::ReleaseMember])
    } else {
        Vec::new()
    };
    let mut release = streams.rng(Stream::Parties, "release");
    let mut index = 0;
    while index < actions.len() {
        let hold = &actions[index];
        if hold.act == Act::HoldMember
            && !actions[index + 1..]
                .iter()
                .any(|act| act.act == Act::ReleaseMember && act.party == hold.party && act.target == hold.target)
        {
            let after = u32::try_from(release.between(0, u64::from(parties::AFTER_MOST))).expect("bounded delay");
            let action = Action { act: Act::ReleaseMember, when: When::Held, after, ..hold.clone() };
            actions.insert(index + 1, action);
        }
        index += 1;
    }
    let planned: Vec<_> = actions
        .iter()
        .filter_map(|action| {
            let kind = acts::kind(action.act);
            kinds.contains(&kind).then(|| (kind, workload.change(&action.target).expect("act goal").0.name.clone()))
        })
        .collect();
    for kind in [people::HOLDS_A_MEMBER, people::RELEASES_A_MEMBER] {
        if kinds.contains(&kind) && !planned.iter().any(|(planned, _)| *planned == kind) {
            ineligible.insert(kind);
        }
    }
    (actions, planned)
}
