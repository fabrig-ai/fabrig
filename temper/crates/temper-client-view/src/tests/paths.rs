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

fn tree(stops: &[Stop<'_>], pressed: Option<&paths::Pressed<'_>>) -> skein_ui::Tree {
    let mut builder = Builder::new(&bounds());
    paths::path(&mut builder, b"The way a change goes", stops, pressed).unwrap();
    builder.finish()
}

fn binding(index: u64) -> skein_ui::Binding {
    skein_ui::Binding { owner: crate::frame::owner(), token: skein_lib::Token::new(index) }
}

fn styled(scene: &Scene, style: skein_ui::Style) -> Vec<u32> {
    (0..u32::try_from(scene.marks().len()).unwrap()).filter(|i| scene.styles(*i).contains(&style)).collect()
}

#[test]
#[expect(clippy::too_many_lines, reason = "the request's three paths share geometry, acts and fallback assertions")]
fn the_three_look_paths_keep_judgement_above_code_and_each_stop_in_words() {
    for rows in [APPROVE, REVIEW, DRAFT] {
        let stops: Vec<_> = rows.iter().map(|(mark, words)| Stop { mark: *mark, words }).collect();
        let bindings: Vec<_> = rows
            .iter()
            .enumerate()
            .map(|(i, (mark, _))| (*mark != Mark::Bar).then_some(binding(u64::try_from(i).unwrap())))
            .collect();
        for pressed in [None, Some(paths::Pressed { drawing: binding(99), stops: &bindings, open: Some(3) })] {
            let tree = tree(&stops, pressed.as_ref());
            let node = &tree.nodes()[0];
            let data = &node.drawing.as_ref().unwrap().data;
            assert_eq!(
                &data[..3],
                &[
                    u8::try_from(rows.len()).unwrap(),
                    u8::from(pressed.is_some()),
                    if pressed.is_some() { 3 } else { 255 }
                ]
            );
            assert_eq!(node.styles.as_slice(), &[look::WAYS_PATH]);
            assert_eq!(node.binding, pressed.as_ref().map(|p| p.drawing));
            assert_eq!(tree.nodes()[1].role, Role::List);
            assert_eq!(tree.nodes()[1].name.as_deref(), Some(b"The way a change goes".as_slice()));
            let buttons: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Button).collect();
            assert_eq!(
                buttons.len(),
                if pressed.is_some() { rows.len() - rows.iter().filter(|r| r.0 == Mark::Bar).count() } else { 0 }
            );
            if pressed.is_some() {
                for (button, (i, _)) in buttons.iter().zip(rows.iter().enumerate().filter(|(_, r)| r.0 != Mark::Bar)) {
                    assert_eq!(button.binding, bindings[i]);
                    assert_eq!(button.states.as_slice().contains(&skein_ui::State::Expanded), i == 3);
                }
            }
            let mut print = skein_ui_text::Print::new(&bounds()).unwrap();
            skein_ui_text::render(&tree, b"", None, &mut print);
            let printed = String::from_utf8_lossy(print.text.as_slice());
            for width in [720_u16, 658, 388, 358] {
                let across = width >= 560;
                let offset = usize::from(width.saturating_sub(360) / 2);
                let boundary = u16::try_from(178 + offset).unwrap();
                let mut scene = Scene::new(&bounds());
                assert!(paths::draw(paths::KIND, data, width, 0, &mut scene));
                let down_end = 54 + 40 * (rows.len() - 2);
                let room = if width < 360 { (14 * 360 + usize::from(width) / 2) / usize::from(width) } else { 14 };
                let height = if across {
                    if pressed.is_some() { if width == 658 { 155 } else { 154 } } else { 140 }
                } else {
                    down_end + 14 + if pressed.is_some() { room } else { 0 }
                };
                assert_eq!(
                    scene.width(),
                    if width == 658 {
                        Some(720)
                    } else if width == 358 {
                        Some(360)
                    } else {
                        None
                    }
                );
                assert_eq!(scene.height(), Some(u16::try_from(height).unwrap()));
                assert_eq!(
                    scene.marks()[0],
                    if across {
                        SceneMark::Line { from: Point { x: 0, y: 72 }, to: Point { x: 720, y: 72 } }
                    } else {
                        SceneMark::Line {
                            from: Point { x: boundary, y: 0 },
                            to: Point { x: boundary, y: u16::try_from(down_end).unwrap() },
                        }
                    }
                );
                assert_eq!(scene.styles(0), &[look::WAYS_AUTH]);
                for (style, above) in [(look::WAYS_BASE, false), (look::WAYS_RISE, true)] {
                    let indices = styled(&scene, style);
                    assert_eq!(indices.len(), 1);
                    let commands = commands(scene.data(indices[0]));
                    assert!(!commands.is_empty());
                    for command in &commands {
                        let points = match command {
                            skein_ui::scene::Command::Move(p) | skein_ui::scene::Command::Line(p) => vec![*p],
                            skein_ui::scene::Command::Cubic(a, b, c) => vec![*a, *b, *c],
                            skein_ui::scene::Command::Quadratic(..) | skein_ui::scene::Command::Close => {
                                panic!("line contains moves, lines and cubics")
                            }
                        };
                        for p in points {
                            if across {
                                assert!(if above { p.y <= 72 } else { p.y >= 72 });
                            } else {
                                assert!(if above { p.x >= boundary } else { p.x <= boundary });
                            }
                        }
                    }
                    if !above {
                        // The first judgement falls over 0.42 of one step.
                        let step = 64000 / (rows.len() - 2);
                        let foot = if across {
                            Point { x: u16::try_from((1200 + step * 42 / 100) / 100).unwrap(), y: 106 }
                        } else {
                            Point { x: u16::try_from(150 + offset).unwrap(), y: 32 }
                        };
                        assert!(commands.iter().any(|command| match command {
                            skein_ui::scene::Command::Cubic(_, _, end) => *end == foot,
                            skein_ui::scene::Command::Move(_)
                            | skein_ui::scene::Command::Line(_)
                            | skein_ui::scene::Command::Quadratic(..)
                            | skein_ui::scene::Command::Close => false,
                        }));
                    }
                }
                let texts = styled(&scene, look::WAYS_WORDS);
                assert_eq!(texts.len(), rows.len());
                for (i, (mark, words)) in rows.iter().enumerate() {
                    let upper = *mark == Mark::Person || *mark == Mark::Agent;
                    let bar = *mark == Mark::Bar;
                    let end = i + 1 == rows.len();
                    let along = if across {
                        if end { 71200 } else { 1200 + 64000 * i / (rows.len() - 2) }
                    } else {
                        if end { down_end * 100 } else { (16 + 40 * i) * 100 }
                    };
                    let (x, y) = if across {
                        (along, if upper { 4000 } else { 10600 })
                    } else {
                        ((offset + if upper { 206 } else { 150 }) * 100, along)
                    };
                    let style = match mark {
                        Mark::Person => look::WAYS_PERSON,
                        Mark::Agent => look::WAYS_AGENT,
                        Mark::Code => look::WAYS_CODE,
                        Mark::Bar => look::WAYS_MAIN,
                    };
                    let text_index = texts[i];
                    let mark_index = text_index - 1;
                    assert_eq!(scene.styles(mark_index), &[style]);
                    let expected = match mark {
                        Mark::Person => SceneMark::Circle {
                            centre: Point { x: u16::try_from(x / 100).unwrap(), y: u16::try_from(y / 100).unwrap() },
                            radius: 7,
                        },
                        Mark::Agent => SceneMark::Path { commands: 6 },
                        Mark::Code => SceneMark::Rectangle {
                            corner: Point {
                                x: u16::try_from((x - 500) / 100).unwrap(),
                                y: u16::try_from((y - 500) / 100).unwrap(),
                            },
                            size: Point { x: 10, y: 10 },
                            radius: 2,
                        },
                        Mark::Bar => SceneMark::Rectangle {
                            corner: Point {
                                x: u16::try_from((x - if across { 150 } else { 1100 }) / 100).unwrap(),
                                y: u16::try_from((y - if across { 1100 } else { 150 }) / 100).unwrap(),
                            },
                            size: if across { Point { x: 3, y: 22 } } else { Point { x: 22, y: 3 } },
                            radius: 1,
                        },
                    };
                    assert_eq!(scene.marks()[usize::try_from(mark_index).unwrap()], expected, "{width}, {i}: {mark:?}");
                    if *mark == Mark::Agent {
                        let star = commands(scene.data(mark_index));
                        assert_eq!(
                            star[0],
                            skein_ui::scene::Command::Move(Point {
                                x: u16::try_from(x / 100).unwrap(),
                                y: u16::try_from((y - 900) / 100).unwrap()
                            })
                        );
                        assert_eq!(
                            star[1],
                            skein_ui::scene::Command::Quadratic(
                                Point {
                                    x: u16::try_from((x + 180) / 100).unwrap(),
                                    y: u16::try_from((y - 180) / 100).unwrap()
                                },
                                Point {
                                    x: u16::try_from((x + 900) / 100).unwrap(),
                                    y: u16::try_from(y / 100).unwrap()
                                }
                            )
                        );
                        for (command, tip) in star.iter().skip(2).take(3).zip([
                            Point { x: u16::try_from(x / 100).unwrap(), y: u16::try_from((y + 900) / 100).unwrap() },
                            Point { x: u16::try_from((x - 900) / 100).unwrap(), y: u16::try_from(y / 100).unwrap() },
                            Point { x: u16::try_from(x / 100).unwrap(), y: u16::try_from((y - 900) / 100).unwrap() },
                        ]) {
                            match command {
                                skein_ui::scene::Command::Quadratic(_, end) => assert_eq!(*end, tip),
                                skein_ui::scene::Command::Move(_)
                                | skein_ui::scene::Command::Line(_)
                                | skein_ui::scene::Command::Cubic(..)
                                | skein_ui::scene::Command::Close => panic!("each star has four curved tips"),
                            }
                        }
                        assert_eq!(star.last(), Some(&skein_ui::scene::Command::Close));
                    }
                    let (at, anchor) = if across {
                        (
                            Point {
                                x: if i == 0 { 2 } else { u16::try_from(x / 100).unwrap() },
                                y: if upper {
                                    22
                                } else if bar {
                                    88
                                } else {
                                    131
                                },
                            },
                            if end {
                                Anchor::End
                            } else if i == 0 {
                                Anchor::Start
                            } else {
                                Anchor::Middle
                            },
                        )
                    } else {
                        (
                            Point {
                                x: u16::try_from(
                                    offset
                                        + if upper {
                                            223
                                        } else if end {
                                            132
                                        } else {
                                            133
                                        },
                                )
                                .unwrap(),
                                y: u16::try_from((y + 400) / 100).unwrap(),
                            },
                            if upper { Anchor::Start } else { Anchor::End },
                        )
                    };
                    assert_eq!(scene.marks()[usize::try_from(text_index).unwrap()], SceneMark::Text { at, anchor });
                    assert_eq!(scene.data(text_index), *words);
                    assert_eq!(scene.styles(text_index).contains(&look::WAYS_OPEN), pressed.is_some() && i == 3);
                    assert_eq!(scene.styles(text_index).contains(&look::WAYS_END), bar);
                    let actor = match mark {
                        Mark::Person => "a person",
                        Mark::Agent => "an agent",
                        Mark::Code => "code",
                        Mark::Bar => "a branch",
                    };
                    assert!(
                        printed.lines().any(|l| l.contains(&format!("{}, {actor}", String::from_utf8_lossy(words)))),
                        "{printed}"
                    );
                }
                let landing = styled(&scene, look::WAYS_LANDING);
                assert_eq!(landing.len(), usize::from(rows == DRAFT));
                for i in landing {
                    assert_eq!(
                        scene.marks()[usize::try_from(i).unwrap()],
                        SceneMark::Group { offset: Point { x: 0, y: 0 }, marks: 2 }
                    );
                    assert_eq!(scene.styles(i + 1), &[look::WAYS_MAIN]);
                }
                let open_x = u16::try_from(12 + 640 * 3 / (rows.len() - 2)).unwrap();
                let groups = styled(&scene, look::WAYS_STOP);
                let hits = styled(&scene, look::WAYS_HIT);
                assert_eq!(groups.len(), buttons.len());
                assert_eq!(hits.len(), buttons.len());
                for ((group, hit), (i, (mark, _))) in
                    groups.iter().zip(&hits).zip(rows.iter().enumerate().filter(|(_, r)| r.0 != Mark::Bar))
                {
                    let act = [u8::try_from(i).unwrap()];
                    assert_eq!(scene.mark_act(*group), Some(act.as_slice()));
                    assert_eq!(scene.mark_act(*hit), Some(act.as_slice()));
                    assert_eq!(
                        scene.marks()[usize::try_from(*group).unwrap()],
                        SceneMark::Group { offset: Point { x: 0, y: 0 }, marks: if i == 3 { 4 } else { 3 } }
                    );
                    let mut resolved = skein_lib::List::with_capacity(1);
                    assert!(scene.act_at(&[texts[i]], Point { x: 0, y: 0 }, &mut resolved));
                    assert_eq!(resolved.as_slice(), act);
                    let upper = *mark == Mark::Person || *mark == Mark::Agent;
                    if i == 3 {
                        assert_eq!(
                            scene.marks()[usize::try_from(*hit).unwrap()],
                            SceneMark::Rectangle {
                                corner: if across {
                                    Point { x: open_x - 30, y: if upper { 6 } else { 94 } }
                                } else {
                                    Point { x: u16::try_from(offset + if upper { 194 } else { 10 }).unwrap(), y: 122 }
                                },
                                size: if across {
                                    Point { x: 60, y: if upper { 52 } else { 44 } }
                                } else {
                                    Point { x: if upper { 150 } else { 152 }, y: 28 }
                                },
                                radius: 0
                            }
                        );
                    }
                }
                let halos = styled(&scene, look::WAYS_HALO);
                assert_eq!(halos.len(), usize::from(pressed.is_some()));
                if let Some(halo) = halos.first() {
                    assert_eq!(
                        scene.marks()[usize::try_from(*halo).unwrap()],
                        SceneMark::Circle {
                            centre: if across {
                                Point { x: open_x, y: if rows == DRAFT { 106 } else { 40 } }
                            } else {
                                Point {
                                    x: u16::try_from(offset + if rows == DRAFT { 150 } else { 206 }).unwrap(),
                                    y: 136,
                                }
                            },
                            radius: 15
                        }
                    );
                }
                let carets = styled(&scene, look::WAYS_CARET);
                assert_eq!(carets.len(), usize::from(pressed.is_some() && across), "caret across alone");
                if let Some(caret) = carets.first() {
                    let c = if width == 658 { 7 } else { 6 };
                    let height = u16::try_from(height).unwrap();
                    assert_eq!(
                        scene.data(*caret),
                        [open_x - c, height, open_x, height - c, open_x + c, height]
                            .into_iter()
                            .flat_map(u16::to_le_bytes)
                            .collect::<Vec<_>>()
                    );
                }
                if pressed.is_none() {
                    for i in 0..scene.marks().len() {
                        assert!(scene.mark_act(u32::try_from(i).unwrap()).is_none());
                    }
                }
                let mut same = Scene::new(&bounds());
                assert!(paths::draw(paths::KIND, data, width, u16::try_from(height).unwrap(), &mut same));
                assert_eq!(same.to_string(), scene.to_string());
            }
        }
    }
}

