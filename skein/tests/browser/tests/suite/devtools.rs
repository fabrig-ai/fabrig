use skein_browser::boundary::{Event, Match, Query, Rect, Refusal, Request, Seen, States};
use skein_browser_world::world::{Fault, World};
use skein_lib::{Time, Token};

const PAGE: Token = Token::new(3);
const READ: Token = Token::new(4);
// The scripted peer's Save button, used as a find's scope.
const SCOPE: u64 = 7;

fn open(requests: bool) -> World {
    let mut world = World::new();
    world.start();
    if requests {
        world.open_with_requests();
    } else {
        world.open();
    }
    world
}

fn methods(world: &World) -> Vec<Box<[u8]>> {
    world
        .sent
        .iter()
        .map(|bytes| {
            let document = skein_browser::wire::decode::Document::parse(
                bytes.strip_suffix(&[0]).expect("NUL"),
                world.env.limits.command,
            )
            .expect("command");
            document.root().get(b"method").expect("method").text().expect("method text")
        })
        .collect()
}

#[test]
fn opening_enables_only_domains_the_page_reports_on() {
    let base: &[&[u8]] = &[
        b"Target.createTarget",
        b"Target.attachToTarget",
        b"Page.enable",
        b"Runtime.enable",
        b"Log.enable",
        b"Inspector.enable",
    ];
    for requests in [false, true] {
        let world = open(requests);
        let sent = methods(&world);
        let page_commands: Vec<_> =
            sent.iter().skip_while(|method| method.as_ref() != b"Target.createTarget").map(AsRef::as_ref).collect();
        let mut expected = base.to_vec();
        if requests {
            expected.extend([b"Network.enable".as_slice(), b"Page.getFrameTree"]);
        }
        expected.extend([b"Emulation.setDeviceMetricsOverride".as_slice(), b"Page.navigate"]);
        assert_eq!(page_commands, expected, "requests: {requests}");
    }
}

#[test]
fn a_boxed_paragraph_find_keeps_matches_in_reply_order() {
    let mut world = open(false);
    // A paragraph's name need not be its text; the text can span child nodes.
    world.query_tree = Some(r#"{"nodes":[
        {"nodeId":"10","backendDOMNodeId":41,"role":{"value":"paragraph"},"name":{"value":"First"},
         "properties":[{"name":"focused","value":{"value":true}}],"childIds":["11","12"]},
        {"nodeId":"11","role":{"value":"StaticText"},"name":{"value":"Read "}},
        {"nodeId":"12","role":{"value":"StaticText"},"name":{"value":"this."}},
        {"nodeId":"20","backendDOMNodeId":9,"role":{"value":"paragraph"},"name":{"value":"Second"},"childIds":["21"]},
        {"nodeId":"21","role":{"value":"StaticText"},"name":{"value":"Read this."}},
        {"nodeId":"30","backendDOMNodeId":13,"role":{"value":"paragraph"},"name":{"value":"Read this."},"childIds":["31"]},
        {"nodeId":"31","role":{"value":"StaticText"},"name":{"value":"Other text."}}
    ]}"#.into());
    let query = Query {
        role: b"paragraph".as_slice().into(),
        name: b"Read this.".as_slice().into(),
        matching: Match::ParagraphText,
        within: Some(SCOPE),
        boxes: true,
    };
    world.ask(Request::Find { page: PAGE, op: READ, query });
    let Some(Event::Found { op, seen, more }) = world.above.pop() else { panic!("expected boxed paragraphs") };
    let expected =
        [(41, b"First".as_slice(), true), (9, b"Second".as_slice(), false)].map(|(node, name, focused)| Seen {
            node,
            role: b"paragraph".as_slice().into(),
            name: name.into(),
            value: Box::default(),
            states: States {
                focused,
                disabled: false,
                checked: false,
                expanded: false,
                selected: false,
                pressed: false,
            },
            // The fake peer's quad (10.25, 10.25) to (110.75, 40.75) in whole CSS pixels.
            rect: Some(Rect { x: 10, y: 10, width: 100, height: 30 }),
        });
    assert_eq!((op, seen.as_slice(), more), (READ, expected.as_slice(), 0));
}

#[test]
fn a_scope_of_an_earlier_document_is_gone() {
    let mut world = open(false);
    world.sent.clear();
    world.fault(b"DOM.resolveNode", Fault::ScopeGone);
    world.ask(Request::Find {
        page: PAGE,
        op: READ,
        query: Query {
            role: b"button".as_slice().into(),
            name: b"Save".as_slice().into(),
            matching: Match::Exact,
            within: Some(SCOPE),
            boxes: false,
        },
    });
    world.tick(Time::from_nanos(50_000_000));
    match world.above.pop() {
        Some(Event::Refused { op, why, .. }) => assert_eq!(
            (op, why, methods(&world)),
            (
                READ,
                Refusal::Gone,
                vec![Box::from(b"Page.bringToFront".as_slice()), Box::from(b"DOM.resolveNode".as_slice())]
            )
        ),
        other => panic!("expected a stale scope to be Gone, got {other:?}"),
    }
}
