use super::*;
use crate::page::Event;
use skein_ui::{Act, LogEdge};

const LIST: NodeId = NodeId(3); // page, main, list
const FOOT: Option<(NodeId, Act)> = Some((LIST, Act::Near { edge: LogEdge::Foot }));

fn reading(b: &mut Builder, bound: bool, continues: bool, busy: bool) {
    main(b);
    b.open(Role::List);
    if bound {
        b.bind(fixtures::binding(1));
    }
    if continues {
        b.state(State::Continues);
    }
    if busy {
        b.state(State::Busy);
    }
    b.open(Role::Item);
    b.text(Run::Plain, b"Entry");
    b.close();
    b.close();
    end(b);
}

fn scroll(page: &mut Page<'_>, top: i32, height: i32, viewport: i32) -> Option<(NodeId, Act)> {
    let mut ops = Vec::new();
    let act = page.act(Event::Scrolled { node: LIST, top, height, viewport }, &mut ops);
    assert!(ops.is_empty(), "a list's scroll changes no following attributes");
    act
}

#[test]
fn a_bound_list_reports_its_foot_as_it_nears_and_the_height_moves() {
    let mut page = page();
    page.apply(&fixtures::patches(&mut View::new(&LIMITS), 0, |b| reading(b, true, true, false)), &mut Vec::new());
    assert_eq!(scroll(&mut page, 800, 1000, 0), None, "not laid out");
    assert_eq!(scroll(&mut page, 500, 1000, 200), None, "one and a half screens away");
    assert_eq!(scroll(&mut page, 501, 1000, 200), FOOT);
    assert_eq!(scroll(&mut page, 501, 1000, 200), None, "one report per measure");
    assert_eq!(scroll(&mut page, 501, 1050, 200), None, "the height took the foot out of reach");
    assert_eq!(scroll(&mut page, 551, 1050, 200), FOOT, "near again");
    assert_eq!(scroll(&mut page, 551, 1000, 200), FOOT, "height moves while near");
    assert_eq!(scroll(&mut page, 551, 1000, 200), None);
    assert_eq!(scroll(&mut page, 0, 100, 200), FOOT, "a short list reports its foot alone");
    assert_eq!(scroll(&mut page, 0, 100, 200), None, "no head follows");
    let mut ops = Vec::new();
    page.apply(&[Patch::Text { node: LIST, text: Box::new([]) }], &mut ops);
    assert_eq!(ops.len(), 2, "the text and measure, with no following after Near");
    assert_eq!(ops.last(), Some(&Op::Measure { node: LIST }));
}

#[test]
fn a_list_reports_each_quarter_screen_toward_its_foot() {
    let mut page = page();
    page.apply(&fixtures::patches(&mut View::new(&LIMITS), 0, |b| reading(b, true, true, false)), &mut Vec::new());
    assert_eq!(scroll(&mut page, 501, 1000, 200), FOOT);
    assert_eq!(scroll(&mut page, 550, 1000, 200), None);
    assert_eq!(scroll(&mut page, 551, 1000, 200), FOOT);
    assert_eq!(scroll(&mut page, 525, 1000, 200), None, "away from the foot");
    assert_eq!(scroll(&mut page, 600, 1000, 200), None);
    assert_eq!(scroll(&mut page, 601, 1000, 200), FOOT);
}

#[test]
fn a_list_holds_near_while_busy_and_reports_when_busy_leaves() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, |b| reading(b, true, true, false)), &mut Vec::new());
    assert_eq!(scroll(&mut page, 600, 1000, 200), FOOT);
    page.apply(&fixtures::patches(&mut view, 1, |b| reading(b, true, true, true)), &mut Vec::new());
    assert_eq!(scroll(&mut page, 600, 1000, 200), None);
    assert_eq!(scroll(&mut page, 650, 1000, 200), None, "a quarter screen while busy");
    assert_eq!(scroll(&mut page, 800, 1100, 200), None, "height and position move while busy");
    page.apply(&fixtures::patches(&mut view, 2, |b| reading(b, true, true, false)), &mut Vec::new());
    assert_eq!(scroll(&mut page, 800, 1100, 200), FOOT, "busy left without another height change");
    assert_eq!(scroll(&mut page, 800, 1100, 200), None);
}

#[test]
fn a_list_needs_a_binding_and_continues_to_report_near() {
    for (bound, continues) in [(false, true), (true, false), (false, false)] {
        let mut page = page();
        page.apply(
            &fixtures::patches(&mut View::new(&LIMITS), 0, |b| reading(b, bound, continues, false)),
            &mut Vec::new(),
        );
        assert_eq!(scroll(&mut page, 0, 100, 200), None, "bound: {bound}, continues: {continues}");
        assert_eq!(scroll(&mut page, 800, 1000, 200), None, "bound: {bound}, continues: {continues}");
    }
}

#[test]
fn an_apply_measures_only_bound_lists_without_following_or_page_scrolling() {
    let (mut page, mut view, mut ops) = (page(), View::new(&LIMITS), Vec::new());
    let patches = fixtures::patches(&mut view, 0, |b| {
        main(b);
        for bound in [true, false, true] {
            b.open(Role::List);
            if bound {
                b.bind(fixtures::binding(1));
            }
            b.close();
        }
        end(b);
    });
    page.apply(&patches, &mut ops);
    assert!(ops.ends_with(&[Op::Measure { node: LIST }, Op::Measure { node: NodeId(5) }]));
    scroll(&mut page, 800, 1000, 200);
    ops.clear();
    page.apply(&[Patch::Text { node: LIST, text: Box::new([]) }], &mut ops);
    assert_eq!(ops.len(), 3, "the text and two measures, with no following or view restoration");
    assert_eq!(&ops[1..], &[Op::Measure { node: LIST }, Op::Measure { node: NodeId(5) }]);
    ops.clear();
    page.apply(&[], &mut ops);
    assert_eq!(ops.len(), 0);
    for key in [b"PageUp".as_slice(), b"PageDown"] {
        assert_eq!(
            page.act(
                Event::Key {
                    key: key.into(),
                    code: key.into(),
                    control: false,
                    meta: false,
                    alt: false,
                    shift: false,
                    in_field: false,
                    composing: false
                },
                &mut ops
            ),
            None
        );
        assert!(ops.is_empty(), "a list takes no page scrolling key");
    }
    assert!(!page.html().contains("data-log-away"));
}
