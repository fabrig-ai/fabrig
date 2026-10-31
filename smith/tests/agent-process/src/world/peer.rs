//! Provider scripts and charter edits for the process world's scenarios.
//! The byte peer owns transport (smith/docs/design/testing.md, sections 4 and 5).
use crate::fake;
use skein_lib::Duration;

/// The run terminal selected by the process scenario's inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TraceCase {
    /// The independent provider calls the granted finish tool.
    Accepted,
    /// The one-turn budget ends after a plaintext response.
    Failed,
    /// A charter model absent from configured declarations is refused before admission.
    Refused,
    /// A settled wait parks immediately under its admitted idle policy.
    Parked,
    /// The parent signals after observing admission.
    Cancelled,
    /// A real delegated child answers before main's accepted finish.
    ChildAnswered,
    /// One native completion calls three children together, each of which answers.
    ThreeChildren,
    /// The independent provider sends a typed HTTP context rejection.
    ProviderRejected,
    /// A read is told, then reasoning stops without a finish and needs a nudge.
    StoppedReasoning,
}

impl TraceCase {
    pub(super) fn peer(self, anthropic: bool, credential: skein_llm::Credential) -> skein_fake_peers::llm::Peer {
        use skein_fake_llm_domain::api;
        if self == Self::ChildAnswered || self == Self::ThreeChildren {
            return child_peer(anthropic, self == Self::ThreeChildren, credential);
        }
        if self == Self::StoppedReasoning {
            return stopped_reasoning_peer();
        }
        let turns = match self {
            Self::Accepted => vec![api::Turn {
                lines: Box::new([api::Line::Call {
                    name: b"finish".as_slice().into(),
                    arguments: br#"{"form":"report","text":"done","fields":{}}"#.as_slice().into(),
                }]),
                finish: api::Finish::ToolCalls,
                tokens: 1,
            }],
            Self::Parked => vec![
                api::Turn {
                    lines: Box::new([api::Line::Call {
                        name: b"wait".as_slice().into(),
                        arguments: b"{}".as_slice().into(),
                    }]),
                    finish: api::Finish::ToolCalls,
                    tokens: 1,
                },
                api::Turn {
                    lines: Box::new([api::Line::Text { text: b"Waiting".as_slice().into() }]),
                    finish: api::Finish::Stop,
                    tokens: 1,
                },
            ],
            Self::Failed | Self::Refused | Self::Cancelled => return fake::peer_for(anthropic),
            Self::ProviderRejected => return fake::rejected(),
            Self::ChildAnswered | Self::ThreeChildren | Self::StoppedReasoning => {
                unreachable!("selected child scripts above")
            }
        };
        fake::configured(
            Box::new([api::Script { cue: Box::new([]), turns: turns.into() }]),
            fake::credential(false),
            Duration::ZERO,
        )
    }
}

fn child_peer(anthropic: bool, three: bool, credential: skein_llm::Credential) -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api;
    let count = if three { 3 } else { 1 };
    let mut scripts = Vec::new();
    let mut calls = Vec::new();
    for index in 0..count {
        let cue = if three { format!("@event-child-{index}") } else { "@event-child".to_owned() };
        scripts.push(api::Script {
            cue: cue.clone().into_bytes().into_boxed_slice(),
            turns: Box::new([api::Turn {
                lines: Box::new([api::Line::Text { text: b"child answer".as_slice().into() }]),
                finish: api::Finish::Stop,
                tokens: 1,
            }]),
        });
        calls.push(api::Line::Call {
            name: b"sub_agent".as_slice().into(),
            arguments: format!("{{\"brief\":\"{cue}\",\"tools\":[],\"max_turns\":2,\"max_seconds\":30}}")
                .into_bytes()
                .into_boxed_slice(),
        });
    }
    scripts.push(api::Script {
        cue: b"@event-main".as_slice().into(),
        turns: Box::new([
            api::Turn { lines: calls.into_boxed_slice(), finish: api::Finish::ToolCalls, tokens: 1 },
            api::Turn {
                lines: Box::new([api::Line::Call {
                    name: b"finish".as_slice().into(),
                    arguments: br#"{"form":"report","text":"done","fields":{}}"#.as_slice().into(),
                }]),
                finish: api::Finish::ToolCalls,
                tokens: 1,
            },
        ]),
    });
    if anthropic {
        fake::configured_anthropic(scripts.into_boxed_slice(), credential)
    } else {
        fake::configured(scripts.into_boxed_slice(), credential, Duration::ZERO)
    }
}

pub(super) fn child_charter(bytes: &[u8]) -> Box<[u8]> {
    crate::wire::edit_charter(bytes, |parts| {
        parts.instructions = b"@event-main".as_slice().into();
        crate::wire::budget(parts, 8, 100);
        let mut models = skein_lib::List::with_capacity(1);
        models.push(parts.main.clone()).expect("one permitted child model");
        parts.models = models;
        let mut tools = parts.tools.clone().into_parts();
        let mut families = tools.families.into_parts();
        families.skein_bools[0] = true;
        families.skein_bools[3] = true;
        tools.families = smith_charter::Families::new(&smith_charter::CEILINGS, families).expect("child authority");
        parts.tools = smith_charter::Tools::new(&smith_charter::CEILINGS, tools).expect("tools");
    })
}

