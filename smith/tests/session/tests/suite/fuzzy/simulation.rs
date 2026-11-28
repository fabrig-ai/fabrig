//! The agent's session child domain at random: many noisy worlds, each settled
//! with every session ended, every way a session ends reached among them.

use smith_session_world::said;

use smith_session_world::bytes;

use std::collections::{BTreeMap, BTreeSet};

use skein_fake_llm_domain::api::{Line, Source};
use skein_world::sweep::{Case, cases, gate};
use smith_domain_session::End;
use smith_domain_session::llm::Failure;
use smith_session_world::provider::SLOT_NAMES;
use smith_session_world::shapes::{Answer, Class, Reference, Sample, answer};
use smith_session_world::translate::Decodings;
use smith_session_world::{Ended, World, noisy, submit_noisily};

const ITERATIONS: u32 = 100_000;
// On the original 1,000 seeds, the old menu escaped twice after a tools
// refusal and twice after a translator refusal.
const ORIGINAL_ESCAPED: u32 = 4;

const REFUSALS: [Answer; 21] = [
    Answer::Outside,
    Answer::TooLong,
    Answer::Protected,
    Answer::NulByte,
    Answer::Malformed,
    Answer::NotFound,
    Answer::NotFile,
    Answer::NotDirectory,
    Answer::NotEmpty,
    Answer::EditNoMatch,
    Answer::EditAmbiguous,
    Answer::EditUnchanged,
    Answer::ExitNonzero,
    Answer::UnknownTool,
    Answer::UnknownJob,
    Answer::NotRead,
    Answer::Stale,
    Answer::ReadOnly,
    Answer::TooLarge,
    Answer::TimedOut,
    Answer::JobsFull,
];

const WORKSPACE_CLASSES: [(Reference, Class); 5] = [
    (Reference::Path, Class::Live),
    (Reference::Path, Class::Foreign),
    (Reference::Path, Class::Unissued),
    (Reference::Offset, Class::Live),
    (Reference::Offset, Class::Unissued),
];

const JOB_CLASSES: [Class; 7] =
    [Class::Live, Class::Held, Class::Consumed, Class::BeforeOpening, Class::Foreign, Class::Unissued, Class::Unknown];

/// Every original scheduled ending class remains mandatory across the full sweep.
const EXPECTED_ENDS: [&str; 11] =
    ["busy", "invalid", "closed", "failed", "timed out", "Turns", "Input", "Output", "CacheRead", "CacheWrite", "Time"];

