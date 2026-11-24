use skein_browser::boundary::{Event, Match, Query, Request};
use skein_browser_world::world::World;
use skein_lib::Token;

const PAGE: Token = Token::new(3);
const FIND: Token = Token::new(4);
const ONE_MATCH: u32 = 1; // Only the second node contains "3 failing".
const NO_MORE_MATCHES: u32 = 0; // That match fits in the default result limit.
const AX_REPLY: &str = r#"{"nodes":[
    {"backendDOMNodeId":7,"role":{"value":"StaticText"},"name":{"value":"Checks: 3 passed"}},
    {"backendDOMNodeId":8,"role":{"value":"StaticText"},"name":{"value":"Checks: 3 failing, 2 passed"}}
]}"#;

#[test]
fn a_find_by_words_keeps_only_the_matching_sentence() {
    let mut world = World::new();
    world.start();
    world.open();
    world.query_tree = Some(AX_REPLY.into());
    world.ask(Request::Find {
        page: PAGE,
        op: FIND,
        query: Query {
            role: b"StaticText".as_slice().into(),
            name: b"3 failing".as_slice().into(),
            matching: Match::Words,
            within: None,
            boxes: false,
        },
    });
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
