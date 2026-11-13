//! The connector's whole change document (domain/forge/change/README.md, 5; client/forge/README.md, 2.1).
use crate::{ChangeRow, Domain, Limits};
use alloc::boxed::Box;
use jig_domain_people::ApplicationScope;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_change as change;
pub use temper_engine_domain_forge_vocabulary::panel::*;

impl Domain {
    fn panel_tiers(&self, verification: &crate::Verification) -> Box<[Box<[u8]>]> {
        let mut lines = List::with_capacity(u32::try_from(verification.tiers.len()).expect("admitted tiers"));
        for tier in &verification.tiers {
            let mut run = None;
            let mut suffix = Box::from(b"".as_slice());
            if let Some(repair) = tier.repaired {
                run = Some(repair.run);
            } else if let Some(verdict) = &tier.verdict {
                run = Some(verdict.run);
                suffix = tier_blame(&verdict.blame);
            } else {
                for ((repository, name), row) in &self.tiers {
                    if *repository == verification.repository
                        && *name == tier.tier
                        && let Some(flight) = &row.record.run
                    {
                        for landing in &flight.landings {
                            if landing.commit == verification.commit {
                                run = Some(flight.number);
                            }
                        }
                    }
                }
                suffix = Box::from(if run.is_some() { b" running".as_slice() } else { b" waiting".as_slice() });
            }
            let number = skein_lib::Decimal::of(run.unwrap_or(0));
            let mut line = List::uncapped();
            for part in [
                tier.tier.as_ref(),
                if run.is_some() { b" run " } else { b"" },
                if run.is_some() { number.as_bytes() } else { b"" },
                &suffix,
            ] {
                for byte in part {
                    line.push(*byte).expect("tier words");
                }
            }
            lines.push(line.into_boxed()).expect("admitted tiers");
        }
        lines.into_boxed()
    }
    /// Purely read a retained change in the project that holds its repository.
    #[must_use]
    pub fn panel(&self, task: u64, scope: ApplicationScope, now: Wall, limits: &Limits) -> Option<Panel> {
        let ApplicationScope::Project(project) = scope else { return None };
        let row = self.changes.get(&task)?;
        let repository = self.repositories.get(&row.repository)?;
        if repository.project != project {
            return None;
        }
        let (step, hold) = self.panel_step(row, now, limits);
        let head = row.change.last_head;
        let panel = Panel {
            way: row.change.way,
            in_landing: if row.change.way == change::Way::Turn {
                match row.change.landing {
                    Some(task) => Some(task),
                    None => match self.newest_landing(task) {
                        Some(landing) => Some(landing.task),
                        None => None,
                    },
                }
            } else {
                None
            },
            task,
            forge: row.repository.forge,
            owner: repository.owner.clone(),
            name: repository.name.clone(),
            branch: row.branch.clone(),
            landing: row.base.clone(),
            head,
            pull: row.pull,
            step,
            hold,
            runs: match row.delegate {
                Some((task, _)) => Some(task),
                None => None,
            },
            ci: self.observed_ci(row),
            ci_required: repository.ci && self.requires(repository, &row.base, self.touched_at(row)).ci,
            gates: panel_gates(row, head),
            approvals: self.panel_approvals(row),
            unmapped_reviews: self.panel_unmapped(row),
            repairs: PanelUsed { used: row.change.repairs, limit: limits.change_policy.repairs },
            resolutions: PanelUsed { used: row.change.resolutions, limit: limits.change_policy.resolutions },
            updates: PanelUsed { used: row.change.updates, limit: limits.change_policy.updates },
            questions: panel_questions(row, limits),
        };
        assert!(panel.within(), "admitted change fits the panel");
        Some(panel)
    }
    fn panel_step(&self, row: &ChangeRow, now: Wall, limits: &Limits) -> (PanelStep, Option<change::Hold>) {
        let mut state = &row.change.state;
        let mut hold = None;
        while let change::State::Held { was, why } = state {
            hold = Some(*why);
            state = was;
        }
        let step = match state {
            change::State::NoChange { .. } | change::State::Ended => PanelStep::Ended,
            change::State::Producing { .. } | change::State::WaitingProducer { .. } => PanelStep::Producing,
            change::State::Opening { .. } => PanelStep::Opening,
            change::State::Recreating { .. } => PanelStep::Recreating,
            change::State::Reopening { .. } => PanelStep::Reopening,
            change::State::Checking { .. } => PanelStep::Checking,
            change::State::Judging { .. } | change::State::Gating { .. } | change::State::CancellingGates { .. } => {
                PanelStep::Gating
            }
            change::State::Ready { .. } => PanelStep::Ready { landing: row.change.landing },
            change::State::Preparing { .. } => PanelStep::Preparing { landing: row.change.landing },
            change::State::Queued { .. } => {
                let (place, ahead) = queue_place(self, row, now, limits);
                PanelStep::Queued { place, ahead }
            }
            change::State::First { .. } => PanelStep::First,
            change::State::Updating { .. } => PanelStep::Updating,
            change::State::Resolving { .. } => PanelStep::Resolving,
            change::State::Repairing { why, .. } => PanelStep::Repairing(*why),
            change::State::Landing { .. } => PanelStep::Landing,
            change::State::Landed { merge, .. } => match self.verifications.get(&(row.repository, *merge)) {
                Some(verification) => PanelStep::Verifying { merge: *merge, tiers: self.panel_tiers(verification) },
                None => PanelStep::Landed { merge: *merge },
            },
            change::State::Red { merge, .. } => {
                PanelStep::Red { merge: *merge, tiers: self.verification_lines(row, *merge) }
            }
            change::State::Cleared { merge } | change::State::Uncleared { merge } => PanelStep::Cleared {
                merge: *merge,
                tiers: self.verification_lines(row, *merge),
                uncleared: *state == change::State::Uncleared { merge: *merge },
            },
            change::State::Held { .. } => unreachable!("held steps unwrapped"),
        };
        (step, hold)
    }
    fn verification_lines(&self, row: &ChangeRow, merge: [u8; 32]) -> Box<[Box<[u8]>]> {
        match self.verifications.get(&(row.repository, merge)) {
            Some(verification) => self.panel_tiers(verification),
            None => Box::new([]),
        }
    }
    fn observed_ci(&self, row: &ChangeRow) -> change::Status {
        let read = if row.terminal() { row.ci.as_ref() } else { self.panel_ci.get(&row.task) };
        match read {
            Some(read) if Some(read.head) == row.change.last_head => read.status,
            Some(_) | None => change::Status::Unknown,
        }
    }
}
fn panel_questions(row: &ChangeRow, limits: &Limits) -> PanelQuestions {
    let mut open = List::with_capacity(8);
    for question in &row.questions.open {
        if open.room() == 0 {
            break;
        }
        open.push(PanelQuestion {
            asker: question.asker,
            number: question.number,
            line: question.line.clone(),
            stands: match &question.stands {
                change::Stands::Taken => PanelStands::Taken,
                change::Stands::Asked => PanelStands::Asked,
                change::Stands::Held { .. } => PanelStands::Held,
            },
        })
        .expect("first eight questions fit");
    }
    PanelQuestions {
        route: row.questions.route,
        taken: PanelUsed { used: row.questions.taken, limit: limits.change_policy.questions },
        open: open.into_boxed(),
    }
}
fn panel_gates(row: &ChangeRow, head: Option<[u8; 32]>) -> Box<[PanelGate]> {
    let mut gates = List::with_capacity(PANEL_GATES);
    for gate in &row.change.gates {
        let mut verdict = change::Status::Unknown;
        for report in &row.verdicts {
            if gate.number == report.number
                && head.is_some()
                && (Some(report.head) == head
                    || (gate.freshness == change::Freshness::Clean && row.change.clean.contains(&report.head)))
            {
                verdict = worse(verdict, report.status);
            }
        }
        // A gate whose review runs at the current head is pending until it reports.
        if let Some((_, change::Delegate::Gate { number, head: asked })) = row.delegate
            && number == gate.number
            && Some(asked) == head
        {
            verdict = worse(verdict, change::Status::Pending);
        }
        gates
            .push(PanelGate { number: gate.number, kind: gate.kind, blocking: gate.blocking, verdict })
            .expect("admitted panel gates");
    }
    gates.into_boxed()
}
fn worse(a: change::Status, b: change::Status) -> change::Status {
    use change::Status::{Failed, Passed, Pending, Unknown};
    match (a, b) {
        (Failed, _) | (_, Failed) => Failed,
        (Pending, _) | (_, Pending) => Pending,
        (Passed, _) | (_, Passed) => Passed,
        (Unknown, Unknown) => Unknown,
    }
}

