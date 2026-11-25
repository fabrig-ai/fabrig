use skein_browser::{
    Limits,
    boundary::{Event, Match, Query, Request},
};
use skein_browser_world::world::World;
use skein_heap::Span;
use skein_lib::Token;

const PAGE: Token = Token::new(3);
const FIND: Token = Token::new(4);
const ONE_MATCH: u32 = 1; // Only the second node contains "3 failing".
const NO_MORE_MATCHES: u32 = 0; // That match fits in the default result limit.
const AX_REPLY: &str = r#"{"nodes":[
    {"backendDOMNodeId":7,"role":{"value":"StaticText"},"name":{"value":"Checks: 3 passed"}},
    {"backendDOMNodeId":8,"role":{"value":"StaticText"},"name":{"value":"Checks: 3 failing, 2 passed"}}
]}"#;

fn find_needle(world: &mut World, needle: &[u8]) {
    world.ask(Request::Find {
        page: PAGE,
        op: FIND,
        query: Query {
            role: b"StaticText".as_slice().into(),
            name: needle.into(),
            matching: Match::Words,
            within: None,
            boxes: false,
        },
    });
}

fn world(limits: Limits, tree: String) -> World {
    let mut world = World::with_limits(limits);
    world.start();
    world.open();
    world.query_tree = Some(tree);
    world
}

#[test]
fn a_find_by_words_keeps_only_the_matching_sentence() {
    let mut world = world(Limits::default(), AX_REPLY.into());
    find_needle(&mut world, b"3 failing");
    match world.above.pop() {
        Some(Event::Found { op, seen, more }) => {
            assert_eq!(op, FIND);
            assert_eq!(seen.len(), ONE_MATCH);
            assert_eq!(more, NO_MORE_MATCHES);
            assert_eq!(seen.get(0).expect("matching sentence").name.as_ref(), b"Checks: 3 failing, 2 passed");
        }
        event => panic!("expected Found for the sentence, got {event:?}"),
    }
}

#[test]
fn a_words_find_counts_overflow_without_materializing_the_matches() {
    const RETAINED: u32 = 1; // Only the first node has room in the result.
    const TEXT_LIMIT: u32 = 8; // Both matches start after the retained prefix.
    const NEEDLE: &[u8] = b"needle";
    let limits = Limits { matches: RETAINED, text: TEXT_LIMIT, ..Limits::default() };
    let mut allocations = Vec::new();
    // Same JSON structure and size: both second nodes have the words, but
    // only one has a backend id. On neither path should it become a Seen.
    for (backend, overflow) in [("null", NO_MORE_MATCHES), ("1234", ONE_MATCH)] {
        let tree = format!(
            r#"{{"nodes":[
            {{"backendDOMNodeId":7,"role":{{"value":"StaticText"}},"name":{{"value":"prefix__ \u006eeedle"}},"value":{{"value":"first"}}}},
            {{"backendDOMNodeId":{backend},"role":{{"value":"StaticText"}},"name":{{"value":"another_ needle"}},"value":{{"value":"second"}}}}
        ]}}"#
        );
        let mut world = world(limits, tree);
        let measured = Span::start();
        find_needle(&mut world, NEEDLE);
        allocations.push(measured.made());
        let Some(Event::Found { op, seen, more }) = world.above.pop() else { panic!("expected Found") };
        let node = seen.get(0).expect("the retained match");
        assert_eq!(
            (op, seen.len(), more, node.node, &*node.name, &*node.value),
            (FIND, RETAINED, overflow, 7, &b"prefix__"[..], &b"first"[..])
        );
    }
    assert_eq!(allocations.first(), allocations.last(), "counting a match must not materialize it");
}
