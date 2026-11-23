//! World-side restart faults (temper/docs/design/testing/restarts.md, 3 and 5).
//! Neither scheduling nor fault draws consume the connector world's generator.
use super::{REPO, World, forge, top};
use skein_forge::api::{Answer, Error, Op, Read, Write};
use skein_forge_client::{Condition, Effect, Entry};
use skein_lib::{Duration, Env, Rng, Time, Wall};
use skein_world::{
    faults::{FaultLedger, Injection, Kind, Struck},
    streams::{Stream, Streams},
    sweep::Sweep,
};
use temper_faults::restarts::{self as kinds, ENGINE_CRASH};

pub const GOAL: u64 = 77;
pub const BRANCH: &[u8] = b"temper/77/g77";
pub const ENDED: u64 = 63;

#[derive(Default)]
pub(super) struct Scripts {
    down: Option<(Wall, Option<Wall>)>,
    find: bool,
    limited: Option<Read>,
    lose_made: bool,
    made_without_answer: u32,
    answers: Vec<(Wall, Op, Error)>,
    synthetic_entries: Vec<u64>,
}

/// One scheduled restart, with one additional kind, drawn before the run.
/// A focused test instead calls `inject` at its named boundary.
pub struct Injector {
    rng: Rng,
    at: u64,
    iteration: u64,
    kind: Option<Kind>,
    horizon: Wall,
    moment: u64,
    crashed: Vec<(Injection, u64)>,
    ledger: FaultLedger,
}

impl Injector {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Streams::new(seed).rng(Stream::Faults, kinds::PART),
            at: 0,
            iteration: 0,
            kind: None,
            horizon: Wall::EPOCH,
            moment: 0,
            crashed: Vec::new(),
            ledger: FaultLedger::new(),
        }
    }

    /// Draw the restart's driver iteration and extra fault from the configured
    /// kinds. A crash accompanies every drawn restart, even a crash-only draw.
    pub fn draw(&mut self, configured: &[Kind], iterations: u64, horizon: Wall) {
        assert!(self.kind.is_none());
        assert_ne!(configured, []);
        self.at = self.rng.below(iterations);
        self.kind = configured
            .get(usize::try_from(self.rng.below(u64::try_from(configured.len()).expect("kinds"))).expect("index"))
            .copied();
        self.horizon = horizon;
        self.moment = self.rng.next_u64();
    }

    /// Whether the workload must first make an uncertain issue write.
    #[must_use]
    pub fn needs_uncertain_write(&self) -> bool {
        self.kind == Some(kinds::FIND_KEEPS_FAILING)
    }

    #[must_use]
    pub fn ledger(&self) -> &FaultLedger {
        &self.ledger
    }

    /// A single iteration of the world's driver; no drain precedes a restart.
    /// The final iteration takes a draw that the time horizon would cut short.
    pub fn step(&mut self, world: &mut World, sweep: &Sweep, end: Time) -> bool {
        if (self.iteration >= self.at || world.last_restart_iteration(end))
            && let Some(kind) = self.kind.take()
        {
            let from = world.wall();
            let remaining = self.horizon.as_nanos().saturating_sub(from.as_nanos());
            let offset = u64::try_from((u128::from(remaining) * u128::from(self.moment)) >> 64).expect("scaled moment");
            let until = Wall::from_nanos(from.as_nanos().saturating_add(offset.max(1)).min(self.horizon.as_nanos()));
            let mut selected = vec![ENGINE_CRASH];
            if kind != ENGINE_CRASH {
                selected.push(kind);
            }
            // Conditions are read here, before injection. This sweep currently
            // has no exclusions; a finding is never screened after it happens.
            selected.retain(|fault| {
                if let Some(exclusion) = sweep.leaves_out(*fault, &[]) {
                    self.ledger.leave_out(exclusion.scenario, *fault, world.now());
                    false
                } else {
                    true
                }
            });
            self.inject(world, &selected, Some(until));
        }
        self.iteration += 1;
        let running = world.iterate(end);
        self.settled(world);
        running
    }

    /// Inject only these kinds at this named restart. `until` is required for
    /// the finite outage; drawn restarts always include `ENGINE_CRASH`.
    pub fn inject(&mut self, world: &mut World, selected: &[Kind], until: Option<Wall>) {
        world.restart_scripts.get_or_insert_with(Scripts::default);
        let from = world.now();
        for kind in selected {
            if *kind == kinds::ENTRIES_HELD_BY_ENDED {
                synthetic_ended_entries(world);
            } else if *kind == kinds::BRANCH_THE_FORGE_LACKS {
                synthetic_missing_branch(world);
            } else if *kind == kinds::FORGE_DOWN_UNTIL {
                let until = until.expect("finite outage's end");
                assert!(until > world.wall());
                world.restart_scripts.as_mut().expect("restart scripts").down = Some((world.wall(), Some(until)));
            } else if *kind == kinds::FORGE_DOWN_FOR_GOOD {
                world.restart_scripts.as_mut().expect("restart scripts").down = Some((world.wall(), None));
            } else if *kind == kinds::FIND_KEEPS_FAILING {
                assert!(
                    world.restart_scripts.as_ref().expect("restart scripts").made_without_answer > 0,
                    "a write made before the restart lost its answer"
                );
                assert!(
                    world.stored.iter().any(|(_, row)| matches!(
                        row,
                        top::Stored::Entry {
                            entry: Entry {
                                effect: Effect { write: Write::CreateIssue { .. }, .. },
                                attempt: Some(_),
                                ..
                            },
                            ..
                        }
                    )),
                    "the lost write is still uncertain at the restart"
                );
                world.restart_scripts.as_mut().expect("restart scripts").find = true;
            } else if *kind == kinds::READ_ALWAYS_LIMITED {
                world.restart_scripts.as_mut().expect("restart scripts").limited =
                    Some(Read::Branch { branch: b"main".as_slice().into() });
            } else {
                assert_eq!(*kind, ENGINE_CRASH, "this world's restart vocabulary");
            }
        }
        world.restart();
        for kind in selected {
            let injection = self.ledger.inject(*kind, from, Struck::default());
            if *kind == ENGINE_CRASH {
                self.crashed.push((injection, world.generation));
            } else if *kind == kinds::ENTRIES_HELD_BY_ENDED || *kind == kinds::BRANCH_THE_FORGE_LACKS {
                self.ledger.stop(injection, from).expect("instant fixture");
            } else if *kind == kinds::FORGE_DOWN_UNTIL {
                self.ledger
                    .stop(injection, Time::from_nanos(until.expect("finite end").as_nanos()))
                    .expect("finite outage");
            }
        }
        self.settled(world);
    }

    pub fn settled(&mut self, world: &World) {
        self.crashed.retain(|(entry, generation)| {
            if let Some((settled, at)) = world.settled_generation
                && *generation == settled
            {
                self.ledger.stop(*entry, at).expect("crash ends at the settle");
                false
            } else {
                true
            }
        });
    }
}

