//! One malformed archive among committed turns (client/domain/testing.md, 4).
use super::{chat_ready, text};
use crate::{Scripted, Test, World, test::list};
use jig_client_domain::contract::*;
use skein_lib::Wall;

fn readable(number: u32, words: &[u8]) -> ActivityLine {
    ActivityLine {
        position: u64::from(number),
        at: Wall::EPOCH,
        event: crate::stories::turn_event(
            9,
            u64::from(number),
            TurnName { attempt: 1, number },
            TurnRead::Decoded(Turn {
                messages: list([Message { speaker: Speaker::Assistant, blocks: list([Block::Text(text(words))]) }]),
                used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
                spent: 1,
                read_through: None,
                cut: false,
            }),
        ),
    }
}
/// Starts a conversation with a malformed turn between two readable ones.
#[must_use]
pub fn world(seed: u64) -> World<Test, Scripted> {
    let mut world = chat_ready(seed);
    let unread = ActivityLine {
        position: 2,
        at: Wall::EPOCH,
        event: Activity::Owner {
            source: Box::new(OwnerActivityLine {
                task: 9,
                position: 2,
                at: Wall::EPOCH,
                line: OwnerActivity::Turn { reply: 2, attempt: 1, turn: 2, read: None },
            }),
            decoded: Some(ArchivedTurn {
                task: 9,
                name: TurnName { attempt: 1, number: 2 },
                cumulative: 2,
                read: None,
                at: Wall::EPOCH,
                turn: TurnRead::Undecodable { why: TurnProblem::Malformed },
            }),
        },
    };
    world
        .front()
        .engine
        .act(
            skein_lib::Time::ZERO,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
                party: 7,
                read: Read::Activity { task: 9, before: None },
                rows: jig_client_scripted_engine::Rows::Activity(vec![
                    readable(3, b"After the malformed turn"),
                    unread,
                    readable(1, b"Before the malformed turn"),
                ]),
            }),
        )
        .expect("whole conversation fits");
    world
}

/// Three rows keep both neighbouring turns in the same bounded window.
#[must_use]
pub fn limits() -> jig_client_test_domain::Limits {
    let mut limits = crate::test::limits();
    limits.jig.rows = 3;
    limits
}
/// Builds the root and view with the story's receiving bounds.
#[must_use]
pub fn client(seed: u64) -> (jig_client_test_domain::Domain, jig_client_test_view::View<'static>) {
    crate::test::new(&limits(), seed)
}
