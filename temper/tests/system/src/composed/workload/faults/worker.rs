//! The checkout's faults (temper/docs/design/testing/worker.md, 3) and the delivered producer's park.
use super::{DISK_FULL, FETCH_CUT, Faults, Injection, Struck, World, parks};
use crate::composed::{engine, run, worker};
use skein_lib::{Duration, Rng, Time, Token};
use temper_fake_checkout::git::Remote;
use temper_worker_checkout_world::translate;
use worker::checkout::git::{Done, Fault, Op};

pub(super) struct Disk {
    headroom: u64,
    duration: Duration,
    capacity: Option<u64>,
    until: Option<Time>,
    full: Option<Injection>,
}

impl Disk {
    pub(super) fn draw(rng: &mut Rng) -> Self {
        Self {
            headroom: rng.between(1, 32),
            duration: Duration::from_millis(rng.between(1, 3)),
            capacity: None,
            until: None,
            full: None,
        }
    }

    pub(super) fn stopped(&self) -> bool {
        self.capacity.is_none() && self.full.is_none()
    }
}

pub(super) struct FetchCut {
    lane: usize,
    owner: Token,
    op: Op,
    deadline: Time,
}

impl World {
    pub(in crate::composed) fn workload_fetch_deadline(&self) -> Option<Time> {
        self.workload_faults.as_ref()?.fetch_cut.as_ref().map(|cut| cut.deadline)
    }

    pub(in crate::composed) fn workload_cut_fetch(&mut self, at: usize) {
        let Some(f) = &mut self.workload_faults else { return };
        let now = self.env.now;
        if !f.fetch_cut.as_ref().is_some_and(|cut| cut.lane == at && cut.deadline <= now) {
            return;
        }
        let cut = f.fetch_cut.take().expect("fetch due to time out");
        self.forge.at(now);
        let lane = &mut self.lanes[at];
        let done = translate::cut(&mut self.forge, &mut lane.disk, cut.op);
        // A fetch the forge cannot serve did not reach the cut.
        if done == (Done::Failed { fault: Fault::TimedOut }) {
            f.fire(FETCH_CUT);
            let struck = Struck { request: Some(format!("hold {} fetch", cut.owner.raw())), ..Struck::default() };
            let injection = f.ledger.inject(FETCH_CUT, now, struck);
            f.ledger.stop(injection, now).expect("a fetch cut is an instant");
            f.stopped(now);
        }
        lane.worker_events.push_back(worker::Event::Done { owner: cut.owner, done });
        let _moves = self.forge.moves();
    }

    pub(in crate::composed) fn workload_cancel_fetch(&mut self, at: usize, owner: Token) {
        if let Some(f) = &mut self.workload_faults
            && f.fetch_cut.as_ref().is_some_and(|cut| cut.lane == at && cut.owner == owner)
        {
            f.fetch_cut = None;
            let done = Done::Failed { fault: Fault::Cancelled };
            self.lanes[at].worker_events.push_back(worker::Event::Done { owner, done });
        }
    }

