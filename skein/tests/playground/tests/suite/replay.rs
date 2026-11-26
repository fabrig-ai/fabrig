//! The session log replay's world tests (skein/docs/design/playground.md, 8).

mod site;

use skein_lib::{Duration, Rng, Time, Wall};
use skein_playground::{
    replay::{self, Differs, LAST_LINE_BYTES, LAST_LINES, Why},
    slices::Slices,
};
use skein_sim_world::{Turns, World};
use skein_ui_web::{
    bridge::{self, Input, Speed, ToPage},
    fetch, log,
};

const SEEDS: [u64; 5] = [1, 7, 19, 97, u64::MAX];
const BUILD: &[u8] = b"another-build";
// Room for all eight inputs and the longest end line.
const LOG_BOUND: u32 = 64 * 1024;

struct Recorded {
    log: log::Read,
    trace: Vec<skein_sim::Entry>,
    reports: Vec<fetch::Report>,
    printed: Vec<bridge::Printed>,
    steps: u64,
    now: Time,
}

fn draw(random: &mut Rng, bound: u64) -> u64 {
    random.next_u64() % bound
}

fn record(seed: u64, build: Option<&[u8]>) -> Recorded {
    record_bounded(seed, build, LOG_BOUND, |_, _| {})
}

fn record_bounded(
    seed: u64,
    build: Option<&[u8]>,
    bound: u32,
    mut on_post: impl FnMut(&log::Log, &ToPage),
) -> Recorded {
    let start = bridge::Start { story: Box::from([]), seed, start: Wall::from_nanos(seed) };
    let (world, reach) = site::world(seed, start.start);
    let mut random = Rng::new(seed);
    let budget = u32::try_from(1 + draw(&mut random, 9)).expect("small budget");
    let mut slices = Slices::new(world, reach, budget, Time::ZERO);
    let mut log = log::Log::new(build, &start, bound);
    let mut schedule = Vec::new();
    let mut at = Time::ZERO;
    for id in 0..4 {
        at = at.saturating_add(Duration::from_nanos(draw(&mut random, 401)));
        schedule.push((
            at,
            Input::Exchange(skein_ui_web::bridge::Exchange {
                id,
                path: b"/".as_slice().into(),
                method: "GET".into(),
                headers: Vec::new(),
                body: Box::from([]),
                bound: 64,
                cache: skein_ui_web::fetch::Cache::Browser,
                keep: Vec::new(),
            }),
        ));
        // Two stamps in one slice, with no digest accumulated between them.
        schedule.push((at, Input::ReadOn));
    }
    let mut schedule = schedule.into_iter().peekable();
    let mut browser = Time::ZERO;
    let mut reports = Vec::new();
    let mut printed = Vec::new();
    let mut rounds = 0;
    loop {
        let pace = match draw(&mut random, 4) {
            0 => Speed::PAUSED,
            1 => Speed(333),
            2 => Speed::ONE,
            _ => Speed(10_000),
        };
        slices.pace(pace, browser);
        browser = browser.saturating_add(Duration::from_nanos(1 + draw(&mut random, 200)));
        while schedule.peek().is_some_and(|(at, _)| *at <= browser) {
            let (_, input) = schedule.next().expect("seeded input peeked");
            log.sent(&input);
            slices.input(input);
        }
        let mut posts = Vec::new();
        slices.slice(browser, &mut posts);
        for post in &posts {
            log.heard(post);
            on_post(&log, post);
            match post {
                ToPage::Report(report) => reports.push(report.clone()),
                ToPage::Printed(line) => printed.push(line.clone()),
                ToPage::Stamp(_) | ToPage::Clock(_) | ToPage::Stopped => {}
                ToPage::Failed { words } => unreachable!("seed {seed}: {words:?}"),
            }
        }
        if schedule.peek().is_none() && reports.len() == 4 && !slices.world().busy() {
            break;
        }
        rounds += 1;
        assert!(rounds < 10_000, "seed {seed}: the site answers all exchanges");
    }
    assert!(site::listening(slices.world()), "seed {seed}: the site keeps listening");
    assert_eq!(printed.len(), 3, "seed {seed}: both printed streams");
    assert!(
        reports.iter().any(|report| matches!(report, fetch::Report::Answered { .. })),
        "seed {seed}: HTTP answered"
    );
    let read = log::read(&log.text()).unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
    if read.end != log::End::Cut {
        assert_eq!(read.entries.len(), 8, "seed {seed}: every input kept");
    }
    Recorded {
        log: read,
        reports,
        printed,
        steps: slices.world().steps(),
        now: slices.world().now(),
        trace: slices.world_mut().parts().0.trace().to_vec(),
    }
}

