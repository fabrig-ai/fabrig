//! A session's park, observed through the host that can wake it again
//! (smith/docs/design/parks.md, 2–3).
use skein_fake_checkout::{Checkout, Exit};
use skein_fake_llm_domain::api::{Line, Part, Script, Turn};
use skein_lib::{Duration, Time, Token};
use skein_world::{
    domain::Span,
    faults::{FaultLedger, Kind, Struck},
    streams::{Stream, Streams},
};
use smith_agent_world::{Boundary, HostSchedule, Job, Settings, World, fixture, scripted_charter};
use smith_domain::session::{
    self,
    llm::{Block, NotRun, Problem, Returned, ToolCall},
    record::{PartKind, Resumed},
};
use smith_domain::{Answered, AnsweredCall, Event, Fact, Transcript, run, tools};
use smith_faults::parks::{
    CUT_TURN, HOST_CALL_ANSWERED, HOST_CALL_UNANSWERED, JOB_RUNNING, KINDS, SUB_AGENT_KEPT, WITHHELD_WAIT,
};
use smith_session_world::shapes::Reached;
use smith_tools_world::fixture::{Fixture, program};

const ROUNDS: u32 = 2000;
const DRAIN: Time = Time::from_nanos(1_000_000_000);
const READ: &[u8] = br#"{"path":"src/lib.rs"}"#;
const SOURCE: &[u8] = b"pub fn answer() -> u32 { 43 }\n";
const WAKE: &[u8] = b"Please continue this parked run.";
const HOST_TOOL: &[u8] = b"host_action";
const HOST_ANSWER: &[u8] = b"opaque host answer: first decision";
const CHILD_CUE: &[u8] = b"@park-child";
const CHILD_REPLY: &[u8] = b"Read the saved source.";

#[derive(Clone, Copy)]
struct Draw {
    kind: Kind,
    resumes: bool,
    cut_answered: bool,
}

impl Draw {
    fn cut(self) -> bool {
        self.cut_answered || matches!(self.kind, CUT_TURN | HOST_CALL_UNANSWERED | SUB_AGENT_KEPT)
    }
}

fn call(name: &[u8], arguments: &[u8]) -> Line {
    Line::Call { name: name.into(), arguments: arguments.into() }
}

fn batch(calls: impl Into<Box<[Line]>>) -> Turn {
    fixture::calls(calls.into(), 1)
}

fn wait() -> Turn {
    batch([call(b"wait", b"{}")])
}

