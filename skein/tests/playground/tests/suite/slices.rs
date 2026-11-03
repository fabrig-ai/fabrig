//! The slices' world tests (playground.md, 8).

mod support;

use skein_lib::{Duration, Rng, Time};
use skein_playground::slices::{Slices, Wake};
use skein_playground_world::scripted::{self, Plan};
use skein_ui_web::bridge::{Clock, Speed, Stamp, Stream, ToPage};
use support::{ANSWER_STEPS, BUDGET, SEEDS, Said};

#[test]
fn a_world_paced_in_slices_ends_as_the_world_run_whole() {
    // Enough inputs to overlap delayed jobs, including several queued in one slice.
    const INPUTS: u32 = 12;
    for seed in SEEDS {
        let mut random = Rng::new(seed);
        let mut plans = Vec::new();
        let mut schedule = Vec::new();
        let mut when = Time::ZERO;
        for id in 0..INPUTS {
            plans.push(Plan {
                steps: u32::try_from(1 + support::draw(&mut random, 9)).expect("small script"),
                delay: Duration::from_nanos(support::draw(&mut random, 2) * support::draw(&mut random, 101)),
            });
            when = when.saturating_add(Duration::from_nanos(support::draw(&mut random, 51)));
            schedule.push((when, scripted::input(id)));
        }
        let budget = u32::try_from(1 + support::draw(&mut random, 7)).expect("small budget");
        let mut slices = support::slices(seed, &plans, budget, true);
        let mut browser = Time::ZERO;
        let mut sent = Vec::new();
        let mut said = Said::default();
        let mut pending = schedule.into_iter().peekable();
        let mut closing = false;
        let mut rounds = 0;
        while !slices.world().settled() {
            slices.pace(support::speed(&mut random), browser);
            browser = browser.saturating_add(Duration::from_nanos(1 + support::draw(&mut random, 200)));
            while pending.peek().is_some_and(|(when, _)| *when <= browser) {
                let (_, input) = pending.next().expect("seeded input peeked");
                slices.input(input.clone());
                sent.push(input);
            }
            if pending.peek().is_none() && !closing {
                slices.input(support::close());
                sent.push(support::close());
                closing = true;
            }
            let before = slices.world().steps();
            let mut posts = Vec::new();
            slices.slice(browser, &mut posts);
            assert!(slices.world().steps() - before <= u64::from(budget), "seed {seed}: the slice's step budget");
            said.hear(posts, seed);
            rounds += 1;
            assert!(rounds < 10_000, "seed {seed}: finite scripts finish");
        }
        assert_eq!(said.stamps.len(), sent.len(), "seed {seed}: each input was stamped");
        let inputs: Vec<_> = said.stamps.iter().copied().zip(sent).collect();
        let (whole, whole_said) = support::whole(seed, &plans, true, &inputs);
        assert!(!whole.trace.is_empty(), "seed {seed}: the whole trace is kept");
        assert_eq!(support::trace(&mut slices), whole.trace, "seed {seed}: every trace entry");
        assert_eq!(slices.world().steps(), whole.steps, "seed {seed}: pacing adds no steps");
        assert_eq!(slices.world().now(), whole.end, "seed {seed}: the ending instant");
        assert_eq!(said.reports.len(), usize::try_from(INPUTS).expect("small script"), "seed {seed}: every answer");
        assert_eq!(said.reports, whole_said.reports, "seed {seed}: reports in order");
        assert_eq!(said.lines, whole_said.lines, "seed {seed}: lines in order");
        assert_eq!(said.stops, 1, "seed {seed}: one stop");
    }
}

#[test]
fn a_paused_world_answers_an_input_at_its_paused_instant() {
    const PAUSED: Time = Time::from_nanos(37);
    for seed in SEEDS {
        let mut slices = support::slices(seed, &[Plan { steps: ANSWER_STEPS, delay: Duration::ZERO }], BUDGET, false);
        slices.world_mut().advance(PAUSED);
        slices.pace(Speed::PAUSED, Time::ZERO);
        slices.input(scripted::input(1));
        let mut posts = Vec::new();
        assert_eq!(slices.slice(Time::from_nanos(100), &mut posts), Wake::Message, "seed {seed}");
        assert_eq!(
            posts,
            vec![
                ToPage::Stamp(Stamp { step: 0, instant: PAUSED }),
                support::answer(1),
                ToPage::Clock(Clock {
                    step: u64::from(ANSWER_STEPS),
                    now: PAUSED,
                    wall: slices.world().wall(),
                    idle: true,
                    next_due: None,
                    memory: 0,
                    slice: Duration::ZERO,
                }),
            ],
            "seed {seed}: input answered at the paused instant"
        );
    }
}

