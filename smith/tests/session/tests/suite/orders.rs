//! Tools entrance settlement and batch boundaries (smith/docs/design/domain/session.md, 5 and 10).

use std::collections::BTreeSet;

use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{Duration, Time, Token};
use smith_domain_session::{self as session, llm};
use smith_domain_tools::{self as tools, Budget, Call, Done, JobEnd, JobNumber, Op, Outcome};
use smith_session_world::recorded::{self, Scripted};
use smith_session_world::{Reclaim, Settings, Span, World, spec};
use smith_tools_world::translate as io;

const ITERATIONS: u32 = 100_000;
const SEED: u64 = 319;
const FILE: &[u8] = b"README.md";
const OLD: &[u8] = b"README.md: old\n";

fn called(id: &[u8], call: Call) -> llm::Block {
    llm::Block::ToolCall {
        id: id.into(),
        name: b"workspace".as_slice().into(),
        input: Box::default(),
        call: llm::Decoded::Owned { call },
        replay: None,
    }
}

fn read(id: &[u8], path: &[u8]) -> llm::Block {
    called(id, Call::Read { path: io::path(path), offset: 1, window: Budget::Default })
}

fn complete(world: &mut Scripted, blocks: Vec<llm::Block>) {
    world.complete(blocks.into(), llm::Stop::ToolUse, llm::Usage::ZERO);
}

fn machine() -> (Scripted, Checkout) {
    let mut checkout = Checkout::new();
    checkout.mkdir(b"/repo");
    checkout.write(b"/repo/README.md", OLD);
    let root = checkout.root(b"/repo");
    let mut opening = recorded::opening(None, u64::MAX);
    opening.spec.authority = tools::Authority {
        cwd: io::names(b"/repo"),
        repos: Box::new([tools::Repo {
            host: Box::default(),
            mount: io::names(b"/repo"),
            root: io::token(root),
            writable: true,
        }]),
        grants: tools::Grants { inspect: true, modify: true, shell: true },
    };
    let mut world = Scripted::new(SEED);
    world.open(opening);
    (world, checkout)
}

fn perform(world: &mut Scripted, checkout: &mut Checkout, owner: Token) {
    let op = world.operations.iter().find(|(pending, _)| *pending == owner).expect("pending operation").1.clone();
    world.step(session::Event::Done { owner, done: io::perform(checkout, op) });
}

fn results(world: &Scripted) -> Vec<(&[u8], &llm::Returned)> {
    world
        .turns
        .last()
        .expect("settled turn")
        .messages
        .last()
        .expect("result message")
        .content
        .iter()
        .map(|block| {
            let llm::Block::ToolResult { id, result } = block else { panic!("result message contains results") };
            (id.as_ref(), result)
        })
        .collect()
}

fn answered(facts: &[session::Fact]) -> Vec<&[u8]> {
    facts
        .iter()
        .filter(|fact| matches!(fact.kind, session::FactKind::Tools { fact: tools::FactKind::Answered { .. }, .. }))
        .map(|fact| fact.call.as_ref().expect("answered call metadata").id.as_ref())
        .collect()
}

fn scripted(mut settings: Settings, lines: Vec<Line>, tokens: u64) -> World {
    settings.network = Span::millis(0, 0);
    settings.provider.calls_per_answer = u32::try_from(lines.len()).expect("small script");
    settings.provider.latency_min = Duration::from_millis(1);
    settings.provider.latency_max = Duration::from_millis(1);
    let script = Script {
        cue: b"You are a coding agent.".as_slice().into(),
        turns: Box::new([Turn { lines: lines.into(), finish: Finish::ToolCalls, tokens }]),
    };
    World::with_scripts(settings, Box::new([script]))
}

