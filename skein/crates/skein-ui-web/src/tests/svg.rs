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
    use skein_ui::{Builder, Limits, Role, flow, markdown};

    // Four nodes, at most four marks each, with 256 data bytes and 25 arrow bytes each.
    const CASE_NODES: u32 = 4;
    const CASE_DATA: u32 = 256;
    let cases = [
        (
            include_bytes!("../../../../docs/design/draft/diagrams/how-hello-works.md").as_slice(),
            640,
            Some(include_str!("../../../../docs/design/draft/diagrams/how-hello-works.640.svg")),
        ),
        (
            include_bytes!("../../../../docs/design/draft/diagrams/how-hello-works.md").as_slice(),
            600,
            Some(include_str!("../../../../docs/design/draft/diagrams/how-hello-works.600.svg")),
        ),
        (
            include_bytes!("../../../../docs/design/draft/diagrams/review.md").as_slice(),
            400,
            Some(include_str!("../../../../docs/design/draft/diagrams/review.400.svg")),
        ),
        (include_bytes!("../../../../docs/design/draft/diagrams/how-hello-works.md").as_slice(), 240, None),
    ];
    for (input, width, expected) in cases {
        let bytes = u32::try_from(input.len()).unwrap();
        let limits = Limits {
            nodes: markdown::max_nodes(bytes).unwrap().checked_add(1).unwrap(),
            depth: 6, // ui.md, 10: the reader's five levels below the page.
            text_per_node: bytes.checked_mul(2).unwrap(),
            text_bytes: markdown::max_text_bytes(bytes).unwrap(),
            markdown_depth: 0,
            drawing_bytes: CASE_DATA,
            scene_marks: flow::marks(CASE_NODES).expect("case marks fit"),
            scene_bytes: flow::scene_bytes(CASE_NODES, CASE_DATA).expect("case bytes fit"),
            ..LIMITS
        };
        let mut builder = Builder::new(&limits);
        builder.open(Role::Page);
        markdown::read(input, 0, &mut builder);
        builder.close();
        let tree = builder.finish();
        let drawing = tree
            .nodes()
            .iter()
            .find(|node| node.role == Role::Drawing)
            .expect("case reads as a drawing")
            .drawing
            .as_ref()
            .expect("drawing has kind and data");
        for height in [0, 74, 999] {
            let mut scene = Scene::new(&limits);
            assert_eq!(
                flow::draw(drawing.kind, drawing.data.as_ref(), width, height, &mut scene),
                expected.is_some(),
                "width {width}, height {height}"
            );
            if let Some(expected) = expected {
                assert_eq!(dom::print(&svg::draw(&scene, &[flow::NAMES], width, height)), expected.trim_end());
            }
        }
    }
}
