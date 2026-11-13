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
