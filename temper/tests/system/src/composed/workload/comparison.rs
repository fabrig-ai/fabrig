//! Comparable composed runs (temper/docs/design/testing/workloads.md, 7).
use super::{World, engine};
use crate::composed::restarts::{Injector, Point, Restart};
use skein_lib::Duration;
use skein_world::faults::FaultLedger;
use temper_engine_domain_world::outcome::{self, Deadlines};
use temper_fake_checkout::git::Remote;
use temper_workload::{
    draw::Workload,
    outcome::{Left, Outcome},
};

pub(super) struct Run {
    goal: String,
    attempt: Option<u64>,
    points: (usize, usize),
    restart: Option<(Point, Restart)>,
    ledger: FaultLedger,
    deadlines: Deadlines,
}

impl Run {
    pub(super) fn new(seed: u64, workload: &Workload, points: Option<(usize, usize)>) -> Self {
        // The dependent goal has not started while the workload is being opened.
        // Its first activation reaches both model queries and host calls.
        let goal = workload.goals.iter().find(|goal| goal.after.is_some()).expect("dependent goal").name.clone();
        Self {
            goal,
            attempt: None,
            points: (0, 0),
            restart: points.map(|(queries, calls)| Injector::new(seed).draw(queries, calls)),
            ledger: FaultLedger::new(),
            deadlines: Deadlines::default(),
        }
    }
}

impl World {
    /// Replay the same draw with a mid-turn point counted on its clean side.
    #[must_use]
    pub fn restarted_workload(seed: u64, points: (usize, usize)) -> Self {
        Self::workload_with_restart(seed, &[], Some(points))
    }

    /// Counts of the dependent goal's first activation, before its first park.
    #[must_use]
    pub fn workload_restart_points(&self) -> (usize, usize) {
        self.workload.as_ref().expect("workload").comparison.as_ref().expect("comparison").points
    }

    #[must_use]
    pub fn workload_restart_ledger(&self) -> &FaultLedger {
        &self.workload.as_ref().expect("workload").comparison.as_ref().expect("comparison").ledger
    }

    #[must_use]
    pub fn workload_outcome(&self) -> Outcome {
        let main = self.forge.branch(super::REMOTE, b"main").expect("main");
        let files = self
            .forge
            .tree(main)
            .into_iter()
            .map(|(path, text)| {
                (String::from_utf8(path).expect("workload path"), String::from_utf8(text).expect("workload text"))
            })
            .collect();
        outcome::read(&self.store, &files, self.workload_names(), self.workload(), self.workload_restart_ledger())
    }

    #[must_use]
    pub fn workload_restart_left(&self) -> Vec<Left> {
        self.workload.as_ref().expect("workload").comparison.as_ref().expect("comparison").deadlines.left(
            &self.store,
            self.workload_names(),
            self.workload(),
        )
    }

    pub(super) fn workload_comparison_assignment(&mut self, assignment: &engine::Assignment) {
        let story = self.workload.as_mut().expect("workload");
        let Some(run) = &mut story.comparison else { return };
        if story.names.task(&run.goal) != Some(assignment.task) || run.attempt.is_some() {
            return;
        }
        run.attempt = Some(assignment.attempt);
        if let Some((point, _)) = run.restart {
            match point {
                Point::Model(query) => self.process.held_query = Some(query),
                Point::HostCall(call) => self.process.held_host_call = Some(call),
            }
        }
    }

    pub(super) fn workload_comparison_tick(&mut self) {
        let Some(mut run) = self.workload.as_mut().expect("workload").comparison.take() else { return };
        let target = self.workload_names().task(&run.goal);
        if self.assignment.as_ref().is_some_and(|a| Some(a.task) == target && Some(a.attempt) == run.attempt)
            && !self.process.prepared()
        {
            run.points = self.process.mid_turn_points();
            if let Some((point, restart)) = run.restart {
                let held = match point {
                    Point::Model(query) => {
                        self.process.agent().prompts().len() == query && self.process.agent().turns().len() + 1 == query
                    }
                    Point::HostCall(_) => self.process.host_call_held(),
                };
                if held
                    && self.worker.retained_turns() == 0
                    && self.store.idle()
                    && self.root_events.is_empty()
                    && self.worker_events.is_empty()
                {
                    eprintln!("restart {} at {point:?}: {restart:?}", run.goal);
                    let injection = self.inject_mid_turn(restart, &mut run.ledger);
                    let mut armed = Deadlines::armed(&self.root, &self.store, self.env.now);
                    self.elapse_absence(Duration::from_millis(100));
                    // Reopen the composed settings, not the single-chat fixture's.
                    let config = temper_engine_domain_world::oracle::config::copy(&self.restart_config);
                    self.reopen_with_config(config);
                    for _ in 0..super::super::STEPS {
                        self.tick();
                        armed.extend(Deadlines::armed(&self.root, &self.store, self.env.now));
                        if self.opened {
                            break;
                        }
                    }
                    assert!(self.opened, "the restarted engine opened within OPENING");
                    run.ledger.stop(injection, self.env.now).expect("one restart interval");
                    run.deadlines.struck(&armed, run.ledger.entries());
                    self.ends.faults_stop(self.env.now);
                    self.join_link();
                    self.release_mid_turn(point);
                    run.restart = None;
                }
            }
        }
        self.workload.as_mut().expect("workload").comparison = Some(run);
    }
}