    pub(super) fn workload_worker_tick(&mut self, f: &mut Faults) -> Result<(), String> {
        if f.due(DISK_FULL) {
            eprintln!("disk capacity at {:?}", self.env.now);
            f.fire(DISK_FULL);
            f.disk.capacity = Some(translate::used(&self.lanes[0].disk) + f.disk.headroom);
            // The capacity is lifted even if a Make frees enough space first.
            f.disk.until = Some(self.env.now.saturating_add(Duration::from_secs(60)));
        }
        if f.disk.until.is_some_and(|until| self.env.now >= until) {
            f.disk.capacity = None;
            f.disk.until = None;
            if let Some(injection) = f.disk.full.take() {
                f.ledger.stop(injection, self.env.now).expect("finite disk pressure");
            }
            f.stopped(self.env.now);
        }
        let Some((name, task, attempt)) = &f.park else { return Ok(()) };
        if f.parked {
            return Ok(());
        }
        // A replacement attempt uses the continuation, not the park script.
        let Some(assignment) = self.lanes[0].assignment.as_ref().filter(|a| a.task == *task && a.attempt == *attempt)
        else {
            f.skip_park();
            return Ok(());
        };
        if self.lanes[0].process.answered.is_none() {
            return Ok(());
        }
        match self.agent().answer() {
            run::Answer::Failed { .. } | run::Answer::Refused(_) => {
                f.skip_park();
                return Ok(());
            }
            run::Answer::Parked { .. } => {}
            answer @ run::Answer::Accepted { .. } => {
                return Err(format!("delivered producer {name} did not park: {answer:?}"));
            }
        }
        let name = name.clone();
        let branch = assignment.workspace.repositories[0].push.as_ref().expect("producer's branch");
        let head = self.forge.branch(super::super::REMOTE, branch).expect("delivered branch");
        let tree = self.forge.tree(head);
        for file in &self.workload().change(&name).expect("parked change").1.files {
            if tree.get(file.path.as_bytes()).map(Vec::as_slice) != Some(file.text.as_bytes()) {
                return Err(format!("{name} parked without delivering {}", file.path));
            }
        }
        f.fire(parks::DELIVERED_THEN_PARKED);
        f.parked = true;
        eprintln!("parked {name} at {:?}", self.env.now);
        let injection = f.ledger.inject(
            parks::DELIVERED_THEN_PARKED,
            self.env.now,
            Struck { request: Some(name), ..Struck::default() },
        );
        f.ledger.stop(injection, self.env.now).expect("park is an instant");
        f.stopped(self.env.now);
        Ok(())
    }

    pub(in crate::composed) fn workload_io(&mut self, at: usize, owner: Token, op: Op, deadline: Time) -> Option<Done> {
        if matches!(op, Op::Fetch { .. }) && self.workload_fetch_due(at) {
            self.workload_faults.as_mut().expect("faulted workload").fetch_cut =
                Some(FetchCut { lane: at, owner, op, deadline });
            return None;
        }
        let lane = &mut self.lanes[at];
        let conflicts = lane.worker_env.limits.checkout.conflicts;
        let Some(capacity) = self.workload_faults.as_ref().and_then(|f| f.disk.capacity) else {
            return Some(translate::perform_within(&mut self.forge, &mut lane.disk, op, conflicts));
        };
        let used = translate::used(&lane.disk);
        let request = format!("hold {} {:?}", owner.raw(), op.kind());
        let writes = translate::writes(&op);
        let done = translate::perform_within_capacity(&mut self.forge, &mut lane.disk, op, conflicts, Some(capacity));
        if let Some(f) = &mut self.workload_faults {
            let refused = matches!(&done, Done::FailedWithOutput { fault: Fault::Broken, diagnostic }
                if diagnostic.output() == b"No space left on device");
            if refused && f.disk.full.is_none() {
                eprintln!("disk refused {request} at {:?}", self.env.now);
                f.disk.until = Some(self.env.now.saturating_add(f.disk.duration));
                f.disk.full = Some(f.ledger.inject(
                    DISK_FULL,
                    self.env.now,
                    Struck { request: Some(request), ..Struck::default() },
                ));
            } else if writes
                && !matches!(done, Done::Failed { .. } | Done::FailedWithOutput { .. })
                && used < capacity
                && let Some(injection) = f.disk.full.take()
            {
                f.ledger.stop(injection, self.env.now).expect("write after disk freed");
                f.stopped(self.env.now);
            }
        }
        Some(done)
    }

    fn workload_fetch_due(&self, at: usize) -> bool {
        let Some(f) = &self.workload_faults else { return false };
        if f.fetch_cut.is_some() || !f.due(FETCH_CUT) {
            return false;
        }
        let lane = &self.lanes[at];
        // The composed workload has one hosted slot. Read the slot, not the
        // last agent assignment: a built-in job can also hold a workspace.
        let goal = lane.worker.host().hosted_at(0).and_then(|(run, _)| self.task_goal(run.raw()));
        goal == f.target(FETCH_CUT) && f.disk.capacity.is_none_or(|capacity| translate::used(&lane.disk) < capacity)
    }

