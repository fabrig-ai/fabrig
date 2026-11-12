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
    for (steps, chat, revision) in [
        (&[Told::Starting { chat: None }][..], None, 0),
        (&[Told::Starting { chat: Some(9) }][..], Some(9), 1),
        (&[Told::Starting { chat: Some(9) }, Told::Starting { chat: Some(9) }][..], Some(9), 1),
        (&[Told::Starting { chat: Some(9) }, Told::Starting { chat: None }][..], None, 2),
        (&[Told::Starting { chat: Some(9) }, Told::Cleared][..], Some(9), 2),
        (&[Told::Starting { chat: Some(9) }, Told::Went][..], None, 2),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        for step in steps {
            tui.step(step.clone());
        }
        assert_eq!((tui.start_chat(), tui.revision()), (chat, revision), "{steps:?}");
    }
    let turn = jig_client::contract::TurnName { attempt: 44, number: 1 };
    let newer = jig_client::contract::TurnName { attempt: 44, number: 2 };
    for (start, told, expected, revision) in [
        (&[][..], Told::ReplyAside { turn }, Some(turn), 1),
        (&[Told::ReplyAside { turn }][..], Told::ReplyAside { turn }, Some(turn), 1),
        (&[Told::ReplyAside { turn }][..], Told::ReplyAside { turn: newer }, Some(newer), 2),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        assert_eq!(tui.reply_aside(), None);
        for step in start {
            tui.step(step.clone());
        }
        tui.step(told);
        assert_eq!((tui.reply_aside(), tui.revision()), (expected, revision));
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

#[test]
fn link_changes_keep_the_command_composers_words() {
    for link in [Link::Starting, Link::Live, Link::Offline, Link::OutOfDate] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Cleared);
        tui.step(Told::CommandWords { value: Box::from(*b"fa") });
        let written = tui.box_written();
        for next in [link, Link::SignedOut] {
            tui.step(Told::Typed { value: Box::from(*b"link") });
            tui.step(Told::Link { link: next });
            assert_eq!(tui.command_words(), b"fa", "{link:?} then {next:?} keeps box words");
            assert_eq!(tui.box_written(), written, "{next:?} keeps the editor's write count");
            assert!(tui.field().is_empty(), "{next:?} still empties the link's field");
        }
    }
}