/// A thousand worlds with random limits, faults, schedules and openers: each
/// settles, with every session ended once and nothing left alive or in flight
/// (checked by `World::run`), and between them they reach every way a session
/// can yield and end.
#[test]
#[expect(clippy::too_many_lines, reason = "the reach checks share one seed sweep")]
fn random_worlds_settle_with_every_session_ended() -> Result<(), Case<String>> {
    let mut sample = Sample::new("random_worlds_settle_with_every_session_ended");
    let mut decodings = Decodings::default();
    let mut ends = BTreeSet::new();
    let mut stops = BTreeSet::new();
    let (mut stale, mut invalid, mut not_run, mut parallel, mut op_timeouts) = (0, 0, 0, 0, 0);
    let mut most_runs = 0;
    let mut concrete_turns = 0;
    let mut failures = BTreeSet::new();
    let mut races = [0; 4];
    let mut served = [0; 6];
    let (mut refused_write_read, mut unknown_jobs, mut full_jobs) = (0, 0, 0);
    let mut job_draws = [0; 2];
    let mut slot_cases = [[0; 2]; SLOT_NAMES.len()];
    let mut held_peaks = [0; 5];
    let mut drawn_job_bounds = [0; 4];
    let mut job_calls: BTreeMap<_, BTreeMap<_, u32>> =
        JOB_CLASSES.into_iter().map(|class| (class.word(), BTreeMap::new())).collect();
    cases(skein_world::this_test!("random_worlds_settle_with_every_session_ended"), 0..1000, |seed| {
        let (world, bound) = random_case(seed, &mut decodings);
        drawn_job_bounds[bound - 1] += 1;
        count_slots(&world, &mut slot_cases, &mut job_draws);
        sample.ran(&world.reached())?;
        stale += world.stats().stale;
        parallel = parallel.max(world.stats().most_parallel);
        most_runs = most_runs.max(world.stats().most_runs);
        op_timeouts += world.stats().op_timeouts;
        invalid += world.told().invalid_calls;
        not_run += world.stats().not_run;
        let stats = world.stats();
        refused_write_read += stats.refused_write_read;
        unknown_jobs += stats.unknown_jobs;
        full_jobs += stats.full_jobs;
        held_peaks.iter_mut().zip(stats.held_peaks).for_each(|(total, peak)| *total += peak);
        let delegated = [
            stats.delegates,
            stats.withdraws,
            stats.answered_after_withdraw,
            stats.delegate_timeouts,
            stats.finishes_refused,
            stats.finishes_accepted,
        ];
        for (count, more) in served.iter_mut().zip(delegated) {
            *count += more;
        }
        let won = [
            stats.answered_after_cancel,
            stats.failed_after_cancel,
            stats.done_after_cancel,
            stats.closed_while_closing,
        ];
        for (race, count) in races.iter_mut().zip(won) {
            *race += count;
        }
        for (_, session) in world.sessions() {
            concrete_turns += session.records.len();
            for call in &session.job_calls {
                *job_calls
                    .get_mut(call.class.word())
                    .expect("all job classes are counted")
                    .entry(answer(call.answer.as_ref()).word())
                    .or_default() += 1;
            }
            if let Some(Ended { end: End::Failed { failure, .. }, .. }) = session.ended {
                failures.insert(format!("{failure:?}"));
            }
            for (stop, _) in &session.yields {
                stops.insert(format!("{stop:?}"));
            }
            let kind = match &session.ended.as_ref().expect("every session ends").end {
                End::Busy => "busy".into(),
                End::Invalid => "invalid".into(),
                End::Closed => "closed".into(),
                End::Failed { failure: Failure::TimedOut { .. }, .. } => "timed out".into(),
                End::Failed { .. } | End::NoSummary { .. } => "failed".into(),
                End::Budget { spent } => format!("{spent:?}"),
                End::TranscriptRefused { .. } | End::PriceOverflow | End::UsageOverflow => {
                    unreachable!("bounded scheduled peers neither restore nor overflow")
                }
            };
            ends.insert(kind);
        }
        Ok(())
    })?;
    sample.write();
    println!("used slot cases (conversation, unissued): {slot_cases:?}");
    println!(
        "canonical scheduled hits: concrete_turns={concrete_turns}, stale={stale}, invalid={invalid}, not_run={not_run}, parallel={parallel}, most_runs={most_runs}, op_timeouts={op_timeouts}, delegated={served:?}, races={races:?}, ends={ends:?}, stops={stops:?}, refused_write_read={refused_write_read}, unknown_jobs={unknown_jobs}, full_jobs={full_jobs}, held_peaks={held_peaks:?}, drawn_job_bounds={drawn_job_bounds:?}"
    );
    for (class, answers) in &job_calls {
        println!("job {class}: {answers:?}");
    }
    let [shown_draws, unissued_draws] = job_draws;
    println!("job calls with slot draws: conversation={shown_draws}, unissued={unissued_draws}; failures={failures:?}");
    if !gate() {
        return Ok(());
    }
    for (slot, [shown, unissued]) in SLOT_NAMES.iter().zip(slot_cases) {
        assert!(shown > 0 && unissued > 0, "slot {slot:?}: {shown} shown, {unissued} unissued cases");
    }
    for answer in REFUSALS {
        assert!(sample.answered(answer) > 0, "the unchanged gate seeds reach {}", answer.word());
    }
    for (reference, class) in WORKSPACE_CLASSES {
        assert!(sample.classed(reference, class) > 0, "the unchanged gate seeds reach {reference:?}={class:?}");
    }
    assert_eq!(ends, EXPECTED_ENDS.into_iter().map(String::from).collect());
    assert!(concrete_turns > 0, "accepted provider completions emitted concrete checked Turns");
    assert!(shown_draws > 0, "some job calls drew a number from the conversation");
    assert!(unissued_draws > 0, "some job calls drew the never-issued number");
    for class in [Class::Live, Class::Consumed, Class::Unissued] {
        assert!(
            !job_calls[class.word()].is_empty() && sample.classed(Reference::Job, class) > 0,
            "the gate's seeds reach {class:?} job calls"
        );
    }
    assert_eq!(stops, ["Done", "Malformed", "Refused", "Truncated"].into_iter().map(String::from).collect());
    assert!(stale > 0, "some continues and closes reached sessions that had ended");
    assert!(parallel > 1, "some reads ran side by side");
    assert!(most_runs > parallel, "an iteration held more runs, ended ones among them, than a batch");
    assert!(
        served.iter().all(|&count| count > 0),
        "the opener served calls, some withdrawn, some answered all the same, some late, finishes refused and accepted: {served:?}"
    );
    assert!(
        races.iter().all(|&count| count > 0),
        "calls, failures and runs won races with cancels, and closes came twice: {races:?}"
    );
    for failure in ["Overloaded", "Unavailable", "Limit { which: Window, bound: 65536 }", "Unauthorized"] {
        assert!(failures.contains(failure), "some session failed as {failure}: {failures:?}");
    }
    assert!(op_timeouts > 0, "some operations of the tools ran out of time");
    assert!(
        refused_write_read > ORIGINAL_ESCAPED,
        "the new weights must improve on {ORIGINAL_ESCAPED} escapes: {refused_write_read}"
    );
    assert!(
        unknown_jobs > 0 && sample.answered(Answer::UnknownJob) > 0,
        "some job calls named a job the session did not hold"
    );
    assert!(
        full_jobs > 0 && sample.answered(Answer::JobsFull) > 0,
        "some yielding commands were refused because all the job cells were occupied"
    );
    assert!(
        invalid > 0 && not_run > 0,
        "some calls were malformed, and some were cut short and not run: {invalid} {not_run}"
    );
    Ok(())
}

