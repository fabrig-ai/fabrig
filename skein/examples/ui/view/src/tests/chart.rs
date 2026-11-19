//! One chart act and its ordinary choices select the same notes.
use super::fixtures;
use crate::{
    Report, State, draw, limits,
    records::{self, Label},
    view,
};
use skein_lib::{List, Queue, Time, Wall};
use skein_ui::{Act, BoundAct, Drawing, Role, View, scene::Scene};

fn state() -> State {
    let mut state = fixtures::state();
    let mut notes = List::with_capacity(64);
    for (number, (title, label)) in
        (0..).zip([(b"An idea".as_slice(), Label::Idea), (b"A task", Label::Task), (b"A reference", Label::Reference)])
    {
        let mut note = fixtures::notes().notes.get(0).unwrap().clone();
        note.number = number;
        note.title = title.into();
        note.label = label;
        notes.push(note).unwrap();
    }
    state.notes = records::Notes { notes };
    state
}
fn drawing(state: &State) -> Drawing {
    let mut tree = View::new(&limits::limits());
    state.build(&mut tree, &mut Queue::with_capacity(1200));
    for node in tree.tree().nodes() {
        if let Some(drawing) = &node.drawing {
            return drawing.clone();
        }
    }
    unreachable!("chart drawing")
}
fn links(state: &State) -> List<Box<[u8]>> {
    let mut tree = View::new(&limits::limits());
    state.build(&mut tree, &mut Queue::with_capacity(1200));
    let mut links = List::with_capacity(3);
    let mut link = false;
    for node in tree.tree().nodes() {
        if link {
            links.push(node.text.get(0).unwrap().text.clone()).unwrap();
        }
        link = node.role == Role::Link;
    }
    links
}
const LABELS: &[(&[u8], &[u8])] = &[(b"\0", b"An idea"), (b"\x01", b"A task"), (b"\x02", b"A reference")];
#[test]
fn a_bars_act_shows_its_labels_notes() {
    for &(data, title) in LABELS {
        let mut state = state();
        state.step(
            Report::Act(BoundAct { binding: view::CHART, act: Act::Drawing { data: data.into() } }),
            Time::ZERO,
            Wall::EPOCH,
            &mut fixtures::duties(),
        );
        assert_eq!(links(&state).as_slice(), &[Box::<[u8]>::from(title)], "{data:?}");
    }
}
#[test]
fn a_choice_shows_its_labels_notes() {
    for ((_, binding), &(_, title)) in view::FILTERS.into_iter().skip(1).zip(LABELS) {
        let mut state = state();
        state.step(
            Report::Act(BoundAct { binding, act: Act::Toggle }),
            Time::ZERO,
            Wall::EPOCH,
            &mut fixtures::duties(),
        );
        assert_eq!(links(&state).as_slice(), &[Box::<[u8]>::from(title)], "{binding:?}");
    }
}
#[test]
fn the_chart_draws_as_its_golden_scene() {
    let drawing = drawing(&state());
    let mut scene = Scene::new(&limits::limits());
    assert!(draw::draw(drawing.kind, &drawing.data, 400, 144, &mut scene));
    let expected = std::fs::read_to_string("src/tests/golden/chart.txt").expect("hand-written golden scene");
    assert_eq!(alloc::fmt::format(format_args!("{scene}")), expected);
}

#[test]
fn all_notes_restores_the_whole_list() {
    let mut state = state();
    for binding in [view::FILTERS[2].1, view::FILTERS[0].1] {
        state.step(
            Report::Act(BoundAct { binding, act: Act::Toggle }),
            Time::ZERO,
            Wall::EPOCH,
            &mut fixtures::duties(),
        );
    }
    assert_eq!(
        links(&state).as_slice(),
        &[
            Box::<[u8]>::from(b"An idea".as_slice()),
            Box::<[u8]>::from(b"A task".as_slice()),
            Box::<[u8]>::from(b"A reference".as_slice()),
        ]
    );
}
#[test]
fn an_unknown_drawing_kind_is_refused() {
    let mut scene = Scene::new(&limits::limits());
    assert!(!draw::draw(skein_ui::DrawingKind { owner: view::OWNER, kind: 1 }, b"", 400, 144, &mut scene));
}
const BAD_DATA: &[&[u8]] = &[b"", b"\0\x04idea", b"A\0\0\0\0\0", b"\0\0\0\0\0\0x"];
#[test]
fn malformed_chart_data_is_refused() {
    for data in BAD_DATA {
        let mut scene = Scene::new(&limits::limits());
        assert!(!draw::draw(draw::KIND, data, 400, 144, &mut scene), "{data:?}");
    }
}
