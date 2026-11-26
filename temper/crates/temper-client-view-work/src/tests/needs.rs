//! Browser needs words, roles and acts (temper/docs/design/client/pages/README.md).
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect semantic subtrees and actions")]
use super::fixtures as f;

use temper_client_domain::{self as d};

pub(super) use super::fixtures::needs::{
    approval_row, approvals, check_size, fixture, kinds, person, proposal, size_cases, stop_root,
};

fn approval_people(people: &[(u64, &str)]) -> Box<[d::forge::approvals::ApprovalPerson]> {
    people
        .iter()
        .map(|&(person, name)| d::forge::approvals::ApprovalPerson { person, name: name.as_bytes().into() })
        .collect()
}

#[test]
#[expect(clippy::too_many_lines, reason = "each row pins a distinct who-sentence rule in 4.1")]
fn approval_context_names_who_signed_and_who_can_still_sign() {
    use crate::pages::approval::{sentences, who};
    use d::forge::approvals::{ApprovalSet, Row, Yours};
    let w = f::browser_words();
    let row = Row {
        role: 2,
        required: 2,
        counted: 1,
        signed: approval_people(&[(2, "Sam")]),
        can_sign: approval_people(&[(1, "Ann"), (3, "Kit")]),
        sets: Box::new([ApprovalSet { number: 1, name: b"payments".as_slice().into() }]),
        ..approval_row()
    };
    assert_eq!(
        sentences(&row, 1, &w).as_ref(),
        [
            Box::<[u8]>::from(b"This project's landing rule 1 asks for 2 approvals by members.".as_slice()),
            b"Signs off changes to payments.".as_slice().into(),
            b"1 of 2 signed: Sam; you or Kit can give the second.".as_slice().into(),
        ]
    );
    for (required, counted, yours, signed, can, more, expected) in [
        (1, 0, Yours::Counts, &[][..], &[(1, "Ann")][..], 0, None),
        (1, 0, Yours::NoAccount, &[], &[(3, "Kit"), (2, "Sam")], 0, Some("Kit or Sam can give it.")),
        (2, 1, Yours::TooFew, &[(2, "Sam")], &[], 0, Some("1 of 2 signed: Sam.")),
        (
            2,
            1,
            Yours::Counts,
            &[(2, "Sam")],
            &[(1, "Ann"), (3, "Kit")],
            3,
            Some("1 of 2 signed: Sam; you, Kit or 3 others can give the second."),
        ),
        (
            4,
            3,
            Yours::Counts,
            &[(2, "Sam")],
            &[(1, "Ann")],
            0,
            Some("3 of 4 signed: Sam and 2 others; you can give the fourth."),
        ),
        (
            4,
            3,
            Yours::Counts,
            &[(2, "Sam"), (3, "Kit")],
            &[(1, "Ann")],
            1,
            Some("3 of 4 signed: Sam, Kit and one other; you or one other can give the fourth."),
        ),
        (
            3,
            2,
            Yours::Counts,
            &[(2, "Sam"), (3, "Kit")],
            &[(1, "Ann")],
            0,
            Some("2 of 3 signed: Sam and Kit; you can give the third."),
        ),
        (
            11,
            10,
            Yours::Counts,
            &[(2, "Sam")],
            &[(1, "Ann")],
            0,
            Some("10 of 11 signed: Sam and 9 others; you can give the next."),
        ),
        (
            9,
            8,
            Yours::Counts,
            &[(2, "Sam")],
            &[(1, "Ann")],
            0,
            Some("8 of 9 signed: Sam and 7 others; you can give the ninth."),
        ),
        (2, 1, Yours::Counts, &[(2, "")], &[(1, "Ann")], 0, Some("1 of 2 signed: someone; you can give the second.")),
        (2, 0, Yours::TooFew, &[], &[(1, "Ann")], 0, Some("You can give the first.")),
        (2, 0, Yours::OtherRole, &[], &[(2, "")], 0, Some("Someone can give the first.")),
        (2, 0, Yours::TooFew, &[], &[], 0, None),
        (2, 1, Yours::TooFew, &[(1, "Ann")], &[], 0, Some("1 of 2 signed: you.")),
    ] {
        let row = Row {
            required,
            counted,
            yours,
            signed: approval_people(signed),
            can_sign: approval_people(can),
            can_sign_more: more,
            ..approval_row()
        };
        assert_eq!(who(&row, 1, &w).as_deref(), expected.map(str::as_bytes), "{row:?}");
    }
    let row = Row {
        yours: Yours::NoAccount,
        whole: false,
        can_sign: approval_people(&[(3, "Kit"), (2, "Sam")]),
        ..approval_row()
    };
    let context = sentences(&row, 1, &w);
    assert_eq!(context[1].as_ref(), b"Kit or Sam can give it.");
    assert_eq!(context[2].as_ref(), b"There are too many reviews to read them all, so one may already count.");
    assert_eq!(context[3].as_ref(), b"temper knows no Forgejo account of yours, so your review wouldn't count.");
}

