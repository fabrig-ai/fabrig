//! Tests for laying a flow out as a scene (skein/docs/design/ui/scenes.md, 5).

use crate::flow::{self, Direction, Node};
use crate::scene::{Mark, Point, Scene};
use crate::tests::fixtures::LIMITS;
use crate::{DrawingKind, Limits};

const PLAIN: Node<'_> = Node { name: b"a", caption: None, strong: false };
const CAPTIONED: Node<'_> = Node { name: b"a", caption: Some(b"c"), strong: true };

fn data(direction: Direction, nodes: &[Node<'_>]) -> skein_lib::List<u8> {
    flow::encode(direction, nodes, u32::MAX).expect("fixture flow encodes")
}

#[test]
fn an_unsupported_flow_leaves_the_scene_untouched() {
    // 1,100 rows at 62 pixels each exceed a u16 figure height (ui/scenes.md, 5).
    const TALL_NODES: u32 = 1_100;
    let lr = data(Direction::LeftToRight, &[PLAIN]);
    let td = data(Direction::TopToBottom, &[PLAIN]);
    let mut tall_nodes = skein_lib::List::with_capacity(TALL_NODES);
    for _ in 0..TALL_NODES {
        tall_nodes.push(PLAIN).expect("fixture node capacity");
    }
    let tall = data(Direction::TopToBottom, tall_nodes.as_slice());
    let hello = data(
        Direction::LeftToRight,
        &[
            Node { name: b"Design draft", caption: Some(b"you write it"), strong: false },
            Node { name: b"Goal", caption: Some(b"you start it"), strong: false },
            Node { name: b"Checked changes", caption: Some(b"CI and a review"), strong: false },
            Node { name: b"main", caption: Some(b"the merge queue"), strong: true },
        ],
    );
    let other = DrawingKind { kind: 1, ..flow::KIND };
    let cases = [
        ("another kind", other, lr.as_slice(), u16::MAX),
        ("no data", flow::KIND, b"".as_slice(), u16::MAX),
        ("malformed data", flow::KIND, b"\0\0\x02\0a", u16::MAX),
        ("malformed tail", flow::KIND, b"\0\0\x01\0a\x04", u16::MAX),
        ("narrow LR", flow::KIND, lr.as_slice(), 121),
        ("narrow TD", flow::KIND, td.as_slice(), 161),
        ("hello at 240", flow::KIND, hello.as_slice(), 240),
        ("height past u16", flow::KIND, tall.as_slice(), u16::MAX),
    ];
    for (reason, kind, bytes, width) in cases {
        let mut scene = Scene::new(&LIMITS);
        scene.circle(Point { x: 0, y: 0 }, 1, &[]);
        let before = alloc::fmt::format(format_args!("{scene}"));
        assert!(!flow::draw(kind, bytes, width, 0, &mut scene), "{reason}");
        assert_eq!(alloc::fmt::format(format_args!("{scene}")), before, "{reason}");
    }
}

#[test]
fn a_flows_scene_depends_on_width_and_says_its_height() {
    // Two 120-wide boxes need 270 pixels horizontally; TD rows are 62 apart.
    let cases = [
        (Direction::LeftToRight, PLAIN, 270, 46),
        (Direction::LeftToRight, CAPTIONED, 270, 74),
        (Direction::LeftToRight, PLAIN, 269, 104),
        (Direction::LeftToRight, CAPTIONED, 269, 104),
        (Direction::TopToBottom, PLAIN, 162, 104),
        (Direction::TopToBottom, CAPTIONED, 185, 104),
    ];
    for (direction, node, width, expected_height) in cases {
        let bytes = data(direction, &[node; 2]);
        let mut scene = Scene::new(&LIMITS);
        let mut previous = None;
        for height in [0, 74, 999, 999] {
            scene.clear();
            assert!(flow::draw(flow::KIND, bytes.as_slice(), width, height, &mut scene));
            assert_eq!(scene.height(), Some(expected_height));
            let printed = alloc::fmt::format(format_args!("{scene}"));
            if let Some(previous) = previous {
                assert_eq!(printed, previous);
            }
            previous = Some(printed);
        }
    }
}

#[test]
fn a_flow_past_scene_capacity_is_refused_whole() {
    // Two boxes, names and captions, one arrow: seven marks, four text bytes + 25 path bytes.
    const MARKS: u32 = 7;
    const BYTES: u32 = 29;
    let bytes = data(Direction::LeftToRight, &[CAPTIONED; 2]);
    for (marks, bytes_limit, accepted) in [
        (MARKS, BYTES, true),
        (0, BYTES, false),
        (1, BYTES, false),
        (2, BYTES, false),
        (3, BYTES, false),
        (MARKS - 1, BYTES, false),
        (MARKS, 0, false),
        (MARKS, 1, false),
        (MARKS, 2, false),
        (MARKS, BYTES - 1, false),
    ] {
        let limits = Limits { scene_marks: marks, scene_bytes: bytes_limit, ..LIMITS };
        let mut scene = Scene::new(&limits);
        assert_eq!(flow::draw(flow::KIND, bytes.as_slice(), 270, 0, &mut scene), accepted);
        assert_eq!(scene.refused(), !accepted);
        assert_eq!(scene.marks().len(), if accepted { 7 } else { 0 });
        assert_eq!(scene.height(), if accepted { Some(74) } else { None });
    }
    let mut scene = Scene::new(&Limits { scene_marks: 0, ..LIMITS });
    scene.circle(Point { x: 0, y: 0 }, 1, &[]);
    assert!(!flow::draw(flow::KIND, bytes.as_slice(), 270, 0, &mut scene));
    assert!(scene.refused());
    assert_eq!(scene.height(), None);
}

#[test]
fn flow_text_width_counts_scalars_and_replacement_characters() {
    // 13 name scalars: ceil(13 * 7.5) + 24 = 122; 19 caption scalars: ceil(19 * 6.5) + 8 = 132.
    let cases = [
        (Node { name: b"abcdefghijklm", ..PLAIN }, 122),
        (Node { name: "ééééééééééééé".as_bytes(), ..PLAIN }, 122),
        (Node { name: &[0xff; 13], ..PLAIN }, 122),
        (Node { name: b"abcdefghijklmn", ..PLAIN }, 129),
        (Node { caption: Some("ééééééééééééééééééé".as_bytes()), ..PLAIN }, 132),
        (Node { caption: Some(&[0xff; 19]), ..PLAIN }, 132),
    ];
    for (node, box_width) in cases {
        let bytes = data(Direction::LeftToRight, &[node]);
        for (width, accepted) in [(box_width + 2, true), (box_width + 1, false)] {
            let mut scene = Scene::new(&LIMITS);
            assert_eq!(flow::draw(flow::KIND, bytes.as_slice(), width, 0, &mut scene), accepted);
            if accepted {
                assert_eq!(
                    scene.marks().first(),
                    Some(&Mark::Rectangle {
                        corner: Point { x: 1, y: 1 },
                        size: Point { x: box_width, y: 44 },
                        radius: 8,
                    })
                );
            }
        }
    }
}
