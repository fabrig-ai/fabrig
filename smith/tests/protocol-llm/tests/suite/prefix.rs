//! Ordered host sections through observed main/child sessions and native restore.
//! Expected system bodies are handwritten outside literals; concrete history,
//! child results, usage and lower settlement come from the composed routes.
//! Contract: domain/run.md, sections 3.1, 3.3, 5.3, 6 and 12;
//! testing-strategy.md, sections 2.3, 6 and 7.

use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Line, Message, Part, Role, Script, Turn};
use skein_lib::{Duration, Token};
use skein_world::domain::Span;
use smith_domain::{Transcript, run, session::llm};
use smith_protocol_llm_world::codec::{self as codec};
use smith_protocol_llm_world::{
    Job, Settings, World,
    client_peer::{self},
};

const BEGIN: &[u8] = b"Begin the work your brief describes.";
const NATIVE_INSTRUCTIONS: &str = "@brief-native PARENT-INSTRUCTIONS: native first activation.";
const FINISH_INPUT: &[u8] =
    br#"{"form":"report","text":"The child inspected the changed workspace.","fields":{"source":"workspace"}}"#;
const FIRST: &[u8] = b"Native structured brief first activation parked.";
const LAST: &[u8] = b"Native structured brief second activation parked.";

// These literals do not call the production renderer or reproduce its algorithm.
// Duplicate Zulu titles and the intervening Alpha title must stay in host order.
fn charter(settings: &Settings, instructions: &str) -> run::Charter {
    use run::charter::{Brief, Endpoint, Grants, Llm, Section, Tools};
    use run::outcome::{FieldRule, OutcomeSpec, TextSpec};
    let llm = Llm {
        effort: None,
        prices: run::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
        dialect: 3,
        account: Some(0),
        endpoint: Endpoint(0),
        model: b"fake-1".as_slice().into(),
        window: smith_protocol_llm_world::MODEL_WINDOW,
        output: 4096,
    };
    run::Charter {
        check_timeout: None,
        mode: run::Mode::Finite,
        reserve: run::charter::Reserve { turns: 0, time: skein_lib::Duration::ZERO },
        instructions: instructions.as_bytes().into(),
        brief: Brief {
            sections: Box::new([
                Section {
                    title: b"Zulu PARENT-TITLE-Z".as_slice().into(),
                    text: "PARENT-BODY-ONE: café\nline one".as_bytes().into(),
                },
                Section {
                    title: b"Alpha PARENT-TITLE-A".as_slice().into(),
                    text: "PARENT-BODY-TWO: λ ends with LF\n".as_bytes().into(),
                },
                Section {
                    title: b"Zulu PARENT-TITLE-Z".as_slice().into(),
                    text: "PARENT-BODY-THREE: 東京\nthird line".as_bytes().into(),
                },
            ]),
        },
        grants: Grants {
            wait: true,
            deliver: None,
            save: false,
            tools: Tools { inspect: true, modify: true, shell: false },
            agents: true,
            host_tools: Box::new([]),
        },
        outcome: OutcomeSpec {
            change: None,
            verdicts: Box::new([]),
            report: Some(TextSpec {
                max: 1024,
                fields: Box::new([FieldRule { name: b"source".as_slice().into(), max: 128 }]),
            }),
            failure: None,
        },
        budget: settings.budget,
        models: Box::new([Llm { effort: None, model: b"fake-2".as_slice().into(), ..llm.clone() }]),
        llm,
        conventions: Some(run::Conventions {
            guide: b"AGENTS.md".as_slice().into(),
            checks: b".temper/pre-pr".as_slice().into(),
        }),
        resume: settings.resume,
        waiting: settings.waiting,
    }
}

