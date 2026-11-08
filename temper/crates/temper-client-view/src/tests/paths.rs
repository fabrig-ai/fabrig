//! The look request's paths, with the words of contracts section 2.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "unit tests inspect scenes and shell output with ordinary Rust"
)]
use super::fixtures as f;
use crate::look;
use crate::paths::{self, Mark, Stop};
use skein_ui::{
    Builder, Role,
    scene::{Anchor, Mark as SceneMark, Point, Scene},
};

const APPROVE: &[(Mark, &[u8])] = &[
    (Mark::Person, b"you ask"),
    (Mark::Agent, b"produce"),
    (Mark::Code, b"check"),
    (Mark::Agent, b"repair"),
    (Mark::Code, b"check"),
    (Mark::Person, b"you approve"),
    (Mark::Code, b"update"),
    (Mark::Agent, b"resolve"),
    (Mark::Code, b"merge"),
    (Mark::Bar, b"main"),
];
const REVIEW: &[(Mark, &[u8])] = &[
    (Mark::Person, b"you ask"),
    (Mark::Agent, b"produce"),
    (Mark::Code, b"check"),
    (Mark::Agent, b"review"),
    (Mark::Agent, b"repair"),
    (Mark::Code, b"check"),
    (Mark::Code, b"update"),
    (Mark::Agent, b"resolve"),
    (Mark::Code, b"merge"),
    (Mark::Bar, b"main"),
];
const DRAFT: &[(Mark, &[u8])] = &[
    (Mark::Person, b"draft"),
    (Mark::Agent, b"plan"),
    (Mark::Agent, "produce ×n".as_bytes()),
    (Mark::Code, b"merge"),
    (Mark::Code, b"check"),
    (Mark::Agent, b"repair"),
    (Mark::Bar, b"goal branch"),
    (Mark::Code, b"check"),
    (Mark::Code, b"update"),
    (Mark::Agent, b"resolve"),
    (Mark::Code, b"merge"),
    (Mark::Code, b"verify"),
    (Mark::Bar, b"main"),
];

fn bounds() -> skein_ui::Limits {
    crate::Limits::of(&f::domain_limits(), &f::words()).unwrap().tree
}