pub(crate) fn ready(row: &ChangeRow) -> Option<change::Ready> {
    let (change::State::Queued { ready_since: since, .. } | change::State::First { ready_since: since, .. }) =
        row.change.state
    else {
        return None;
    };
    Some(change::Ready { task: row.task, priority: row.priority, since })
}
fn queue_place(d: &Domain, row: &ChangeRow, now: Wall, limits: &Limits) -> (u32, Box<[u64]>) {
    let (ready, repair) = crate::domain::queue_ready(d, row, limits);
    let mut own = None;
    for entry in &ready {
        if entry.task == row.task {
            own = Some(*entry);
        }
    }
    let own = match own {
        Some(own) => own,
        None => change::Ready {
            task: row.task,
            priority: row.priority,
            since: row.change.ready_since.unwrap_or(row.change.since),
        },
    };
    let mut place = 1_u32;
    for entry in &ready {
        if queue_before(entry, &own, repair, limits, now) {
            place = place.checked_add(1).expect("bounded queue place");
        }
    }
    let mut ahead = List::with_capacity(PANEL_AHEAD);
    // Repeated selection retains only the nearest predecessors, without sorting a second queue.
    let mut cursor = own;
    while ahead.room() > 0 {
        let mut nearest = None;
        for entry in &ready {
            if queue_before(entry, &cursor, repair, limits, now) {
                nearest = match nearest {
                    Some(current) if queue_before(entry, &current, repair, limits, now) => Some(current),
                    Some(_) | None => Some(*entry),
                };
            }
        }
        let Some(entry) = nearest else { break };
        if ahead.room() > 0 {
            ahead.push(entry.task).expect("nearest ahead room");
        }
        cursor = entry;
    }
    (place, ahead.into_boxed())
}
pub(crate) fn queue_before(
    a: &change::Ready,
    b: &change::Ready,
    repair: Option<u64>,
    limits: &Limits,
    now: Wall,
) -> bool {
    if Some(a.task) == repair {
        return a.task != b.task;
    }
    if Some(b.task) == repair {
        return false;
    }
    change::Ready::before(a, b, limits.queue_window, now)
}