type Facts<'a> =
    (u16, Option<jig_client::contract::ObjectKey>, Option<&'a crate::Notice>, u64, u64, u64, Option<u64>, bool);

#[test]
fn the_surface_state_changes_only_when_its_facts_change() {
    let object = jig_client::contract::ObjectKey::Task(9);
    let first_notice = crate::Notice::Unknown { words: Box::from(*b"/font") };
    let notice = Told::Notice { notice: Some(first_notice.clone()) };
    let aside = Told::Aside { object: Some(object) };
    let size = Told::Size { columns: 80 };
    let live = Told::Link { link: Link::Live };
    let out = Told::Link { link: Link::SignedOut };
    // columns, aside, notice, focus, box writes, revision.
    let rows: &[(&str, &[Told], Facts<'_>)] = &[
        ("initial", &[], (0, None, None, 1, 0, 0, None, false)),
        ("zero columns", &[Told::Size { columns: 0 }], (0, None, None, 1, 0, 0, None, false)),
        ("size", core::slice::from_ref(&size), (80, None, None, 1, 0, 1, None, false)),
        ("same size", &[size.clone(), size.clone()], (80, None, None, 1, 0, 1, None, false)),
        ("resize", &[size.clone(), Told::Size { columns: 40 }], (40, None, None, 1, 0, 2, None, false)),
        ("went", &[Told::Went], (0, None, None, 2, 1, 1, None, false)),
        ("went twice", &[Told::Went, Told::Went], (0, None, None, 3, 2, 2, None, false)),
        ("cleared", &[Told::Cleared], (0, None, None, 1, 1, 1, None, false)),
        ("cleared twice", &[Told::Cleared, Told::Cleared], (0, None, None, 1, 2, 2, None, false)),
        ("aside", core::slice::from_ref(&aside), (0, Some(object), None, 1, 0, 1, None, false)),
        ("same aside", &[aside.clone(), aside.clone()], (0, Some(object), None, 1, 0, 1, None, false)),
        ("clear aside", &[aside.clone(), Told::Aside { object: None }], (0, None, None, 1, 0, 2, None, false)),
        (
            "different aside",
            &[aside.clone(), Told::Aside { object: Some(jig_client::contract::ObjectKey::Task(10)) }],
            (0, Some(jig_client::contract::ObjectKey::Task(10)), None, 1, 0, 2, None, false),
        ),
        ("no aside", &[Told::Aside { object: None }], (0, None, None, 1, 0, 0, None, false)),
        ("notice", core::slice::from_ref(&notice), (0, None, Some(&first_notice), 1, 0, 1, None, false)),
        ("same notice", &[notice.clone(), notice.clone()], (0, None, Some(&first_notice), 1, 0, 1, None, false)),
        ("clear notice", &[notice.clone(), Told::Notice { notice: None }], (0, None, None, 1, 0, 2, None, false)),
        (
            "different notice",
            &[notice.clone(), Told::Notice { notice: Some(crate::Notice::Unknown { words: Box::from(*b"/goal") }) }],
            (0, None, Some(&crate::Notice::Unknown { words: Box::from(*b"/goal") }), 1, 0, 2, None, false),
        ),
        (
            "cleared keeps notice",
            &[notice.clone(), Told::Cleared],
            (0, None, Some(&first_notice), 1, 1, 2, None, false),
        ),
        ("went keeps notice", &[notice.clone(), Told::Went], (0, None, Some(&first_notice), 2, 1, 2, None, false)),
        (
            "signing in replaces notice",
            &[notice.clone(), Told::Notice { notice: Some(crate::Notice::SigningIn) }],
            (0, None, Some(&crate::Notice::SigningIn), 1, 0, 2, None, false),
        ),
        ("no notice", &[Told::Notice { notice: None }], (0, None, None, 1, 0, 0, None, false)),
        ("first link", core::slice::from_ref(&live), (0, None, None, 2, 0, 1, None, false)),
        ("same link", &[live.clone(), live.clone()], (0, None, None, 2, 0, 1, None, false)),
        ("sign in shown again", &[live.clone(), out.clone()], (0, None, None, 3, 0, 2, None, false)),
        ("same sign out", &[live.clone(), out.clone(), out.clone()], (0, None, None, 3, 0, 2, None, false)),
        ("sign in again", &[live.clone(), out.clone(), live.clone()], (0, None, None, 4, 0, 3, None, false)),
        ("left off", &[Told::LeftOff { task: Some(7) }], (0, None, None, 1, 0, 1, Some(7), false)),
        (
            "same left off",
            &[Told::LeftOff { task: Some(7) }, Told::LeftOff { task: Some(7) }],
            (0, None, None, 1, 0, 1, Some(7), false),
        ),
        (
            "clear left off",
            &[Told::LeftOff { task: Some(7) }, Told::LeftOff { task: None }],
            (0, None, None, 1, 0, 2, None, false),
        ),
        ("no left off", &[Told::LeftOff { task: None }], (0, None, None, 1, 0, 0, None, false)),
        ("went clears left off", &[Told::LeftOff { task: Some(7) }, Told::Went], (0, None, None, 2, 1, 2, None, false)),
        ("boxed", &[Told::Boxed], (0, None, None, 1, 0, 1, None, true)),
        ("same boxed", &[Told::Boxed, Told::Boxed], (0, None, None, 1, 0, 1, None, true)),
    ];
    let mut failures = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (name, steps, want) in rows {
        let mut tui = Tui::new(f::ADDRESS);
        for told in *steps {
            tui.step(told.clone());
        }
        let found = (
            tui.columns(),
            tui.aside(),
            tui.notice(),
            tui.focus(),
            tui.box_written(),
            tui.revision(),
            tui.left_off(),
            tui.boxed(),
        );
        if found != *want {
            failures.push(*name).unwrap();
        }
    }
    assert!(failures.is_empty(), "surface state rows: {failures:?}");
    walk_rows();
    for link in [Link::Starting, Link::Live, Link::Offline, Link::OutOfDate] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Link { link });
        assert_eq!(tui.focus(), 2, "{link:?}");
        tui.step(Told::Link { link: Link::Live });
        assert_eq!(tui.focus(), 2, "{link:?}");
    }
}

