//! Provider-to-machine job stories (smith/docs/design/domain/session.md, 10).

use skein_fake_llm_domain::api::{Line, Script, Turn};
use skein_lib::{Duration, Time};
use smith_domain_session::{
    llm::{Block, Endpoint, NotRun, Problem, Returned},
    record,
};
use smith_domain_tools::{BytesClamp, Exit, Grants, JobEnd, JobNumber, Outcome, TimeoutClamp};
use smith_session_world::fixture::{calls, says};
use smith_session_world::recorded::{Scripted, opening};
use smith_session_world::shapes::{Answer, Class};
use smith_session_world::{JobCall, Settings, Span, World, bytes, check_job_call, spec, tickets::Tickets, translate};

const SEED: u64 = 1;
const ITERATIONS: u32 = 10_000;
const PROVIDER_LATENCY: Duration = Duration::from_millis(20);
const TOOL_MS: u64 = 500;
const TURN_TOKENS: u64 = 48;
const FIRST_JOB: JobNumber = JobNumber(1);
const SECOND_JOB: u32 = 2;
const THIRD_JOB: u32 = 3;
const NEVER_STARTED_JOB: u32 = 7;
const SYSTEM_CUE: &[u8] = b"You are a coding agent.";
const OTHER_CUE: &[u8] = b"You are the other coding agent.";
const JOB_TASK: &[u8] = b"exercise jobs";
const FIRST_BLOCK: usize = 0;
const FIRST_CALL: u16 = 1;
const JOB_CELLS: usize = 4;
const JOB_OUTPUT: &[u8] = b"ls: done\n";
const README_CONTENT: &[u8] = b"README.md: hello, world\n";
const READ_README: &[u8] = br#"{"path":"README.md"}"#;
const WRITE_NOTES: &[u8] = br#"{"path":"notes.md","content":"notes.md: noted"}"#;

fn call(name: &[u8], arguments: &[u8]) -> Line {
    Line::Call { name: bytes(name), arguments: bytes(arguments) }
}