fn run(log: &log::Read) -> Result<World<site::Process>, Box<Differs>> {
    let (world, reach) = site::world(log.seed, log.start);
    replay::replay(world, reach, log, &mut ()).map_err(Box::new)
}

fn take_large_answer(process: &mut skein_playground_world::scripted::Process, reports: &mut Vec<fetch::Report>) {
    // One 64 KiB body, well past the diagnostic's 200-byte prefix.
    const BODY_BYTES: usize = 64 * 1024;
    skein_playground_world::scripted::take(process, reports);
    for report in reports {
        if let fetch::Report::Answered { body, .. } = report {
            *body = vec![b'a'; BODY_BYTES].into_boxed_slice();
        }
    }
}

#[test]
fn a_large_report_digests_whole_but_diagnoses_its_prefix() {
    use skein_playground_world::scripted::{self, Plan};
    const EXCHANGE: u32 = 4;
    // SHA-256 of the two empty SHA-256 digests and the zero stopped byte.
    const EMPTY: log::Said = log::Said([
        0x82, 0xc0, 0xf0, 0xf2, 0x59, 0xe8, 0xcf, 0xfe, 0xce, 0xad, 0x54, 0x32, 0x5f, 0xad, 0xb4, 0x8f, 0x15, 0xa4,
        0xe7, 0x61, 0xda, 0xe5, 0xff, 0xaf, 0x31, 0x20, 0x99, 0x93, 0xea, 0xcb, 0xb2, 0x4d,
    ]);
    // Same fold, with the complete canonical answer (including its newline) in the reports digest.
    const ANSWER: log::Said = log::Said([
        0xb2, 0x4f, 0x56, 0x38, 0x1d, 0x5e, 0x96, 0xf0, 0xb0, 0x4e, 0x0b, 0x34, 0xe5, 0x22, 0x13, 0x21, 0xbb, 0xac,
        0x30, 0xf3, 0xf5, 0xf2, 0xa3, 0xd2, 0xb6, 0x49, 0x40, 0xfa, 0x5d, 0x6e, 0xbc, 0x2a,
    ]);
    let seed = 19;
    let (world, reach) = scripted::world(seed, &[Plan { steps: 1, delay: Duration::ZERO }], false);
    let input = scripted::input(EXCHANGE);
    let read = log::Read {
        build: None,
        seed,
        start: world.wall(),
        entries: vec![log::Entry { stamp: bridge::Stamp { step: 0, instant: Time::ZERO }, input, said: EMPTY }],
        end: log::End::At { step: 1, instant: Time::ZERO, said: EMPTY },
    };
    let error =
        replay::replay(world, skein_playground::slices::Reach { take: take_large_answer, ..reach }, &read, &mut ())
            .expect_err("the end claims the report never arrived");
    let Why::Said { expected, found, last } = error.why else { panic!("expected a digest mismatch") };
    assert_eq!((error.line, error.build, error.input, expected, found), (3, None, None, EMPTY, ANSWER));
    // After the 30-byte prefix, 170 base64 bytes fit: 42 triples of 'a', then 'YW'.
    let answer_prefix = format!("answered id=4 status=200 body={}YW...", "YWFh".repeat(42));
    assert!(last.last() == Some(&answer_prefix), "the diagnostic keeps exactly the 200-byte prefix and ...");
}