pub(super) fn random_case(seed: u64, decodings: &mut Decodings) -> (World, usize) {
    let mut settings = noisy(seed);
    settings.provider.capture_answers = 16;
    settings.provider.capture_bytes = 32_768;
    let bound = usize::try_from(settings.agent.tools.jobs).expect("the drawn bound is at most four");
    let mut world = World::new(settings);
    world.omit_trace();
    submit_noisily(&mut world, &settings, seed);
    world.run_with_decodings(ITERATIONS, decodings);
    (world, bound)
}

fn count_slots(world: &World, total: &mut [[u32; 2]; SLOT_NAMES.len()], job_draws: &mut [u32; 2]) {
    let mut case = [[false; 2]; SLOT_NAMES.len()];
    for answer in world.capture().answers {
        for draw in answer.slots {
            let slot = usize::try_from(draw.slot).expect("bounded slot");
            let source = usize::from(draw.source == Source::Unissued);
            case[slot][source] = true;
            if SLOT_NAMES[slot] == b"job"
                && !draw.class.is_empty()
                && matches!(&answer.answer.lines[usize::try_from(draw.call).expect("captured position")],
                    Line::Call { name, .. } if name.as_ref() == b"job")
            {
                job_draws[source] += 1;
            }
        }
    }
    for (counts, seen) in total.iter_mut().zip(case) {
        for (count, reached) in counts.iter_mut().zip(seen) {
            *count += u32::from(reached);
        }
    }
}

#[test]
fn drawn_small_windows_compact_within_them() {
    use smith_domain_session::{Event, llm, record};
    use smith_session_world::recorded::{Scripted, opening};
    let mut rng = skein_lib::Rng::new(4712);
    let mut sample = Sample::new("drawn_small_windows_compact_within_them");
    for seed in 0..100 {
        let mut world = Scripted::with_shapes(seed);
        world.omit_replay();
        world.env.limits.spend = 10_000;
        let mut start = opening(None, world.env.limits.spend);
        start.spec.window = u32::try_from(rng.between(2_000, 3_000)).expect("small window");
        start.spec.output = 512;
        let window = start.spec.window;
        world.open(start);
        let text =
            |bytes: &[u8]| Box::new([llm::Block::Text { text: bytes.into(), replay: None }]) as Box<[llm::Block]>;
        let usage =
            llm::Usage { input_tokens: Some(u64::from(window) * 4 / 5), output_tokens: Some(32), ..llm::Usage::ZERO };
        world.complete(text(b"answer"), llm::Stop::EndTurn, usage);
        world.step(Event::Continue { session: world.session.expect("opened"), content: bytes(b"continue") });
        world.complete(text(b"summary"), llm::Stop::EndTurn, llm::Usage { output_tokens: Some(8), ..usage });
        assert!(matches!(world.turns.last().expect("window turn").kind, record::Kind::Window(_)));
        let prompt = world.prompts.last().expect("new window prompt");
        assert_eq!(
            smith_session_world::check_prompt_bound(prompt.input_bytes().expect("bounded") - 7 + 8, window),
            Ok(()),
        );
        world.close();
        sample.ran(world.reached()).expect("the case holds only shapes in form");
    }
    sample.write();
}