const DUE: u64 = 3_610_000_000_000;
const BEFORE_DUE: skein_lib::Wall = skein_lib::Wall::from_nanos(DUE - 1);
const AT_DUE: skein_lib::Wall = skein_lib::Wall::from_nanos(DUE);

fn bounded_needs() -> d::Domain {
    use super::fixtures::needs::{deliver_asked, event, fixture_watched};
    use jig_client::contract::{self as c, AtBound, waits::Bound};
    use skein_lib::Queue;
    const COUNT: u32 = 5;
    let (mut root, watch, decided) = fixture_watched("A question?", false, COUNT, false, d::address::Filter::ALL, None);
    deliver_asked(&mut root, decided, None);
    let mut rows = root.pages().inbox().unwrap().rows.clone();
    for (index, then) in [(0, AtBound::Overdue), (1, AtBound::PassUp), (COUNT - 1, AtBound::Overdue)] {
        rows.get_mut(index).unwrap().bound = Some(Bound::At { at: AT_DUE, then });
    }
    event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(f::documents::inbox(
                        rows,
                        COUNT,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
        &mut Queue::uncapped(),
    );
    let inbox = root.pages().inbox().unwrap();
    assert!(inbox.cards.get(0).unwrap().page.is_some(), "the first need is a read card");
    assert_eq!(inbox.cards.len(), COUNT - 1, "the last need has only its row");
    root
}

fn inbox_tree(root: &d::Domain, words: &crate::words::Words, times: &mut crate::words::Times) -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&f::tree_limits());
    crate::pages::inbox::main(
        &mut b,
        root,
        words,
        &mut crate::bindings::Bindings::new(256),
        32,
        times,
        &crate::forge_sign_in::Forges::new(),
    );
    b.finish()
}

#[test]
fn needs_you_marks_only_overdue_bounds_after_the_age() {
    use crate::{look, words::Times};
    let root = bounded_needs();
    let words = f::browser_words();
    let mut times = Times::new(BEFORE_DUE);
    let before = inbox_tree(&root, &words, &mut times);
    assert!(!core::str::from_utf8(&f::text(&before)).unwrap().contains("Overdue"));
    assert_eq!(times.next, Some(AT_DUE), "a redraw is due even if the displayed age stays the same");

    let tree = inbox_tree(&root, &words, &mut Times::new(AT_DUE));
    assert!(f::find(&tree, skein_ui::Role::Text, b"oldest first").is_some());
    let mut overdue = 0_u32;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::ASK_META) {
            let text = f::node_text(&tree, index);
            let text = core::str::from_utf8(&text).unwrap();
            if text.contains("Overdue") {
                assert!(text.ends_with(" · 1 h · Overdue"), "{text}");
                let parts = f::inbox::subtree(&tree, index);
                assert!(parts.iter().any(|part| part.styles.as_slice().contains(&look::NEED_OVERDUE)));
                assert_eq!(node.name.is_some(), overdue == 0, "first the open card, then the row-only need");
                overdue += 1;
            }
        }
    }
    assert_eq!(overdue, 2, "one open card and one closed row, but never the pass-up need");
}

#[test]
fn terminal_inbox_marks_overdue_after_the_wait() {
    use crate::words::{Shell, Times};
    let root = bounded_needs();
    let words = f::english(Shell::Terminal);
    for (wall, marked) in [(BEFORE_DUE, 0), (AT_DUE, 2)] {
        let tree = inbox_tree(&root, &words, &mut Times::new(wall));
        let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
        skein_ui_text::render(&tree, b"temper", None, &mut print);
        let text = core::str::from_utf8(print.text.as_slice()).unwrap();
        assert_eq!(text.matches("overdue").count(), marked, "{text}");
        assert_eq!(text.matches("1 h · overdue").count(), marked, "{text}");
    }
}