#[test]
fn a_log_recorded_in_slices_replays() {
    for seed in SEEDS {
        let recorded = record(seed, None);
        let mut world = run(&recorded.log).unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        assert_eq!(world.steps(), recorded.steps, "seed {seed}: no extra steps");
        assert_eq!(world.now(), recorded.now, "seed {seed}: the final instant");
        assert_eq!(world.parts().0.trace(), recorded.trace, "seed {seed}: the whole trace through the end");
        assert!(!world.settled(), "seed {seed}: a snapshot leaves the site running");
    }
}

#[test]
fn a_snapshot_after_each_message_replays() {
    let empty = log::Saying::new().said();
    for seed in SEEDS {
        let mut saw_stamp_after_output = false;
        record_bounded(seed, None, LOG_BOUND, |log, post| {
            let snapshot =
                log::read(&log.text()).unwrap_or_else(|error| panic!("seed {seed} after {post:?}: {error:?}"));
            run(&snapshot).unwrap_or_else(|error| panic!("seed {seed} after {post:?}: {error:?}"));
            if let ToPage::Stamp(stamp) = post {
                let entry = snapshot.entries.last().expect("the stamp keeps an input");
                assert_eq!(
                    snapshot.end,
                    log::End::At { step: stamp.step, instant: stamp.instant, said: empty },
                    "seed {seed}: the snapshot ends at the stamp with nothing said since it"
                );
                saw_stamp_after_output |= entry.said != empty;
            }
        });
        assert!(saw_stamp_after_output, "seed {seed}: a stamp followed a nonempty fold");
    }
}

fn differs(log: &log::Read) -> Differs {
    match run(log) {
        Ok(_) => panic!("seed {}: the changed log replayed", log.seed),
        Err(error) => *error,
    }
}

#[test]
fn a_replay_that_differs_fails_at_its_first_input() {
    for seed in SEEDS {
        let recorded = record(seed, None);
        for (index, entry) in recorded.log.entries.iter().enumerate() {
            let mut changed = recorded.log.clone();
            changed.entries.get_mut(index).expect("kept input").said = log::Said([0; 32]);
            let line = u32::try_from(index + 2).expect("short log");
            let Differs { line: found_line, build, input, why } = differs(&changed);
            assert_eq!((found_line, build, input), (line, None, Some(log::input_text(&entry.input))), "seed {seed}");
            let Why::Said { expected, found, last } = why else { panic!("seed {seed}: digest difference") };
            assert_eq!(expected, log::Said([0; 32]), "seed {seed}: log digest");
            assert_eq!(found, entry.said, "seed {seed}: replay digest");
            assert!(last.len() <= LAST_LINES, "seed {seed}: bounded history");
            assert_ne!(entry.said, log::Said([0; 32]), "seed {seed}: digest changed");
        }
        let mut changed = recorded.log;
        let entry = changed.entries.get_mut(1).expect("second input of the slice");
        let found = entry.stamp;
        entry.stamp.instant = if entry.stamp.instant == Time::ZERO { Time::from_nanos(1) } else { Time::ZERO };
        let expected = entry.stamp;
        let input = Some(log::input_text(&entry.input));
        assert_eq!(
            differs(&changed),
            Differs { line: 3, build: None, input, why: Why::Moment { expected, found } },
            "seed {seed}: second stamp at another instant"
        );
    }
}

#[test]
fn a_replay_of_another_build_goes_on() {
    for seed in SEEDS {
        let mut recorded = record(seed, Some(BUILD));
        assert_eq!(recorded.log.build.as_deref(), Some(BUILD), "seed {seed}");
        assert!(run(&recorded.log).is_ok(), "seed {seed}: the digest decides across builds");
        let entry = recorded.log.entries.last_mut().expect("recorded inputs");
        let said = entry.said;
        let input = Some(log::input_text(&entry.input));
        entry.said = log::Said([0; 32]);
        let Differs { line, build, input: found_input, why } = differs(&recorded.log);
        assert_eq!(
            (line, build, found_input),
            (9, Some(BUILD.into()), input),
            "seed {seed}: a difference names its build"
        );
        let Why::Said { expected, found, last } = why else { panic!("seed {seed}: digest difference") };
        assert_eq!(expected, log::Said([0; 32]), "seed {seed}: log digest");
        assert_eq!(found, said, "seed {seed}: replay digest");
        assert!(last.len() <= LAST_LINES, "seed {seed}: bounded history");
    }
}