/// Order: `tools_requests_dispatched` (phase C).
#[test]
fn tools_requests_dispatched_before_the_next_batch() {
    const JOBS: usize = 3; // Multiple CancelIo effects from one CancelJobs entrance.
    let (mut world, mut checkout) = machine();
    let mut jobs = Vec::new();
    for _ in 0..JOBS {
        complete(
            &mut world,
            vec![called(
                b"build",
                Call::Shell {
                    command: b"cargo build".as_slice().into(),
                    timeout: tools::Timeout::Default,
                    output: Budget::Default,
                    after: tools::Yield::Asked(Duration::from_secs(1)),
                },
            )],
        );
        let owner = world.operations[0].0;
        world.step(session::Event::Done {
            owner,
            done: Done::Held { head: Box::default(), tail: Box::default(), dropped: 0 },
        });
        jobs.push(owner);
    }
    let first_fact = world.facts.len();
    complete(
        &mut world,
        vec![
            read(b"refused-first", b"../outside"),
            read(b"refused-second", b"../outside"),
            read(b"live", FILE),
            called(
                b"poll",
                Call::Job { job: JobNumber(1), wait: tools::Wait::Default, output: Budget::Default, stop: false },
            ),
            read(b"later", FILE),
        ],
    );
    let live = world.operations[0].0;
    assert_eq!(world.operations.len(), 1, "the later batch has not dispatched");
    assert_eq!(world.job_acts.len(), 1, "the current entrance translated the job read");
    assert_eq!(
        answered(&world.facts[first_fact..]),
        [b"refused-first".as_slice(), b"refused-second"],
        "both immediate terminals crossed the tools seam before another entrance"
    );
    world.step(session::Event::CancelJobs { session: world.session.expect("opened") });
    assert_eq!(world.cancelled_operations, jobs, "every cancellation from the same tools entrance is translated");
    assert_eq!(world.jobs_cancelled, 0, "terminals are still owed");
    for &owner in jobs.iter().rev() {
        world.step(session::Event::Done {
            owner,
            done: Done::Job { head: Box::default(), tail: Box::default(), dropped: 0, end: JobEnd::Cancelled },
        });
        assert_eq!(
            world.operations.iter().map(|(owner, _)| *owner).collect::<Vec<_>>(),
            [live],
            "no later batch bypasses the current live IO"
        );
    }
    assert_eq!(world.jobs_cancelled, 1, "the tools' settlement terminal was translated");
    assert!(world.job_acts.is_empty() && world.held.is_empty());
    assert_eq!(
        answered(&world.facts[first_fact..]),
        [b"refused-first".as_slice(), b"refused-second", b"poll"],
        "all refusals and the cancelled poll are translated while the later batch still waits"
    );
    perform(&mut world, &mut checkout, live);
    assert_eq!(world.operations.len(), 1, "the later batch now dispatches");
    let later = world.operations[0].0;
    assert_ne!(live, later, "the later batch has its own request identity");
    perform(&mut world, &mut checkout, later);
    let observed = results(&world);
    assert_eq!(
        observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        [b"refused-first".as_slice(), b"refused-second", b"live", b"poll", b"later"]
    );
    assert!(
        observed[..2]
            .iter()
            .all(|(_, result)| matches!(result, llm::Returned::Owned { outcome: Outcome::Outside { .. } }))
    );
    assert!(matches!(observed[3].1, llm::Returned::Owned { outcome: Outcome::Job { end: JobEnd::Cancelled, .. } }));
    world.close();
}

