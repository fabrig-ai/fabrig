use skein_ui::{
    Style,
    scene::{Anchor, Command, Point, Scene},
};

use super::fixtures::{LIMITS, NAMES, OWNER};
use crate::{dom, svg};

#[test]
fn a_scene_draws_as_its_golden_svg() {
    let mut scene = Scene::new(&LIMITS);
    let p = Point { x: 1, y: 2 };
    let q = Point { x: 3, y: 4 };
    assert!(scene.group(p, 6, &[]));
    assert!(scene.line(p, q, &[Style { owner: OWNER, token: 0 }]));
    assert!(scene.hint(b"Line & hint"));
    assert!(scene.act(&[0, 255]));
    assert!(scene.polyline(&[p, q], &[]));
    assert!(scene.path(
        &[Command::Move(p), Command::Line(q), Command::Quadratic(p, q), Command::Cubic(p, q, p), Command::Close],
        &[]
    ));
    assert!(scene.rect(p, q, 2, &[]));
    assert!(scene.circle(q, 3, &[]));
    assert!(scene.text(p, Anchor::Middle, b"Words", &[]));
    assert_eq!(dom::print(&svg::draw(&scene, NAMES, 100, 50)), include_str!("golden/scene.svg").trim_end());
}

#[test]
fn a_scene_uses_its_own_height_with_the_box_width() {
    for height in [0, 74, u16::MAX] {
        let mut scene = Scene::new(&LIMITS);
        scene.set_height(height);
        let printed = dom::print(&svg::draw(&scene, NAMES, 100, 50));
        assert_eq!(
            printed,
            format!("<svg aria-hidden=\"true\" width=\"100\" height=\"{height}\" viewBox=\"0 0 100 {height}\"></svg>")
        );
    }
}

#[test]
fn a_refused_scene_draws_nothing() {
    let mut limits = LIMITS;
    limits.scene_marks = 0;
    let mut scene = Scene::new(&limits);
    scene.circle(Point { x: 0, y: 0 }, 1, &[]);
    assert_eq!(dom::print(&svg::draw(&scene, NAMES, 100, 50)), "");
}

#[test]
fn a_marks_text_is_escaped_never_markup() {
    let mut scene = Scene::new(&LIMITS);
    scene.text(Point { x: 0, y: 0 }, Anchor::Start, b"<b>", &[]);
    assert_eq!(
        dom::print(&svg::draw(&scene, NAMES, 10, 20)),
        "<svg aria-hidden=\"true\" width=\"10\" height=\"20\" viewBox=\"0 0 10 20\"><text x=\"0\" y=\"0\" text-anchor=\"start\">&lt;b&gt;</text></svg>"
    );
}

#[test]
fn a_flows_scene_prints_its_case_byte_for_byte() {
    use skein_ui::{
        Limits,
        flow::{self, Direction, Node},
    };

    // Four nodes, at most four marks each, with 256 data bytes and 25 arrow bytes each.
    const CASE_NODES: u32 = 4;
    const CASE_DATA: u32 = 256;
    let limits = Limits {
        scene_marks: flow::marks(CASE_NODES).expect("case marks fit"),
        scene_bytes: flow::scene_bytes(CASE_NODES, CASE_DATA).expect("case bytes fit"),
        ..LIMITS
    };
    let hello = [
        Node { name: b"Design draft", caption: Some(b"you write it"), strong: false },
        Node { name: b"Goal", caption: Some(b"you start it"), strong: false },
        Node { name: b"Checked changes", caption: Some(b"CI and a review"), strong: false },
        Node { name: b"main", caption: Some(b"the merge queue"), strong: true },
    ];
    let review = [
        Node { name: b"A change pushed", caption: None, strong: false },
        Node { name: b"A review by an agent", caption: None, strong: false },
        Node { name: b"Landed", caption: None, strong: true },
    ];
    let cases = [
        (
            Direction::LeftToRight,
            hello.as_slice(),
            640,
            include_str!("../../../../docs/design/draft/diagrams/how-hello-works.640.svg"),
        ),
        (
            Direction::LeftToRight,
            hello.as_slice(),
            600,
            include_str!("../../../../docs/design/draft/diagrams/how-hello-works.600.svg"),
        ),
        (
            Direction::TopToBottom,
            review.as_slice(),
            400,
            include_str!("../../../../docs/design/draft/diagrams/review.400.svg"),
        ),
    ];
    for (direction, nodes, width, expected) in cases {
        let data = flow::encode(direction, nodes, CASE_DATA).expect("case data fits");
        for height in [0, 74, 999] {
            let mut scene = Scene::new(&limits);
            assert!(flow::draw(flow::KIND, data.as_slice(), width, height, &mut scene));
            assert_eq!(dom::print(&svg::draw(&scene, &[flow::NAMES], width, height)), expected.trim_end());
        }
    }
}