// front, printed link, log, box state, question, /log, revision.
type FrontFacts<'a> = (bool, Option<skein_lib::Secret>, &'a [&'a [u8]], crate::BoxState<'a>, bool, bool, u64);

#[test]
#[expect(clippy::too_many_lines, reason = "named rows pin the front's facts and revision changes together")]
fn the_front_states_steps_are_its_table() {
    use crate::{BoxLine, BoxState};
    use skein_lib::{Secret, bytes::copy_of};
    let link = Told::BoxLine { line: BoxLine::Link { url: Secret::new(copy_of(b"the printed link")) } };
    let another = Told::BoxLine { line: BoxLine::Link { url: Secret::new(copy_of(b"another printed link")) } };
    let restarting = Told::BoxLine { line: BoxLine::Restarting { step: copy_of(b"forgejo-3") } };
    let running = Told::BoxLine { line: BoxLine::Running { step: copy_of(b"forgejo-3") } };
    let gave_up = Told::BoxLine { line: BoxLine::GaveUp { step: copy_of(b"serve") } };
    let mut logs = skein_lib::List::with_capacity(51);
    let mut kept = skein_lib::List::with_capacity(50);
    for number in 0..51 {
        let entry = copy_of(skein_lib::Decimal::of(number).as_bytes());
        if number > 0 {
            kept.push(entry.clone()).unwrap();
        }
        logs.push(Told::BoxLine { line: BoxLine::Log { line: entry } }).unwrap();
    }
    let mut kept_refs = skein_lib::List::with_capacity(50);
    for entry in &kept {
        kept_refs.push(entry.as_ref()).unwrap();
    }
    let rows: &[(&str, &[Told], FrontFacts<'_>)] = &[
        ("initial", &[], (false, None, &[], BoxState::Running, false, false, 0)),
        ("front", &[Told::Front], (true, None, &[], BoxState::Running, false, false, 1)),
        ("same front", &[Told::Front, Told::Front], (true, None, &[], BoxState::Running, false, false, 1)),
        (
            "the printed link",
            core::slice::from_ref(&link),
            (false, Some(Secret::new(copy_of(b"the printed link"))), &[], BoxState::Running, false, false, 1),
        ),
        (
            "same printed link",
            &[link.clone(), link.clone()],
            (false, Some(Secret::new(copy_of(b"the printed link"))), &[], BoxState::Running, false, false, 1),
        ),
        (
            "another printed link",
            &[link.clone(), another],
            (false, Some(Secret::new(copy_of(b"another printed link"))), &[], BoxState::Running, false, false, 2),
        ),
        (
            "last fifty oldest first",
            logs.as_slice(),
            (false, None, kept_refs.as_slice(), BoxState::Running, false, false, 51),
        ),
        (
            "restarting",
            core::slice::from_ref(&restarting),
            (false, None, &[], BoxState::Restarting { step: b"forgejo-3" }, false, false, 1),
        ),
        (
            "same restarting",
            &[restarting.clone(), restarting.clone()],
            (false, None, &[], BoxState::Restarting { step: b"forgejo-3" }, false, false, 1),
        ),
        (
            "another step running",
            &[restarting.clone(), Told::BoxLine { line: BoxLine::Running { step: copy_of(b"other") } }],
            (false, None, &[], BoxState::Restarting { step: b"forgejo-3" }, false, false, 1),
        ),
        ("restarted", &[restarting.clone(), running.clone()], (false, None, &[], BoxState::Running, false, false, 2)),
        ("already running", &[running], (false, None, &[], BoxState::Running, false, false, 0)),
        (
            "gave up",
            &[restarting.clone(), gave_up.clone()],
            (false, None, &[], BoxState::GaveUp { step: b"serve" }, false, false, 2),
        ),
        (
            "gave up wins",
            &[gave_up.clone(), restarting.clone()],
            (false, None, &[], BoxState::GaveUp { step: b"serve" }, false, false, 2),
        ),
        (
            "given up for good",
            &[gave_up, Told::BoxLine { line: BoxLine::GaveUp { step: copy_of(b"other") } }],
            (false, None, &[], BoxState::GaveUp { step: b"serve" }, false, false, 1),
        ),
        ("quitting", &[Told::Quitting], (false, None, &[], BoxState::Running, true, false, 1)),
        ("same quitting", &[Told::Quitting, Told::Quitting], (false, None, &[], BoxState::Running, true, false, 1)),
        ("stay", &[Told::Quitting, Told::Stay], (false, None, &[], BoxState::Running, false, false, 2)),
        ("already staying", &[Told::Stay], (false, None, &[], BoxState::Running, false, false, 0)),
        ("log", &[Told::Log], (false, None, &[], BoxState::Running, false, true, 1)),
        ("same log", &[Told::Log, Told::Log], (false, None, &[], BoxState::Running, false, true, 1)),
        ("send lowers log", &[Told::Log, Told::Cleared], (false, None, &[], BoxState::Running, false, false, 2)),
        ("screen lowers log", &[Told::Log, Told::Went], (false, None, &[], BoxState::Running, false, false, 2)),
    ];
    let mut failures = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (name, steps, want) in rows {
        let mut tui = Tui::new(f::ADDRESS);
        for told in *steps {
            tui.step(told.clone());
        }
        let mut log = skein_lib::List::with_capacity(50);
        for entry in tui.box_log() {
            log.push(entry).unwrap();
        }
        let found = (
            tui.front(),
            tui.printed().cloned(),
            log.as_slice(),
            tui.box_state(),
            tui.quitting(),
            tui.log_shown(),
            tui.revision(),
        );
        if found != *want {
            failures.push(*name).unwrap();
        }
    }
    assert!(failures.is_empty(), "front state rows: {failures:?}");
}

// Walk (by key, taken, shown object, shown other), brief, opener, revision.
type WalkFacts<'a> = (Option<(bool, bool, bool, bool)>, bool, Option<&'a [u8]>, u64);

