//! The composed park on nothing with a running job (temper/docs/design/testing/workloads.md, 5).
use super::{FaultLedger, Faults, Kind, Struck, World};
use crate::composed::{Agent, Key, Script, call, engine, run, tasks};
use jig_core_kit::{
    cells::{Column, Pairing, Park},
    parks::PARKS_ON_NOTHING,
};
use skein_lib::Time;
use skein_world::streams::{Stream, Streams};
use smith_domain::{Fact, session, tools};
use std::collections::BTreeSet;
use temper_workload::draw::Workload;

pub(super) use jig_core_kit::cells::JOB_RUNNING_PARKS_ON_NOTHING as CELL;
pub(super) use smith_faults::parks::JOB_RUNNING;
pub(super) const KINDS: [Kind; 2] = [JOB_RUNNING, CELL];
const JOB: u32 = 1;
const NOTICE: &[u8] =
    b"Nothing more will come: your delegates have ended and no question waits. Finish with your result.";

pub(super) struct Target {
    pub(super) goal: String,
    pub(super) producer: String,
    kinds: Vec<Kind>,
    cue: Box<[u8]>,
}

impl Target {
    pub(super) fn draw(
        streams: Streams,
        workload: &Workload,
        kinds: &[Kind],
        ineligible: &mut BTreeSet<Kind>,
    ) -> Option<Self> {
        let eligible: Vec<_> = workload
            .goals
            .iter()
            .flat_map(|goal| goal.changes.iter().filter(|change| !change.asks).map(move |change| (goal, change)))
            .collect();
        let index =
            streams.rng(Stream::Faults, "cells").below(u64::try_from(eligible.len()).expect("bounded producers"));
        let kinds: Vec<_> = KINDS.into_iter().filter(|kind| kinds.contains(kind)).collect();
        if kinds.is_empty() {
            return None;
        }
        if let Some((goal, change)) = eligible.get(usize::try_from(index).expect("producer index")) {
            Some(Self {
                goal: goal.name.clone(),
                producer: change.name.clone(),
                kinds,
                cue: super::super::scripts::change_cue(change).into_bytes().into(),
            })
        } else {
            ineligible.extend(kinds);
            None
        }
    }

    pub(super) fn planned(&self) -> impl Iterator<Item = (Kind, &str)> {
        self.kinds.iter().map(|&kind| (kind, self.goal.as_str()))
    }
}

pub(super) struct Driver {
    target: Target,
    phase: Phase,
    pairing: Pairing,
    deferred_park: Option<(String, u64, u64)>,
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Drawn,
    First(Park),
    Parked(Park),
    Resumed(Park),
    Refused(Park),
    Ineligible,
}

impl Driver {
    pub(super) fn new(target: Target) -> Self {
        Self { target, phase: Phase::Drawn, pairing: Pairing::new(), deferred_park: None }
    }

    pub(super) fn retire(&mut self, ledger: &mut FaultLedger) {
        if matches!(self.phase, Phase::Drawn) {
            self.skip(ledger);
        }
    }

    fn skip(&mut self, ledger: &mut FaultLedger) {
        self.phase = Phase::Ineligible;
        for &kind in &self.target.kinds {
            ledger.ineligible(kind);
        }
    }
}

impl World {
    pub(in crate::composed) fn workload_park_deferred(&self) -> bool {
        self.workload_faults.as_ref().and_then(|f| f.cell.as_ref()).is_some_and(|cell| cell.deferred_park.is_some())
    }

    pub(in crate::composed) fn workload_cell_script(
        &mut self,
        assignment: &engine::Assignment,
        script: &mut Script,
    ) -> bool {
        let Some(f) = &mut self.workload_faults else { return false };
        match arm_workload_cell(f, assignment, script, &self.store, self.env.now) {
            Ok(first) => first,
            Err(finding) => {
                self.finding.get_or_insert(finding);
                false
            }
        }
    }
}