    pub(in crate::composed) fn workload_choose_park(&mut self, assignment: &engine::Assignment) {
        let Some(f) = &self.workload_faults else { return };
        if f.park.is_some() || !f.due(parks::DELIVERED_THEN_PARKED) {
            return;
        }
        let goal = self.workload().goal(f.target(parks::DELIVERED_THEN_PARKED).expect("park target")).expect("goal");
        let words = &self.root.tasks().task(assignment.task).expect("assigned task").spec.words;
        let name = goal
            .changes
            .iter()
            .find(|change| {
                !change.asks && String::from_utf8_lossy(words).contains(&super::super::scripts::change_cue(change))
            })
            .map(|change| change.name.clone());
        if let Some(name) = name {
            let continuing = super::super::scripts::continuing(self, assignment.task);
            let f = self.workload_faults.as_mut().expect("faults");
            if continuing {
                f.skip_park();
            } else {
                f.park = Some((name, assignment.task, assignment.attempt));
            }
        }
    }

    pub(super) fn judge_workload_park(&self) -> Result<(), String> {
        use crate::composed::{Key, tasks};
        use jig_domain_core as core;
        let f = self.workload_faults.as_ref().expect("faults");
        let Some((name, producer, parked)) = &f.park else { return Ok(()) };
        let key = |attempt| Key::Core(core::Key::Tasks(tasks::Key::Attempt { task: *producer, attempt }));
        let attempt = |number| match self.store.rows.get(&key(number)) {
            Some(engine::Record::Core(core::Record::Tasks(tasks::Stored::Attempt(row)))) => row.end,
            _ => None,
        };
        if attempt(*parked) != Some(tasks::EndKind::Parked) {
            return Err(format!("{name}: its delivered attempt {parked} did not record the park"));
        }
        let story = self.workload.as_ref().expect("workload");
        // Judge the attempts after the park, not starts before it. An overlap
        // may interrupt a resumed attempt; it is not the producer's finish.
        let mut finished = false;
        for assignment in story.assignments.iter().filter(|a| a.task == *producer && a.attempt > *parked) {
            match attempt(assignment.attempt) {
                Some(tasks::EndKind::Finished) if !finished => finished = true,
                Some(tasks::EndKind::Suspended { .. } | tasks::EndKind::Failed { .. } | tasks::EndKind::Refused)
                    if !finished => {}
                end => return Err(format!("{name}: continuation {} ended {end:?}", assignment.attempt)),
            }
        }
        if !finished || !self.workload_task_ended(*producer) {
            return Err(format!("{name}: the same producer did not finish after its park"));
        }
        Ok(())
    }

    pub(in crate::composed) fn workload_parks(&self, task: u64) -> bool {
        self.workload_faults
            .as_ref()
            .is_some_and(|f| !f.parked && f.park.as_ref().is_some_and(|(_, producer, _)| *producer == task))
    }

    pub(in crate::composed) fn workload_answer_park(&mut self) {
        let Some(f) = &self.workload_faults else { return };
        if !f.parked {
            return;
        }
        let (name, _, _) = f.park.as_ref().expect("parked producer");
        let marker = format!("@park:{name}@");
        let party = self.workload().change(name).expect("change").0.party;
        let answer = self
            .workload_tasks()
            .find_map(|row| row.spec.words.starts_with(marker.as_bytes()).then_some((row.number, row.revisions.spec)));
        let Some((task, revision)) = answer else { return };
        if !self.workload.as_mut().expect("workload").answered.insert(task) {
            return;
        }
        self.workload_enqueue(
            party,
            crate::composed::people::Ask::AnswerPerson {
                project: 1,
                task,
                revision,
                result: crate::composed::people::PersonResult::Report {
                    words: b"Finish the delivered change.".as_slice().into(),
                },
            },
        );
    }
}

impl Faults {
    fn skip_park(&mut self) {
        self.park = None;
        self.fire(parks::DELIVERED_THEN_PARKED);
        self.ledger.ineligible(parks::DELIVERED_THEN_PARKED);
    }
}