#[test]
fn an_input_entered_at_a_stamp_replays_at_it() {
    const BETWEEN: Time = Time::from_nanos(23);
    let plans =
        [Plan { steps: 1, delay: Duration::from_nanos(40) }, Plan { steps: ANSWER_STEPS, delay: Duration::ZERO }];
    for seed in SEEDS {
        let mut slices = support::slices(seed, &plans, BUDGET, true);
        let mut said = Said::default();
        let mut posts = Vec::new();
        slices.input(scripted::input(1));
        slices.slice(Time::ZERO, &mut posts);
        said.hear(posts, seed);
        let mut posts = Vec::new();
        slices.slice(BETWEEN, &mut posts);
        said.hear(posts, seed);
        slices.input(scripted::input(2));
        slices.input(support::close());
        let mut posts = Vec::new();
        slices.slice(BETWEEN, &mut posts);
        said.hear(posts, seed);
        assert_eq!(
            said.stamps.get(1),
            Some(&Stamp { step: 1, instant: BETWEEN }),
            "seed {seed}: input between due instants"
        );
        let mut browser = BETWEEN;
        while !slices.world().settled() {
            browser = browser.saturating_add(Duration::from_nanos(100));
            let mut posts = Vec::new();
            slices.slice(browser, &mut posts);
            said.hear(posts, seed);
        }
        let inputs: Vec<_> =
            said.stamps.iter().copied().zip([scripted::input(1), scripted::input(2), support::close()]).collect();
        let (whole, whole_said) = support::whole(seed, &plans, true, &inputs);
        assert_eq!(support::trace(&mut slices), whole.trace, "seed {seed}: replay at the stamp");
        assert_eq!(said.reports, whole_said.reports, "seed {seed}");
        assert_eq!(said.lines, whole_said.lines, "seed {seed}");
    }
}

#[test]
fn a_paused_world_moves_no_clock() {
    const PAUSED: Time = Time::from_nanos(37);
    for seed in SEEDS {
        let mut slices = support::slices(seed, &[Plan { steps: 1, delay: Duration::from_nanos(100) }], BUDGET, false);
        slices.world_mut().advance(PAUSED);
        slices.pace(Speed::PAUSED, Time::ZERO);
        slices.input(scripted::input(1));
        let wall = slices.world().wall();
        for browser in [Time::ZERO, Time::from_nanos(100), Time::from_nanos(u64::MAX)] {
            let mut posts = Vec::new();
            assert_eq!(slices.slice(browser, &mut posts), Wake::Message, "seed {seed}");
            assert_eq!(slices.world().now(), PAUSED, "seed {seed}: no monotonic clock movement");
            assert_eq!(slices.world().wall(), wall, "seed {seed}: no wall clock movement");
            assert_eq!(slices.world().steps(), 1, "seed {seed}: no step at a future deadline");
        }
    }
}

#[test]
fn a_slice_steps_at_most_its_budget() {
    // A long job stays busy across several slices even with the largest budget here.
    const LONG_JOB: u32 = 100;
    for seed in SEEDS {
        for budget in [1, 2, 7] {
            let mut slices = support::slices(seed, &[Plan { steps: LONG_JOB, delay: Duration::ZERO }], budget, false);
            slices.input(scripted::input(1));
            slices.pace(Speed::PAUSED, Time::ZERO);
            for browser in [Time::ZERO, Time::from_nanos(100), Time::from_nanos(200)] {
                let before = slices.world().steps();
                let mut posts = Vec::new();
                assert_eq!(slices.slice(browser, &mut posts), Wake::Now, "seed {seed}: busy even paused");
                assert_eq!(slices.world().steps() - before, u64::from(budget), "seed {seed}: budget {budget}");
                assert_eq!(slices.world().now(), Time::ZERO, "seed {seed}");
            }
        }
    }
}

#[test]
fn an_idle_world_wakes_when_its_next_due_comes_at_its_speed() {
    const BROWSER: Time = Time::from_nanos(13);
    for seed in SEEDS {
        for (speed, wake) in [
            (Speed::ONE, Wake::At(Time::from_nanos(114))),
            (Speed(10_000), Wake::At(Time::from_nanos(24))),
            (Speed::PAUSED, Wake::Message),
        ] {
            let mut slices =
                support::slices(seed, &[Plan { steps: 1, delay: Duration::from_nanos(101) }], BUDGET, false);
            slices.input(scripted::input(1));
            slices.slice(Time::ZERO, &mut Vec::new());
            slices.pace(speed, BROWSER);
            let mut posts = Vec::new();
            assert_eq!(slices.slice(BROWSER, &mut posts), wake, "seed {seed}: speed {speed:?}, rounded up");
            assert_eq!(slices.world().now(), Time::ZERO, "seed {seed}: pacing alone moves no clock");
        }
    }
}

