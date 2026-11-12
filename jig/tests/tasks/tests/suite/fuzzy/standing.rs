//! Seeded standing lifecycles keep conservation through period and run interleavings.
use super::super::standing_lifecycle::{cancel, read, wake, world};
use jig_domain_tasks::{Active, Cause, End, Event, Funder, Key, Party, Phase, TaskResult};
use jig_tasks_world::{Reply, task};
use skein_lib::Rng;

#[test]
fn random_standing_periods_runs_finishes_and_delegates_end_only_by_cancel() {
    let mut renewals = 0;
    for seed in [871, 877, 881, 883, 887, 907, 911, 919] {
        let mut rng = Rng::new(seed);
        let mut world = world(seed);
        let mut period = 0;
        let mut child = 2;
        let mut attempt = 1;
        for _ in 0..32 {
            match rng.below(6) {
                0 if period < 8 => {
                    period += 1 + rng.below(3);
                    world.open_period(period, 100);
                    world.renew_standing(period);
                }
                1 if !world.runs.contains_key(&1) => {
                    if world.record(1).phase == Phase::Active(Active::Idle) {
                        wake(&mut world);
                    }
                    if world.activations.contains(&1) {
                        world.claim(1, attempt);
                        attempt += 1;
                        read(&mut world);
                    }
                }
                2 if world.runs.contains_key(&1) => {
                    let end = match rng.below(3) {
                        0 => End::Parked,
                        1 => End::Finished {
                            result: TaskResult::Report { words: b"done".as_slice().into() },
                            cancel_delegates: false,
                        },
                        _ => End::Finished {
                            result: TaskResult::Failure { reason: b"failed".as_slice().into() },
                            cancel_delegates: false,
                        },
                    };
                    let row = world.record(1);
                    let spend = rng.below(row.run_reserved.min(15) + 1);
                    world.terminal_cause(1, end, Cause::Priced { cumulative: spend });
                }
                3 if child <= 8 && !world.runs.contains_key(&1) => {
                    let row = world.record(1);
                    if row.numbers.budget - row.numbers.spent - row.numbers.reserved >= 10 {
                        let mut member = task(child, &[]);
                        member.funder = Funder::Task(1);
                        member.numbers.budget = 10;
                        member.authority.budget.spend = 10;
                        assert_eq!(world.make(Party::Task(1), vec![member]), Reply::Made(vec![child]));
                        child += 1;
                    }
                }
                4 => {
                    let selected = world.record(1).delegates.first().copied();
                    if let Some(selected) = selected {
                        world.claim_fresh(selected);
                        world.terminal_cause(
                            selected,
                            End::Finished {
                                result: TaskResult::Report { words: Box::new([]) },
                                cancel_delegates: false,
                            },
                            Cause::Priced { cumulative: rng.below(11) },
                        );
                        world.settle(selected);
                    }
                }
                5 if world.runs.is_empty() => world.restart(),
                _ => {}
            }
            assert!(world.records.contains_key(&Key::Live(1)), "seed {seed}");
            assert!(matches!(world.record(1).phase, Phase::Active(_)), "seed {seed}: no finish closes a standing task");
            assert!(!world.results.contains_key(&1), "seed {seed}: only cancel ends a standing task");
            if world.record(1).funder != (Funder::Period { project: 1, period: 0 }) {
                renewals += 1;
            }
        }
        cancel(&mut world);
        assert!(matches!(world.results[&1], jig_domain_tasks::Ending::Cancelled { .. }));
        // Even a later period cannot revive the cancelled standing task.
        world.send(Event::RenewStanding { task: 1, period: period + 1 });
        assert!(!world.records.contains_key(&Key::Live(1)));
    }
    assert!(renewals > 0, "the random stream exercises renewals under the accounting referee");
}
