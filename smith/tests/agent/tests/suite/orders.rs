//! Root handoff orders under concurrent session and host pressure.
//! Contract: smith/docs/design/domain/run.md, sections 3.2, 5.3 and 11;
//! skein/docs/design/draft/dynamic-memory.md, section 4.6.

use std::collections::{BTreeMap, BTreeSet};

use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{Duration, ReplyTo, Time, Token};
use skein_world::domain::Span;
use smith_agent_world::orders::{Entrance, EntranceObservation, HandoffRequest};
use smith_agent_world::{Boundary, CompletionTerminal, Job, Settings, World, scripted_charter};
use smith_domain::{Event, Fact, Grant, GrantName, Window, run, session};

// Four adjacent read-effect delegates fill the world's parallel batch.
const CHILDREN: u32 = 4;
// Three batches reuse conversation slots and produce twelve distinct openings.
const BATCHES: u32 = 3;
const ITERATIONS: u32 = 10_000;
const ACTIVATION: u64 = 37;
const HOST: Token = Token::new(1);
const ACKNOWLEDGED: Time = Time::from_nanos(1_000_000_000);
const FILE: &[u8] = b"order fixture\n";
const FINISH: &[u8] = br#"{"report":"All handoffs settled.","source":"workspace"}"#;

fn settings(seed: u64) -> Settings {
    let calm = Settings::calm(seed);
    Settings {
        job: Job::Reporting,
        writable: false,
        network: Span::millis(1, 1),
        provider: skein_fake_llm_domain::Config {
            latency_min: Duration::from_millis(100),
            latency_max: Duration::from_millis(100),
            ..calm.provider
        },
        ..calm
    }
}

fn charter(settings: &Settings, instructions: &[u8]) -> run::Charter {
    let mut charter = scripted_charter(settings);
    charter.instructions = instructions.into();
    charter.brief.sections = Box::default();
    charter.conventions = None;
    charter
}

fn calls(lines: Vec<Line>) -> Turn {
    Turn { lines: lines.into(), finish: Finish::ToolCalls, tokens: 1 }
}

fn call(name: &[u8], arguments: &[u8]) -> Line {
    Line::Call { name: name.into(), arguments: arguments.into() }
}

