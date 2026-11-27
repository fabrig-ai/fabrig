//! The delegate's withheld wait (jig/docs/design/domain/testing-cells.md, 5).
use jig_channel_kit::fixtures as channel;
use skein_fake_llm_domain::{
    Domain,
    api::{Finish, Line, Menu, Script, Scripts, Turn},
};

pub(crate) const TOOLS: u64 = jig_domain_core::ToolFamilies::JIG.delegate.0;
pub(crate) const NOTICE: &[u8] =
    b"Nothing more will come: your delegates have ended and no question waits. Finish with your result.";
pub(crate) const CHILD_RESULT: &[u8] = b"Delegate done";
const CHILD_SPEC: &[u8] = b"Wait alone";

pub(crate) fn document(engine: &str) -> String {
    engine
        .replace(r#""tools": 0"#, &format!(r#""tools": {TOOLS}"#))
        .replace(r#""tasks.project_tasks":1"#, r#""tasks.project_tasks":2"#)
        .replace(r#""tasks.tree_tasks":1"#, r#""tasks.tree_tasks":2"#)
        // A single agent place: the chat parks awaiting its delegate, then the delegate runs.
        .replace(r#""waiting_ms": 10000"#, r#""waiting_ms": 0"#)
}

pub(crate) fn worker_agent() -> jig_test_service::Agent {
    let mut agent = channel::agent();
    // The delegate schema is part of the charter.
    agent.host_limits.charter_bytes = 16_384;
    agent
}

pub(crate) fn provider(seed: u64) -> skein_fake_peers::llm::Peer {
    channel::provider_with_domain(
        (smith_agent_process_world::fake::loopback(), 443).into(),
        skein_llm::Credential::bearer(channel::TOKEN.into()),
        skein_fake_peers::Transport::Plaintext,
        skein_fake_llm_protocol::documents::Provider::OpenAi,
        |config| {
            let call = |name: &[u8], arguments: &[u8]| Turn {
                lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
                finish: Finish::ToolCalls,
                tokens: 1,
            };
            let delegate = serde_json::json!({"batch":[{
                "executor":{"kind":"agent","charter":1},
                "spec":{"words":std::str::from_utf8(CHILD_SPEC).expect("child words")},
                "contract":{"kind":"report","words":32},
                "authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},
                    "budget":{"spend":20},"notes":0}
            }]})
            .to_string();
            let finish = |words: &[u8]| {
                let result = serde_json::json!({"form":"report","text":std::str::from_utf8(words).expect("report words"),"fields":{}}).to_string();
                call(b"finish", result.as_bytes())
            };
            Domain::played(
                config,
                seed,
                Scripts {
                    cued: Box::new([
                        Script {
                            cue: b"Hello".as_slice().into(),
                            turns: Box::new([
                                call(b"delegate", delegate.as_bytes()),
                                call(b"wait", b"{}"),
                                finish(super::ANSWER),
                            ]),
                        },
                        Script {
                            cue: CHILD_SPEC.into(),
                            turns: Box::new([call(b"wait", b"{}"), finish(CHILD_RESULT)]),
                        },
                    ]),
                    asked: Box::new([]),
                    otherwise: Box::new([]),
                },
                Menu { arguments: Box::new([]), invalid: Box::new([]), slots: Box::new([]) },
            )
            .expect("bounded park story")
        },
    )
}