fn walk_rows() {
    use jig_client::contract::ObjectKey;
    // The contract holds 64 objects; the next remains not shown, without evicting one.
    const HELD: u32 = 64;
    let object = ObjectKey::Question { task: 9, message: 1 };
    let other = ObjectKey::Question { task: 9, message: 2 };
    let walk = Told::Walk { by_key: true };
    let walked = Told::Walked { object };
    let note = Told::NotOpened { address: Some(Box::from(*b"http://example.test/tasks/9")) };
    let rows: &[(&str, &[Told], WalkFacts<'_>)] = &[
        ("initial walk", &[], (None, false, None, 0)),
        ("n", core::slice::from_ref(&walk), (Some((true, false, false, false)), false, None, 1)),
        ("same walk", &[walk.clone(), walk.clone()], (Some((true, false, false, false)), false, None, 1)),
        ("/needs", &[Told::Walk { by_key: false }], (Some((false, false, false, false)), false, None, 1)),
        ("walked", &[walk.clone(), walked.clone()], (Some((true, true, true, false)), false, None, 2)),
        (
            "same walked",
            &[walk.clone(), walked.clone(), walked.clone()],
            (Some((true, true, true, false)), false, None, 2),
        ),
        ("walked outside walk", core::slice::from_ref(&walked), (None, false, None, 0)),
        (
            "restart walk",
            &[walk.clone(), walked.clone(), walk.clone()],
            (Some((true, false, false, false)), false, None, 3),
        ),
        ("end walk", &[walk.clone(), Told::EndWalk], (None, false, None, 2)),
        ("no walk to end", &[Told::EndWalk], (None, false, None, 0)),
        ("went ends walk", &[walk.clone(), Told::Went], (None, false, None, 2)),
        ("cleared ends walk", &[walk.clone(), Told::Cleared], (None, false, None, 2)),
        ("typing ends walk", &[walk.clone(), Told::Typed { value: Box::from(*b"words") }], (None, false, None, 2)),
        (
            "empty typing keeps walk",
            &[walk.clone(), Told::Typed { value: Box::default() }],
            (Some((true, false, false, false)), false, None, 1),
        ),
        ("brief", &[Told::Brief], (None, true, None, 1)),
        ("same brief", &[Told::Brief, Told::Brief], (None, true, None, 1)),
        ("went clears brief", &[Told::Brief, Told::Went], (None, false, None, 2)),
        ("send clears brief", &[Told::Brief, Told::Cleared], (None, false, None, 2)),
        ("opener", core::slice::from_ref(&note), (None, false, Some(b"http://example.test/tasks/9"), 1)),
        ("same opener", &[note.clone(), note.clone()], (None, false, Some(b"http://example.test/tasks/9"), 1)),
        ("no opener address", &[Told::NotOpened { address: None }], (None, false, None, 0)),
        ("clear opener", &[note.clone(), Told::NotOpened { address: None }], (None, false, None, 2)),
        ("went clears opener", &[note.clone(), Told::Went], (None, false, None, 2)),
        ("send clears opener", &[note.clone(), Told::Cleared], (None, false, None, 2)),
        (
            "end keeps notes",
            &[Told::Brief, note.clone(), walk.clone(), Told::EndWalk],
            (None, true, Some(b"http://example.test/tasks/9"), 4),
        ),
        ("went clears all", &[Told::Brief, note.clone(), walk.clone(), Told::Went], (None, false, None, 4)),
        ("send clears all", &[Told::Brief, note.clone(), walk.clone(), Told::Cleared], (None, false, None, 4)),
    ];
    let mut failures = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (name, steps, want) in rows {
        let mut tui = Tui::new(f::ADDRESS);
        for told in *steps {
            tui.step(told.clone());
        }
        let mut walk = None;
        if let Some(held) = tui.walk() {
            walk = Some((held.by_key(), held.taken(), held.shown(&object), held.shown(&other)));
        }
        if (walk, tui.brief_shown(), tui.not_opened(), tui.revision()) != *want {
            failures.push(*name).unwrap();
        }
    }
    assert!(failures.is_empty(), "walk and note rows: {failures:?}");
    assert_eq!(crate::limits::WALK, HELD);
    for count in [HELD, HELD + 1] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Walk { by_key: false });
        for task in 0..count {
            tui.step(Told::Walked { object: ObjectKey::Task(u64::from(task)) });
        }
        let walk = tui.walk().unwrap();
        assert!(walk.taken());
        for task in 0..HELD {
            assert!(walk.shown(&ObjectKey::Task(u64::from(task))), "{count}: {task}");
        }
        assert!(!walk.shown(&ObjectKey::Task(u64::from(HELD))));
    }
}