#[test]
fn an_idle_world_with_nothing_due_waits_for_a_message() {
    for seed in SEEDS {
        let mut slices = support::slices(seed, &[], BUDGET, false);
        let mut posts = Vec::new();
        assert_eq!(slices.slice(Time::from_nanos(100), &mut posts), Wake::Message, "seed {seed}");
        assert!(!slices.world().settled(), "seed {seed}: the visitor waits for input");
        assert_eq!(slices.world().now(), Time::from_nanos(100), "seed {seed}: an idle clock moves alone");
        assert_eq!(slices.world().steps(), 0, "seed {seed}: no gratuitous step");
    }
}

#[test]
fn the_posts_keep_their_order() {
    // Two one-step answers fit, with room left to discover settlement.
    const STEPS: u64 = 2;
    let plans = [Plan { steps: 1, delay: Duration::ZERO }; 2];
    for seed in SEEDS {
        let mut slices = support::slices(seed, &plans, BUDGET, true);
        for input in [scripted::input(1), scripted::input(2), support::close()] {
            slices.input(input);
        }
        // The loop appends to the caller's existing posts.
        let prefix = support::line(Stream::Output, b"before the slice");
        let mut posts = vec![prefix.clone()];
        assert_eq!(slices.slice(Time::ZERO, &mut posts), Wake::Message, "seed {seed}");
        let stamp = ToPage::Stamp(Stamp { step: 0, instant: Time::ZERO });
        assert_eq!(
            posts,
            vec![
                prefix,
                stamp.clone(),
                stamp.clone(),
                stamp,
                support::answer(1),
                support::answer(2),
                support::line(Stream::Output, b"one"),
                support::line(Stream::Output, b"two"),
                support::line(Stream::Error, b"error"),
                ToPage::Stopped,
                ToPage::Clock(Clock {
                    step: STEPS,
                    now: Time::ZERO,
                    wall: slices.world().wall(),
                    idle: true,
                    next_due: None,
                    memory: 0,
                    slice: Duration::ZERO,
                }),
            ],
            "seed {seed}: stamps, reports, lines, stop, clock"
        );
    }
}

#[test]
fn a_settled_world_says_it_stopped_once() {
    for seed in SEEDS {
        let mut slices = support::slices(seed, &[], BUDGET, false);
        slices.input(support::close());
        let mut said = Said::default();
        let mut posts = Vec::new();
        assert_eq!(slices.slice(Time::ZERO, &mut posts), Wake::Message, "seed {seed}");
        said.hear(posts, seed);
        assert_eq!(said.stops, 1, "seed {seed}: stopped at settlement");
        let steps = slices.world().steps();
        let now = slices.world().now();
        for browser in [Time::from_nanos(10), Time::from_nanos(1000), Time::from_nanos(u64::MAX)] {
            let mut posts = Vec::new();
            assert_eq!(slices.slice(browser, &mut posts), Wake::Message, "seed {seed}");
            assert_eq!(
                posts,
                vec![ToPage::Clock(Clock {
                    step: steps,
                    now,
                    wall: slices.world().wall(),
                    idle: true,
                    next_due: None,
                    memory: 0,
                    slice: Duration::ZERO,
                })],
                "seed {seed}: a settled slice posts its clock alone"
            );
            said.hear(posts, seed);
        }
        assert_eq!(said.stops, 1, "seed {seed}: no second stop");
    }
}

#[test]
fn elapsed_time_a_busy_slice_could_not_use_is_dropped() {
    for seed in SEEDS {
        let mut slices = support::slices(seed, &[Plan { steps: ANSWER_STEPS, delay: Duration::ZERO }], 1, false);
        slices.input(scripted::input(1));
        for browser in [Time::from_nanos(1000), Time::from_nanos(2000), Time::from_nanos(2000)] {
            slices.slice(browser, &mut Vec::new());
            assert_eq!(slices.world().now(), Time::ZERO, "seed {seed}: time dropped at the budget");
        }
        slices.slice(Time::from_nanos(2001), &mut Vec::new());
        assert_eq!(slices.world().now(), Time::from_nanos(1), "seed {seed}: no catch-up");
    }
}

