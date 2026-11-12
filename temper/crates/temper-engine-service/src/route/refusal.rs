//! The fleet's reason in a start's words (protocol/cluster.md, 3).
use alloc::boxed::Box;
use jig_domain_fleet::{AgentShortfall, HelloRefusal, HelloRefused};
use skein_lib::{Decimal, Writer};

/// The refusal's text and the engine's line (protocol/cluster.md, 3): at most 256 bytes.
pub(super) fn text(hello: Option<HelloRefused>) -> Box<[u8]> {
    let Some(hello) = hello else { return b"not admitted".as_slice().into() };
    match hello.why {
        HelloRefusal::Agent { why } => {
            let prefix = match why {
                AgentShortfall::Model { charter, model }
                | AgentShortfall::Window { charter, model }
                | AgentShortfall::Output { charter, model } => joined(&[
                    b"charter ",
                    Decimal::of(u64::from(charter)).as_bytes(),
                    b", model ",
                    Decimal::of(u64::from(model)).as_bytes(),
                    b": ",
                ]),
                AgentShortfall::Turns { charter }
                | AgentShortfall::Time { charter }
                | AgentShortfall::Waiting { charter }
                | AgentShortfall::Checks { charter } => {
                    joined(&[b"charter ", Decimal::of(u64::from(charter)).as_bytes(), b": "])
                }
                AgentShortfall::Spend
                | AgentShortfall::Message
                | AgentShortfall::Inbox
                | AgentShortfall::Unacknowledged => Box::default(),
            };
            let words = match hello.figures {
                Some(figures) => crate::startup::admission_words(why, figures.offered, figures.required),
                None => match why {
                    AgentShortfall::Model { .. } => crate::startup::admission_words(why, 0, 0),
                    AgentShortfall::Window { .. }
                    | AgentShortfall::Output { .. }
                    | AgentShortfall::Turns { .. }
                    | AgentShortfall::Time { .. }
                    | AgentShortfall::Waiting { .. }
                    | AgentShortfall::Checks { .. }
                    | AgentShortfall::Spend
                    | AgentShortfall::Message
                    | AgentShortfall::Inbox
                    | AgentShortfall::Unacknowledged => b"not admitted".as_slice().into(),
                },
            };
            joined(&[&prefix, &words])
        }
        HelloRefusal::JobKinds => b"its job kinds are not ascending and distinct".as_slice().into(),
        HelloRefusal::ReservedChannel => b"channel zero is the engine's own host".as_slice().into(),
        HelloRefusal::Workers { limit } => {
            joined(&[b"no room: fleet.workers is ", Decimal::of(u64::from(limit)).as_bytes()])
        }
        HelloRefusal::Position { worker } => joined(&[
            b"worker position ",
            Decimal::of(u64::from(worker)).as_bytes(),
            b" is not configured or is held by a live channel",
        ]),
        HelloRefusal::StopBound { grace } => joined(&[
            b"its stop bound is not shorter than the engine's grace of ",
            Decimal::of(grace.as_nanos().checked_div(1_000_000).expect("nonzero milliseconds divisor")).as_bytes(),
            b" ms",
        ]),
    }
}

fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded refusal pieces");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured refusal length");
    }
    writer.finish()
}
