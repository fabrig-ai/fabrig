//! The answered chat's script, authority and small model menu (service-testing.md, 1).
use jig_channel_kit::fixtures as channel;
use skein_fake_llm_domain::Domain;
use skein_fake_llm_domain::api::{Bounds, Capture, Finish, Line, Menu, Rules, Script, Scripts, Seeds, Slot, Turn};
use skein_world::streams::{Stream, Streams};

const FAMILIES: jig_domain_core::ToolFamilies = jig_domain_core::ToolFamilies::JIG;
pub(crate) const TOOLS: u64 = FAMILIES.delegate.0 | FAMILIES.message.0 | FAMILIES.control.0 | FAMILIES.subscribe.0;
// delegate; message and introduce; cancel, amend and release; subscribe and unsubscribe.
const HOST_TOOLS: u32 = 8;
// One call before and one after cancel suffice for the two refusal classes.
pub(crate) const ROUNDS: usize = 1;
const CALLS_PER_ANSWER: u32 = 1;
// Four families carry the full host tool schemas, including delegate and amend.
const CHARTER_BYTES: u64 = 32_768;
// Five model answers, a reserve turn, and two turns of headroom.
const RUN_TURNS: u32 = 8;
pub(crate) const DELEGATE: u64 = 2;
pub(crate) const NEVER_ISSUED: u64 = 999_999;
const TASK_WEIGHT: u32 = 150;
const QUESTION_WEIGHT: u32 = 300;

pub(crate) fn document(engine: &str) -> String {
    engine
        .replace(r#""tools": 0"#, &format!(r#""tools": {TOOLS}"#))
        .replace(
            r#"{"kind": "charter", "number": 1}"#,
            r#"{"kind": "charter", "number": 1}, {"kind": "role", "number": 0}"#,
        )
        .replace(r#""tasks.project_tasks":1"#, r#""tasks.project_tasks":2"#)
        .replace(r#""tasks.tree_tasks":1"#, r#""tasks.tree_tasks":2"#)
        .replace(r#""turns": 4"#, &format!(r#""turns": {RUN_TURNS}"#))
}

pub(crate) fn agent_configuration() -> smith_agent_service::Config {
    channel::agent_configuration_for_host_tools(HOST_TOOLS)
}

pub(crate) fn worker_agent() -> jig_test_service::Agent {
    let mut agent = channel::agent_with_limits(&agent_configuration().limits);
    agent.host_limits.charter_bytes = CHARTER_BYTES;
    agent.bounds.turns = RUN_TURNS;
    agent
}

pub(crate) fn provider(seed: u64) -> skein_fake_peers::llm::Peer {
    channel::provider_with_domain(
        (smith_agent_process_world::fake::loopback(), 443).into(),
        skein_llm::Credential::bearer(channel::TOKEN.into()),
        skein_fake_peers::Transport::Plaintext,
        skein_fake_llm_protocol::documents::Provider::OpenAi,
        |config| {
            config.calls_per_answer = CALLS_PER_ANSWER;
            let streams = Streams::new(seed);
            Domain::drawn(
                config,
                Seeds::Parts {
                    agent: streams.seed_of(Stream::Parties, "agent/model"),
                    faults: streams.seed_of(Stream::Faults, "llm"),
                    scheduling: streams.seed_of(Stream::Scheduling, "llm"),
                },
                Scripts {
                    cued: Box::new([Script { cue: Box::default(), turns: turns() }]),
                    asked: Box::new([]),
                    otherwise: Box::new([]),
                },
                menu(),
                Rules { cued: Box::new([]), asked: Box::new([]) },
                Capture::none(),
            )
            .expect("bounded drawn chat script and menu")
        },
    )
}

fn turns() -> Box<[Turn]> {
    let call = |name: &[u8], arguments: &[u8]| Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    };
    let stretch = || Turn {
        lines: Box::new([Line::Drawn {
            rounds: u32::try_from(ROUNDS).expect("stretch rounds"),
            tools: Box::new([b"message".as_slice().into(), b"subscribe".as_slice().into()]),
        }]),
        finish: Finish::ToolCalls,
        tokens: 0,
    };
    Box::new([
        call(b"delegate", br#"{"batch":[{"executor":{"kind":"person","person":1},"spec":{"words":"Help"},"contract":{"kind":"report","words":32},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":0},"notes":0}}]}"#),
        stretch(),
        call(b"cancel", br#"{"target":2,"reason":"No longer needed"}"#),
        stretch(),
        call(b"finish", br#"{"form":"report","text":"World answer","fields":{}}"#),
    ])
}

fn menu() -> Menu {
    // The fake draws a name and a body independently. `kind` serves subscribe;
    // `form`, `words` and `question` serve message.
    Menu {
        arguments: Box::new([
            br#"{"target":"{{#task}}","form":"words","words":"drawn","kind":"task"}"#.as_slice().into(),
            br#"{"target":"{{#task}}","form":"question","words":"drawn","kind":"task"}"#.as_slice().into(),
            br#"{"target":"{{#task}}","form":"answer","words":"drawn","kind":"task","question":"{{#question}}"}"#
                .as_slice()
                .into(),
        ]),
        invalid: Box::new([]),
        slots: Box::new([
            slot(b"task", b"Delegated tasks: ", b" at", TASK_WEIGHT),
            slot(b"question", b"Sent message ", b"", QUESTION_WEIGHT),
        ]),
    }
}

fn slot(name: &[u8], before: &[u8], after: &[u8], weight: u32) -> Slot {
    Slot {
        name: name.into(),
        before: before.into(),
        after: after.into(),
        with: Box::new([]),
        foreign: 0,
        unissued: NEVER_ISSUED.to_string().into_bytes().into_boxed_slice(),
        weight,
        numeric: Some(Bounds { low: 1, high: NEVER_ISSUED - 1 }),
    }
}