pub(super) fn environment_peer() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api;
    fake::configured(
        Box::new([api::Script {
            cue: Box::new([]),
            turns: Box::new([
                api::Turn {
                    lines: Box::new([api::Line::Call {
                        name: b"shell".as_slice().into(),
                        arguments: br#"{"command":"true"}"#.as_slice().into(),
                    }]),
                    finish: api::Finish::ToolCalls,
                    tokens: 1,
                },
                api::Turn {
                    lines: Box::new([api::Line::Text { text: b"done".as_slice().into() }]),
                    finish: api::Finish::Stop,
                    tokens: 1,
                },
            ]),
        }]),
        fake::credential(false),
        Duration::ZERO,
    )
}

fn stopped_reasoning_peer() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api;
    fake::reasoning(Box::new([api::Script {
        cue: Box::new([]),
        turns: Box::new([
            api::Turn {
                lines: Box::new([api::Line::Call {
                    name: b"read".as_slice().into(),
                    arguments: br#"{"path":"problem.txt"}"#.as_slice().into(),
                }]),
                finish: api::Finish::ToolCalls,
                tokens: 1,
            },
            api::Turn {
                lines: Box::new([
                    api::Line::Opaque {
                        bytes: format!(
                            r#"{{"id":"r2","type":"reasoning","encrypted_content":"{}","summary":[]}}"#,
                            "r".repeat(43_688)
                        )
                        .into_bytes()
                        .into(),
                    },
                    api::Line::Opaque {
                        bytes: format!(
                            r#"{{"id":"r3","type":"reasoning","encrypted_content":"{}","summary":[]}}"#,
                            "r".repeat(43_688)
                        )
                        .into_bytes()
                        .into(),
                    },
                    api::Line::Opaque {
                        bytes: format!(
                            r#"{{"id":"{}","type":"message","phase":"final_answer","role":"assistant","content":[{{"type":"output_text","text":"{}","annotations":[]}}]}}"#,
                            "m".repeat(54),
                            "t".repeat(903)
                        )
                        .into_bytes()
                        .into(),
                    },
                ]),
                finish: api::Finish::Stop,
                tokens: 1,
            },
            api::Turn {
                lines: Box::new([api::Line::Call {
                    name: b"finish".as_slice().into(),
                    arguments: br#"{"form":"report","text":"REASONING COMPLETE","fields":{}}"#.as_slice().into(),
                }]),
                finish: api::Finish::ToolCalls,
                tokens: 1,
            },
        ]),
    }]))
}

/// The provider behavior selected from the world's independent inputs.
pub(super) enum Peer<'a> {
    Scripted(&'a [skein_fake_llm_domain::api::Turn]),
    Refreshing,
    Environment,
    FullCompletion,
    Tracing(TraceCase),
    Ordinary,
}

impl<'a> Peer<'a> {
    pub fn select(world: &'a super::World) -> Self {
        match (
            world.script.as_deref(),
            world.refreshing.is_some(),
            world.environment,
            world.full_completion,
            world.setup.tracing,
        ) {
            (Some(turns), _, _, _, _) => Self::Scripted(turns),
            (None, true, _, _, _) => Self::Refreshing,
            (None, false, true, _, _) => Self::Environment,
            (None, false, false, true, _) => Self::FullCompletion,
            (None, false, false, false, Some((case, _))) => Self::Tracing(case),
            (None, false, false, false, None) => Self::Ordinary,
        }
    }

    pub fn make(self, world: &super::World) -> skein_fake_peers::llm::Peer {
        match self {
            Self::Scripted(turns) => fake::scripted(turns.into(), world.credential(), world.anthropic),
            Self::Refreshing => fake::configured(
                Box::new([skein_fake_llm_domain::api::Script {
                    cue: Box::new([]),
                    turns: Box::new([skein_fake_llm_domain::api::Turn {
                        lines: Box::new([skein_fake_llm_domain::api::Line::Text { text: Box::from(&b"Hello"[..]) }]),
                        finish: skein_fake_llm_domain::api::Finish::Stop,
                        tokens: 1,
                    }]),
                }]),
                crate::scenario_credential(false, false),
                Duration::from_secs(1),
            ),
            Self::Environment => environment_peer(),
            Self::FullCompletion => full_completion(),
            Self::Tracing(case) => case.peer(world.anthropic, world.credential()),
            Self::Ordinary => fake::peer(),
        }
    }
}

fn full_completion() -> skein_fake_peers::llm::Peer {
    fake::configured(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: Box::new([]),
            turns: Box::new([skein_fake_llm_domain::api::Turn {
                lines: (0..crate::limits().llm.adapter.client.output_items)
                    .map(|index| skein_fake_llm_domain::api::Line::Text {
                        // First response metadata is item_1_<index> and final_answer.
                        text: vec![
                            b'x';
                            usize::try_from(
                                crate::limits().llm.adapter.client.answer
                                    / crate::limits().llm.adapter.client.output_items
                                    + u32::from(
                                        index
                                            < crate::limits().llm.adapter.client.answer
                                                % crate::limits().llm.adapter.client.output_items
                                    )
                            )
                            .expect("bounded text per part")
                                - format!("item_1_{index}").len()
                                - b"final_answer".len()
                                - 9
                        ]
                        .into_boxed_slice(),
                    })
                    .collect(),
                finish: skein_fake_llm_domain::api::Finish::Stop,
                tokens: 1,
            }]),
        }]),
        fake::credential(false),
        Duration::ZERO,
    )
}
