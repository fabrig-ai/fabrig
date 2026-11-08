use super::fixtures as f;
use crate::{Browser, Local, Refusal, Signing, Told, Tui};
use jig_client::Link;

type Row<'a> = (&'a [Told], Told, Signing, &'a [u8], bool, bool, u64, u64, u64);
type LocalRow<'a> = (&'a [Told], Told, Local, Signing, &'a [u8], bool, bool);
type ApprovalRow<'a> = (&'a [Told], Told, Option<Browser>, bool, u64);

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks the TUI state and its revision changes")]
fn the_states_steps_are_its_table() {
    use Signing::{Answered, Asked, Idle, Refused};
    let typed = Told::Typed { value: Box::from(*b"link") };
    let refused = Told::Refused { refusal: Refusal::NotALink };
    let linked = Told::Link { link: Link::Live };
    let out = Told::Link { link: Link::SignedOut };
    let answered = Told::Answered { lost: false };
    let tui = Tui::new(f::ADDRESS);
    assert_eq!(
        (tui.field(), tui.signing(), tui.signed_out(), tui.unkept(), tui.openings(), tui.revision()),
        (b"".as_slice(), &Idle, false, false, 1, 0)
    );
    let rows: &[Row<'_>] = &[
        (&[], typed.clone(), Idle, b"link", false, false, 1, 1, 0),
        (&[], Told::Typed { value: Box::from([]) }, Idle, b"", false, false, 1, 0, 0),
        (core::slice::from_ref(&typed), typed.clone(), Idle, b"link", false, false, 1, 1, 0),
        (&[typed.clone(), refused.clone()], typed.clone(), Idle, b"link", false, false, 1, 3, 0),
        (&[Told::Asked, answered.clone()], typed.clone(), Idle, b"link", false, false, 1, 3, 1),
        (&[Told::Asked], typed.clone(), Asked, b"link", false, false, 1, 2, 1),
        (core::slice::from_ref(&typed), refused.clone(), Refused(Refusal::NotALink), b"link", false, false, 1, 2, 0),
        (core::slice::from_ref(&refused), refused.clone(), Refused(Refusal::NotALink), b"", false, false, 1, 1, 0),
        (
            &[],
            Told::Refused { refusal: Refusal::OtherAddress { address: Box::from(*b"localhost:8367") } },
            Refused(Refusal::OtherAddress { address: Box::from(*b"localhost:8367") }),
            b"",
            false,
            false,
            1,
            1,
            0,
        ),
        (core::slice::from_ref(&typed), Told::Asked, Asked, b"", false, false, 1, 2, 1),
        (&[Told::Asked], Told::Asked, Asked, b"", false, false, 1, 1, 1),
        (&[Told::Asked], answered.clone(), Answered { lost: false }, b"", false, false, 1, 2, 1),
        (&[Told::Asked], Told::Answered { lost: true }, Answered { lost: true }, b"", false, false, 1, 2, 1),
        (&[], answered.clone(), Idle, b"", false, false, 1, 0, 0),
        (
            &[typed.clone(), refused.clone()],
            answered.clone(),
            Refused(Refusal::NotALink),
            b"link",
            false,
            false,
            1,
            2,
            0,
        ),
        (
            &[Told::Asked, answered.clone()],
            Told::Answered { lost: true },
            Answered { lost: false },
            b"",
            false,
            false,
            1,
            2,
            1,
        ),
        (&[typed.clone(), refused.clone()], linked.clone(), Idle, b"", false, false, 1, 3, 1),
        (core::slice::from_ref(&linked), linked.clone(), Idle, b"", false, false, 1, 1, 1),
        (&[linked.clone(), typed.clone(), refused.clone()], out.clone(), Idle, b"", true, false, 2, 4, 2),
        (&[typed.clone(), refused.clone()], out.clone(), Refused(Refusal::NotALink), b"link", false, false, 1, 2, 0),
        (&[linked.clone(), out.clone()], out.clone(), Idle, b"", true, false, 2, 2, 2),
        (&[linked.clone(), out.clone()], linked.clone(), Idle, b"", false, false, 2, 3, 3),
        (&[], Told::Kept { written: false }, Idle, b"", false, true, 1, 1, 0),
        (&[], Told::Kept { written: true }, Idle, b"", false, false, 1, 0, 0),
        (&[Told::Kept { written: false }], Told::Kept { written: false }, Idle, b"", false, true, 1, 1, 0),
        (&[Told::Kept { written: false }], Told::Kept { written: true }, Idle, b"", false, false, 1, 2, 0),
    ];
    for (index, (start, told, signing, field, signed_out, unkept, openings, revision, written)) in
        rows.iter().enumerate()
    {
        let mut tui = Tui::new(f::ADDRESS);
        for step in *start {
            tui.step(step.clone());
        }
        tui.step(told.clone());
        assert_eq!(
            (tui.signing(), tui.field(), tui.signed_out(), tui.unkept(), tui.openings(), tui.revision(), tui.written()),
            (signing, *field, *signed_out, *unkept, *openings, *revision, *written),
            "row {index}: {told:?}"
        );
        assert_eq!(tui.address(), f::ADDRESS);
    }
    let rows: &[ApprovalRow<'_>] = &[
        (&[], Told::Browse, Some(Browser::Opening), false, 1),
        (&[Told::Browse], Told::Browse, Some(Browser::Opening), false, 1),
        (&[Told::Asked], Told::Browse, Some(Browser::Opening), false, 2),
        (&[Told::Browse, Told::Asked], Told::Browse, Some(Browser::Opening), false, 3),
        (&[Told::Browse], Told::Opened { opened: true }, Some(Browser::Opened), false, 2),
        (&[Told::Browse], Told::Opened { opened: false }, Some(Browser::None), false, 2),
        (&[Told::Opened { opened: true }], Told::Opened { opened: true }, Some(Browser::Opened), false, 1),
        (&[Told::Opened { opened: false }], Told::Opened { opened: false }, Some(Browser::None), false, 1),
        (&[Told::Opened { opened: true }], Told::Browse, Some(Browser::Opening), false, 2),
        (&[Told::Opened { opened: false }], Told::Browse, Some(Browser::Opening), false, 2),
        (&[], Told::Asked, None, true, 1),
        (&[Told::Asked], Told::Asked, None, true, 1),
        (&[Told::Asked, Told::Browse], Told::Asked, Some(Browser::Opening), true, 3),
        (&[Told::Asked, Told::Browse, Told::Asked], Told::Asked, Some(Browser::Opening), true, 3),
        (&[Told::Asked], Told::Opened { opened: true }, Some(Browser::Opened), true, 2),
        (&[Told::Asked], Told::Opened { opened: false }, Some(Browser::None), true, 2),
        (&[Told::Browse, Told::Asked, linked.clone()], out.clone(), None, false, 4),
        (&[Told::Opened { opened: false }, Told::Asked, linked.clone()], out.clone(), None, false, 4),
        (&[Told::Browse, Told::Asked, linked.clone(), out.clone()], out.clone(), None, false, 4),
        (&[Told::Browse, Told::Asked], out.clone(), Some(Browser::Opening), true, 2),
    ];
    assert_eq!((tui.browser(), tui.pasted()), (None, false));
    for (index, (start, told, browser, pasted, revision)) in rows.iter().enumerate() {
        let mut tui = Tui::new(f::ADDRESS);
        for step in *start {
            tui.step(step.clone());
        }
        tui.step(told.clone());
        assert_eq!(
            (tui.browser(), tui.pasted(), tui.revision()),
            (*browser, *pasted, *revision),
            "approval row {index}: {told:?}"
        );
    }
    let local_rows: &[LocalRow<'_>] = &[
        (&[], Told::Local { local: Local::Idle }, Local::Idle, Idle, b"", false, false),
        (&[], Told::Local { local: Local::Signing }, Local::Signing, Idle, b"", false, true),
        (&[], Told::Local { local: Local::Ended }, Local::Ended, Idle, b"", false, true),
        (
            &[Told::Local { local: Local::Signing }],
            Told::Local { local: Local::Idle },
            Local::Idle,
            Idle,
            b"",
            false,
            true,
        ),
        (
            &[Told::Local { local: Local::Signing }],
            Told::Local { local: Local::Signing },
            Local::Signing,
            Idle,
            b"",
            false,
            false,
        ),
        (
            &[Told::Local { local: Local::Signing }],
            Told::Local { local: Local::Ended },
            Local::Ended,
            Idle,
            b"",
            false,
            true,
        ),
        (
            &[Told::Local { local: Local::Ended }],
            Told::Local { local: Local::Idle },
            Local::Idle,
            Idle,
            b"",
            false,
            true,
        ),
        (
            &[Told::Local { local: Local::Ended }],
            Told::Local { local: Local::Signing },
            Local::Signing,
            Idle,
            b"",
            false,
            true,
        ),
        (
            &[Told::Local { local: Local::Ended }],
            Told::Local { local: Local::Ended },
            Local::Ended,
            Idle,
            b"",
            false,
            false,
        ),
        (&[Told::Local { local: Local::Signing }], linked.clone(), Local::Idle, Idle, b"", false, true),
        (&[Told::Local { local: Local::Ended }], linked.clone(), Local::Idle, Idle, b"", false, true),
        (&[Told::Local { local: Local::Ended }], Told::Asked, Local::Idle, Asked, b"", true, true),
        (&[Told::Asked, Told::Local { local: Local::Ended }], Told::Asked, Local::Idle, Asked, b"", true, true),
        (
            &[Told::Local { local: Local::Signing }, Told::Asked, answered.clone(), typed.clone()],
            Told::Local { local: Local::Ended },
            Local::Ended,
            Idle,
            b"",
            false,
            true,
        ),
        (
            &[Told::Local { local: Local::Ended }, typed.clone(), refused.clone()],
            Told::Local { local: Local::Ended },
            Local::Ended,
            Idle,
            b"",
            false,
            true,
        ),
        (
            &[Told::Local { local: Local::Signing }, Told::Asked],
            Told::Local { local: Local::Ended },
            Local::Ended,
            Idle,
            b"",
            false,
            true,
        ),
        (&[Told::Local { local: Local::Signing }], out.clone(), Local::Signing, Idle, b"", false, false),
        (&[Told::Local { local: Local::Ended }], out.clone(), Local::Ended, Idle, b"", false, false),
    ];
    assert_eq!(tui.local(), Local::Idle);
    let mut failures = skein_lib::List::with_capacity(u32::try_from(local_rows.len()).unwrap());
    for (index, (start, told, local, signing, field, pasted, changed)) in local_rows.iter().enumerate() {
        let mut tui = Tui::new(f::ADDRESS);
        for step in *start {
            tui.step(step.clone());
        }
        let before = tui.revision();
        tui.step(told.clone());
        let found = (tui.local(), tui.signing().clone(), tui.field(), tui.pasted(), tui.revision() - before);
        let expected = (*local, signing.clone(), *field, *pasted, u64::from(*changed));
        if found != expected {
            failures.push((index, (found.0, found.1, Box::<[u8]>::from(found.2), found.3, found.4), expected)).unwrap();
        }
    }
    assert!(failures.is_empty(), "local rows: {failures:?}");
    for local in [Local::Signing, Local::Ended] {
        for link in [Link::Starting, Link::Live, Link::Offline, Link::OutOfDate] {
            let mut tui = Tui::new(f::ADDRESS);
            tui.step(linked.clone());
            tui.step(Told::Local { local });
            let before = tui.revision();
            tui.step(Told::Link { link });
            assert_eq!((tui.local(), tui.signing(), tui.revision()), (Local::Idle, &Idle, before + 1));
            tui.step(out.clone());
            assert_eq!((tui.signed_out(), tui.openings()), (true, 2));
        }
    }
    for link in [Link::Starting, Link::Live, Link::Offline, Link::OutOfDate] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(typed.clone());
        tui.step(Told::Link { link });
        assert_eq!((tui.signing(), tui.field(), tui.revision()), (&Idle, b"".as_slice(), 2));
        tui.step(out.clone());
        assert_eq!((tui.signed_out(), tui.openings()), (true, 2), "{link:?} counts as linked");
    }
}
