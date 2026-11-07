use super::fixtures as f;
use crate::{Refusal, Signing, Told, Tui};
use jig_client::Link;

type Row<'a> = (&'a [Told], Told, Signing, &'a [u8], bool, bool, u64, u64, u64);

#[test]
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
    for link in [Link::Starting, Link::Live, Link::Offline, Link::OutOfDate] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(typed.clone());
        tui.step(Told::Link { link });
        assert_eq!((tui.signing(), tui.field(), tui.revision()), (&Idle, b"".as_slice(), 2));
        tui.step(out.clone());
        assert_eq!((tui.signed_out(), tui.openings()), (true, 2), "{link:?} counts as linked");
    }
}