/// Order: `batch_promoted_after_reclaim` (phase C).
#[test]
fn immediate_batches_are_promoted_after_reclaim_before_close() {
    const SESSIONS: usize = 3; // One closed at the boundary, two continuing under load.
    const PARALLEL: u32 = 2;
    const CALLS: usize = 7; // Four batches at width two, so a boundary close leaves work unstarted.
    const TOKENS: u64 = 40; // The scripted tool response's declared output usage.
    let calm = Settings::calm(SEED);
    let settings = Settings { agent: session::Limits { parallel_tools: PARALLEL, ..calm.agent }, ..calm };
    let line = Line::Call { name: b"read".as_slice().into(), arguments: br#"{"path":"src/lib.rs"}"#.as_slice().into() };
    let mut world = scripted(settings, vec![line; CALLS], TOKENS);
    let openers: Vec<_> = (0..SESSIONS)
        .map(|_| {
            let mut opening = spec(b"refused reads");
            opening.authority.grants.inspect = false;
            world.submit(Time::ZERO, opening)
        })
        .collect();
    let mut closed = false;
    let mut batches = Vec::new();
    world.run_observed(ITERATIONS, |world, point| match point {
        Reclaim::Before => {
            let runs = world.domain().runs();
            if runs > 0 {
                assert!(!world.domain().is_ready(), "newly resting batches cannot resume before reclaim");
                batches.push(runs);
                if !closed {
                    world.close(openers[0]);
                    closed = true;
                }
            }
        }
        Reclaim::After => assert_eq!(world.domain().runs(), 0, "retired slots are freed before any batch reuses them"),
    });
    assert_eq!(batches, [6, 6, 4, 2], "one batch per session per reclaim interval");
    assert_eq!(world.stats().ops, 0, "all reads were refused at the tools entrance");
    for (index, opener) in openers.into_iter().enumerate() {
        let session = world.session(opener);
        assert_eq!(session.ended.as_ref().expect("settled session").end, session::End::Closed);
        let outcomes: Vec<_> = session.records[0]
            .messages
            .last()
            .expect("results")
            .content
            .iter()
            .map(|block| {
                let llm::Block::ToolResult { result, .. } = block else { panic!("tool result") };
                result
            })
            .collect();
        let refused = if index == 0 { 4 } else { CALLS };
        assert!(
            outcomes[..refused]
                .iter()
                .all(|result| matches!(result, llm::Returned::Owned { outcome: Outcome::NotGranted }))
        );
        assert!(
            outcomes[refused..]
                .iter()
                .all(|result| matches!(result, llm::Returned::NotRun { why: llm::NotRun::Stopped }))
        );
        assert_eq!(outcomes.len(), CALLS);
    }
}

fn line(name: &[u8], arguments: &[u8]) -> Line {
    Line::Call { name: name.into(), arguments: arguments.into() }
}

fn loaded_content(block: &llm::Block) -> &[u8] {
    let llm::Block::ToolResult { result: llm::Returned::Owned { outcome: Outcome::Read { content, .. } }, .. } = block
    else {
        panic!("read result")
    };
    content
}

/// Order: `batch_accepts(effect)`, `batch_settled` (phase C).
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the scripted reverse-terminal referee and its ordered transcript expectations form one scenario"
)]
fn reverse_read_terminals_preserve_write_isolation_and_result_order() {
    const SESSIONS: usize = 4; // The observed session and three unrelated sessions with live IO.
    const CALLS: usize = 7; // Four reads (one delegated), one write, then two reads.
    const TOKENS: u64 = 48; // Scripted response usage, as in the existing batching scenario.
    const SOURCE: &[u8] = b"src/lib.rs";
    const GUIDE: &[u8] = b"docs/guide.md";
    const OLD_SOURCE: &[u8] = b"src/lib.rs: pub fn answer() -> u32 { 42 }\n";
    const NEW_SOURCE: &[u8] = b"src/lib.rs: rewritten\n";
    const FIRST_DELAYS: [u64; 3] = [60, 50, 40]; // Reverse the owned reads after the delegated read (1 ms).
    const LAST_DELAYS: [u64; 2] = [60, 40]; // Reverse the two reads after the write.
    const PEER_DELAY: u64 = 500; // Other sessions' reads remain live through both observed batches.
    let calm = Settings::calm(SEED);
    let settings = Settings { tool: Span::millis(30, 30), serve: 1000, serving: Span::millis(1, 1), ..calm };
    let mut world = scripted(
        settings,
        vec![
            line(b"read", br#"{"path":"src/lib.rs"}"#),
            line(b"read", br#"{"path":"README.md"}"#),
            line(b"read", br#"{"path":"docs/guide.md"}"#),
            line(b"lookup", br#"{"path":"README.md"}"#),
            line(b"write", br#"{"path":"src/lib.rs","content":"src/lib.rs: rewritten\n"}"#),
            line(b"read", br#"{"path":"src/lib.rs"}"#),
            line(b"read", br#"{"path":"README.md"}"#),
        ],
        TOKENS,
    );
    let openers: Vec<_> = (0..SESSIONS).map(|_| world.submit(Time::ZERO, spec(b"reverse read terminals"))).collect();
    let mut delayed = BTreeSet::new();
    let mut tracked = Vec::new();
    let mut terminals = Vec::new();
    let mut batches = 0;
    world.run_observed(ITERATIONS, |world, point| {
        if point != Reclaim::Before {
            return;
        }
        let target = openers[0];
        // All opens share time zero, so each opener exists before the first observation.
        let pending = world.pending_io(target);
        let delegates = world.pending_delegates(target);
        let owners: Vec<_> =
            pending.iter().map(|(owner, _)| *owner).chain(delegates.iter().map(|(owner, _)| *owner)).collect();
        tracked.retain(|(owner, label)| {
            if owners.contains(owner) {
                return true;
            }
            terminals.push(*label);
            false
        });
        let mut deliveries = Vec::new();
        if pending.len() > 1 && pending.iter().any(|(owner, _)| !delayed.contains(owner)) {
            assert!(
                openers[1..].iter().all(|&opener| !world.pending_io(opener).is_empty()),
                "three unrelated sessions have IO in flight during the reversed batch"
            );
            let (paths, labels, delays): (&[&[u8]], &[&str], &[u64]) = if batches == 0 {
                assert_eq!(delegates.len(), 1, "a delegated read joins the owned reads");
                assert_eq!(delegates[0].1.position, 3, "the fourth provider call is delegated");
                tracked.push((delegates[0].0, "lookup"));
                (&[SOURCE, FILE, GUIDE], &["source", "readme", "guide"], &FIRST_DELAYS)
            } else {
                assert!(delegates.is_empty());
                (&[SOURCE, FILE], &["after-source", "after-readme"], &LAST_DELAYS)
            };
            assert_eq!(pending.len(), paths.len(), "every adjacent owned read starts together");
            for ((path, label), delay) in paths.iter().zip(labels).zip(delays) {
                let owner = pending
                    .iter()
                    .find_map(|(owner, op)| {
                        matches!(op, Op::Load { at, .. } if at.path.as_ref() == *path).then_some(*owner)
                    })
                    .expect("the batch's read reached IO");
                tracked.push((owner, *label));
                deliveries.push((owner, world.now().saturating_add(Duration::from_millis(*delay))));
                delayed.insert(owner);
            }
            batches += 1;
        }
        for &opener in &openers[1..] {
            for (owner, _) in world.pending_io(opener) {
                if delayed.insert(owner) {
                    deliveries.push((owner, world.now().saturating_add(Duration::from_millis(PEER_DELAY))));
                }
            }
        }
        for (owner, at) in deliveries {
            world.deliver_io_at(owner, at);
        }
    });
    assert_eq!(batches, 2, "both read batches were reversed under unrelated queue load");
    assert_eq!(terminals, ["lookup", "guide", "readme", "source", "after-readme", "after-source"]);
    assert_eq!(world.stats().most_parallel, 4, "the delegated read shares the first batch");
    for opener in openers {
        let observed = world.session(opener);
        assert_eq!(observed.ended.as_ref().expect("settled session").end, session::End::Closed);
        let messages = &observed.records[0].messages;
        let calls: Vec<_> = messages[1]
            .content
            .iter()
            .filter_map(|block| if let llm::Block::ToolCall { id, .. } = block { Some(id) } else { None })
            .collect();
        let results = &messages[2].content;
        assert_eq!(results.len(), CALLS);
        let ids: Vec<_> = results
            .iter()
            .map(|block| {
                let llm::Block::ToolResult { id, .. } = block else { panic!("result message") };
                id
            })
            .collect();
        assert_eq!(ids, calls, "results retain provider call order despite reverse terminals");
        assert_eq!(loaded_content(&results[0]), OLD_SOURCE);
        assert!(matches!(&results[3], llm::Block::ToolResult { result: llm::Returned::Text { error: false, .. }, .. }));
        assert!(matches!(
            &results[4],
            llm::Block::ToolResult {
                result: llm::Returned::Owned { outcome: Outcome::Written { created: false } },
                ..
            }
        ));
        assert_eq!(loaded_content(&results[5]), NEW_SOURCE, "later reads see the stored write");
    }
}