fn workspace() -> (Checkout, run::Workspace) {
    let mut disk = Checkout::new();
    disk.mkdir(b"work");
    disk.mkdir(b"archive");
    disk.write(b"work/AGENTS.md", b"WORK-GUIDE: inspect data.txt.\n");
    disk.write(b"archive/AGENTS.md", b"ARCHIVE-GUIDE: keep every byte.");
    disk.write(b"work/.temper/pre-pr", b"#!checks");
    disk.write(b"archive/.temper/pre-pr", b"#!readonly-check-must-not-run");
    disk.write(b"work/.git/HEAD", b"host-owned-git-metadata");
    disk.write(b"work/data.txt", b"old");
    disk.write(b"archive/data.txt", b"unchanged");
    let mounted = run::Workspace {
        directories: Box::new([
            run::Directory {
                host: Box::from(b"/host".as_slice()),
                name: b"work".as_slice().into(),
                root: Token::new(disk.root(b"work")),
                writable: true,
                git: true,
                conflicts: Box::new([]),
            },
            run::Directory {
                host: Box::from(b"/host".as_slice()),
                name: b"archive".as_slice().into(),
                root: Token::new(disk.root(b"archive")),
                writable: false,
                git: false,
                conflicts: Box::new([]),
            },
        ]),
    };
    (disk, mounted)
}

fn call(name: &[u8], arguments: &[u8]) -> Line {
    Line::Call { name: name.into(), arguments: arguments.into() }
}

fn calls(lines: Vec<Line>, tokens: u64) -> Turn {
    smith_protocol_llm_world::fixture::calls(lines.into(), tokens)
}

fn says(text: &[u8], tokens: u64) -> Turn {
    smith_protocol_llm_world::fixture::says(Box::new([Line::Text { text: text.into() }]), tokens)
}

fn contains(bytes: &[u8], part: &[u8]) -> bool {
    skein_lib::bytes::find(bytes, part).is_some()
}

fn text(text: &[u8]) -> Part {
    Part::Text { text: text.into() }
}

fn message(role: Role, parts: Vec<Part>) -> Message {
    Message { role, parts: parts.into() }
}

fn part_bytes(part: &Part) -> usize {
    match part {
        Part::Text { text } | Part::Opaque { bytes: text } => text.len(),
        Part::ToolCall { id, name, arguments } => id.len() + name.len() + arguments.len(),
        Part::ToolOutput { id, output, .. } => id.len() + output.len(),
    }
}

fn observe_prefix_and_usage(
    world: &World,
    prefixes: &[Vec<Message>],
    outputs: &[u64],
    cache_write: bool,
    reasoning: Option<u64>,
) {
    assert_eq!(world.prompts().len(), prefixes.len());
    assert_eq!(world.turns().len(), prefixes.len());
    let requests = world
        .protocol_calls()
        .iter()
        .map(|binding| smith_protocol_llm_world::referee::NativeRequest::from_http(&binding.request))
        .collect::<Vec<_>>();
    for adjacent in requests.windows(2) {
        assert_eq!(adjacent[0].system, adjacent[1].system, "exact native system bytes");
        assert_eq!(adjacent[0].tools, adjacent[1].tools, "exact native tool-schema bytes");
        assert!(adjacent[1].history.starts_with(&adjacent[0].history), "exact native history items form a byte prefix");
    }
    for (index, ((query, turn), prefix)) in world.prompts().iter().zip(world.turns()).zip(prefixes).enumerate() {
        assert_eq!(query.model.as_ref(), b"fake-1");
        assert_eq!(query.messages.as_ref(), prefix, "whole native history and concrete feedback at completion {index}");
        let fixed = query.system.len()
            + query
                .tools
                .iter()
                .map(|tool| tool.name.len() + tool.description.len() + tool.parameters.len())
                .sum::<usize>();
        let prompt_bytes = fixed + prefix.iter().flat_map(|message| &message.parts).map(part_bytes).sum::<usize>();
        let tokens = u64::try_from(prompt_bytes / 4).expect("bounded outside fixture");
        // Only an earlier observed request wrote its prompt. Its answer was not
        // written until the next request includes it. A new World has no writes.
        let read = if index == 0 {
            0
        } else {
            let previous = &prefixes[index - 1];
            u64::try_from(
                (fixed + previous.iter().flat_map(|message| &message.parts).map(part_bytes).sum::<usize>()) / 4,
            )
            .expect("bounded previously written prompt")
        };
        let fresh_tokens = tokens - read;
        let input = if cache_write { 0 } else { fresh_tokens };
        let write = if cache_write { Some(fresh_tokens) } else { None };
        assert_eq!(
            [
                turn.usage.input_tokens,
                turn.usage.output_tokens,
                turn.usage.cache_read_tokens,
                turn.usage.cache_write_tokens
            ],
            [Some(input), Some(outputs[index]), Some(read), write],
            "all four usage fields independently priced from literal system and native query"
        );
        assert_eq!(turn.usage.reasoning_tokens, reasoning, "literal peer reasoning reporting");
    }
}

