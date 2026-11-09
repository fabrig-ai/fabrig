//! Exact memory answers (temper/docs/design/domain/agent.md, 8.4.4).
use skein_lib::List;
use temper_engine_domain::{CallAnswer, RememberDid as D, RememberRefusal as R, Remembered};

// The party scope holds at most twelve lines (domain/agent.md, 8.4.4).
const BOUND: u32 = 12;

fn full_list() -> List<Box<[u8]>> {
    List::from_boxed(Box::new([
        b"Brief".as_slice().into(),
        b"Ask first".as_slice().into(),
        b"Explain".as_slice().into(),
        b"Read".as_slice().into(),
        b"Check".as_slice().into(),
        b"Plan".as_slice().into(),
        b"Write".as_slice().into(),
        b"Review".as_slice().into(),
        b"Test".as_slice().into(),
        b"Report".as_slice().into(),
        b"Keep".as_slice().into(),
        b"Finish".as_slice().into(),
    ]))
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds every memory answer and refusal in its exact words")]
fn remember_answers_say_what_changed_or_why_not() {
    let rows = [
        (
            Remembered::Done { did: D::Kept, line: 2, lines: List::from_boxed(Box::new([])), bound: BOUND },
            "remember: line 2 kept; 0 of 12 lines.\nNone kept.",
            false,
        ),
        (
            Remembered::Done {
                did: D::Kept,
                line: 2,
                lines: List::from_boxed(Box::new([b"Brief".as_slice().into()])),
                bound: BOUND,
            },
            "remember: line 2 kept; 1 of 12 lines.\n1. Brief",
            false,
        ),
        (
            Remembered::Done { did: D::Kept, line: 2, lines: full_list(), bound: BOUND },
            "remember: line 2 kept; 12 of 12 lines.\n1. Brief\n2. Ask first\n3. Explain\n4. Read\n5. Check\n6. Plan\n7. Write\n8. Review\n9. Test\n10. Report\n11. Keep\n12. Finish",
            false,
        ),
        (
            Remembered::Done { did: D::Changed, line: 2, lines: List::from_boxed(Box::new([])), bound: BOUND },
            "remember: line 2 changed; 0 of 12 lines.\nNone kept.",
            false,
        ),
        (
            Remembered::Done {
                did: D::Changed,
                line: 2,
                lines: List::from_boxed(Box::new([b"Brief".as_slice().into()])),
                bound: BOUND,
            },
            "remember: line 2 changed; 1 of 12 lines.\n1. Brief",
            false,
        ),
        (
            Remembered::Done { did: D::Changed, line: 2, lines: full_list(), bound: BOUND },
            "remember: line 2 changed; 12 of 12 lines.\n1. Brief\n2. Ask first\n3. Explain\n4. Read\n5. Check\n6. Plan\n7. Write\n8. Review\n9. Test\n10. Report\n11. Keep\n12. Finish",
            false,
        ),
        (
            Remembered::Done { did: D::Dropped, line: 2, lines: List::from_boxed(Box::new([])), bound: BOUND },
            "remember: line 2 dropped; 0 of 12 lines.\nNone kept.",
            false,
        ),
        (
            Remembered::Done {
                did: D::Dropped,
                line: 2,
                lines: List::from_boxed(Box::new([b"Brief".as_slice().into()])),
                bound: BOUND,
            },
            "remember: line 2 dropped; 1 of 12 lines.\n1. Brief",
            false,
        ),
        (
            Remembered::Done { did: D::Dropped, line: 2, lines: full_list(), bound: BOUND },
            "remember: line 2 dropped; 12 of 12 lines.\n1. Brief\n2. Ask first\n3. Explain\n4. Read\n5. Check\n6. Plan\n7. Write\n8. Review\n9. Test\n10. Report\n11. Keep\n12. Finish",
            false,
        ),
        (Remembered::Refused(R::NotOneLine), "remember: a line is one line of text, not empty; nothing changed.", true),
        (
            Remembered::Refused(R::TooLong { bytes: 140, bound: 120 }),
            "remember: the line is 140 bytes, past its bound of 120; nothing changed.",
            true,
        ),
        (
            Remembered::Refused(R::Full { bound: BOUND }),
            "remember: the list is full, 12 lines; ask which line to change or drop. Nothing changed.",
            true,
        ),
        (
            Remembered::Refused(R::NoLine { line: 7, lines: 3 }),
            "remember: no line 7; the list has 3. Nothing changed.",
            true,
        ),
        (
            Remembered::Refused(R::NoLine { line: 7, lines: 0 }),
            "remember: no line 7; the list is empty. Nothing changed.",
            true,
        ),
        (
            Remembered::Refused(R::Moved { action: D::Kept, line: 2 }),
            "remember: the list changed while a line was kept; nothing changed. Call again.",
            true,
        ),
        (
            Remembered::Refused(R::Moved { action: D::Changed, line: 2 }),
            "remember: the list changed while line 2 was changed; nothing changed. Call again.",
            true,
        ),
        (
            Remembered::Refused(R::Moved { action: D::Dropped, line: 2 }),
            "remember: the list changed while line 2 was dropped; nothing changed. Call again.",
            true,
        ),
        (Remembered::Refused(R::NoPerson), "remember: this chat keeps nothing for a person; nothing changed.", true),
        (Remembered::Refused(R::Busy), "remember: busy; nothing changed. Call again.", true),
        (Remembered::Refused(R::Unread), "remember: the list could not be read; nothing changed.", true),
        (Remembered::Refused(R::Note(jig_domain_notes::Refusal::Full)), "remember: Note refused: full", true),
    ];
    for (remembered, expected, error) in rows {
        let (body, failed) = crate::answer(&CallAnswer::Remember(Box::new(remembered))).into_parts();
        assert_eq!(body.as_ref(), expected.as_bytes(), "{expected}");
        assert_eq!(failed, error, "{expected}");
    }
}
