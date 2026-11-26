//! Provider-to-machine job stories (smith/docs/design/domain/session.md, 10).

use skein_fake_llm_domain::api::{Line, Script, Turn};
use skein_lib::{Duration, Time};
use smith_domain_session::llm::{Block, NotRun, Problem, Returned};
use smith_domain_tools::{Exit, Grants, JobEnd, JobNumber, Outcome};
use smith_session_world::fixture::{calls, says};
use smith_session_world::recorded::{Scripted, opening};
use smith_session_world::{Settings, Span, World, bytes, spec, tickets::Tickets, translate};

const SEED: u64 = 1;
const ITERATIONS: u32 = 10_000;
const PROVIDER_LATENCY: Duration = Duration::from_millis(20);
const TOOL_MS: u64 = 500;
const TURN_TOKENS: u64 = 48;
const FIRST_JOB: JobNumber = JobNumber(1);
const NEVER_STARTED_JOB: u32 = 7;
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

fn polling() -> Turn {
    calls(Box::new([call(b"job", br#"{"job":1}"#)]), TURN_TOKENS)
}

fn run_story(mut turns: Vec<Turn>) -> (World, u64) {
    let mut settings = Settings::calm(SEED);
    (settings.provider.latency_min, settings.provider.latency_max) = (PROVIDER_LATENCY, PROVIDER_LATENCY);
    settings.tool = Span::millis(TOOL_MS, TOOL_MS);
    turns.push(says(Box::new([Line::Text { text: bytes(b"done") }]), TURN_TOKENS));
    let script = Script { cue: bytes(b"You are a coding agent."), turns: turns.into() };
    let mut world = World::with_scripts(settings, Box::new([script]));
    let opener = world.submit(Time::ZERO, spec(b"exercise jobs"));
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
    let (world, opener) = run_story(vec![calls(Box::new([launching()]), TURN_TOKENS), polling()]);
    assert!(matches!(
        results(&world, opener).as_slice(),
        [Returned::Owned { outcome: Outcome::Running { job: FIRST_JOB, .. } }, Returned::Owned {
            outcome: Outcome::Job { job: FIRST_JOB, end: JobEnd::Ended(Exit::Code { code: 0 }), head, tail, dropped: 0, .. }
        }] if head.as_ref() == JOB_OUTPUT && tail.is_empty()
    ));
    assert_eq!(world.told().jobs_ended, 0, "the read reported the only job end");
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
    let (world, opener) = run_story(vec![calls(Box::new([launching()]), TURN_TOKENS), change, polling()]);
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
