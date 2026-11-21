//! Seeded filing, triage and judging in one tier (domain/red-repairs.md, 10).
use super::*;
use crate::findings::{BlameAct, FindingRepair, RepairAct, RepairJudging};
use crate::settings::{RepairGoal, Repairs};
use alloc::collections::{BTreeMap, BTreeSet};
use skein_lib::Rng;

const PROJECT: u32 = 5;
const TRIAGER: Filer = Filer::Task(40);
const REPAIRS: Repairs = Repairs {
    goals: 2,
    attempts: 2,
    deadline_ms: 86_400_000,
    goal: RepairGoal { charter: 1, llm: None, priority: 7, budget: 20 },
};
const TURNS: u64 = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Move {
    Run { many: bool },
    Repair,
    Route,
    Backlog,
    Drop,
    Blame { landing: bool },
    Land,
    End { ending: ReleaseEnding, landed: bool },
    Hold,
    Release { judging: bool },
    Restart,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    findings: Vec<Finding>,
    tier: TierRow,
    log: Option<LandingLog>,
}

struct World {
    d: Domain,
    env: Env<Limits>,
    goals: BTreeMap<u64, bool>, // Live goals and whether they have landed.
    ended: BTreeSet<u64>,
}

impl World {
    fn new() -> Self {
        let mut env = env();
        env.limits.findings_per_project = 3;
        let mut d = one();
        d.take_project_repairs(PROJECT, Ok(REPAIRS));
        d.landing.projects.get_mut(&PROJECT).unwrap().2.verdict.suspects = p::Suspects::TellAll;
        let tier = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
        tier.record.run = None;
        tier.result = None;
        tier.task = None;
        Self { d, env, goals: BTreeMap::new(), ended: BTreeSet::new() }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            findings: self.d.findings(PROJECT).map(|(_, finding)| finding.clone()).collect(),
            tier: row(&self.d).clone(),
            log: self.d.landing_log(REPO).cloned(),
        }
    }

    fn restart(&mut self) -> Move {
        let saved = self.snapshot();
        self.d = Self::new().d;
        for finding in &saved.findings {
            outputs(&mut self.d, Event::Restore { record: Stored::Finding(finding.clone()) });
        }
        outputs(&mut self.d, Event::Restore { record: Stored::Tier(saved.tier.clone()) });
        if let Some(log) = &saved.log {
            outputs(&mut self.d, Event::Restore { record: Stored::LandingLog(log.clone()) });
        }
        assert_eq!(self.snapshot(), saved);
        Move::Restart
    }

    fn check(&self, seed: u64, turn: u64, action: Option<Move>) {
        let at = format!("seed {seed}, turn {turn}, {action:?}");
        let mut ids = BTreeSet::new();
        let mut live = BTreeSet::new();
        for (_, finding) in self.d.findings(PROJECT) {
            if let Some(repair) = &finding.repair {
                assert!(repair.attempts <= REPAIRS.attempts, "{at}: finding {} over attempt cap", finding.number);
                if repair.until.is_some() {
                    assert!(!self.ended.contains(&repair.goal), "{at}: ended repair goal still counts as live");
                    live.insert(repair.goal);
                }
            }
            if finding.uncleared_red() {
                let sig = finding.signature.as_ref().unwrap();
                for id in &sig.ids {
                    assert!(ids.insert((sig.repository, &sig.source, id)), "{at}: duplicate failing id {id:?}");
                }
                if finding.state == FindingState::Routed
                    && let Some(goal) = finding.goal
                    && self.ended.contains(&goal)
                {
                    assert!(
                        finding.repair.as_ref().is_some_and(|repair| repair.goal == goal && repair.judging.is_some()),
                        "{at}: red routed to ended goal {goal} without a pending judgement: {finding:?}"
                    );
                }
            }
        }
        assert!(live.len() <= usize::from(REPAIRS.goals), "{at}: too many live repair goals {live:?}");
        let tier = row(&self.d);
        for filing in &tier.waiting {
            let sig = &filing.signature;
            for id in &sig.ids {
                assert!(ids.insert((sig.repository, &sig.source, id)), "{at}: waiting duplicate of {id:?}");
            }
        }
        assert!(tier.waiting.len() <= 8, "{at}: too many waiting filings");
    }

    fn commit(&mut self, write: FindingWrite) {
        self.d.commit_finding(&self.env.limits, write, &mut Queue::uncapped());
    }

    fn returns(&mut self, requests: Vec<Request>) {
        for request in requests {
            if let Request::ReturnFinding(row) = request {
                self.commit(FindingWrite { row, erase: None });
            }
        }
    }

    fn run(&mut self, rng: &mut Rng) -> Option<Move> {
        if row(&self.d).hold.is_some() {
            return None;
        }
        let mut failed = Vec::new();
        let mut originally = Vec::new();
        for id in [A, B, C, OWN, b"extra::a", b"extra::b"] {
            if rng.chance(600) {
                originally.push(id);
                if rng.chance(800) {
                    failed.push(id);
                }
            }
        }
        failed.sort_unstable();
        let many = rng.chance(150);
        let failing = if many { p::Failures::Many } else { failures(&failed) };
        let range: Box<[p::Landing]> = self
            .d
            .landing_log(REPO)
            .into_iter()
            .flat_map(|log| log.landings.iter())
            .filter(|landing| landing.tiers.iter().any(|tier| tier.verdict == LoggedVerdict::Verifying))
            .map(|landing| p::Landing { commit: landing.commit, key: landing.commit })
            .collect();
        let judged_tip = row(&self.d).record.judged;
        let last_run = row(&self.d).record.last_run;
        next(&mut self.d, failing.clone(), &originally);
        let tier = self.d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
        tier.record.judged = judged_tip;
        tier.record.last_run = last_run;
        let run = tier.record.run.as_mut().unwrap();
        let number = run.number;
        let tip = range.last().map_or(commit(self.env.wall.as_nanos()), |last| last.commit);
        run.tip = tip;
        run.key = tip;
        run.landings = range;
        tier.result.as_mut().unwrap().candidate = tip;
        let (calls, out) = judged(&mut self.d, &self.env);
        self.returns(out);
        take(&mut self.d, &self.env, calls);
        assert_eq!(row(&self.d).record.last_run, Some(number), "the drawn run was judged");
        assert_eq!(row(&self.d).record.failing, failing, "the drawn failures were judged");
        Some(Move::Run { many })
    }

    fn repair(&mut self, number: u32, goal: u64) -> Option<Move> {
        let act = RepairAct { more: Box::new([]), words: b"Fix what is red.".as_slice().into() };
        let mut writes = self.d.prepare_repair_findings(PROJECT, number, &act).ok()?.into_vec();
        // The world supplies jig's fresh goal and stamps the admitted write as the root does.
        assert_eq!(writes.len(), 1);
        let mut write = writes.pop().unwrap();
        let prior = write.row.repair.take();
        write.row.repair = Some(FindingRepair {
            goal,
            made: true,
            attempts: prior.as_ref().map_or(1, |repair| repair.attempts.checked_add(1).expect("bounded attempts")),
            cancelled: false,
            failing: prior.map_or_else(Box::default, |repair| repair.failing),
            until: Some(Wall::from_nanos(
                self.env
                    .wall
                    .as_nanos()
                    .checked_add(Duration::from_millis(REPAIRS.deadline_ms).as_nanos())
                    .expect("bounded test wall"),
            )),
            judging: None,
        });
        write.row.state = FindingState::Routed;
        write.row.goal = Some(goal);
        self.commit(write);
        self.goals.insert(goal, false);
        Some(Move::Repair)
    }

    fn triage(&mut self, rng: &mut Rng, goal: u64) -> Option<Move> {
        let candidates: Vec<_> =
            self.d.findings(PROJECT).filter(|(_, row)| row.uncleared_red()).map(|(_, row)| row.number).collect();
        let number = pick(rng, candidates.into_iter())?;
        let (action, act) = match rng.below(8) {
            0..=3 => return self.repair(number, goal),
            4 => (Move::Route, Triage::Route(goal)),
            5 => (Move::Backlog, Triage::Backlog),
            6 => (Move::Drop, Triage::Drop(b"triaged".as_slice().into())),
            _ => {
                let landing = match self.d.landing_log(REPO) {
                    Some(log) if rng.chance(700) => pick(rng, log.landings.iter().map(|landing| landing.commit)),
                    _ => None,
                };
                (
                    Move::Blame { landing: landing.is_some() },
                    Triage::Blame(BlameAct { landing, why: b"Better evidence".as_slice().into() }),
                )
            }
        };
        let write = self.d.prepare_triage_finding(TRIAGER, TriageFinding { project: PROJECT, number, act }).ok()?;
        self.commit(write);
        if action == Move::Route {
            self.goals.insert(goal, false);
        }
        Some(action)
    }

    fn land(&mut self, rng: &mut Rng) -> Option<Move> {
        let candidates: Vec<_> = self.goals.iter().filter_map(|(&goal, &landed)| (!landed).then_some(goal)).collect();
        let goal = pick(rng, candidates.into_iter())?;
        let mut change = fixtures::opening_row();
        change.goal = Some(goal);
        change.repository = REPO;
        change.base = b"main".as_slice().into();
        let task = change.task;
        outputs(&mut self.d, Event::Restore { record: Stored::Change(Box::new(change)) });
        landing_log::landed(
            &mut self.d,
            self.env.wall,
            commit(self.env.wall.as_nanos()),
            task,
            Box::new([]),
            &mut Queue::uncapped(),
        );
        self.goals.insert(goal, true);
        Some(Move::Land)
    }

    fn end(&mut self, rng: &mut Rng) -> Option<Move> {
        let goal = pick(rng, self.goals.keys().copied())?;
        let landed = self.goals.remove(&goal).unwrap();
        self.ended.insert(goal);
        let ending = match rng.below(3) {
            0 => ReleaseEnding::Done,
            1 => ReleaseEnding::Failed,
            _ => ReleaseEnding::Cancelled,
        };
        let mut out = Queue::uncapped();
        self.d.judge_goal_findings(&self.env, PROJECT, goal, ending, &mut out);
        self.returns(collect(out).into_vec());
        Some(Move::End { ending, landed })
    }

    fn toggle_hold(&mut self) -> Move {
        let (action, act) = if row(&self.d).hold.is_none() {
            (Move::Hold, TierAct::Hold { reason: b"Maintenance".as_slice().into() })
        } else {
            (Move::Release { judging: self.waits_for_release() }, TierAct::Release)
        };
        let mut out = Queue::uncapped();
        let token = Token::new(self.env.wall.as_nanos());
        step(
            &mut self.d,
            &self.env,
            Event::TierAct {
                request: token,
                token,
                party: 44,
                call: TierActRequest { project: PROJECT, repository: None, tier: TIER.into(), act },
            },
            &mut out,
        );
        self.returns(collect(out).into_vec());
        assert_eq!(row(&self.d).hold.is_some(), action == Move::Hold);
        if matches!(action, Move::Release { .. }) {
            assert!(!self.waits_for_release(), "release must assess all findings waiting on it");
        }
        action
    }

    fn waits_for_release(&self) -> bool {
        self.d.findings(PROJECT).any(|(_, finding)| {
            finding.uncleared_red()
                && finding.repair.as_ref().is_some_and(|repair| repair.judging == Some(RepairJudging::Release))
        })
    }
}

fn pick<T>(rng: &mut Rng, mut choices: impl ExactSizeIterator<Item = T>) -> Option<T> {
    if choices.len() == 0 {
        None
    } else {
        choices.nth(usize::try_from(rng.below(u64::try_from(choices.len()).unwrap())).unwrap())
    }
}

fn commit(turn: u64) -> api::Commit {
    let mut commit = [0; 32];
    commit[..8].copy_from_slice(&turn.to_be_bytes());
    commit
}

pub(crate) fn random_repairs(seed: u64) -> Vec<(Option<Move>, Snapshot)> {
    let mut rng = Rng::new(seed);
    let mut world = World::new();
    let mut trace = Vec::new();
    for turn in 1..=TURNS {
        world.env.wall = Wall::from_nanos(turn);
        let action = match rng.below(24) {
            0..=8 => world.run(&mut rng),
            9..=15 => world.triage(&mut rng, turn.checked_add(1000).expect("fresh scripted goal")),
            16..=18 => world.land(&mut rng),
            19..=21 => world.end(&mut rng),
            22 => Some(world.toggle_hold()),
            _ => Some(world.restart()),
        };
        world.check(seed, turn, action);
        trace.push((action, world.snapshot()));
    }
    trace
}