fn arm_workload_cell(
    f: &mut Faults,
    assignment: &engine::Assignment,
    script: &mut Script,
    store: &temper_engine_domain_world::store::Store,
    now: Time,
) -> Result<bool, String> {
    let Some(cell) = &mut f.cell else { return Ok(false) };
    let task = assignment.task;
    let attempt = assignment.attempt;
    match cell.phase {
        Phase::Drawn if attempt == assignment.conversation && script.cue == cell.target.cue => {
            let park = Park { task, attempt, number: 1 };
            let mut turns = vec![
                call(b"shell", br#"{"command":"park-cell-job","yield_seconds":0}"#),
                call(b"wait", b"{}"),
                call(b"job", br#"{"job":1,"wait_seconds":0}"#),
            ];
            turns.extend(std::mem::take(&mut script.turns));
            script.turns = turns.into_boxed_slice();
            // Keep the delivered park in the script, but observe it only
            // once this producer has resumed to do its work.
            if f.park.as_ref().is_some_and(|(_, producer, first)| (*producer, *first) == (task, attempt)) {
                cell.deferred_park = f.park.take();
            }
            cell.phase = Phase::First(park);
            Ok(true)
        }
        Phase::Parked(park) if task == park.task && attempt != park.attempt => {
            if attempt_end(store, park.task, park.attempt) != Some(tasks::EndKind::Parked) {
                return Err(format!("{}: {park:?} did not commit its parked end", cell.target.producer));
            }
            if !assignment
                .messages
                .iter()
                .any(|message| message.kind == tasks::MessageKind::NothingAwaited && message.words.as_ref() == NOTICE)
            {
                return Err(format!(
                    "{}: {park:?} resumed without the Nothing left to await notice",
                    cell.target.producer
                ));
            }
            record_half(&mut f.ledger, PARKS_ON_NOTHING, now, park)?;
            let paired = cell.pairing.fell(park, PARKS_ON_NOTHING, now, &mut f.ledger)?;
            if !paired.is_some_and(|paired| paired.kind == CELL && paired.column == Column::Resumes) {
                return Err(format!("{park:?}: expected the job-running cell to pair on its wake, got {paired:?}"));
            }
            if let Some((name, task, _)) = cell.deferred_park.take() {
                f.park = Some((name, task, attempt));
            }
            cell.phase = Phase::Resumed(park);
            f.stopped(now);
            Ok(false)
        }
        _ => Ok(false),
    }
}

impl World {
    pub(super) fn workload_cell_tick(&self, f: &mut Faults) -> Result<(), String> {
        let (Some(cell), Some(assignment)) = (&mut f.cell, &self.lanes[0].assignment) else { return Ok(()) };
        let process = &self.lanes[0].process;
        match cell.phase {
            Phase::First(park) if (assignment.task, assignment.attempt) != (park.task, park.attempt) => {
                // Another fault replaced the attempt before it could park.
                cell.skip(&mut f.ledger);
                if let Some(deferred) = cell.deferred_park.take() {
                    f.park = Some(deferred);
                }
            }
            Phase::First(park) if process.answered.is_some() => {
                let agent = process.agent();
                match agent.answer() {
                    run::Answer::Parked { .. } => {
                        if !job_ended_before_park(agent) {
                            return Err(format!(
                                "{}: {park:?} did not tell the running job's end before its parked answer",
                                cell.target.producer
                            ));
                        }
                        record_half(&mut f.ledger, JOB_RUNNING, self.env.now, park)?;
                        if let Some(early) = cell.pairing.fell(park, JOB_RUNNING, self.env.now, &mut f.ledger)? {
                            return Err(format!("{park:?} paired without its wake: {early:?}"));
                        }
                        cell.phase = Phase::Parked(park);
                        f.stopped(self.env.now);
                    }
                    run::Answer::Failed { failure: run::RunFailure::Cancelled | run::RunFailure::Model(_), .. } => {
                        cell.skip(&mut f.ledger);
                        if let Some(deferred) = cell.deferred_park.take() {
                            f.park = Some(deferred);
                        }
                    }
                    answer => {
                        return Err(format!(
                            "{}: {park:?} expected to park before working, got {answer:?}",
                            cell.target.producer
                        ));
                    }
                }
            }
            Phase::Resumed(park) if assignment.task == park.task && !process.prepared() => {
                if let Some(turn) = process.agent().turns().first() {
                    if !unknown_job(turn) {
                        return Err(format!(
                            "{}: {park:?} did not refuse job {JOB} as unknown on resume",
                            cell.target.producer
                        ));
                    }
                    cell.phase = Phase::Refused(park);
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn judge_workload_cell(&self) -> Result<(), String> {
        let Some(cell) = self.workload_faults.as_ref().and_then(|f| f.cell.as_ref()) else { return Ok(()) };
        match cell.phase {
            Phase::Ineligible => Ok(()),
            Phase::Refused(park) => {
                let finished = self.workload.as_ref().expect("workload").assignments.iter().any(|assignment| {
                    assignment.task == park.task
                        && assignment.attempt > park.attempt
                        && attempt_end(&self.store, park.task, assignment.attempt) == Some(tasks::EndKind::Finished)
                });
                if !finished || !self.workload_task_ended(park.task) {
                    return Err(format!(
                        "{}: {park:?} did not finish after the unknown job refusal",
                        cell.target.producer
                    ));
                }
                eprintln!(
                    "park cell: {} on {}, {park:?}; job {JOB} ended before park, was unknown on resume, and the producer finished",
                    cell.target.producer, cell.target.goal
                );
                Ok(())
            }
            phase => Err(format!("{}: incomplete park cell: {phase:?}", cell.target.producer)),
        }
    }
}

fn attempt_end(store: &temper_engine_domain_world::store::Store, task: u64, attempt: u64) -> Option<tasks::EndKind> {
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Attempt { task, attempt }));
    match store.rows.get(&key) {
        Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row)))) => row.end,
        _ => None,
    }
}

