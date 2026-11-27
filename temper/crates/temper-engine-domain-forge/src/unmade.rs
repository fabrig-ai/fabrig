//! Start’s taskless landings and holds (temper/docs/design/domain/landing-liveness.md, 6.2 and landing-forming.md, 4).
use crate::{Domain, Key, LandingRow, Limits, Request, Stored, Writer, landing};
use skein_forge::api::{Answer, Commit, Error, Read, Repository};
use skein_lib::{Env, List, Queue, Set, Time, Token};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as policy;
use temper_engine_domain_forge_vocabulary::writes::Seen;

impl Domain {
    /// End unmade landings and drop taskless holds that no live task writes under.
    pub fn link_landings(&mut self, tasks: &jig_domain_tasks::Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
        if self.restore_fit_failure().is_some() {
            return;
        }
        self.join_changes();
        let mut owners = Set::with_capacity(self.entries.len());
        for (_, entry) in &self.entries {
            owners.insert(entry.entry.task).expect("at most one owner per outbox entry");
        }
        let mut unmade = List::with_capacity(self.landing.rows.len());
        for (task, row) in &self.landing.rows {
            if !landing::matches_terminal(&row.landing.phase)
                && row.entries.is_empty()
                && !owners.contains(task)
                && tasks.task(*task).is_none()
            {
                unmade.push(*task).expect("bounded restored landings");
            }
        }
        for task in unmade.into_boxed() {
            let row = self.landing.rows.get(&task).expect("selected live row");
            if let Some(top) = unknown_merge(row) {
                self.landing.unmade.insert(task, Readback { top, stage: Stage::Fresh }).expect("one readback per row");
            } else {
                // Link commits the repaired rows, but cannot step jig's
                // tasks before jig has linked its own census projections.
                let mut ending = Queue::uncapped();
                end_unmade(self, env, task, None, false, &mut ending);
                while let Some(request) = ending.pop() {
                    if let Request::WakeProcedure { task } = request {
                        self.procedure_wakes.insert(task, true).expect("one pending wake per change");
                    } else {
                        out.push(request);
                    }
                }
            }
        }
        let mut abandoned = List::with_capacity(self.holds.len());
        for (name, hold) in &self.holds {
            if tasks.task(hold.task).is_some() || self.forge_ended(name.forge) {
                continue;
            }
            let mut written = match hold.writer {
                Some(Writer::Run { task, .. } | Writer::Lost { task, .. }) => tasks.task(task).is_some(),
                Some(Writer::Entry(_)) | None => false,
            };
            for writer in &hold.design_writers {
                written |= tasks.task(writer.task).is_some();
            }
            if !written {
                abandoned.push(name.clone()).expect("bounded restored holds");
            }
        }
        for name in abandoned.into_boxed() {
            let hold = self.holds.remove(&name).expect("selected taskless hold");
            let task = hold.task;
            self.reconciliation.forget(&Stored::Hold(hold), &env.limits);
            out.push(Request::Erase { key: Key::Hold(name.clone()) });
            out.push(Request::HoldDropped { name, task });
        }
    }
}

// The readback holds the row and its train, but has no durable state of its own.
#[derive(Clone, Copy, Hash, Debug)]
pub(crate) struct Readback {
    top: u16,
    stage: Stage,
}

#[derive(Clone, Copy, Hash, Debug)]
enum Stage {
    Fresh,
    Due { at: Time, tip: Option<Commit> },
    Reading { owner: Token, tip: Option<Commit> },
}

impl Readback {
    pub(crate) const fn due(&self) -> Option<Time> {
        match self.stage {
            Stage::Due { at, .. } => Some(at),
            Stage::Fresh | Stage::Reading { .. } => None,
        }
    }
}

pub(crate) fn waiting_member(d: &Domain, change: u64) -> bool {
    match d.landing_of(change) {
        Some(task) => d.landing.unmade.contains_key(&task),
        None => false,
    }
}

fn unknown_merge(row: &LandingRow) -> Option<u16> {
    let at_turn = match &row.landing.phase {
        policy::Phase::Turn { .. } => true,
        policy::Phase::Held { was, .. } => crate::views::matches_turn(was),
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Landed
        | policy::Phase::Cancelled => false,
    };
    // A saved landed prefix is an outcome; the turn's transient effect result is not stored.
    if at_turn && row.landing.round.merges != 0 { policy::top(&row.landing) } else { None }
}

fn target(row: &LandingRow) -> Option<Repository> {
    Some(landing::repository(row.landing.members.first()?))
}

pub(crate) fn deadline(d: &Domain) -> Option<Time> {
    let mut next: Option<Time> = None;
    for (_, readback) in &d.landing.unmade {
        if let Some(at) = readback.due() {
            next = Some(match next {
                Some(old) => at.min(old),
                None => at,
            });
        }
    }
    next
}

pub(crate) fn advance(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.stopping || d.landing.unmade.is_empty() {
        return;
    }
    let mut ready = List::with_capacity(d.landing.unmade.len());
    for (task, readback) in &d.landing.unmade {
        let row = d.landing.rows.get(task).expect("readback keeps its row");
        let repository = target(row).expect("a turn has members");
        let tip = match readback.stage {
            Stage::Fresh if crate::domain::repository_fresh(d, repository) => None,
            Stage::Due { at, tip } if at <= env.now => tip,
            Stage::Fresh | Stage::Due { .. } | Stage::Reading { .. } => continue,
        };
        ready.push((*task, repository, readback.top, tip)).expect("bounded readbacks");
    }
    for (task, repository, top, tip) in ready.into_boxed() {
        let row = d.landing.rows.get(&task).expect("readback keeps its row");
        let read = match tip {
            Some(before) => Read::Commits { before, after: candidate(row, top), page: 1 },
            None => Read::Branch { branch: row.landing.branch.clone() },
        };
        d.sequence = d.sequence.checked_add(1).expect("read sequence");
        let owner = Token::new(d.sequence | (1_u64 << 62));
        d.landing.unmade.get_mut(&task).expect("selected readback").stage = Stage::Reading { owner, tip };
        crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
    }
}

