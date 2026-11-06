//! Public landings and queues (temper/docs/design/client/forge.md, 11.2–11.3).
use super::{Names, PublicAsk, phase, w};
use crate::{Domain, Limits};
use alloc::boxed::Box;
use skein_lib::{List, Map};
use temper_engine_domain_forge_change as change;
struct Queue {
    repository: Box<[u8]>,
    ready: List<change::Ready>,
    named: bool,
}
impl Domain {
    pub(super) fn make_landings(
        &self,
        ask: &PublicAsk<'_>,
        limits: &Limits,
        names: &mut Names,
    ) -> Option<w::PublicLandings> {
        let l = &w::CEILINGS;
        let mut public = Map::with_capacity(limits.landings);
        let mut turn = None;
        for (_, row) in &self.landing.rows {
            if self.public_landing(ask, &row.landing) {
                if crate::views::matches_turn(&row.landing.phase) {
                    turn = row.landing.members.first();
                }
                public.insert(row.landing.task, &row.landing).ok()?;
            }
        }
        // Landing tasks select recency but never cross the public boundary.
        let turn = match turn {
            Some(member) => Some(names.name(member.change)?),
            None => None,
        };
        let mut landings = List::with_capacity(l.public_landings_landings);
        for (_, row) in public.iter().rev() {
            if landings.room() == 0 {
                break;
            }
            let mut members = List::with_capacity(l.public_landing_line_members);
            let mut landed = List::with_capacity(l.public_landing_line_landed);
            for member in &row.members {
                members.push(names.name(member.change)?).ok()?;
                if let Some(hash) = member.landed {
                    landed.push(hash).ok()?;
                }
            }
            landings
                .push(
                    w::PublicLandingLine::new(
                        l,
                        w::PublicLandingLineParts { members, phase: phase(&row.phase), landed },
                    )
                    .ok()?,
                )
                .ok()?;
        }
        let mut queues = self.public_queues(ask, limits)?;
        let mut made = List::with_capacity(l.public_landings_queues);
        while made.room() > 0 {
            let mut first = None;
            for (index, queue) in queues.iter().enumerate() {
                if queue.named {
                    continue;
                }
                let index = u32::try_from(index).ok()?;
                first = match first {
                    Some(previous) if !queue_before(queue, queues.get(previous)?, names) => Some(previous),
                    Some(_) | None => Some(index),
                };
            }
            let Some(first) = first else {
                break;
            };
            let queue = queues.get_mut(first)?;
            queue.named = true;
            let mut changes = List::with_capacity(l.public_queue_changes);
            for entry in &queue.ready {
                if changes.room() == 0 {
                    break;
                }
                let goal = ask.handed(entry.task)?.goal?;
                let task = names.name(entry.task)?;
                let goal = names.name(goal)?;
                changes
                    .push(
                        w::PublicQueued::new(
                            l,
                            w::PublicQueuedParts {
                                task,
                                goal,
                                ready_since: entry.since.as_nanos().checked_div(1_000_000_000)?,
                            },
                        )
                        .ok()?,
                    )
                    .ok()?;
            }
            made.push(w::PublicQueue::new(l, w::PublicQueueParts { changes }).ok()?).ok()?;
        }
        w::PublicLandings::new(l, w::PublicLandingsParts { turn, landings, queues: made }).ok()
    }
    fn public_queues(&self, ask: &PublicAsk<'_>, limits: &Limits) -> Option<List<Queue>> {
        let mut queues = List::with_capacity(limits.changes);
        let mut seen = List::with_capacity(limits.changes);
        for (_, row) in &self.changes {
            if self.public_change(ask, row.task).is_none() {
                continue;
            }
            let Some(goal) = ask.handed(row.task)?.goal else {
                continue;
            };
            if ask.handed(goal).is_none() || crate::panel::ready(row).is_none() {
                continue;
            }
            let key = (row.repository, row.base.as_ref());
            if seen.as_slice().contains(&key) {
                continue;
            }
            seen.push(key).ok()?;
            let (candidates, repair) = crate::domain::queue_ready(self, row, limits);
            let mut ready = List::with_capacity(limits.changes);
            for candidate in &candidates {
                if self.public_change(ask, candidate.task).is_none() {
                    continue;
                }
                let Some(goal) = ask.handed(candidate.task)?.goal else {
                    continue;
                };
                if ask.handed(goal).is_some() {
                    ready.push(*candidate).ok()?;
                }
            }
            let ready = super::order_ready(ready, repair, limits, ask.now)?;
            let queue = Queue { repository: self.repository_address(row)?, ready, named: false };
            queues.push(queue).ok()?;
        }
        Some(queues)
    }
}
fn queue_before(a: &Queue, b: &Queue, names: &Names) -> bool {
    let first_a = a.ready.as_slice().first().expect("nonempty public queue");
    let first_b = b.ready.as_slice().first().expect("nonempty public queue");
    let a = (
        &a.repository,
        first_a.since.as_nanos().checked_div(1_000_000_000).expect("seconds divisor"),
        names.place(first_a.task).unwrap_or(u16::MAX),
    );
    let b = (
        &b.repository,
        first_b.since.as_nanos().checked_div(1_000_000_000).expect("seconds divisor"),
        names.place(first_b.task).unwrap_or(u16::MAX),
    );
    a < b
}