fn launching() -> Line {
    call(b"shell", br#"{"command":"ls","yield_seconds":0}"#)
}

fn polling(number: u32) -> Turn {
    let input = format!(r#"{{"job":{number}}}"#);
    calls(Box::new([call(b"job", input.as_bytes())]), TURN_TOKENS)
}

fn done() -> Turn {
    says(Box::new([Line::Text { text: bytes(b"done") }]), TURN_TOKENS)
}

fn script(cue: &[u8], turns: Vec<Turn>) -> Script {
    Script { cue: bytes(cue), turns: turns.into() }
}

fn story_world(scripts: Box<[Script]>) -> World {
    let mut settings = Settings::calm(SEED);
    (settings.provider.latency_min, settings.provider.latency_max) = (PROVIDER_LATENCY, PROVIDER_LATENCY);
    settings.tool = Span::millis(TOOL_MS, TOOL_MS);
    World::with_scripts(settings, scripts)
}

fn run_story(mut turns: Vec<Turn>) -> (World, u64) {
    turns.push(done());
    let mut world = story_world(Box::new([script(SYSTEM_CUE, turns)]));
    let opener = world.submit(Time::ZERO, spec(JOB_TASK));
    world.run(ITERATIONS);
    (world, opener)
}

fn results(world: &World, opener: u64) -> Vec<&Returned> {
    world
        .session(opener)
        .records
        .iter()
        .flat_map(|turn| &turn.messages)
        .flat_map(|message| &message.content)
        .filter_map(|block| match block {
            Block::ToolResult { result, .. } => Some(result),
            _ => None,
        })
        .collect()
}

#[test]
fn the_provider_sees_job_only_with_the_shell_grant() {
    let all: &[&[u8]] = &[b"read", b"list", b"search", b"write", b"edit", b"shell", b"job"];
    let shell_only: &[&[u8]] = &[b"shell", b"job"];
    let without_shell: &[&[u8]] = &[b"read", b"list", b"search", b"write", b"edit"];
    for (grants, names, shell_schema) in [
        (Grants { inspect: true, modify: true, shell: true }, all, true),
        (Grants { inspect: false, modify: false, shell: true }, shell_only, true),
        (Grants { inspect: true, modify: true, shell: false }, without_shell, false),
    ] {
        let mut scripted = Scripted::new(SEED);
        let mut opening = opening(None, scripted.env.limits.spend);
        opening.spec.authority.grants = grants;
        opening.spec.delegated = Box::default();
        scripted.open(opening);
        let query = translate::query(scripted.prompts.pop().expect("opening prompt"), &Tickets::default());
        assert_eq!(query.tools.iter().map(|tool| tool.name.as_ref()).collect::<Vec<_>>(), names);
        assert_eq!(
            query.tools.iter().any(|tool| {
                tool.name.as_ref() == b"shell"
                    && skein_lib::bytes::find(&tool.parameters, b"\"yield_seconds\"").is_some()
            }),
            shell_schema
        );
        scripted.close();
    }
}

#[test]
fn a_yielded_shell_is_read_to_its_exit() {
    let (world, opener) = run_story(vec![calls(Box::new([launching()]), TURN_TOKENS), polling(FIRST_JOB.0)]);
    assert!(matches!(
        results(&world, opener).as_slice(),
        [Returned::Owned { outcome: Outcome::Running { job: FIRST_JOB, .. } }, Returned::Owned {
            outcome: Outcome::Job { job: FIRST_JOB, end: JobEnd::Ended(Exit::Code { code: 0 }), head, tail, dropped: 0, .. }
        }] if head.as_ref() == JOB_OUTPUT && tail.is_empty()
    ));
    assert!(matches!(
        world.session(opener).job_calls.as_slice(),
        [JobCall {
            class: Class::Live,
            job: Some(FIRST_JOB),
            answer: Some(Returned::Owned {
                outcome: Outcome::Job { job: FIRST_JOB, end: JobEnd::Ended(Exit::Code { code: 0 }), .. }
            }),
            ..
        }]
    ));
    assert_eq!(world.told().jobs_ended, 0, "the read reported the only job end");
}

#[test]
fn an_unread_end_is_held_at_the_poll() {
    let launch_then_read = calls(Box::new([launching(), call(b"read", READ_README)]), TURN_TOKENS);
    let (world, opener) = run_story(vec![launch_then_read, polling(FIRST_JOB.0)]);
    let polls = &world.session(opener).job_calls;
    assert!(
        matches!(
            polls.as_slice(),
            [JobCall {
                class: Class::Held,
                job: Some(FIRST_JOB),
                answer: Some(Returned::Owned {
                    outcome: Outcome::Job { job: FIRST_JOB, end: JobEnd::Ended(Exit::Code { code: 0 }), .. }
                }),
                ..
            }]
        ),
        "{polls:?}"
    );
}

#[test]
fn another_poll_of_the_read_end_is_consumed() {
    let launch = calls(Box::new([launching()]), TURN_TOKENS);
    let (world, opener) = run_story(vec![launch, polling(FIRST_JOB.0), polling(FIRST_JOB.0)]);
    let polls = &world.session(opener).job_calls;
    assert!(
        matches!(polls.as_slice(), [_, JobCall {
        class: Class::Consumed, job: Some(FIRST_JOB), answer: Some(Returned::Invalid {
            problem: Problem::UnknownJob { job }
        }), ..
    }] if *job == FIRST_JOB.0),
        "{polls:?}"
    );
}

#[test]
fn a_poll_of_a_never_issued_number_is_unissued() {
    let (world, opener) = run_story(vec![polling(SECOND_JOB)]);
    let polls = &world.session(opener).job_calls;
    assert!(
        matches!(
            polls.as_slice(),
            [JobCall {
                class: Class::Unissued,
                job: Some(JobNumber(SECOND_JOB)),
                answer: Some(Returned::Invalid { problem: Problem::UnknownJob { job: SECOND_JOB } }),
                ..
            }]
        ),
        "{polls:?}"
    );
}

#[test]
fn a_job_number_issued_by_another_session_is_foreign() {
    let launch = calls(Box::new([launching()]), TURN_TOKENS);
    let mut world = story_world(Box::new([
        script(SYSTEM_CUE, vec![launch, done()]),
        script(OTHER_CUE, vec![polling(FIRST_JOB.0), done()]),
    ]));
    world.submit(Time::ZERO, spec(JOB_TASK));
    world.run(ITERATIONS);
    let other = world.submit(world.now(), smith_domain_session::Spec { system: bytes(OTHER_CUE), ..spec(JOB_TASK) });
    world.run(ITERATIONS);
    let polls = &world.session(other).job_calls;
    assert!(
        matches!(polls.as_slice(), [JobCall {
        class: Class::Foreign, job: Some(FIRST_JOB), answer: Some(Returned::Invalid {
            problem: Problem::UnknownJob { job }
        }), ..
    }] if *job == FIRST_JOB.0),
        "{polls:?}"
    );
}

#[test]
fn a_job_restored_from_the_transcript_is_before_opening() {
    // The prior kit issues jobs 1 through 3. The fake resumes its script after the two recorded answers.
    let launches = calls((0..THIRD_JOB).map(|_| launching()).collect(), TURN_TOKENS);
    let mut world = story_world(Box::new([script(SYSTEM_CUE, vec![launches, done(), polling(SECOND_JOB), done()])]));
    let old = world.submit(Time::ZERO, spec(JOB_TASK));
    world.run(ITERATIONS);
    let transcript = record::Transcript {
        version: record::VERSION,
        endpoint: Some(Endpoint(0)),
        dialect: 2,
        turns: world.session(old).records.clone().into(),
    };
    let resumed = world.submit_restored(world.now(), spec(JOB_TASK), transcript);
    world.run(ITERATIONS);
    let polls = &world.session(resumed).job_calls;
    assert!(
        matches!(
            polls.as_slice(),
            [JobCall {
                class: Class::BeforeOpening,
                job: Some(JobNumber(SECOND_JOB)),
                answer: Some(Returned::Invalid { problem: Problem::UnknownJob { job: SECOND_JOB } }),
                ..
            }]
        ),
        "{polls:?}"
    );
}

#[test]
fn a_translator_refusal_cannot_name_a_job() {
    let (world, opener) = run_story(vec![polling(0)]);
    assert_eq!(
        world.session(opener).job_calls,
        [JobCall {
            sequence: 1,
            position: FIRST_BLOCK,
            class: Class::Unknown,
            job: None,
            answer: Some(Returned::Invalid { problem: Problem::BadValue { field: bytes(b"job") } }),
        }]
    );
}

#[test]
fn the_referee_rejects_an_owned_answer_for_consumed_jobs() {
    let call = JobCall {
        sequence: 3,
        position: FIRST_BLOCK,
        class: Class::Consumed,
        job: Some(FIRST_JOB),
        answer: Some(Returned::Owned {
            outcome: Outcome::Job {
                job: FIRST_JOB,
                end: JobEnd::Running,
                head: Box::default(),
                tail: Box::default(),
                dropped: 0,
                output: BytesClamp::Within,
                wait: TimeoutClamp::Within,
            },
        }),
    };
    let finding = check_job_call(&call).expect_err("a consumed job can no longer answer");
    assert!(
        finding.starts_with("turn 3 block 0: Consumed job Some(JobNumber(1)) must be answered as unknown"),
        "{finding}"
    );
}

#[test]
fn held_commands_do_not_occupy_the_batch_for_other_tools() {
    let change = calls(
        Box::new([
            call(b"read", READ_README),
            call(b"read", br#"{"path":"src/lib.rs"}"#),
            call(b"write", br#"{"path":"README.md","content":"README.md: revised"}"#),
        ]),
        TURN_TOKENS,
    );
    let (world, opener) = run_story(vec![calls(Box::new([launching()]), TURN_TOKENS), change, polling(FIRST_JOB.0)]);
    let answers = results(&world, opener);
    assert!(
        matches!(
            answers.as_slice(),
            [
                Returned::Owned { outcome: Outcome::Running { job: FIRST_JOB, .. } },
                Returned::Owned { outcome: Outcome::Read { .. } },
                Returned::Owned { outcome: Outcome::Read { .. } },
                Returned::Owned { outcome: Outcome::Written { created: false } },
                Returned::Owned {
                    outcome: Outcome::Job { job: FIRST_JOB, end: JobEnd::Ended(Exit::Code { code: 0 }), .. }
                }
            ]
        ),
        "{answers:?}"
    );
    assert_eq!(world.stats().most_parallel, 2, "the two reads ran together, the write alone");
}

#[test]
fn an_unknown_job_suppresses_the_write_but_not_the_read() {
    let unknown = format!(r#"{{"job":{NEVER_STARTED_JOB}}}"#);
    let turn = calls(
        Box::new([call(b"job", unknown.as_bytes()), call(b"write", WRITE_NOTES), call(b"read", READ_README)]),
        TURN_TOKENS,
    );
    let (world, opener) = run_story(vec![turn]);
    assert!(matches!(
        results(&world, opener).as_slice(),
        [Returned::Invalid { problem: Problem::UnknownJob { job: NEVER_STARTED_JOB } },
        Returned::NotRun { why: NotRun::AfterFailedWrite { call: FIRST_CALL } },
        Returned::Owned { outcome: Outcome::Read { content, .. } }] if content.as_ref() == README_CONTENT
    ));
    let stats = world.stats();
    assert_eq!((stats.refused_write_read, stats.unknown_jobs, stats.ops), (1, 1, 1));
    let reached = world.reached();
    assert!(reached.contains(
        "job:unknown_job:job=unissued,write:not_run_after_failure,read:ok end=unknown begin=unknown wake=unknown"
    ));
    assert!(reached.answered(Answer::UnknownJob));
}

#[test]
fn a_fifth_yielding_shell_is_refused_and_close_drains_four() {
    let commands = (0..=JOB_CELLS).map(|_| launching()).collect();
    let (world, opener) = run_story(vec![calls(commands, TURN_TOKENS)]);
    let answers = results(&world, opener);
    let (refusal, held) = answers.split_last().expect("the yielding shells answered");
    let job_cells = u32::try_from(JOB_CELLS).expect("the fixture has four cells");
    assert_eq!(**refusal, Returned::Invalid { problem: Problem::JobsFull { jobs: job_cells } });
    let numbers: Vec<_> = held
        .iter()
        .filter_map(|answer| match answer {
            Returned::Owned { outcome: Outcome::Running { job, .. } } => Some(job.0),
            _ => None,
        })
        .collect();
    assert_eq!(held.len(), JOB_CELLS);
    assert!(numbers.into_iter().eq(1..=job_cells));
    assert_eq!(world.stats().full_jobs, 1);
    assert_eq!(world.stats().op_cancels, job_cells, "close cancelled every held command");
    assert_eq!(world.told().jobs_ended, job_cells, "close reported all four unread ends");
}

#[test]
fn drawn_job_slots_name_shown_and_unissued_jobs_and_replay() {
    use skein_fake_llm_domain::api::Source;
    use smith_session_world::provider::UNISSUED_JOB;

    // The model polls the never-issued number, then a job whose end is held.
    const DRAWN_POLLS_SEED: u64 = 1;
    const DRAWN_ROUNDS: u32 = 10;
    const CAPTURE_ANSWERS: u32 = 16;
    const CAPTURE_BYTES: u32 = 16_384;
    let mut settings = Settings::calm(DRAWN_POLLS_SEED);
    settings.provider.tool_rounds = DRAWN_ROUNDS;
    settings.provider.capture_answers = CAPTURE_ANSWERS;
    settings.provider.capture_bytes = CAPTURE_BYTES;
    settings.tool = Span::millis(TOOL_MS, TOOL_MS);
    let opening = || {
        let mut opening = spec(JOB_TASK);
        opening.authority.grants = Grants { inspect: false, modify: false, shell: true };
        opening
    };
    let mut world = World::new(settings);
    let opener = world.submit(Time::ZERO, opening());
    world.run(ITERATIONS);
    let session = world.session(opener);
    let capture = world.capture();
    assert!(!capture.full);
    let polls: Vec<_> = capture
        .answers
        .iter()
        .flat_map(|drawn| {
            drawn.slots.iter().filter(|slot| {
                matches!(&drawn.answer.lines[usize::try_from(slot.call).expect("call position")],
                    Line::Call { name, .. } if name.as_ref() == b"job")
            })
        })
        .collect();
    let named: Vec<_> = session.job_calls.iter().filter(|call| call.job.is_some()).collect();
    assert_eq!(polls.len(), named.len(), "each named job call had a slot draw");
    for (draw, mention) in polls.into_iter().zip(named) {
        let number = mention.job.expect("a named job");
        assert_eq!(&*draw.value, number.0.to_string().as_bytes());
        assert_eq!(&*draw.class, mention.class.word().as_bytes());
        assert_eq!(draw.source, if number.0 == UNISSUED_JOB { Source::Unissued } else { Source::Conversation },);
    }
    assert!(session.job_calls.iter().any(|call| {
        call.class == Class::Unissued
            && call.job == Some(JobNumber(UNISSUED_JOB))
            && matches!(call.answer, Some(Returned::Invalid { problem: Problem::UnknownJob { job: UNISSUED_JOB } }))
    }));
    let read = session
        .job_calls
        .iter()
        .find(|call| matches!(call.class, Class::Live | Class::Held))
        .expect("a draw named the session's own job");
    assert!(
        session.records.iter().take_while(|turn| turn.sequence < read.sequence).any(|turn| {
            turn.messages.iter().flat_map(|message| &message.content).any(|block| {
            matches!(block, Block::ToolResult { result: Returned::Owned { outcome: Outcome::Running { job, .. } }, .. }
                if Some(*job) == read.job)
        })
        }),
        "the model was shown that job before the poll"
    );

    // Strict replay cannot silently redraw any answer that was not captured.
    settings.provider.strict = true;
    let mut replay = World::with_replay(settings, capture.clone());
    let replay_opener = replay.submit(Time::ZERO, opening());
    replay.run(ITERATIONS);
    let played = replay.session(replay_opener);
    assert_eq!(
        (world.trace(), world.stats(), world.told(), world.now()),
        (replay.trace(), replay.stats(), replay.told(), replay.now())
    );
    assert_eq!(
        (&session.records, &session.job_calls, &session.yields, &session.ended),
        (&played.records, &played.job_calls, &played.yields, &played.ended)
    );
    assert_eq!(capture, replay.capture());
}
