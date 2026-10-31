use crate::state::{
    objects::{Objects, Source},
    windows::*,
};
pub(in crate::tests) use crate::tests::kit::text;
use jig_client::contract::*;
use jig_client::thread::window::{Edge, Edges, End};
use skein_lib::{List, Wall};

fn rows<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let mut rows = List::with_capacity(3);
    for value in values {
        rows.push(value).ok().unwrap();
    }
    rows
}

fn edges() -> Edges {
    Edges {
        before: Edge::More(Cursor::new(Box::from(b"before".as_slice()), 1024).expect("bounded opaque cursor")),
        after: Edge::More(Cursor::new(Box::from(b"after".as_slice()), 1024).expect("bounded opaque cursor")),
    }
}
fn chip(task: u64) -> Chip {
    Chip {
        task,
        title: DocumentText::complete(text(b"title")),
        executor: ChipExecutor::Party { party: 7 },
        phase: Phase::Idle,
        details: Box::new(crate::tests::state::domain::objects::phase_details(Phase::Idle)),
    }
}
fn conversation(task: u64) -> TaskLine {
    crate::tests::fixtures::task_line(chip(task), 1, Requester::Person(7), Wall::EPOCH)
}

#[test]
fn source_replacement_keeps_shared_names_and_admits_in_order() {
    let mut objects = Objects::new(2, 2);
    let a = Source::Window { number: 1 };
    let b = Source::Window { number: 2 };
    let first = ObjectKey::Task(1);
    let second = ObjectKey::Task(2);
    let third = ObjectKey::Task(3);
    assert_eq!(objects.replace(a, &[first, second, third]), 1);
    let id = objects.find(first).unwrap();
    assert_eq!(objects.replace(b, &[first]), 0);
    let c = Source::Window { number: 3 };
    assert_eq!(objects.replace(c, &[first]), 1);
    assert!(!objects.named(first, c));
    assert_eq!(objects.replace(a, &[second]), 0);
    assert_eq!(objects.find(first), Some(id));
    assert_eq!(objects.replace(b, &[]), 0);
    assert!(objects.find(first).is_none());
    assert_eq!(objects.replace(a, &[third, first, second]), 1);
    assert!(objects.find(third).is_some());
    assert!(objects.find(first).is_some());
    assert!(objects.find(second).is_none());
    objects.reclaim();
    assert_eq!(objects.replace(a, &[first]), 0);
    assert_eq!(objects.len(), 1);
}

fn object_window_slides_from_both_ends_and_gates_live_rows() {
    let mut window = ObjectWindow::new(2);
    assert_eq!(
        window.slide(
            rows([ObjectKey::Task(1), ObjectKey::Task(2)]),
            Edges { before: Edge::End, after: Edge::End },
            End::After
        ),
        0
    );
    assert_eq!(window.slide(rows([ObjectKey::Task(3)]), edges(), End::After), 1);
    assert_eq!(window.rows.as_slice(), &[ObjectKey::Task(2), ObjectKey::Task(3)]);
    assert_eq!(window.edges, edges());
    assert!(!window.live(rows([ObjectKey::Task(4)]), edges(), End::After));
    assert_eq!(window.slide(rows([ObjectKey::Task(1)]), edges(), End::Before), 1);
    assert_eq!(window.rows.as_slice(), &[ObjectKey::Task(1), ObjectKey::Task(2)]);
    assert_eq!(window.edges, edges());
    window.edges.before = Edge::End;
    assert!(window.live(rows([ObjectKey::Task(0)]), edges(), End::Before));
    assert_eq!(window.rows.as_slice(), &[ObjectKey::Task(0), ObjectKey::Task(1)]);
}

fn conversation_window_slides_from_both_ends_and_gates_live_rows() {
    let mut window = ConversationWindow::new(2);
    assert_eq!(
        window.slide(
            rows([conversation(1), conversation(2)]),
            Edges { before: Edge::End, after: Edge::End },
            End::After
        ),
        0
    );
    assert_eq!(window.slide(rows([conversation(3)]), edges(), End::After), 1);
    assert_eq!(window.rows.as_slice(), &[conversation(2), conversation(3)]);
    assert_eq!(window.edges, edges());
    assert!(!window.live(rows([conversation(4)]), edges(), End::After));
    assert_eq!(window.slide(rows([conversation(1)]), edges(), End::Before), 1);
    assert_eq!(window.rows.as_slice(), &[conversation(1), conversation(2)]);
    assert_eq!(window.edges, edges());
    window.edges.before = Edge::End;
    assert!(window.live(rows([conversation(0)]), edges(), End::Before));
    assert_eq!(window.rows.as_slice(), &[conversation(0), conversation(1)]);
}

#[test]
fn object_and_conversation_windows_slide_from_both_ends_and_gate_live_rows() {
    object_window_slides_from_both_ends_and_gates_live_rows();
    conversation_window_slides_from_both_ends_and_gates_live_rows();
}