#[test]
fn no_drawn_session_ends_because_its_history_grew() {
    use smith_domain_session::{Event, llm, record};
    use smith_session_world::recorded::{Scripted, opening, written_calls};
    let mut rng = skein_lib::Rng::new(4717);
    let mut sample = Sample::new("no_drawn_session_ends_because_its_history_grew");
    for seed in 0..100 {
        let mut world = Scripted::with_shapes(seed);
        world.omit_replay();
        world.env.limits.history_bytes = rng.between(24_576, 32_768);
        world.open(opening(None, world.env.limits.spend));
        world.complete(written_calls(16), llm::Stop::ToolUse, llm::Usage::ZERO);
        for _ in 0..16 {
            let owner = *world.delegated.last().expect("every call runs");
            let count = usize::try_from(rng.between(15_000, 16_384)).expect("bounded text");
            world.step(Event::Answered {
                owner,
                text: vec![b'r'; count].into_boxed_slice(),
                error: false,
                spent: 0,
                after: smith_domain_session::After::Complete,
            });
        }
        assert!(world.end.is_none());
        world.complete(said(b"summary"), llm::Stop::EndTurn, llm::Usage::ZERO);
        assert!(matches!(world.turns.last().expect("window turn").kind, record::Kind::Window(_)));
        world.close();
        sample.ran(world.reached()).expect("the case holds only shapes in form");
    }
    sample.write();
}

/// Every native observation survives with no session fact allowance.
#[test]
fn observations_are_lossless_without_a_session_fact_allowance_over_many_seeds() {
    let mut sample = Sample::new("observations_are_lossless_without_a_session_fact_allowance_over_many_seeds");
    let mut decodings = Decodings::default();
    for seed in 0..100 {
        let mut run = |fold_shapes| {
            let settings = noisy(seed);
            let mut world = World::new(settings);
            submit_noisily(&mut world, &settings, seed);
            if fold_shapes {
                world.run_with_decodings(ITERATIONS, &mut decodings);
                sample.ran(&world.reached()).expect("the case holds only shapes in form");
            } else {
                world.run(ITERATIONS);
            }
            (world.stats(), world.told(), world.trace().to_vec())
        };
        let normal = run(true);
        assert!(normal.1.ended > 0 && normal.0.calls > 0, "seed {seed}: calls settled");
        assert_eq!(normal, run(false), "seed {seed}: complete observations and request order survive");
    }
    sample.write();
}

#[test]
fn drawn_ratios_and_rendered_sizes_send_nothing_past_the_line() {
    use skein_lib::{Rng, Time};
    use smith_session_world::{Count, Settings, spec};
    const SEEDS: u64 = 1_000;
    const LINE_MIN: u64 = 512;
    const LINE_MAX: u64 = 2_048;
    const TASK_BYTES: u64 = 32;
    let mut ratios = BTreeSet::new();
    let (mut guarded, mut refused, mut expanded) = (0, 0, 0);
    let mut rng = Rng::new(4102);
    let mut sample = Sample::new("drawn_ratios_and_rendered_sizes_send_nothing_past_the_line");
    let mut decodings = Decodings::default();
    for seed in 0..SEEDS {
        let calm = Settings::calm(seed);
        let settings = Settings { nudges: Count { min: 0, max: 2 }, ..calm };
        let mut world = World::new(settings);
        let line = rng.between(LINE_MIN, LINE_MAX);
        let bytes = usize::try_from(rng.between(1, TASK_BYTES)).expect("small task");
        let opener = world.submit(Time::ZERO, smith_domain_session::Spec { line, ..spec(&vec![b'x'; bytes]) });
        world.run_with_decodings(ITERATIONS, &mut decodings);
        sample.ran(&world.reached()).expect("the case holds only shapes in form");
        for prompt in &world.session(opener).measures {
            guarded += u32::from(prompt.guard.is_some());
            refused += u32::from(!prompt.sent);
            if prompt.sent && prompt.ordinary {
                assert!(prompt.input <= line, "seed {seed}: {prompt:?}");
            }
            for block in &prompt.blocks {
                ratios.insert(block.ratio);
                assert!((block.counted..=block.counted * 6).contains(&block.rendered), "seed {seed}: {block:?}");
                expanded += u32::from(block.rendered > block.counted);
            }
        }
    }
    sample.write();
    assert_eq!(ratios, (1..=u64::from(skein_llm::TOKEN_BYTES)).collect());
    assert!(guarded > 0 && refused > 0 && expanded > 0, "the sweep exercised drawn additions and unsent guards");
}