#[test]
fn a_cut_log_replays_to_its_last_input() {
    for seed in SEEDS {
        let recorded = record(seed, None);
        let cut = record_bounded(seed, None, 512, |_, _| {}).log;
        assert_eq!(cut.end, log::End::Cut, "seed {seed}: the text bound cuts the log");
        assert!(!cut.entries.is_empty() && cut.entries.len() < recorded.log.entries.len(), "seed {seed}: kept prefix");
        assert!(recorded.log.entries.starts_with(&cut.entries), "seed {seed}: all kept digests agree");
        let last = cut.entries.last().expect("kept input").stamp;
        let mut world = run(&cut).unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        assert_eq!((world.steps(), world.now()), (last.step, last.instant), "seed {seed}: no step after last input");
        let prefix = world.parts().0.trace().to_vec();
        assert!(recorded.trace.starts_with(&prefix), "seed {seed}: trace ends at the cut");
        assert!(world.busy(), "seed {seed}: the final input is still queued");
    }
}

#[test]
fn another_version_is_refused() {
    for seed in SEEDS {
        let text = format!("skein-playground-log 1 build=- seed={seed} start=0\ncut\n");
        assert_eq!(
            log::read(text.as_bytes()),
            Err(log::Refused::Version { found: b"1".as_slice().into() }),
            "seed {seed}"
        );
    }
}

#[derive(Default)]
struct Counter {
    turns: u64,
    instants: Vec<Time>,
}
impl<P> Turns<P> for Counter {
    fn turned(&mut self, now: Time, _: &[P]) {
        self.turns += 1;
        self.instants.push(now);
    }
}

#[test]
fn the_referee_observes_each_turn() {
    for seed in SEEDS {
        let recorded = record(seed, None);
        let (world, reach) = site::world(seed, recorded.log.start);
        let mut counter = Counter::default();
        let world = replay::replay(world, reach, &recorded.log, &mut counter)
            .unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        assert!(world.steps() > 0, "seed {seed}: turns occurred");
        assert_eq!(counter.turns, 2 * world.steps(), "seed {seed}: both processes turn each step");
        assert!(
            counter.instants.windows(2).all(|pair| pair.first() <= pair.last()),
            "seed {seed}: turn instants never go back"
        );
    }
}

#[test]
fn the_last_line_checks_its_moment_and_digest() {
    for seed in SEEDS {
        let recorded = record(seed, None);
        let mut changed = recorded.log;
        let log::End::At { step, instant, said } = changed.end else { panic!("seed {seed}: complete log") };
        changed.end = log::End::At { step, instant, said: log::Said([0; 32]) };
        let Differs { line, build, input, why } = differs(&changed);
        assert_eq!((line, build, input), (10, None, None), "seed {seed}: end digest");
        let Why::Said { expected, found, last } = why else { panic!("seed {seed}: digest difference") };
        assert_eq!(expected, log::Said([0; 32]), "seed {seed}: log digest");
        assert_eq!(found, said, "seed {seed}: replay digest");
        assert!(last.len() <= LAST_LINES, "seed {seed}: bounded history");
        changed.end = log::End::At { step: step + 1, instant, said };
        let error = differs(&changed);
        assert_eq!((error.line, error.build, error.input), (10, None, None), "seed {seed}: last line's moment");
        assert!(matches!(error.why, Why::Moment { .. }), "seed {seed}: a step with no work due");
    }
}