fn commands(data: &[u8]) -> Vec<skein_ui::scene::Command> {
    use skein_ui::scene::Command;
    let mut reader = skein_lib::Reader::new(data);
    let mut result = Vec::new();
    while !reader.is_empty() {
        let c = match reader.u8().unwrap() {
            b'M' => Command::Move(read_point(&mut reader)),
            b'L' => Command::Line(read_point(&mut reader)),
            b'Q' => Command::Quadratic(read_point(&mut reader), read_point(&mut reader)),
            b'C' => Command::Cubic(read_point(&mut reader), read_point(&mut reader), read_point(&mut reader)),
            b'Z' => Command::Close,
            _ => panic!("known command"),
        };
        result.push(c);
    }
    result
}

fn read_point(reader: &mut skein_lib::Reader<'_>) -> Point {
    Point {
        x: u16::from_le_bytes([reader.u8().unwrap(), reader.u8().unwrap()]),
        y: u16::from_le_bytes([reader.u8().unwrap(), reader.u8().unwrap()]),
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks path data, layout thresholds and exact scene bounds")]
fn path_bounds_refuse_bad_data_and_draw_any_keeps_flows() {
    let maximum = vec![Stop { mark: Mark::Agent, words: &[b'x'; 64] }; 32];
    let bindings = vec![Some(binding(1)); 32];
    let pressed = paths::Pressed { drawing: binding(99), stops: &bindings, open: Some(3) };
    let maximum_tree = tree(&maximum, Some(&pressed));
    let data = &maximum_tree.nodes()[0].drawing.as_ref().unwrap().data;
    assert_eq!(data.len(), usize::try_from(paths::DATA).unwrap());
    assert_eq!(bounds().drawing_bytes, paths::DATA);
    assert_eq!(bounds().scene_marks, paths::MARKS);
    assert_eq!(bounds().scene_bytes, paths::SCENE_BYTES);
    let mut maximum_bounds = bounds();
    maximum_bounds.scene_marks = paths::MARKS;
    maximum_bounds.scene_bytes = paths::SCENE_BYTES;
    let mut scene = Scene::new(&maximum_bounds);
    assert!(paths::draw_any(paths::KIND, data, 1568, 0, &mut scene));
    assert_eq!(scene.marks().len(), usize::try_from(paths::MARKS).unwrap());
    assert!(!scene.refused());
    scene.clear();
    assert!(paths::draw(paths::KIND, data, 1567, 0, &mut scene));
    assert_eq!(scene.height(), Some(1294));
    scene.clear();
    for (bytes, expected) in [(&[][..], None), (&[3][..], Some(3)), (&[3, 0][..], None), (&[255][..], Some(255))] {
        assert_eq!(paths::pressed_stop(bytes), expected);
    }
    for (width, rows, height, drawn) in [
        (358, APPROVE, 388, Some(360)),
        (559, APPROVE, 388, None),
        (560, APPROVE, 140, Some(720)),
        (607, DRAFT, 508, None),
        (608, DRAFT, 140, Some(720)),
    ] {
        let stops: Vec<_> = rows.iter().map(|(mark, words)| Stop { mark: *mark, words }).collect();
        let t = tree(&stops, None);
        assert!(paths::draw(paths::KIND, &t.nodes()[0].drawing.as_ref().unwrap().data, width, 0, &mut scene));
        assert_eq!(scene.height(), Some(height));
        assert_eq!(scene.width(), drawn);
        scene.clear();
    }
    for (stops, expected) in [
        (vec![], paths::Problem::Stops),
        (vec![Stop { mark: Mark::Code, words: b"x" }; 33], paths::Problem::Stops),
        (vec![Stop { mark: Mark::Code, words: b"" }], paths::Problem::Words),
        (vec![Stop { mark: Mark::Code, words: &[b'x'; 65] }], paths::Problem::Words),
    ] {
        let mut builder = Builder::new(&bounds());
        assert_eq!(paths::path(&mut builder, b"Path", &stops, None), Err(expected));
        assert!(builder.finish().nodes().is_empty());
    }
    for data in [
        b"".as_slice(),
        &[0],
        &[33],
        &[1, 0, 255, 4, 1, b'x'],
        &[1, 0, 255, 0, 0],
        &[1, 0, 255, 0, 65],
        &[1, 0, 255, 0, 2, b'x'],
        &[1, 0, 255, 0, 1, b'x', 0],
        &[1, 2, 255, 0, 1, b'x'],
        &[1, 0, 0, 0, 1, b'x'],
        &[1, 1, 1, 0, 1, b'x'],
        &[1, 1, 0, 3, 1, b'x'],
    ] {
        assert!(!paths::draw(paths::KIND, data, 640, 0, &mut scene));
        assert!(scene.marks().is_empty());
    }
    for open in [None, Some(0), Some(9), Some(32)] {
        let stops: Vec<_> = APPROVE.iter().map(|(mark, words)| Stop { mark: *mark, words }).collect();
        let bindings: Vec<_> = APPROVE.iter().map(|(mark, _)| (*mark != Mark::Bar).then_some(binding(1))).collect();
        let pressed = paths::Pressed { drawing: binding(99), stops: &bindings, open };
        let tree = tree(&stops, Some(&pressed));
        let bytes = &tree.nodes()[0].drawing.as_ref().unwrap().data;
        assert_eq!(bytes[2], if open == Some(0) { 0 } else { 255 });
        let mut state = Scene::new(&bounds());
        assert!(paths::draw(paths::KIND, bytes, 720, 0, &mut state));
        assert_eq!(styled(&state, look::WAYS_HALO).len(), usize::from(open == Some(0)));
        assert_eq!(state.height(), Some(if open == Some(0) { 154 } else { 140 }));
    }
    for field in ["scene_marks", "scene_bytes"] {
        let mut tight = maximum_bounds;
        if field == "scene_marks" {
            tight.scene_marks -= 1;
        } else {
            tight.scene_bytes -= 1;
        }
        let mut short = Scene::new(&tight);
        assert!(!paths::draw(paths::KIND, data, 1568, 0, &mut short), "{field} is tight");
        assert!(short.refused());
        assert!(short.marks().is_empty());
    }
    for field in ["scene_marks", "scene_bytes"] {
        let mut tiny = bounds();
        if field == "scene_marks" {
            tiny.scene_marks = 1;
        } else {
            tiny.scene_bytes = 1;
        }
        let mut scene = Scene::new(&tiny);
        assert!(!paths::draw(paths::KIND, data, 1568, 0, &mut scene));
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