#[must_use]
pub fn goal_name() -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([
            b"temper".as_slice().into(),
            b"77".as_slice().into(),
            b"g77".as_slice().into(),
        ])),
    }
}

/// A held goal branch absent from the forge and with no creation entry. No
/// legitimate sequence of events makes this synthetic restart snapshot.
pub fn synthetic_missing_branch(world: &mut World) {
    assert!(!world.has_branch(BRANCH));
    world.hold_task(GOAL);
    world.store_before_restart(top::Stored::Hold(top::Hold {
        name: goal_name(),
        task: GOAL,
        writer: None,
        beside: None,
        design_writers: Box::new([]),
        drift: None,
    }));
}

/// Cancelled landings retain every write-entry place as held gate dispatches.
/// Existing entry numbers are retained, but their entries become cancelled
/// dispatches too. No legitimate sequence of events makes this synthetic
/// restart snapshot.
pub fn synthetic_ended_entries(world: &mut World) {
    world.hold_task(ENDED);
    let mut entries: Vec<_> = world
        .stored
        .iter()
        .filter_map(|(_, row)| match row {
            top::Stored::Entry { entry, .. } => Some(entry.number),
            _ => None,
        })
        .collect();
    let mut next = entries.iter().copied().max().unwrap_or(0) + 1;
    while entries.len() < usize::try_from(world.env.limits.entries).expect("entry room") {
        entries.push(next);
        next += 1;
    }
    world.store_before_restart(top::Stored::Landing(synthetic_cancelled_landing(&entries)));
    for number in entries {
        world.store_before_restart(synthetic_held_dispatch(number));
        world.restart_scripts.get_or_insert_with(Scripts::default).synthetic_entries.push(number);
    }
}

/// A cancelled owner retaining the named dispatch entries. No legitimate
/// sequence of events makes this synthetic restart snapshot.
#[must_use]
pub fn synthetic_cancelled_landing(entries: &[u64]) -> top::LandingRow {
    use temper_engine_domain_forge_landing as landing;
    top::LandingRow {
        last_event: None,
        turn_since: None,
        waiting: None,
        room: None,
        job: None,
        landing: landing::Landing {
            last_left: None,
            searched: None,
            task: ENDED,
            project: 5,
            phase: landing::Phase::Cancelled,
            members: Box::new([]),
            preparation: 0,
            stale_proofs: 0,
            branch: b"main".as_slice().into(),
            round: landing::Round::new(b"temper/63/l63".as_slice().into()),
            last_prepare: None,
        },
        asks: Box::new([]),
        entries: entries
            .iter()
            .map(|entry| top::LandingEntry {
                preparation: None,
                base: None,
                entry: *entry,
                member: 0,
                gate: 0,
                operation: top::Operation::Dispatch,
                run: None,
            })
            .collect(),
        follows: None,
        flake_reruns: Box::new([]),
    }
}