#[test]
fn the_theme_list_steps() {
    use crate::{Scheme, theme::ThemeChoice};
    let light = Told::Theme { choice: ThemeChoice::Light, terminal: Some(Scheme::Dark) };
    let open = Told::ThemeList { open: true };
    let close = Told::ThemeList { open: false };
    for (name, steps, choice, terminal, opened, revision) in [
        ("default", &[][..], ThemeChoice::Auto, None, false, 0),
        (
            "same default",
            &[Told::Theme { choice: ThemeChoice::Auto, terminal: None }][..],
            ThemeChoice::Auto,
            None,
            false,
            0,
        ),
        ("told", core::slice::from_ref(&light), ThemeChoice::Light, Some(Scheme::Dark), false, 1),
        ("same told", &[light.clone(), light.clone()][..], ThemeChoice::Light, Some(Scheme::Dark), false, 1),
        (
            "terminal changed",
            &[light.clone(), Told::Theme { choice: ThemeChoice::Light, terminal: None }][..],
            ThemeChoice::Light,
            None,
            false,
            2,
        ),
        ("open", core::slice::from_ref(&open), ThemeChoice::Auto, None, true, 1),
        ("same open", &[open.clone(), open.clone()][..], ThemeChoice::Auto, None, true, 1),
        ("close", &[open.clone(), close.clone()][..], ThemeChoice::Auto, None, false, 2),
        ("same close", &[close.clone(), close.clone()][..], ThemeChoice::Auto, None, false, 0),
        (
            "cleared",
            &[light.clone(), open.clone(), Told::Cleared][..],
            ThemeChoice::Light,
            Some(Scheme::Dark),
            false,
            3,
        ),
        ("went", &[light.clone(), open.clone(), Told::Went][..], ThemeChoice::Light, Some(Scheme::Dark), false, 3),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        for told in steps {
            tui.step(told.clone());
        }
        assert_eq!(
            (tui.theme_choice(), tui.terminal_scheme(), tui.theme_list(), tui.revision()),
            (choice, terminal, opened, revision),
            "{name}"
        );
    }
}