fn record_half(ledger: &mut FaultLedger, kind: Kind, now: Time, park: Park) -> Result<(), String> {
    let half = ledger.inject(kind, now, Struck { request: Some(format!("task {}", park.task)), ..Struck::default() });
    ledger.stop(half, now)
}

fn job_ended_before_park(agent: &Agent) -> bool {
    let mut facts = agent.facts().iter();
    let running = facts.find_map(|fact| match fact {
        Fact::Session {
            fact:
                session::Fact {
                    kind:
                        session::FactKind::Tools {
                            opener,
                            fact:
                                tools::FactKind::Answered {
                                    tool: tools::Tool::Shell,
                                    verdict: tools::Verdict::Running,
                                    job: Some(tools::JobNumber(JOB)),
                                    ..
                                },
                        },
                    ..
                },
        } => Some(*opener),
        _ => None,
    });
    let Some(opener) = running else { return false };
    let mut ended = false;
    for fact in facts {
        match fact {
            Fact::Session {
                fact:
                    session::Fact {
                        kind:
                            session::FactKind::JobEnded {
                                opener: owner,
                                job: tools::JobNumber(JOB),
                                end: tools::JobEnd::Cancelled,
                                ..
                            },
                        ..
                    },
            } if *owner == opener => ended = true,
            Fact::Run {
                fact:
                    run::facts::Fact {
                        kind: run::facts::FactKind::Answered { answer: run::facts::Answered::Parked, .. },
                        ..
                    },
                ..
            } => return ended,
            _ => {}
        }
    }
    false
}

fn unknown_job(turn: &session::record::Turn) -> bool {
    use session::llm::{Block, Problem, Returned};
    turn.messages.iter().flat_map(|message| &message.content).any(|block| {
        matches!(
            block,
            Block::ToolResult { result: Returned::Invalid { problem: Problem::UnknownJob { job: JOB } }, .. }
        )
    })
}