#[test]
fn a_slice_visits_each_due_instant_before_its_target() {
    const JOBS: usize = 3;
    // Each delayed job takes a step to start waiting and a step to answer.
    const STEPS: u64 = 6;
    const BUDGET_WITH_ROOM: u32 = 10;
    for seed in SEEDS {
        let mut slices =
            support::slices(seed, &[Plan { steps: 1, delay: Duration::from_nanos(5) }; JOBS], BUDGET_WITH_ROOM, false);
        for id in 0..u32::try_from(JOBS).expect("small script") {
            slices.input(scripted::input(id));
        }
        let mut posts = Vec::new();
        assert_eq!(slices.slice(Time::from_nanos(20), &mut posts), Wake::Message, "seed {seed}");
        assert_eq!(
            slices.world().now(),
            Time::from_nanos(20),
            "seed {seed}: reaches the target after the due instants"
        );
        assert_eq!(slices.world().steps(), STEPS, "seed {seed}: only the due work steps");
        let mut said = Said::default();
        said.hear(posts, seed);
        let inputs: Vec<_> = said
            .stamps
            .iter()
            .copied()
            .zip((0..u32::try_from(JOBS).expect("small script")).map(scripted::input))
            .chain([(Stamp { step: STEPS, instant: Time::from_nanos(20) }, support::close())])
            .collect();
        let (whole, whole_said) =
            support::whole(seed, &[Plan { steps: 1, delay: Duration::from_nanos(5) }; JOBS], false, &inputs);
        slices.input(support::close());
        slices.slice(Time::from_nanos(20), &mut Vec::new());
        assert_eq!(support::trace(&mut slices), whole.trace, "seed {seed}: steps occur at each deadline");
        assert_eq!(said.reports, whole_said.reports, "seed {seed}");
    }
}

#[test]
fn a_slices_clock_scales_elapsed_time_and_saturates_at_the_far_future() {
    for seed in SEEDS {
        for (speed, start, browser, expected) in [
            (Speed::ONE, Time::ZERO, Time::from_nanos(123), Time::from_nanos(123)),
            (Speed(500), Time::ZERO, Time::from_nanos(u64::MAX), Time::from_nanos(9_223_372_036_854_775_807)),
            (Speed(u32::MAX), Time::ZERO, Time::from_nanos(u64::MAX), Time::from_nanos(u64::MAX)),
            (Speed::ONE, Time::from_nanos(u64::MAX - 2), Time::from_nanos(3), Time::from_nanos(u64::MAX)),
        ] {
            let mut slices = support::slices(seed, &[], BUDGET, false);
            slices.world_mut().advance(start);
            slices.pace(speed, Time::ZERO);
            slices.slice(browser, &mut Vec::new());
            assert_eq!(slices.world().now(), expected, "seed {seed}: speed {speed:?}");
            assert_eq!(slices.world().steps(), 0, "seed {seed}: scaling an idle clock adds no steps");
        }
    }
}

#[test]
fn a_wake_scales_the_due_span_before_saturating_its_browser_instant() {
    const BROWSER: Time = Time::from_nanos(17);
    for seed in SEEDS {
        for (speed, expected) in
            [(Speed(1), Time::from_nanos(u64::MAX)), (Speed(u32::MAX), Time::from_nanos(4_294_967_297_017))]
        {
            let mut slices =
                support::slices(seed, &[Plan { steps: 1, delay: Duration::from_nanos(u64::MAX) }], BUDGET, false);
            slices.input(scripted::input(1));
            slices.slice(Time::ZERO, &mut Vec::new());
            slices.pace(speed, BROWSER);
            assert_eq!(slices.slice(BROWSER, &mut Vec::new()), Wake::At(expected), "seed {seed}: speed {speed:?}");
        }
    }
}

#[test]
fn a_new_loop_counts_elapsed_time_from_its_browser_instant() {
    for seed in SEEDS {
        let (world, reach) = scripted::world(seed, &[], false);
        // Process is not Copy; passing Reach twice requires its unconditional Copy impl.
        let copied = reach;
        assert_eq!(reach.at, copied.at, "seed {seed}");
        let mut slices = Slices::new(world, reach, BUDGET, Time::from_nanos(100));
        slices.slice(Time::from_nanos(103), &mut Vec::new());
        assert_eq!(slices.world().now(), Time::from_nanos(3), "seed {seed}");
    }
}

#[test]
#[should_panic(expected = "the visitor is an admitted process")]
fn a_visitor_index_must_name_a_process() {
    let (world, mut reach) = scripted::world(SEEDS[0], &[], false);
    reach.at = world.processes().len();
    let _slices = Slices::new(world, reach, BUDGET, Time::ZERO);
}

#[test]
#[should_panic(expected = "a slice has at least one step")]
fn a_slice_must_have_a_nonzero_budget() {
    let _slices = support::slices(SEEDS[0], &[], 0, false);
}