#[test]
fn the_box_stages_and_terminal_precedence_hold() {
    use crate::{BoxEnd, BoxLine, BoxStage, BoxState};
    use skein_lib::bytes::copy_of;
    let ended = BoxStage::Ended { end: BoxEnd::Exited { code: 1 } };
    let failed = BoxStage::Failed { why: copy_of(b"no link") };
    let stages = [
        BoxStage::Finding,
        BoxStage::Fetching { done: 5, total: None },
        BoxStage::Fetching { done: 12, total: Some(24) },
        BoxStage::Starting,
        BoxStage::Running,
        BoxStage::Stopping,
        ended.clone(),
    ];
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Front);
    for (index, stage) in stages.iter().enumerate() {
        tui.step(Told::Stage { stage: stage.clone() });
        assert_eq!(tui.box_stage(), stage);
        assert_eq!(tui.revision(), u64::try_from(index).unwrap() + 1);
        tui.step(Told::Stage { stage: stage.clone() });
        assert_eq!(tui.revision(), u64::try_from(index).unwrap() + 1, "same stage");
    }
    for terminal in [&ended, &failed] {
        for next in stages.iter().chain(core::iter::once(&failed)) {
            let mut tui = Tui::new(f::ADDRESS);
            tui.step(Told::Stage { stage: terminal.clone() });
            let revision = tui.revision();
            tui.step(Told::Stage { stage: next.clone() });
            let accepted = terminal == &failed && next == &ended;
            assert_eq!(tui.box_stage(), if accepted { next } else { terminal });
            assert_eq!(tui.revision(), revision + u64::from(accepted));
        }
    }
    let restarting = Told::BoxLine { line: BoxLine::Restarting { step: copy_of(b"serve") } };
    let gave_up = Told::BoxLine { line: BoxLine::GaveUp { step: copy_of(b"serve") } };
    let stopping = Told::Stage { stage: BoxStage::Stopping };
    let end = Told::Stage { stage: ended };
    for (steps, want, note) in [
        (&[][..], BoxState::Running, None),
        (&[restarting.clone()][..], BoxState::Restarting { step: b"serve" }, None),
        (&[restarting.clone(), stopping.clone()][..], BoxState::Stopping, None),
        (&[stopping.clone(), restarting.clone()][..], BoxState::Stopping, None),
        (&[restarting, stopping.clone(), end.clone()][..], BoxState::Stopped { end: BoxEnd::Exited { code: 1 } }, None),
        (
            &[end.clone()][..],
            BoxState::Stopped { end: BoxEnd::Exited { code: 1 } },
            Some(b"The box stopped: exited 1. /log shows why; Ctrl+C closes.".as_slice()),
        ),
        (
            &[gave_up.clone(), stopping, end.clone()][..],
            BoxState::GaveUp { step: b"serve" },
            Some(b"The box gave up: serve. /log shows why; Ctrl+C closes.".as_slice()),
        ),
        (
            &[end, gave_up][..],
            BoxState::GaveUp { step: b"serve" },
            Some(b"The box gave up: serve. /log shows why; Ctrl+C closes.".as_slice()),
        ),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Front);
        for told in steps {
            tui.step(told.clone());
        }
        assert_eq!(tui.box_state(), want, "{steps:?}");
        assert_eq!(crate::view::front::gave_up_note(&tui, &f::words()).as_deref(), note, "{steps:?}");
    }
    // The bound is on the kept bytes, before invalid UTF-8 is replaced for display.
    let bound = usize::try_from(crate::limits::LOG_LINE).unwrap();
    let oversized = [0xff; 513];
    for size in [bound - 1, bound, bound + 1] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Stage { stage: BoxStage::Failed { why: copy_of(&oversized[..size]) } });
        assert_eq!(tui.box_stage(), &BoxStage::Failed { why: copy_of(&oversized[..size.min(bound)]) });
    }
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Stage { stage: BoxStage::Failed { why: copy_of(b"first\nsecond") } });
    assert_eq!(tui.box_stage(), &BoxStage::Failed { why: copy_of(b"first") });
}