fn tree(stops: &[Stop<'_>]) -> skein_ui::Tree {
    let mut builder = Builder::new(&bounds());
    paths::path(&mut builder, b"The way a change goes", stops).unwrap();
    builder.finish()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks the three paths, their stop geometry and fallback words")]
fn the_three_look_paths_keep_judgement_above_code_and_each_stop_in_words() {
    for rows in [APPROVE, REVIEW, DRAFT] {
        let stops: Vec<_> = rows.iter().map(|(mark, words)| Stop { mark: *mark, words }).collect();
        let tree = tree(&stops);
        let data = &tree.nodes()[0].drawing.as_ref().unwrap().data;
        assert_eq!(tree.nodes()[1].role, Role::List);
        assert_eq!(tree.nodes()[1].name.as_deref(), Some(b"The way a change goes".as_slice()));
        let mut print = skein_ui_text::Print::new(&bounds()).unwrap();
        skein_ui_text::render(&tree, b"", None, &mut print);
        let printed = String::from_utf8_lossy(print.text.as_slice());
        let mut scene = Scene::new(&bounds());
        assert!(paths::draw(paths::KIND, data, 640, 0, &mut scene));
        assert_eq!(scene.height(), Some(140));
        assert_eq!(scene.marks().len(), 2 + 2 * rows.len());
        assert_eq!(scene.marks()[0], SceneMark::Line { from: Point { x: 0, y: 72 }, to: Point { x: 640, y: 72 } });
        assert_eq!(scene.styles(0), &[look::WAYS_AUTH]);
        assert_eq!(scene.styles(1), &[look::WAYS_BASE]);
        assert_eq!(scene.marks()[1], SceneMark::Path { commands: u32::try_from(1 + 3 * (rows.len() - 1)).unwrap() });
        let mut line = skein_lib::Reader::new(scene.data(1));
        assert_eq!(line.u8(), Some(b'M'));
        assert_eq!(read_point(&mut line), Point { x: 12, y: 40 });
        for (i, (mark, words)) in rows.iter().enumerate() {
            let x = u16::try_from(12 + 616 * i / (rows.len() - 1)).unwrap();
            let upper = *mark == Mark::Person || *mark == Mark::Agent;
            let mark_index = u32::try_from(2 + 2 * i).unwrap();
            let expected = match mark {
                Mark::Person => SceneMark::Circle { centre: Point { x, y: 40 }, radius: 7 },
                Mark::Agent => SceneMark::Path { commands: 6 },
                Mark::Code => {
                    SceneMark::Rectangle { corner: Point { x: x - 5, y: 101 }, size: Point { x: 10, y: 10 }, radius: 2 }
                }
                Mark::Bar => {
                    SceneMark::Rectangle { corner: Point { x: x - 1, y: 95 }, size: Point { x: 3, y: 22 }, radius: 1 }
                }
            };
            assert_eq!(scene.marks()[usize::try_from(mark_index).unwrap()], expected, "{i}: {mark:?}");
            let (style, name, actor) = match mark {
                Mark::Person => (look::WAYS_PERSON, b"temper-ways-person".as_slice(), "a person"),
                Mark::Agent => (look::WAYS_AGENT, b"temper-ways-agent".as_slice(), "an agent"),
                Mark::Code => (look::WAYS_CODE, b"temper-ways-code".as_slice(), "code"),
                Mark::Bar => (look::WAYS_BAR, b"temper-ways-bar".as_slice(), "a branch"),
            };
            assert_eq!(scene.styles(mark_index), &[style]);
            assert_eq!(look::NAMES.style(style), Some(name));
            if *mark == Mark::Agent {
                let mut star = skein_lib::Reader::new(scene.data(mark_index));
                assert_eq!(star.u8(), Some(b'M'));
                assert_eq!(read_point(&mut star), Point { x, y: 31 });
                for tip in
                    [Point { x: x + 9, y: 40 }, Point { x, y: 49 }, Point { x: x - 9, y: 40 }, Point { x, y: 31 }]
                {
                    assert_eq!(star.u8(), Some(b'Q'));
                    let _control = read_point(&mut star);
                    assert_eq!(read_point(&mut star), tip);
                }
                assert_eq!(star.u8(), Some(b'Z'));
                assert!(star.is_empty());
            }
            let y = if upper {
                22
            } else if *mark == Mark::Bar {
                88
            } else {
                131
            };
            let anchor = if i == 0 {
                Anchor::Start
            } else if i + 1 == rows.len() {
                Anchor::End
            } else {
                Anchor::Middle
            };
            assert_eq!(
                scene.marks()[usize::try_from(mark_index + 1).unwrap()],
                SceneMark::Text { at: Point { x: if i == 0 { 2 } else { x }, y }, anchor }
            );
            assert_eq!(scene.data(mark_index + 1), *words);
            assert_eq!(scene.styles(mark_index + 1), &[look::WAYS_WORDS]);
            assert!(
                printed.lines().any(|line| line.contains(&format!("{}, {actor}", String::from_utf8_lossy(words)))),
                "{printed}"
            );
            if i != 0 {
                assert_eq!(line.u8(), Some(b'C'));
                let _control = read_point(&mut line);
                assert_eq!(read_point(&mut line).y, 106);
                assert_eq!(read_point(&mut line).y, 106);
                assert_eq!(line.u8(), Some(b'L'));
                assert_eq!(read_point(&mut line).y, 106);
                assert_eq!(line.u8(), Some(b'C'));
                assert_eq!(read_point(&mut line).y, 106);
                let _control = read_point(&mut line);
                assert_eq!(read_point(&mut line), Point { x, y: if upper { 40 } else { 106 } });
            }
        }
        assert!(line.is_empty());
        let before = scene.to_string();
        let mut same = Scene::new(&bounds());
        assert!(paths::draw(paths::KIND, data, 640, 140, &mut same));
        assert_eq!(same.to_string(), before);
        scene.clear();
        assert!(!paths::draw(paths::KIND, data, 240, 0, &mut scene));
        assert!(scene.marks().is_empty());
    }
}

fn read_point(reader: &mut skein_lib::Reader<'_>) -> Point {
    Point {
        x: u16::from_le_bytes([reader.u8().unwrap(), reader.u8().unwrap()]),
        y: u16::from_le_bytes([reader.u8().unwrap(), reader.u8().unwrap()]),
    }
}

#[test]
fn path_bounds_refuse_bad_data_and_draw_any_keeps_flows() {
    let maximum = vec![Stop { mark: Mark::Agent, words: &[b'x'; 64] }; 32];
    let tree = tree(&maximum);
    let data = &tree.nodes()[0].drawing.as_ref().unwrap().data;
    assert_eq!(data.len(), usize::try_from(paths::DATA).unwrap());
    let mut maximum_bounds = bounds();
    maximum_bounds.scene_marks = paths::MARKS;
    maximum_bounds.scene_bytes = paths::SCENE_BYTES;
    let mut scene = Scene::new(&maximum_bounds);
    assert!(paths::draw_any(paths::KIND, data, 1512, 0, &mut scene));
    assert_eq!(scene.marks().len(), usize::try_from(paths::MARKS).unwrap());
    assert!(!scene.refused());
    scene.clear();
    assert!(!paths::draw(paths::KIND, data, 1511, 0, &mut scene));
    for (stops, expected) in [
        (vec![], paths::Problem::Stops),
        (vec![Stop { mark: Mark::Code, words: b"x" }; 33], paths::Problem::Stops),
        (vec![Stop { mark: Mark::Code, words: b"" }], paths::Problem::Words),
        (vec![Stop { mark: Mark::Code, words: &[b'x'; 65] }], paths::Problem::Words),
    ] {
        let mut builder = Builder::new(&bounds());
        assert_eq!(paths::path(&mut builder, b"Path", &stops), Err(expected));
        assert!(builder.finish().nodes().is_empty());
    }
    for data in
        [b"".as_slice(), &[0], &[33], &[1, 4, 1, b'x'], &[1, 0, 0], &[1, 0, 65], &[1, 0, 2, b'x'], &[1, 0, 1, b'x', 0]]
    {
        assert!(!paths::draw(paths::KIND, data, 640, 0, &mut scene));
        assert!(scene.marks().is_empty());
    }
    for field in ["scene_marks", "scene_bytes"] {
        let mut tiny = bounds();
        if field == "scene_marks" {
            tiny.scene_marks = 1;
        } else {
            tiny.scene_bytes = 1;
        }
        let mut scene = Scene::new(&tiny);
        assert!(!paths::draw(paths::KIND, data, 1512, 0, &mut scene));
        assert!(scene.refused());
        assert!(scene.marks().is_empty());
    }
    let flow = skein_ui::flow::encode(
        skein_ui::flow::Direction::LeftToRight,
        &[
            skein_ui::flow::Node { name: b"draft", caption: None, strong: false },
            skein_ui::flow::Node { name: b"main", caption: None, strong: true },
        ],
        2048,
    )
    .unwrap();
    assert!(!paths::draw(skein_ui::flow::KIND, flow.as_slice(), 640, 0, &mut scene));
    assert!(paths::draw_any(skein_ui::flow::KIND, flow.as_slice(), 640, 0, &mut scene));
    let mut expected = Scene::new(&bounds());
    assert!(skein_ui::flow::draw(skein_ui::flow::KIND, flow.as_slice(), 640, 0, &mut expected));
    assert_eq!(scene.to_string(), expected.to_string());
    scene.clear();
    assert!(!paths::draw_any(
        skein_ui::DrawingKind { owner: crate::frame::owner(), kind: 1 },
        data,
        640,
        0,
        &mut scene
    ));
}
