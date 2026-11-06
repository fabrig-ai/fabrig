//! A committed chat reply followed by a parked run on the real transport.
use skein_fake_llm_domain::api;

pub const WORDS: &str = "Hello";
pub const REPLY: &[u8] = b"World answer";
pub const EARLY: &[u8] = b"World";

#[must_use]
pub fn chat_scripts(chats: usize) -> Box<[api::Script]> {
    (0..chats)
        .map(|_| api::Script {
            cue: Box::default(),
            turns: Box::new([api::Turn {
                lines: Box::new([
                    api::Line::Text { text: REPLY.into() },
                    api::Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                ]),
                finish: api::Finish::ToolCalls,
                tokens: 1,
            }]),
        })
        .collect()
}

/// One chat reads two documentation topics in successive turns, then replies and parks.
#[must_use]
pub fn call_line_scripts() -> Box<[api::Script]> {
    let mut turns: Vec<_> = [br#"{"topic":"hold"}"#.as_slice(), br#"{"topic":"chat"}"#.as_slice()]
        .into_iter()
        .map(|arguments| api::Turn {
            lines: Box::new([api::Line::Call { name: b"read_docs".as_slice().into(), arguments: arguments.into() }]),
            finish: api::Finish::ToolCalls,
            tokens: 1,
        })
        .collect();
    turns.push(api::Turn {
        lines: Box::new([
            api::Line::Text { text: REPLY.into() },
            api::Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        finish: api::Finish::ToolCalls,
        tokens: 1,
    });
    Box::new([api::Script { cue: Box::default(), turns: turns.into_boxed_slice() }])
}

/// The reply's distinguishable prefixes and the hold before its suffix.
pub const FIRST: &[u8] = b"The answer";
pub const SECOND: &[u8] = b" is forty-two.";
pub const LIVE: &[u8] = b"live-reply";

/// One chat writes a prefix, waits for its reader, then completes and parks.
#[must_use]
pub fn live_scripts() -> Box<[api::Script]> {
    Box::new([api::Script {
        cue: Box::default(),
        turns: Box::new([api::Turn {
            lines: Box::new([
                api::Line::Pieces {
                    pieces: Box::new([
                        api::Piece { text: FIRST.into(), hold: None },
                        api::Piece { text: SECOND.into(), hold: Some(LIVE.into()) },
                    ]),
                },
                api::Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
            ]),
            finish: api::Finish::ToolCalls,
            tokens: 1,
        }]),
    }])
}