fn read() -> Turn {
    calls(vec![call(b"read", br#"{"path":"README.md"}"#)])
}

fn stopped() -> Turn {
    Turn { lines: Box::new([Line::Text { text: b"child settled".as_slice().into() }]), finish: Finish::Stop, tokens: 1 }
}

fn child(cue: &str, model: Option<&str>) -> Line {
    let selected = model.map_or_else(String::new, |model| format!(r#","llm":"{model}""#));
    call(b"sub_agent", format!(r#"{{"brief":"{cue}","tools":["inspect"]{selected}}}"#).as_bytes())
}

fn reads(world: &World) -> usize {
    world
        .boundaries()
        .iter()
        .filter(|(_, boundary)| matches!(boundary, Boundary::Io { op: smith_domain::tools::Op::Load { .. } }))
        .count()
}

fn fixture(settings: &Settings, charter: run::Charter, scripts: Vec<Script>, window: Window) -> World {
    let mut disk = Checkout::new();
    disk.mkdir(b"work");
    disk.write(b"work/README.md", FILE);
    let workspace = run::Workspace {
        directories: Box::new([run::Directory {
            host: b"/host".as_slice().into(),
            name: b"work".as_slice().into(),
            root: Token::new(disk.root(b"work")),
            writable: false,
            git: false,
            conflicts: Box::default(),
        }]),
    };
    let start = Event::Start {
        messages: Box::new([run::Message {
            finish_requested: false,
            name: Token::new(9),
            label: b"host".as_slice().into(),
            text: b"Run the handoff scenario.".as_slice().into(),
        }]),
        reply_to: ReplyTo::new(HOST),
        host_run: HOST,
        identity: None,
        activation: ACTIVATION,
        window,
        charter,
        workspace: Some(workspace),
        transcript: None,
        answered: Box::default(),
        grants: Box::new([
            Grant { renewable: true, name: GrantName { account: 0, generation: 1 }, valid: Duration::from_secs(7200) },
            Grant { renewable: false, name: GrantName { account: 1, generation: 1 }, valid: Duration::from_millis(1) },
        ]),
    };
    let mut world = World::with_workspace_scripts_start(*settings, disk, scripts.into(), start);
    world.record_entrances();
    world
}

/// Order: `run_handoffs_drained`, with `child_handoffs_drained` at return (phase C).
#[test]
fn run_handoffs_drained_before_later_session_requests() {
    const SEED: u64 = 2001;
    const MAIN: &[u8] = b"@handoff-main";
    let settings = settings(SEED);
    let charter = charter(&settings, MAIN);
    let mut scripts = Vec::new();
    let mut children = Vec::new();
    for index in 0..CHILDREN {
        let cue = format!("@handoff-child-{index}");
        children.push(child(&cue, None));
        let turns = if index == 0 { vec![stopped()] } else { vec![read(), stopped()] };
        scripts.push(Script { cue: cue.into_bytes().into(), turns: turns.into() });
    }
    scripts.push(Script {
        cue: MAIN.into(),
        turns: vec![read(), calls(children), calls(vec![call(b"finish", FINISH)])].into(),
    });
    let largest = smith_domain::max_turn_bytes(&settings.limits).expect("bounded host turn");
    let mut world = fixture(&settings, charter, scripts, Window { turns: 1, bytes: largest });
    world.acknowledge_at(ACKNOWLEDGED, 1);
    for _ in 0..ITERATIONS {
        world.drive(1);
        if !world.turns().is_empty() {
            break;
        }
    }
    assert_eq!(world.turns().len(), 1, "seed {SEED}: the concrete turn reaches the host");
    assert_eq!(world.prompts().len(), 1, "seed {SEED}: its successor waits for host credit");
    // Child zero ends while its siblings continue through filesystem work and
    // another provider request. The parent then cancels the remaining work.
    for _ in 0..ITERATIONS {
        world.drive(1);
        if world.entrances().iter().flat_map(|entry| &entry.requests).any(|request| {
            matches!(request, HandoffRequest::Complete { thread, name, .. } if *thread != 0 && name.sequence > 1)
        }) {
            break;
        }
    }
    world.cancel_run();
    world.run(ITERATIONS);
    let mut turns = BTreeSet::new();
    let mut threads = BTreeSet::new();
    let mut cancels = 0;
    for entrance in world.entrances() {
        for request in &entrance.requests {
            match request {
                HandoffRequest::Turn { sequence } => {
                    turns.insert(*sequence);
                }
                HandoffRequest::Complete { name, thread, .. } => {
                    threads.insert(*thread);
                    if *thread == 0 && name.sequence > 1 {
                        assert!(entrance.at >= ACKNOWLEDGED, "seed {SEED}: host credit delays dispatch");
                        assert!(
                            turns.contains(&(name.sequence - 1)),
                            "seed {SEED}: every main provider request follows its concrete root turn"
                        );
                    }
                }
                HandoffRequest::Cancel { .. } => {
                    cancels += 1;
                }
            }
        }
    }
    assert_eq!(threads.len(), usize::try_from(CHILDREN + 1).expect("main and children"));
    assert!(cancels > 0, "seed {SEED}: unrelated provider cancellations reached the root");
    assert!(world.completions().iter().any(|completion| completion.started >= ACKNOWLEDGED));
    assert!(
        world
            .completions()
            .iter()
            .any(|completion| matches!(completion.terminal, Some((_, CompletionTerminal::Cancelled))))
    );
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::Failure::Cancelled, .. }));
}

/// Order: `child_facts_gathered_before_handoff` (phase C).
#[test]
fn child_facts_gathered_before_open_handoff_under_load() {
    const SEED: u64 = 2003;
    const MAIN: &[u8] = b"@opening-main";
    let mut settings = settings(SEED);
    settings.limits.session.retries = 0;
    let mut charter = charter(&settings, MAIN);
    // This model's grant expires before child creation. Its openings are
    // observed before the unsent provider failure closes their sessions.
    charter.models[0].account = Some(1);
    let main_llm = charter.llm.clone();
    let child_llm = charter.models[0].clone();
    let mut scripts = Vec::new();
    let mut parent_turns = Vec::new();
    for batch in 0..BATCHES {
        let mut children = Vec::new();
        for index in 0..CHILDREN {
            let cue = format!("@opening-child-{batch}-{index}");
            children.push(child(&cue, if index % 2 == 0 { None } else { Some("fake-2") }));
            scripts.push(Script { cue: cue.into_bytes().into(), turns: Box::new([stopped()]) });
        }
        parent_turns.push(calls(children));
    }
    parent_turns.push(calls(vec![call(b"finish", FINISH)]));
    scripts.push(Script { cue: MAIN.into(), turns: parent_turns.into() });
    let mut world = fixture(&settings, charter, scripts, Window { turns: u32::MAX, bytes: u64::MAX });
    world.hold_fact_writes();
    world.run(ITERATIONS);
    assert!(world.facts().is_empty(), "seed {SEED}: the outer writer is still delayed");
    world.flush_fact_writes();
    let mut ordinals = Vec::new();
    let mut failed_rounds = BTreeSet::new();
    for entrance in world.entrances() {
        let mut opened = 0;
        let mut failed = 0;
        for fact in &entrance.facts {
            if let Fact::Run { fact: run::facts::Fact { kind, .. }, opening } = fact {
                if let run::facts::FactKind::Opened { .. } = kind {
                    let opening = opening.as_ref().expect("queued Open supplies every opening's metadata");
                    assert_eq!(opening.activation, ACTIVATION, "seed {SEED}");
                    let expected =
                        if opening.ordinal != 0 && opening.ordinal % 2 == 0 { &child_llm } else { &main_llm };
                    assert_eq!(&opening.llm, expected, "seed {SEED}: ordinal {}", opening.ordinal);
                    ordinals.push(opening.ordinal);
                    opened += 1;
                }
                if matches!(kind, run::facts::FactKind::Ended { end: run::facts::EndKind::Fault, .. }) {
                    failed += 1;
                }
            }
        }
        if opened == CHILDREN && failed == CHILDREN / 2 {
            failed_rounds.insert(entrance.round);
        }
    }
    assert_eq!(ordinals, (0..=BATCHES * CHILDREN).collect::<Vec<_>>(), "seed {SEED}: no opening lookup was lost");
    assert_eq!(
        failed_rounds.len(),
        usize::try_from(BATCHES).expect("opening batches"),
        "seed {SEED}: child creation and unsent failures share each loaded entrance"
    );
    let recorded: Vec<_> = world.entrances().iter().flat_map(|entrance| &entrance.facts).collect();
    assert_eq!(recorded, world.facts().iter().collect::<Vec<_>>(), "delayed writing preserves every fact in order");
    assert!(world.entrances().iter().any(|entrance| entrance.entrance == Entrance::Resume));
    assert!(matches!(world.answer(), run::Answer::Accepted { .. }));
}

/// Order: `handoff_promoted_after_reclaim` (phase C).
#[test]
fn handoff_promoted_after_reclaim_for_returns_and_closes() {
    const SEED: u64 = 2002;
    // Eight write-effect finish calls exceed one parallel batch and are each
    // rejected immediately for omitting the charter's required source field.
    const REJECTIONS: u32 = 8;
    const MAIN: &[u8] = b"@reclaim-main";
    const REJECTED: &[u8] = br#"{"report":"Missing the required source field."}"#;
    let settings = settings(SEED);
    let charter = charter(&settings, MAIN);
    let mut scripts = Vec::new();
    let mut children = Vec::new();
    for index in 0..CHILDREN {
        let cue = format!("@reclaim-child-{index}");
        children.push(child(&cue, None));
        scripts.push(Script { cue: cue.into_bytes().into(), turns: vec![read(), stopped()].into() });
    }
    scripts.push(Script {
        cue: MAIN.into(),
        turns: vec![
            calls((0..REJECTIONS).map(|_| call(b"finish", REJECTED)).collect()),
            calls(children),
            calls(vec![call(b"finish", FINISH)]),
        ]
        .into(),
    });
    let mut world = fixture(&settings, charter, scripts, Window { turns: u32::MAX, bytes: u64::MAX });
    // Stop outside a round with child filesystem work admitted. Closing
    // then walks root, conversations, sessions and tools through their terminals.
    for _ in 0..ITERATIONS {
        world.drive(1);
        if reads(&world) == usize::try_from(CHILDREN).expect("child reads") {
            break;
        }
    }
    world.cancel_run();
    world.run(ITERATIONS);
    assert_eq!(reads(&world), usize::try_from(CHILDREN).expect("child reads"));
    assert_returns_cross_reclaim(world.entrances(), SEED, REJECTIONS);
    assert_closes_cross_reclaim(world.entrances(), SEED);
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::Failure::Cancelled, .. }));
}

fn assert_returns_cross_reclaim(entries: &[EntranceObservation], seed: u64, rejections: u32) {
    let mut returned = BTreeMap::new();
    let mut answered = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        for fact in &entry.facts {
            if let Fact::Run {
                fact:
                    run::facts::Fact {
                        kind: run::facts::FactKind::Returned { call, result: run::facts::Return::Rejected, .. },
                        ..
                    },
                ..
            } = fact
            {
                returned.insert(*call, index);
                assert!(!entry.ready, "seed {seed}: a new Return is not eligible before reclaim");
            }
            if let Fact::Session {
                fact: session::Fact { kind: session::FactKind::DelegateAnswered { .. }, call: Some(call), .. },
            } = fact
                && let Some(start) = returned.get(&call.owner)
            {
                let original = &entries[*start];
                assert!(
                    entry.round > original.round,
                    "seed {seed}: call {:?} cannot pass its reclaim point",
                    call.owner
                );
                assert_eq!(entry.entrance, Entrance::Resume);
                assert!(
                    entries[*start..index].iter().any(|entry| {
                        entry.entrance == Entrance::Reclaim && entry.round == original.round && entry.ready
                    }),
                    "seed {seed}: reclaim promotes this deferred Return"
                );
                answered.insert(call.owner);
            }
        }
    }
    assert_eq!(returned.len(), usize::try_from(rejections).expect("rejected calls"));
    assert_eq!(answered.len(), returned.len(), "every rejected call crossed reclaim before delivery");
}

