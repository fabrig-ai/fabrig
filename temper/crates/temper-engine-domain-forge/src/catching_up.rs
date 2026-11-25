//! The restart's two halves, reported per forge (domain/forge/catching-up.md, 2 and 5).
use super::{Domain, Handoff, RestorePhase, fresh_reads_in, goal_branch};
use crate::{CatchingUp, Limits, Request, RestartStage, age_words};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Time, bytes};

pub(super) fn deadline(d: &Domain) -> Option<Time> {
    if d.stopping {
        return None;
    }
    match &d.catching_up_report {
        Some(clock) => Some(clock.deadline()),
        None => None,
    }
}

/// The same clock as full-table reports, ending only when both halves are in.
pub(super) fn advance(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.stopping {
        d.catching_up_report = None;
        return;
    }
    if !d.restart_settling && fresh_reads_in(d) {
        if let Some(ended) = d.catching_up_report.take()
            && ended.reported()
        {
            out.push(Request::FullTableFact { words: b"the forge has caught up".as_slice().into() });
        }
        return;
    }
    let Some(clock) = &mut d.catching_up_report else { return };
    let Some(age) = clock.due(env.now) else { return };
    let fresh = d.restart_waits(RestartStage::ReadAfresh);
    let settling = d.restart_waits(RestartStage::Settled);
    let waits = match (fresh.is_empty(), settling.is_empty()) {
        (true, true) => b"nothing named".as_slice().into(),
        (false, true) => fresh,
        (true, false) => settling,
        (false, false) => bytes::concat(&[&fresh, b", ", &settling]),
    };
    out.push(Request::FullTableFact {
        words: bytes::concat(&[b"the forge catching up for ", &age_words(age), b": ", &waits]),
    });
}

impl Domain {
    #[must_use]
    pub fn catching_up(&self, forge: u16) -> Option<CatchingUp> {
        if !self.configured.contains_key(&forge)
            || (!self.restart_settling && (self.restore_phase == RestorePhase::Restoring || fresh_reads_in(self)))
        {
            return None;
        }
        let mut state = CatchingUp {
            fresh: self.client.fresh_owed(forge),
            gathers: self.landing.gathers_owed(forge),
            ci: 0,
            goal_files: 0,
            lost_writers: 0,
            goal_branches: 0,
            uncertain: self.client.finds_owed(forge),
        };
        for (_, pending) in &self.pending_ci {
            if pending.repository.forge == forge {
                state.ci = state.ci.checked_add(1).expect("CI reads fit their limit");
            }
        }
        for (_, pending) in &self.goal_file_reads {
            if pending.repository.forge == forge {
                state.goal_files = state.goal_files.checked_add(1).expect("goal-file reads fit their limit");
            }
        }
        for (_, pending) in &self.lost {
            if pending.name.forge == forge {
                state.lost_writers = state.lost_writers.checked_add(1).expect("lost writers fit their limit");
            }
        }
        for (_, pending) in &self.makings {
            if pending.name.forge == forge && goal_branch::read_out(pending) {
                state.goal_branches = state.goal_branches.checked_add(1).expect("goal branches fit their limit");
            }
        }
        for (_, row) in &self.entries {
            if row.entry.repository.forge == forge && row.handoff == Handoff::Restored && row.entry.attempt.is_some() {
                state.uncertain = state.uncertain.checked_add(1).expect("entries fit their limit");
            }
        }
        Some(state)
    }
}

/// Mark only when the published state changes, including its disappearance.
pub(super) fn mark(d: &mut Domain, out: &mut Queue<Request>) {
    for (forge, _) in &d.configured {
        let state = d.catching_up(*forge);
        if state == d.catching_up_seen.get(forge).copied() {
            continue;
        }
        match state {
            Some(state) => {
                d.catching_up_seen.insert(*forge, state).expect("one catching-up state per configured forge");
            }
            None => {
                d.catching_up_seen.remove(forge);
            }
        }

        out.push(Request::MarkedForges);
        for (repository, adopted) in &d.repositories {
            if repository.forge == *forge {
                out.push(Request::Marked { tasks: Box::new([crate::changes_subject(adopted.project)]) });
            }
        }
    }
}