fn tier_blame(blame: &temper_engine_domain_forge_tiers::Blame) -> Box<[u8]> {
    use temper_engine_domain_forge_tiers::{Blame, Failures};
    let (kind, failures) = match blame {
        Blame::Cleared => return Box::new([]),
        Blame::Owns(failures) => (b" owns ".as_slice(), failures),
        Blame::Suspect(failures) => (b" suspected of ".as_slice(), failures),
    };
    let mut line = List::uncapped();
    for part in [b" \xc2\xb7".as_slice(), kind] {
        for byte in part {
            line.push(*byte).expect("tier words");
        }
    }
    match failures {
        Failures::Many => {
            for byte in b"many tests" {
                line.push(*byte).expect("tier words");
            }
        }
        Failures::Tests(tests) => {
            let count = skein_lib::Decimal::of(skein_lib::len_u64(tests.len()));
            for part in [count.as_bytes(), if tests.len() == 1 { b" test: " } else { b" tests: " }] {
                for byte in part {
                    line.push(*byte).expect("tier words");
                }
            }
            for (at, test) in tests.iter().enumerate() {
                if at > 0 {
                    for byte in b", " {
                        line.push(*byte).expect("tier words");
                    }
                }
                for byte in test {
                    line.push(*byte).expect("tier words");
                }
            }
        }
    }
    line.into_boxed()
}