fn assert_closes_cross_reclaim(entries: &[EntranceObservation], seed: u64) {
    let cancel = entries.iter().position(|entry| entry.entrance == Entrance::Cancel).expect("host cancelled");
    let cancellation = &entries[cancel];
    assert!(!cancellation.ready, "seed {seed}: new Close handoffs wait for reclaim");
    assert!(cancellation.requests.is_empty(), "no close cascade runs inside the cancellation entrance");
    let mut closed = BTreeSet::new();
    let mut kits = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate().skip(cancel) {
        for fact in &entry.facts {
            if let Fact::Session { fact: session::Fact { kind, .. } } = fact {
                if let session::FactKind::Tools { opener, fact: smith_domain::tools::FactKind::Closed { .. } } = kind {
                    assert!(entry.round > cancellation.round, "seed {seed}: the kit closes after reclaim");
                    assert!(entries[cancel..index].iter().any(|entry| {
                        entry.entrance == Entrance::Reclaim && entry.round == cancellation.round && entry.ready
                    }));
                    kits.insert(*opener);
                }
                if let session::FactKind::Ended { opener, .. } = kind {
                    assert!(entry.round > cancellation.round, "seed {seed}: the session closes after reclaim");
                    closed.insert(*opener);
                }
            }
        }
    }
    assert_eq!(closed.len(), usize::try_from(CHILDREN + 1).expect("main and children"));
    assert_eq!(kits, closed, "every session settles its own tools in the close cascade");
}