#[test]
fn five_native_completions_keep_the_wire_prefix_and_replay_each_previous_answer() {
    for dialect in smith_protocol_llm_world::fixture::Dialect::ALL {
        let mut bounds = smith_protocol_llm::fixture::small(skein_llm_kit::limits(), codec::stated());
        bounds.client.http.request = 16384;
        bounds.client.request = 16384;
        let mut settings = Settings {
            job: Job::Waiting,
            waiting: Duration::from_secs(1),
            network: Span::millis(1, 1),
            ..smith_protocol_llm_world::fixture::settings(1103)
        };
        settings.limits.session.completion_bytes =
            codec::completion_bound(&bounds.client, settings.limits.decoded_call_bytes)
                .expect("observed receiving reservation");
        settings.limits.session.completion_blocks = bounds.client.output_items;
        settings.limits.session.failure_bytes = bounds.client.detail_bytes;
        let scripts = Box::new([Script {
            cue: b"@brief-native".as_slice().into(),
            turns: Box::new([
                calls(vec![call(b"read", br#"{"path":"data.txt"}"#)], 2),
                calls(vec![call(b"read", br#"{"path":"data.txt"}"#)], 3),
                calls(vec![call(b"read", br#"{"path":"data.txt"}"#)], 5),
                calls(vec![call(b"read", br#"{"path":"data.txt"}"#)], 7),
                calls(vec![Line::Text { text: FIRST.into() }, call(b"wait", b"{}")], 11),
            ]),
        }]);
        let (disk, mounted) = workspace();
        let mut world = World::with_workspace_protocol_charter(
            settings,
            None,
            Some(mounted),
            disk,
            (dialect.configuration(), bounds),
            scripts,
            charter(&settings, NATIVE_INSTRUCTIONS),
        );
        world.run(100_000);
        let mut history = vec![message(Role::User, vec![text(BEGIN)])];
        let mut prefixes = vec![history.clone()];
        for id in
            [b"call_0000000000000001", b"call_0000000000000002", b"call_0000000000000003", b"call_0000000000000004"]
        {
            history.extend([
                message(
                    Role::Assistant,
                    vec![Part::ToolCall {
                        id: id.as_slice().into(),
                        name: b"read".as_slice().into(),
                        arguments: br#"{"path":"data.txt"}"#.as_slice().into(),
                    }],
                ),
                message(
                    Role::User,
                    vec![Part::ToolOutput {
                        id: id.as_slice().into(),
                        output: b"1: old\n(1 of 1 lines; skipped 0) [end of file]".as_slice().into(),
                        is_error: false,
                    }],
                ),
            ]);
            prefixes.push(history.clone());
        }
        observe_prefix_and_usage(&world, &prefixes, &[2, 3, 5, 7, 11], true, dialect.reasoning());
        assert_eq!(world.protocol_calls().len(), 5, "five observed native Client requests");
    }
}

#[test]
fn anthropic_breakpoints_account_for_thinking_and_a_large_tool_batch() {
    const THINKING: &[u8] =
        br#"{"type":"thinking","thinking":"Inspect the whole batch.","signature":"fixture-signature"}"#;
    let declared = skein_llm::Declared {
        tools: 16,
        history_items: 256,
        window: 342,
        output: 4096,
        reasoning_item: 2048,
        tool_payload: 342,
        calls_per_response: 16,
        conversations: 1,
    };
    let mut client = skein_llm::Limits::derive(&declared).expect("native batch bounds").anthropic;
    let pieces = skein_llm_kit::limits();
    client.http.request = pieces.http.request;
    client.http.read = pieces.http.read;
    client.http.send = pieces.http.send;
    client.sse.chunk = pieces.sse.chunk;
    let bounds = smith_protocol_llm::fixture::small(client, codec::stated());
    let mut settings = Settings {
        job: Job::Waiting,
        waiting: Duration::from_secs(1),
        network: Span::millis(1, 1),
        ..smith_protocol_llm_world::fixture::settings(1104)
    };
    settings.limits.decoded_call_bytes = u64::from(declared.calls_per_response)
        .checked_mul(u64::from(bounds.client.input))
        .and_then(|payload| {
            payload.checked_add(
                u64::from(bounds.client.output_items)
                    .checked_mul(u64::try_from(core::mem::size_of::<llm::Decoded>()).expect("decoded cell"))?,
            )
        })
        .expect("observed batch inputs plus fallback cells");
    settings.limits.session.completion_bytes =
        codec::completion_bound(&bounds.client, settings.limits.decoded_call_bytes)
            .expect("observed batch and history receiving reservation");
    settings.limits.session.completion_blocks = bounds.client.output_items;
    settings.limits.session.failure_bytes = bounds.client.detail_bytes;
    let batch = 16_usize;
    // Each read asks a window its whole result fits, so every result stays below the line (smith/docs/design/domain/session.md, 8.1).
    let read = || call(b"read", br#"{"path":"data.txt","window":64}"#);
    let scripts = Box::new([Script {
        cue: b"@brief-native".as_slice().into(),
        turns: Box::new([
            calls(vec![Line::Opaque { bytes: THINKING.into() }, read()], 2),
            calls((0..batch).map(|_| read()).collect(), 3),
            calls(vec![Line::Text { text: FIRST.into() }, call(b"wait", b"{}")], 5),
        ]),
    }]);
    let (disk, mounted) = workspace();
    let mut world = World::with_workspace_protocol_charter(
        settings,
        None,
        Some(mounted),
        disk,
        (client_peer::configurations().into_iter().nth(1).expect("Anthropic fixture"), bounds),
        scripts,
        charter(&settings, NATIVE_INSTRUCTIONS),
    );
    world.run(100_000);
    let mut history = vec![message(Role::User, vec![text(BEGIN)])];
    let mut prefixes = vec![history.clone()];
    let read_call = |number: usize| Part::ToolCall {
        id: format!("call_{number:016x}").into_bytes().into(),
        name: b"read".as_slice().into(),
        arguments: br#"{"path":"data.txt","window":64}"#.as_slice().into(),
    };
    let read_result = |number: usize| Part::ToolOutput {
        id: format!("call_{number:016x}").into_bytes().into(),
        output: b"1: old\n(1 of 1 lines; skipped 0) [end of file]".as_slice().into(),
        is_error: false,
    };
    history.extend([
        message(Role::Assistant, vec![Part::Opaque { bytes: THINKING.into() }, read_call(1)]),
        message(Role::User, vec![read_result(1)]),
    ]);
    prefixes.push(history.clone());
    history.extend([
        message(Role::Assistant, (2..batch + 2).map(read_call).collect()),
        message(Role::User, (2..batch + 2).map(read_result).collect()),
    ]);
    prefixes.push(history);

    observe_prefix_and_usage(&world, &prefixes, &[2, 3, 5], true, None);
    let turns = world.turns();
    let largest = turns
        .iter()
        .map(|turn| turn.messages.iter().map(|message| message.content.len()).sum::<usize>())
        .max()
        .expect("three captured tool turns");
    assert_eq!(largest, batch * 2, "full call batch followed by every concrete result");
    assert_eq!(batch, 16, "the scripted call batch measured independently of history room");
    assert!(turns.iter().skip(1).all(|turn| turn.usage.cache_read_tokens.unwrap_or(0) > 0));
    assert!(turns.iter().all(|turn| turn.usage.cache_write_tokens.unwrap_or(0) > 0));
}

fn affinity_scripts() -> Box<[Script]> {
    Box::new([
        Script {
            cue: b"@brief-native".as_slice().into(),
            turns: Box::new([
                calls(vec![Line::Text { text: FIRST.into() }, call(b"wait", b"{}")], 3),
                calls(vec![call(b"sub_agent", br#"{"brief":"@affinity-child","tools":[]}"#)], 3),
                calls(vec![Line::Text { text: LAST.into() }, call(b"wait", b"{}")], 3),
            ]),
        },
        Script { cue: b"@affinity-child".as_slice().into(), turns: Box::new([says(b"Child retains the run key.", 3)]) },
    ])
}

#[test]
fn identityless_resume_keeps_the_recorded_run_key_for_main_and_a_new_child() {
    for dialect in smith_protocol_llm_world::fixture::Dialect::ALL {
        let mut bounds = smith_protocol_llm::fixture::small(skein_llm_kit::limits(), codec::stated());
        bounds.client.http.request = 16384;
        bounds.client.request = 16384;
        let mut settings = Settings {
            job: Job::Waiting,
            waiting: Duration::from_secs(1),
            network: Span::millis(1, 1),
            ..smith_protocol_llm_world::fixture::settings(1110)
        };
        settings.limits.session.completion_bytes =
            codec::completion_bound(&bounds.client, settings.limits.decoded_call_bytes)
                .expect("observed receiving reservation");
        settings.limits.session.completion_blocks = bounds.client.output_items;
        settings.limits.session.failure_bytes = bounds.client.detail_bytes;
        let (disk, mounted) = workspace();
        let mut first = World::with_workspace_protocol_charter(
            settings,
            None,
            Some(mounted),
            disk,
            (dialect.configuration(), bounds),
            affinity_scripts(),
            charter(&settings, NATIVE_INSTRUCTIONS),
        );
        first.run(100_000);
        assert_eq!(first.turns().len(), 1);
        assert!(matches!(first.answer(), run::Answer::Parked { .. }));
        let affinity = match &first.turns()[0].kind {
            smith_domain::session::record::Kind::First(opener) => opener.affinity,
            smith_domain::session::record::Kind::Ordinary | smith_domain::session::record::Kind::Window(_) => {
                panic!("fresh opening requires a first turn")
            }
        };
        assert_eq!(affinity.thread, 0);
        let original = smith_protocol_llm_world::referee::NativeRequest::from_http(&first.protocol_calls()[0].request);
        let saved = Transcript {
            version: first.turns()[0].version,
            endpoint: first.turns()[0].endpoint,
            dialect: first.turns()[0].dialect,
            turns: first.turns().into(),
        };
        settings.resume = true;
        settings.seed = 999_110;
        let (disk, mounted) = workspace();
        let mut resumed = World::with_workspace_protocol_charter(
            settings,
            Some(saved),
            Some(mounted),
            disk,
            (dialect.configuration(), bounds),
            affinity_scripts(),
            charter(&settings, NATIVE_INSTRUCTIONS),
        );
        resumed.run(100_000);
        assert!(matches!(resumed.answer(), run::Answer::Parked { .. }));
        assert_eq!(
            resumed.turns().iter().map(|turn| (turn.sequence, turn.kind.clone())).collect::<Vec<_>>(),
            [(2, smith_domain::session::record::Kind::Ordinary), (3, smith_domain::session::record::Kind::Ordinary),]
        );
        assert_eq!(resumed.prompts().len(), 3, "two main requests and one observed child request");
        for (query, binding) in resumed.prompts().iter().zip(resumed.protocol_calls()) {
            if original.affinity.is_some() {
                assert_eq!(query.caching, skein_fake_llm_domain::api::Caching::Scope(affinity.key));
            } else {
                assert!(
                    matches!(query.caching, skein_fake_llm_domain::api::Caching::Marks(_)),
                    "Anthropic ignores affinity and uses its native breakpoints"
                );
            }
            let observed = smith_protocol_llm_world::referee::NativeRequest::from_http(&binding.request);
            match (&original.affinity, &observed.affinity) {
                (Some(original), Some(observed)) => {
                    assert_eq!(observed, original, "the recorded run cache key survives the new seed");
                }
                (None, None) => {}
                other => panic!("both requests use the same dialect: {other:?}"),
            }
            assert!(binding.terminal.is_some(), "component terminal settles every request");
        }
    }
}

mod open;