/// One held dispatch of a cancelled landing. No legitimate sequence of events
/// makes this synthetic restart snapshot.
#[must_use]
pub fn synthetic_held_dispatch(number: u64) -> top::Stored {
    top::Stored::Entry {
        entry: Entry {
            number,
            task: ENDED,
            repository: REPO,
            effect: Effect {
                write: Write::Dispatch {
                    workflow: b"gate.yml".as_slice().into(),
                    branch: b"main".as_slice().into(),
                    inputs: Box::new([]),
                },
                condition: Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
        held: true,
    }
}

impl World {
    pub(super) fn forget_synthetic_entry(&mut self, number: u64) {
        if let Some(scripts) = &mut self.restart_scripts {
            scripts.synthetic_entries.retain(|laid| *laid != number);
        }
    }

    pub(super) fn changing_synthetic_entry(&mut self, record: &top::Stored) {
        if self.restart_scripts.is_some()
            && let top::Stored::Entry { entry, .. } = record
            && self.stored.get(&top::Key::Entry(entry.number)) != Some(record)
        {
            self.forget_synthetic_entry(entry.number);
        }
    }

    /// Only the still-held rows laid by the impossible ended-owner fixture
    /// are exempt from their release rule (testing/restarts.md, 3).
    pub(super) fn synthetic_held_entries(&self) -> Vec<u64> {
        let Some(scripts) = &self.restart_scripts else { return Vec::new() };
        scripts
            .synthetic_entries
            .iter()
            .copied()
            .filter(|number| {
                let Some(top::Stored::Entry { entry, held: true }) = self.stored.get(&top::Key::Entry(*number)) else {
                    return false;
                };
                entry.number == *number && entry.task == ENDED && self.top.inspection_entry(*number) == Some(entry)
            })
            .collect()
    }

    fn last_restart_iteration(&self, end: Time) -> bool {
        self.to_top.is_empty()
            && self.to_forge.is_empty()
            && !self.top.is_ready()
            && self.top.next_deadline().is_none_or(|at| at >= end)
            && self.top.next_wall_deadline().is_none_or(|at| at.as_nanos() >= end.as_nanos())
            && self.forge.next_deadline().is_none_or(|at| at >= end)
    }

    /// Lose successful issue-creation answers only after the fake made the writes.
    pub fn lose_made_write_answers(&mut self) {
        self.restart_scripts.get_or_insert_with(Scripts::default).lose_made = true;
    }

    #[must_use]
    pub fn made_without_answer(&self) -> u32 {
        self.restart_scripts.as_ref().map_or(0, |scripts| scripts.made_without_answer)
    }

    #[must_use]
    pub fn dropped_answers(&self) -> u32 {
        self.dropped_answers
    }

    /// The current generation's settle, observed at the connector boundary.
    #[must_use]
    pub fn restart_settled_at(&self) -> Option<Time> {
        self.settled_generation.filter(|(generation, _)| *generation == self.generation).map(|(_, at)| at)
    }

    /// Each refused or limited call, as answered at the world's fake boundary.
    #[must_use]
    pub fn restart_answers(&self) -> &[(Wall, Op, Error)] {
        self.restart_scripts.as_ref().map_or(&[], |scripts| scripts.answers.as_slice())
    }

    pub(super) fn restart_refusal(&mut self, op: &Op) -> Option<Error> {
        let scripts = self.restart_scripts.as_ref()?;
        let down =
            scripts.down.is_some_and(|(from, until)| self.wall() >= from && until.is_none_or(|end| self.wall() < end));
        let find =
            scripts.find && matches!(op, Op::Read(Read::Items { kind: Some(skein_forge::api::Kind::Issue), .. }));
        if down || find {
            let error = Error::Unavailable;
            let at = self.wall();
            self.restart_scripts.as_mut().expect("restart scripts").answers.push((at, op.clone(), error));
            Some(error)
        } else {
            None
        }
    }

    pub(super) fn restart_call_config(&mut self, event: &forge::Event, env: &mut Env<forge::Config>) {
        if let forge::Event::Call { op, user, .. } = event
            && let Some(read) = &self.restart_scripts.as_ref().expect("restart scripts").limited
            && *op == crate::translate::op(&Op::Read(read.clone()), &self.env.limits.client)
        {
            env.limits.rate_limit = 1;
            env.limits.rate_window = Duration::from_secs(1);
            // Consume this user's allowance first. The real call below asks
            // the fake to refuse it, and all other routes keep their config.
            let _ = forge::rate(&mut self.forge, env, forge::RateKey::User(*user), 1, env.limits.rate_window);
        }
    }

    pub(super) fn restart_loses_answer(&mut self, op: &Op, result: &Result<Answer, Error>) -> bool {
        let scripts = self.restart_scripts.as_mut().expect("restart scripts");
        if scripts.lose_made && matches!(op, Op::Write(Write::CreateIssue { .. })) && result.is_ok() {
            scripts.made_without_answer += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn restart_answer(&mut self, op: &Op, result: &Result<Answer, Error>) {
        if let Err(error @ Error::RateLimited { .. }) = result {
            let at = self.wall();
            self.restart_scripts.as_mut().expect("restart scripts").answers.push((at, op.clone(), *error));
        }
    }
}