fn candidate(row: &LandingRow, top: u16) -> Commit {
    row.landing.members.get(usize::from(top)).expect("turn's top").candidate.expect("proved candidate")
}

pub(crate) fn reader(d: &Domain, owner: Token) -> Option<u64> {
    for (task, readback) in &d.landing.unmade {
        match readback.stage {
            Stage::Reading { owner: pending, .. } if owner == pending => return Some(*task),
            Stage::Fresh | Stage::Due { .. } | Stage::Reading { .. } => {}
        }
    }
    None
}

pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    if d.stopping {
        return;
    }
    let readback = d.landing.unmade.get_mut(&task).expect("selected readback");
    let tip = match readback.stage {
        Stage::Reading { tip, .. } => tip,
        Stage::Fresh | Stage::Due { .. } => unreachable!("only a read in flight answers"),
    };
    let top = readback.top;
    if tip.is_some() {
        match crate::domain::ancestry_status(result) {
            change::Status::Passed => end_unmade(d, env, task, Some(top), true, out),
            change::Status::Failed => end_unmade(d, env, task, None, true, out),
            change::Status::Unknown | change::Status::Pending => {
                readback.stage = Stage::Due { at: env.now.saturating_add(env.limits.client.backoff), tip };
            }
        }
    } else {
        tip_answered(d, env, task, top, result, out);
    }
}

fn tip_answered(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    top: u16,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let due = Stage::Due { at: env.now.saturating_add(env.limits.client.backoff), tip: None };
    let tip = match result {
        Ok(Answer::Commit(tip)) => Some(tip),
        Err(Error::Missing) => None,
        Ok(_) | Err(_) => {
            d.landing.unmade.get_mut(&task).expect("readback").stage = due;
            return;
        }
    };
    let row = d.landing.rows.get(&task).expect("readback keeps its row");
    let member = row.landing.members.get(usize::from(top)).expect("turn's top");
    let base = member.base;
    let candidate = candidate(row, top);
    let branch = row.landing.branch.clone();
    let seen = landing::observe_tip(d, &env.limits, landing::repository(member), &branch, tip, out);
    if let Seen::Behind { .. } | Seen::Owed = seen {
        d.landing.unmade.get_mut(&task).expect("readback").stage = due;
    } else if tip == Some(candidate) {
        end_unmade(d, env, task, Some(top), true, out);
    } else if tip == base || tip.is_none() {
        end_unmade(d, env, task, None, false, out);
    } else {
        d.landing.unmade.get_mut(&task).expect("readback").stage = Stage::Due { at: env.now, tip };
    }
}

fn end_unmade(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    landed_top: Option<u16>,
    readback: bool,
    out: &mut Queue<Request>,
) {
    d.landing.unmade.remove(&task);
    let mut row = d.landing.rows.get(&task).expect("selected unmade landing").clone();
    let mut waiting = List::with_capacity(env.limits.landing_members);
    let mut landed = List::with_capacity(env.limits.landing_members);
    let mut unrecorded = List::with_capacity(env.limits.landing_members);
    if let Some(top) = landed_top {
        landing::settle_turn_merge(d, &env.limits, &row, top, true, out);
        let prior = row.clone();
        for (index, member) in row.landing.members.iter_mut().enumerate() {
            if index <= usize::from(top) && member.left.is_none() && member.landed.is_none() {
                member.landed = member.candidate;
                member.told = member.landed.is_some();
            }
        }
        crate::landing_history::transition(d, env.wall, &prior, &row, out);
        for (index, (old, member)) in prior.landing.members.iter().zip(&row.landing.members).enumerate() {
            if old.landed.is_none()
                && let Some(prefix) = member.landed
            {
                let position = u16::try_from(index).expect("bounded member position");
                landing::hand_landed(d, env.wall, &row, position, prefix, out);
                out.push(Request::LandingNews { task, member: position, step: crate::LandingNews::Landed });
                landed.push(member.change).expect("bounded landing members");
                if member.combined.or(row.landing.round.combined) == Some(policy::Combine::Squash) && !member.recorded {
                    unrecorded.push(member.change).expect("bounded landing members");
                }
            }
        }
        row.landing.round.told |= !landed.is_empty();
        landing::save(d, env, row.clone(), out);
    }
    landing::end_row(d, env, task, out);
    for member in &row.landing.members {
        if member.left.is_none()
            && member.landed.is_none()
            && d.landing_of(member.change).is_none()
            && let Some(change) = d.changes.get(&member.change)
            && crate::domain::landed_commit(&change.change.state).is_none()
        {
            waiting.push(member.change).expect("bounded landing members");
        }
    }
    if let Some(repository) = target(&row) {
        out.push(Request::LandingUnmade {
            task,
            repository,
            branch: row.landing.branch,
            changes: waiting.into_boxed(),
            readback,
            landed: landed.into_boxed(),
            unrecorded: unrecorded.into_boxed(),
        });
    }
}