#[test]
fn a_settled_worlds_stop_is_folded_once() {
    use skein_playground_world::scripted::{self, Plan};
    for seed in SEEDS {
        let plans = [Plan { steps: 3, delay: Duration::from_nanos(17) }];
        let (world, reach) = scripted::world(seed, &plans, true);
        let start = bridge::Start { story: Box::from([]), seed, start: world.wall() };
        let mut log = log::Log::new(None, &start, 4096);
        let mut slices = Slices::new(world, reach, 2, Time::ZERO);
        for input in [scripted::input(1), Input::Abandon { exchange: 0 }] {
            log.sent(&input);
            slices.input(input);
        }
        let mut browser = Time::ZERO;
        let mut stops = 0;
        for round in 0..20 {
            let mut posts = Vec::new();
            slices.slice(browser, &mut posts);
            for post in posts {
                if post == ToPage::Stopped {
                    stops += 1;
                }
                log.heard(&post);
            }
            browser = browser.saturating_add(Duration::from_nanos(10));
            if round == 19 {
                assert!(slices.world().settled(), "seed {seed}: delayed script finishes");
            }
        }
        assert_eq!(stops, 1, "seed {seed}: repeated settled slices say stopped once");
        let read = log::read(&log.text()).unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        let (world, reach) = scripted::world(seed, &plans, true);
        let mut world =
            replay::replay(world, reach, &read, &mut ()).unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        assert!(world.settled(), "seed {seed}: settlement replays");
        assert_eq!(
            world.parts().0.trace(),
            slices.world_mut().parts().0.trace(),
            "seed {seed}: delayed turns and prints"
        );
    }
}

#[test]
fn an_empty_cut_returns_without_a_turn() {
    for seed in SEEDS {
        let read =
            log::Read { build: None, seed, start: Wall::from_nanos(seed), entries: Vec::new(), end: log::End::Cut };
        let world = run(&read).unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        assert_eq!(world.steps(), 0, "seed {seed}: empty cut");
        assert_eq!(world.now(), Time::ZERO, "seed {seed}: empty cut");
        let mut read = read;
        read.end = log::End::At { step: 0, instant: Time::ZERO, said: log::Saying::new().said() };
        let world = run(&read).unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        assert_eq!(world.steps(), 0, "seed {seed}: snapshot before the first slice");
    }
}

#[test]
fn a_stamp_behind_the_world_names_the_first_bad_line() {
    for seed in SEEDS {
        let mut recorded = record(seed, Some(BUILD));
        let (index, entry) = recorded
            .log
            .entries
            .iter_mut()
            .enumerate()
            .find(|(index, entry)| index % 2 == 1 && entry.stamp.step > 0)
            .unwrap_or_else(|| panic!("seed {seed}: site setup took turns before an input"));
        let found = entry.stamp;
        entry.stamp.step -= 1;
        let expected = entry.stamp;
        let line = u32::try_from(index + 2).expect("short log");
        let input = Some(log::input_text(&entry.input));
        assert_eq!(
            differs(&recorded.log),
            Differs { line, build: Some(BUILD.into()), input, why: Why::Moment { expected, found } },
            "seed {seed}: step count already past the stamp"
        );
    }
}

#[test]
fn a_said_difference_shows_its_last_lines() {
    for seed in SEEDS {
        let mut recorded = record(seed, None);
        let log::End::At { step, instant, .. } = recorded.log.end else { panic!("seed {seed}: complete log") };
        recorded.log.end = log::End::At { step, instant, said: log::Said([0; 32]) };
        let Why::Said { last, .. } = differs(&recorded.log).why else { panic!("seed {seed}: digest difference") };
        assert_eq!(last.len(), LAST_LINES, "seed {seed}: history spans inputs");
        assert!(last.iter().all(|line| line.len() <= LAST_LINE_BYTES + 3), "seed {seed}: bounded line bytes");
        let entry = recorded.log.entries.last().expect("recorded input");
        let input = format!(
            "input step={} instant={} {}",
            entry.stamp.step,
            entry.stamp.instant.as_nanos(),
            log::input_text(&entry.input)
        );
        assert!(last.contains(&input), "seed {seed}: last input in the log's text");
        let heard: Vec<_> = recorded
            .reports
            .iter()
            .map(log::report_line)
            .chain(recorded.printed.iter().map(log::printed_line))
            .map(|line| line.trim_end_matches('\n').to_owned())
            .collect();
        for line in &last {
            assert!(
                line.starts_with("input step=") || heard.contains(line) || line == "stopped",
                "seed {seed}: line in the log's text: {line}"
            );
        }
    }
}