fn finish() -> Turn {
    batch([call(b"finish", br#"{"report":"Resumed.","source":"workspace"}"#)])
}

fn scripts(draw: Draw, charter: &run::Charter) -> Box<[Script]> {
    let host = || call(HOST_TOOL, b"{}");
    let shell = || call(b"shell", br#"{"command":"park command"}"#);
    let opening = match draw.kind {
        WITHHELD_WAIT => vec![wait()],
        JOB_RUNNING => vec![
            batch([call(b"shell", br#"{"command":"park command","yield_seconds":0}"#)]),
            wait(),
            batch([
                call(b"job", br#"{"job":1,"wait_seconds":5}"#),
                call(b"edit", br#"{"path":"src/lib.rs","edits":[{"old":"43","new":"44"}]}"#),
                call(b"read", READ),
            ]),
        ],
        CUT_TURN => vec![batch([call(b"read", READ), shell(), call(b"list", br#"{"path":"."}"#)])],
        HOST_CALL_ANSWERED if draw.cut_answered => vec![batch([host(), shell()])],
        HOST_CALL_ANSWERED => vec![batch([host()]), wait()],
        HOST_CALL_UNANSWERED => vec![batch([host()])],
        SUB_AGENT_KEPT => vec![batch([
            call(b"read", READ),
            call(b"sub_agent", br#"{"brief":"@park-child","tools":["inspect","shell"],"max_turns":3}"#),
        ])],
        _ => unreachable!("park kind from KINDS"),
    };
    let mut scripts = vec![Script {
        cue: charter.brief.sections[0].text.clone(),
        turns: opening.into_iter().chain([finish()]).collect(),
    }];
    if draw.kind == SUB_AGENT_KEPT {
        scripts.push(Script {
            cue: CHILD_CUE.into(),
            turns: Box::new([
                batch([call(b"shell", br#"{"command":"park command"}"#)]),
                fixture::says(Box::new([Line::Text { text: CHILD_REPLY.into() }]), 1),
            ]),
        });
    }
    scripts.into()
}

fn eligible(kind: Kind, charter: &run::Charter) -> bool {
    let grants = &charter.grants;
    match kind {
        WITHHELD_WAIT => grants.wait,
        JOB_RUNNING => grants.wait && grants.tools.shell && grants.tools.modify && grants.tools.inspect,
        CUT_TURN => grants.tools.inspect && grants.tools.shell,
        HOST_CALL_ANSWERED | HOST_CALL_UNANSWERED => {
            (kind != HOST_CALL_ANSWERED || grants.wait)
                && grants.host_tools.iter().any(|t| t.name.as_ref() == HOST_TOOL)
        }
        SUB_AGENT_KEPT => grants.agents && grants.tools.inspect && grants.tools.shell,
        _ => unreachable!("park kind from KINDS"),
    }
}

fn io(world: &World) -> impl Iterator<Item = &tools::Op> {
    world.boundaries().iter().filter_map(|(_, boundary)| match boundary {
        Boundary::Io { op } => Some(op),
        _ => None,
    })
}

fn blocks(turn: &smith_domain::Turn) -> impl Iterator<Item = &Block> {
    turn.messages.iter().flat_map(|message| &message.content)
}

fn calls(turn: &smith_domain::Turn) -> impl Iterator<Item = &ToolCall> {
    blocks(turn).filter_map(Block::tool_call)
}

fn result(block: &Block) -> Option<(&[u8], &Returned)> {
    match block {
        Block::ToolResult { id, result } => Some((id.as_ref(), result)),
        _ => None,
    }
}

fn results(turn: &smith_domain::Turn) -> impl Iterator<Item = (&[u8], &Returned)> {
    blocks(turn).filter_map(result)
}

fn outputs(query: &skein_fake_llm_domain::api::Query) -> impl Iterator<Item = (&[u8], &[u8], bool)> {
    query.messages.iter().flat_map(|message| &message.parts).filter_map(|part| match part {
        Part::ToolOutput { id, output, is_error } => Some((id.as_ref(), output.as_ref(), *is_error)),
        _ => None,
    })
}

pub(super) fn run(seed: u64, reached: &mut Reached) -> FaultLedger {
    let mut ledger = FaultLedger::new();
    let mut rng = Streams::new(seed).rng(Stream::Faults, "parks");
    // One choice followed by a fixed outcome slot for each kind, even if the charter cannot take it.
    for number in [1, 2] {
        let host_tools = number == 1 || !seed.is_multiple_of(2);
        let index = usize::try_from(rng.below(KINDS.len() as u64)).expect("park index");
        let outcomes: [bool; KINDS.len()] = core::array::from_fn(|_| rng.below(2) == 1);
        let kind = KINDS[index];
        let draw = Draw { kind, resumes: outcomes[index], cut_answered: number == 2 && kind == HOST_CALL_ANSWERED };
        let calm = Settings::calm(seed);
        let settings = Settings {
            job: Job::HostTools,
            provider: skein_fake_llm_domain::Config {
                latency_min: Duration::from_millis(100),
                latency_max: Duration::from_millis(100),
                ..calm.provider
            },
            waiting: Duration::from_secs(1),
            network: Span::millis(0, 0),
            host: if draw.kind == HOST_CALL_UNANSWERED { HostSchedule::Withdrawn } else { HostSchedule::Answer },
            host_tool_timeout: Duration::from_secs(10),
            ..calm
        };
        let charter = charter(&settings, host_tools);
        if !eligible(draw.kind, &charter) {
            ledger.ineligible(draw.kind);
            continue;
        }
        drive(draw, &settings, charter, number, &mut ledger, reached);
    }
    ledger
}

fn charter(settings: &Settings, host_tools: bool) -> run::Charter {
    let mut charter = scripted_charter(settings);
    if !host_tools {
        charter.grants.host_tools = Box::default();
    }
    charter
}

fn activation(
    draw: Draw,
    settings: &Settings,
    charter: run::Charter,
    number: u64,
    workspace: run::Workspace,
    disk: Checkout,
) -> World {
    let scripts = scripts(draw, &charter);
    let start = smith_domain::Start {
        host_run: Token::new(number),
        workspace: Some(workspace),
        ..fixture::start(charter, true)
    };
    World::with_workspace_scripts_start(*settings, disk, scripts, Event::Start(start))
}

fn drive(
    draw: Draw,
    settings: &Settings,
    first_charter: run::Charter,
    number: u64,
    ledger: &mut FaultLedger,
    reached: &mut Reached,
) {
    let request = format!("run {number}");
    let finding = &format!("{request}, {} {}", draw.kind.name, if draw.resumes { "resumes" } else { "ends" });
    eprintln!("seed {}: {finding}", settings.seed);
    let mut fixture = Fixture::new();
    fixture.checkout.write(b"work/temper/src/lib.rs", SOURCE);
    fixture.checkout.program(b"park command", program(60_000, b"still working", Exit::Code(0), &[]));
    let workspace = run::Workspace {
        directories: Box::new([run::Directory {
            root: fixture.repos[0].root,
            host: b"/work/temper".as_slice().into(),
            name: b"work".as_slice().into(),
            results: None,
            hooks: None,
            writable: true,
            git: true,
            conflicts: Box::default(),
        }]),
    };
    let kind = draw.kind;
    let host_tools = !first_charter.grants.host_tools.is_empty();
    let mut first = activation(draw, settings, first_charter, number, workspace.clone(), fixture.checkout);
    if draw.cut() {
        first.tell_parts();
        // Replies take 100 ms; the command and host relay outlast the grace.
        first.drain_with_grace_at(DRAIN, Some(Duration::from_millis(10)));
    }
    assert!(first.drive(ROUNDS), "{finding}: park did not settle");
    observe_terminal(&first, draw, finding);
    let at = observe(&first, draw, finding);
    let injection = ledger.inject(kind, at, Struck { request: Some(request), ..Struck::default() });
    ledger.stop(injection, at).expect("instant park observation");
    reached.turns(first.turns());
    if !draw.resumes {
        ends(&first, finding);
        return;
    }
    let parts: Box<[Resumed]> = if draw.cut() {
        // At this cut the host kept its decision, not a second copy as a result part.
        first
            .parts()
            .iter()
            .filter(|(_, _, _, part)| !draw.cut_answered || !host_result(&part.body))
            .map(|(_, _, turn, part)| Resumed { activation: 1, turn: *turn, part: part.clone() })
            .collect()
    } else {
        Box::default()
    };
    let opening = first.turns().first().unwrap_or_else(|| &parts[0].part.body);
    let history = Transcript {
        version: opening.version,
        endpoint: opening.endpoint,
        dialect: opening.dialect,
        turns: first.turns().into(),
    };
    let settings = Settings { resume: true, host: HostSchedule::Answer, ..*settings };
    let charter = charter(&settings, host_tools);
    let mut next = activation(draw, &settings, charter, number, workspace, first.take_checkout());
    next.resume_history(history);
    next.resume_parts(parts, 2);
    next.waking_messages(Box::new([run::Message {
        finish_requested: false,
        name: Token::new(91),
        label: b"host".as_slice().into(),
        text: WAKE.into(),
    }]));
    if kind == HOST_CALL_ANSWERED {
        next.resume_answers(Box::new([answered(&first, finding)]));
    }
    assert!(next.drive(ROUNDS), "{finding}: resumed activation did not settle");
    resumed(&first, &next, draw, finding);
    reached.turns(next.turns());
}

fn answered(world: &World, finding: &str) -> AnsweredCall {
    let [submission] = world.host_submissions() else { panic!("{finding}: expected one host submission") };
    let [(relay, _, run::HostReply::Answered(answer))] = world.host_terminals() else {
        panic!("{finding}: expected one recorded host answer");
    };
    assert_eq!(*relay, submission.relay, "{finding}: host decision's relay");
    AnsweredCall { name: submission.name, tool: submission.tool.clone(), answer: Answered::Host(answer.clone()) }
}

fn host_result(turn: &smith_domain::Turn) -> bool {
    results(turn).any(|(_, r)| {
        matches!(r, Returned::Text { text, error: false, .. }
        if text.ends_with(HOST_ANSWER))
    })
}

fn observe_terminal(world: &World, draw: Draw, finding: &str) {
    if draw.cut() {
        let armed = if draw.kind == HOST_CALL_UNANSWERED {
            world.host_submissions().first().is_some_and(|call| call.at < DRAIN && DRAIN < call.deadline)
        } else {
            world
                .boundaries()
                .iter()
                .any(|(at, b)| *at < DRAIN && matches!(b, Boundary::Io { op: tools::Op::Spawn { .. } }))
        };
        assert!(armed, "{finding}: drain did not reach a running call");
        assert!(
            matches!(world.answer(), run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 0, .. }),
            "{finding}: expected a cut, got {:?}\n{}",
            world.answer(),
            world.trace().join("\n")
        );
    } else {
        assert!(
            matches!(world.answer(), run::Answer::Parked { .. }),
            "{finding}: expected parked, got {:?}",
            world.answer()
        );
    }
}

fn told_turn(world: &World, number: u32, finding: &str) -> Time {
    world
        .messages_seen()
        .iter()
        .find_map(|(at, seen)| {
            matches!(seen, smith_agent_world::messages_referee::Seen::Turn { number: n, .. } if *n == number)
                .then_some(*at)
        })
        .unwrap_or_else(|| panic!("{finding}: turn {number} was not told"))
}

fn observe(world: &World, draw: Draw, finding: &str) -> Time {
    match draw.kind {
        WITHHELD_WAIT => {
            let turn = world.turns().last().unwrap_or_else(|| panic!("{finding}: no last turn"));
            assert!(
                matches!(calls(turn).collect::<Vec<_>>().as_slice(), [call] if call.name.as_ref() == b"wait")
                    && results(turn).next().is_none()
                    && !world.waiting().is_empty(),
                "{finding}: wait was not withheld"
            );
            told_turn(world, u32::try_from(world.turns().len()).expect("bounded turns"), finding)
        }
        JOB_RUNNING => {
            assert!(
                world.turns().iter().flat_map(results).any(|(_, result)| matches!(result,
                Returned::Owned { outcome: tools::Outcome::Running { job, .. } } if job.0 == 1)),
                "{finding}: shell did not yield job 1"
            );
            let ends: Vec<_> = world
                .facts_at_answer()
                .iter()
                .filter_map(|fact| match fact {
                    Fact::Session { fact }
                        if matches!(fact.kind, session::FactKind::JobEnded {
                    job, end: tools::JobEnd::Cancelled, ..
                } if job.0 == 1) =>
                    {
                        Some(fact.at)
                    }
                    _ => None,
                })
                .collect();
            let [at] = ends.as_slice() else { panic!("{finding}: job 1 not told ended once before answer: {ends:?}") };
            *at
        }
        CUT_TURN => {
            let [answer, read, _, interrupted] = world.parts() else {
                panic!("{finding}: expected answer, read result, command start and interrupted result");
            };
            assert_eq!(answer.3.kind, PartKind::Answer, "{finding}: main answer part");
            assert!(
                matches!(results(&read.3.body).next(), Some((_, Returned::Owned { outcome: tools::Outcome::Read { content, .. } })) if content.as_ref() == SOURCE),
                "{finding}: missing kept read result"
            );
            assert!(
                matches!(
                    results(&interrupted.3.body).next(),
                    Some((_, Returned::Owned { outcome: tools::Outcome::Interrupted { .. } }))
                ),
                "{finding}: command interruption wasn't kept"
            );
            interrupted.0
        }
        HOST_CALL_ANSWERED => {
            let decision = answered(world, finding);
            assert_eq!(decision.name.activation, 1, "{finding}: host decision's activation");
            if draw.cut_answered {
                world
                    .parts()
                    .iter()
                    .find(|(_, _, _, part)| part.kind == PartKind::Result && host_result(&part.body))
                    .unwrap_or_else(|| panic!("{finding}: answered host call not told in a part"))
                    .0
            } else {
                assert!(
                    matches!(world.turns(), [host, waiting] if host_result(host)
                    && calls(waiting).any(|call| call.name.as_ref() == b"wait") && results(waiting).next().is_none()),
                    "{finding}: host answer not told before the withheld wait"
                );
                told_turn(world, 1, finding)
            }
        }
        HOST_CALL_UNANSWERED => {
            let [submission] = world.host_submissions() else {
                panic!("{finding}: expected one unanswered submission")
            };
            assert!(
                matches!(world.parts(), [(_, _, _, part)] if part.kind == PartKind::Answer)
                    && world.host_decisions() == 0
                    && matches!(world.host_withdrawals(), [(_, relay)] if *relay == submission.relay),
                "{finding}: expected a withdrawn host call without a result part or decision"
            );
            world.answered_at()
        }
        SUB_AGENT_KEPT => {
            let child = world.parts().iter().filter(|(_, _, _, part)| part.thread != 0).collect::<Vec<_>>();
            assert!(
                matches!(child.as_slice(), [(_, _, _, a), _, (_, _, _, r)]
                if a.kind == PartKind::Answer && a.call == 1 && r.call == 1 && r.kind == PartKind::Result),
                "{finding}: child answer and its command's interruption were not kept: {child:?}"
            );
            child.last().expect("kept child parts").0
        }
        _ => unreachable!("park kind from KINDS"),
    }
}

fn ends(world: &World, finding: &str) {
    assert_eq!(world.requests_after_answer(), 0, "{finding}: request or notice after the answer");
    assert_eq!(world.facts_at_answer().len(), world.facts().len(), "{finding}: facts told after the answer");
}

fn resumed(first: &World, next: &World, draw: Draw, finding: &str) {
    let kind = draw.kind;
    assert!(matches!(next.answer(), run::Answer::Accepted { .. }), "{finding}: resumed answer {:?}", next.answer());
    let expected = if matches!(kind, JOB_RUNNING | SUB_AGENT_KEPT) { 2 } else { 1 };
    assert_eq!(
        next.completions().len(),
        expected,
        "{finding}: unexpected completion (a kept answer may have been asked again)"
    );
    if kind == HOST_CALL_ANSWERED {
        assert!(next.host_submissions().is_empty(), "{finding}: host asked again");
        let answers = outputs(&next.prompts()[0]).filter(|(_, output, err)| !err && output.ends_with(HOST_ANSWER));
        assert_eq!(answers.count(), 1, "{finding}: recorded answer not sent to model once");
        if draw.cut_answered {
            assert!(host_result(rebuilt(first, next, finding)), "{finding}: answered call not filled from Start");
            assert!(io(next).next().is_none(), "{finding}: interrupted shell ran again");
            return;
        }
    }
    match kind {
        WITHHELD_WAIT | JOB_RUNNING | HOST_CALL_ANSWERED => {
            let wait = calls(first.turns().last().expect("parked turn"))
                .find(|call| call.name.as_ref() == b"wait")
                .unwrap_or_else(|| panic!("{finding}: no wait to resume"));
            let waking: Vec<_> = outputs(&next.prompts()[0])
                .filter(|(_, output, _)| skein_lib::bytes::find(output, WAKE).is_some())
                .collect();
            assert!(
                matches!(waking.as_slice(), [(id, output, false)] if *id == wait.id.as_ref() && skein_lib::bytes::count(output, WAKE, 2) == 1),
                "{finding}: first resumed request didn't answer the wait once: {waking:?}"
            );
            if kind == JOB_RUNNING {
                resumed_job(next, finding);
            }
        }
        CUT_TURN => {
            let turn = rebuilt(first, next, finding);
            assert!(
                matches!(io(next).collect::<Vec<_>>().as_slice(), [tools::Op::List { .. }]),
                "{finding}: only the call without a part should run"
            );
            let kept: Vec<_> = [1, 3].into_iter().flat_map(|index| results(&first.parts()[index].3.body)).collect();
            assert_eq!(results(turn).take(2).collect::<Vec<_>>(), kept, "{finding}: kept results changed");
            assert!(
                matches!(results(turn).last(), Some((_, Returned::Owned { outcome: tools::Outcome::Listed { .. } }))),
                "{finding}: missing result of the unstarted call"
            );
        }
        HOST_CALL_UNANSWERED => {
            rebuilt(first, next, finding);
            let [old] = first.host_submissions() else { panic!("{finding}: expected one first activation relay") };
            let new = answered(next, finding);
            assert_eq!(new.name, run::CallName { activation: 2, ..old.name }, "{finding}: incorrect new call identity");
            assert_eq!(new.tool, old.tool, "{finding}: different host tool on resume");
            assert_eq!(next.host_submissions()[0].input, old.input, "{finding}: different host input on resume");
        }
        SUB_AGENT_KEPT => {
            rebuilt(first, next, finding);
            let (_, _, _, old) = first
                .parts()
                .iter()
                .find(|(_, _, _, p)| p.thread != 0 && p.kind == PartKind::Answer)
                .unwrap_or_else(|| panic!("{finding}: no kept child answer"));
            let child: Vec<_> =
                next.parts().iter().filter(|(_, _, _, part)| part.thread != 0).map(|(_, _, _, part)| part).collect();
            assert!(
                child.iter().all(|part| part.call == old.call)
                    && matches!(child.last(), Some(new) if new.kind == PartKind::Answer && new.body.sequence == old.body.sequence + 1),
                "{finding}: no new child answer part at the kept call position"
            );
            assert!(io(next).next().is_none(), "{finding}: main read or child command ran again");
            assert!(
                outputs(&next.prompts()[0]).any(|(_, output, _)| skein_lib::bytes::find(
                    output,
                    b"interrupted by a worker restart"
                )
                .is_some()),
                "{finding}: child did not see its kept interruption"
            );
            assert!(
                outputs(&next.prompts()[1]).any(|(_, output, _)| skein_lib::bytes::find(output, CHILD_REPLY).is_some()),
                "{finding}: main didn't get the continued child's answer"
            );
        }
        _ => unreachable!("park kind from KINDS"),
    }
}

fn rebuilt<'a>(first: &World, next: &'a World, finding: &str) -> &'a smith_domain::Turn {
    let answer = &first.parts()[0].3.body;
    let turn = next.turns().first().unwrap_or_else(|| panic!("{finding}: cut turn was not rebuilt"));
    assert_eq!(turn.sequence, answer.sequence, "{finding}: rebuilt a different turn");
    assert_eq!(
        calls(turn).collect::<Vec<_>>(),
        calls(answer).collect::<Vec<_>>(),
        "{finding}: rebuilt different calls"
    );
    assert_eq!(
        results(turn).map(|(id, _)| id).collect::<Vec<_>>(),
        calls(answer).map(|call| call.id.as_ref()).collect::<Vec<_>>(),
        "{finding}: rebuilt results out of call order"
    );
    turn
}

fn resumed_job(next: &World, finding: &str) {
    let turn = next.turns().first().unwrap_or_else(|| panic!("{finding}: no job batch turn"));
    let message = turn.messages.last().unwrap_or_else(|| panic!("{finding}: stale job batch did not answer"));
    let returned: Vec<_> = message.content.iter().filter_map(result).collect();
    assert!(
        matches!(returned.as_slice(), [
        (_, Returned::Invalid { problem: Problem::UnknownJob { job: 1 } }),
        (_, Returned::NotRun { why: NotRun::AfterFailedWrite { call: 1 } }),
        (_, Returned::Owned { outcome: tools::Outcome::Read { content, .. } }),
    ] if content.as_ref() == SOURCE),
        "{finding}: stale job batch results: {returned:?}"
    );
    assert_eq!(
        returned.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        calls(turn).map(|c| c.id.as_ref()).collect::<Vec<_>>(),
        "{finding}: stale job batch result identities"
    );
    let started: Vec<_> = next
        .facts()
        .iter()
        .filter_map(|fact| match fact {
            Fact::Session { fact } => match fact.kind {
                session::FactKind::Tools { fact: tools::FactKind::Started { tool, .. }, .. } => Some(tool),
                _ => None,
            },
            Fact::Run { .. } => None,
        })
        .collect();
    assert_eq!(started, [tools::Tool::Read], "{finding}: job or edit passed the tools entrance");
    assert!(
        matches!(io(next).collect::<Vec<_>>().as_slice(), [tools::Op::Load { .. }]),
        "{finding}: only the next batch's read reaches IO"
    );
}
