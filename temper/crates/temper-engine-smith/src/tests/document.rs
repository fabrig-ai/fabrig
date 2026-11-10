//! Exact document answers (temper/docs/design/domain/agent/ask-temper.md, 3).
use temper_engine_domain::{
    CallAnswer, DocumentAction as A, DocumentOutcome as O, DocumentRefusal as R, Documented, NoteFamily,
};

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds each document answer and refusal in its exact words")]
fn document_answers_say_what_changed_or_why_not() {
    let rows = [
        (
            A::Write,
            "Title",
            O::Written { name: 12, revision: 1, lines: 64, added: 64, removed: 0, first: 1 },
            "Note 12 revision 1 written · 64 lines\ndocument: \"Title\" written, 64 lines.",
            false,
        ),
        (
            A::Write,
            "Title",
            O::Written { name: 12, revision: 2, lines: 66, added: 18, removed: 6, first: 31 },
            "Note 12 revision 2 written · +18 −6 · line 31 · 66 lines\ndocument: \"Title\" written whole again, 66 lines.",
            false,
        ),
        (
            A::Append,
            "Open questions",
            O::Written { name: 12, revision: 3, lines: 76, added: 18, removed: 6, first: 31 },
            "Note 12 revision 3 written · +18 −6 · line 31 · 76 lines\ndocument: ## Open questions appended, 76 lines.",
            false,
        ),
        (
            A::Replace,
            "Approvals",
            O::Written { name: 12, revision: 3, lines: 76, added: 18, removed: 6, first: 31 },
            "Note 12 revision 3 written · +18 −6 · line 31 · 76 lines\ndocument: ## Approvals replaced, 76 lines.",
            false,
        ),
        (
            A::Write,
            "",
            O::Refused(R::NoTitle),
            "document: write takes the whole document, its first line # TITLE; nothing changed.",
            true,
        ),
        (
            A::Write,
            "",
            O::Refused(R::TitlePastBound { bytes: 300, bound: 256 }),
            "document: the title is 300 bytes, past its bound of 256; nothing changed.",
            true,
        ),
        (
            A::Append,
            "",
            O::Refused(R::NotOneSection),
            "document: append takes one section, its first line ## HEADING and no other heading; nothing changed.",
            true,
        ),
        (
            A::Replace,
            "",
            O::Refused(R::NotOneSection),
            "document: replace takes one section, its first line ## HEADING and no other heading; nothing changed.",
            true,
        ),
        (
            A::Append,
            "",
            O::Refused(R::NoDocument),
            "document: no document yet; write it whole first, its first line # TITLE.",
            true,
        ),
        (
            A::Replace,
            "Approvals",
            O::Refused(R::NoSection {
                headings: Box::new([b"Landing".as_slice().into(), b"Open questions".as_slice().into()]),
                more: false,
            }),
            "document: no section ## Approvals; its sections: ## Landing · ## Open questions. Nothing changed.",
            true,
        ),
        (
            A::Replace,
            "Approvals",
            O::Refused(R::NoSection { headings: Box::new([]), more: false }),
            "document: no section ## Approvals; it has none. Nothing changed.",
            true,
        ),
        (
            A::Replace,
            "Approvals",
            O::Refused(R::NoSection { headings: Box::new([b"Landing".as_slice().into()]), more: true }),
            "document: no section ## Approvals; its sections: ## Landing · …. Nothing changed.",
            true,
        ),
        (
            A::Replace,
            "Approvals",
            O::Refused(R::NoSection { headings: Box::new([]), more: true }),
            "document: no section ## Approvals; its sections: …. Nothing changed.",
            true,
        ),
        (
            A::Append,
            "Approvals",
            O::Refused(R::SectionThere),
            "document: ## Approvals is there already; replace it. Nothing changed.",
            true,
        ),
        (
            A::Append,
            "",
            O::Refused(R::PastBound { bytes: 70000, bound: 65536 }),
            "document: the document would be 70000 bytes, past its bound of 65536; nothing changed.",
            true,
        ),
        (
            A::Replace,
            "Approvals",
            O::Refused(R::Moved),
            "document: the document changed while ## Approvals was written; nothing changed. Call again: it applies to the newest.",
            true,
        ),
        (
            A::Write,
            "Title",
            O::Refused(R::Moved),
            "document: the document changed while \"Title\" was written; nothing changed. Call again: it applies to the newest.",
            true,
        ),
        (A::Write, "", O::Refused(R::Busy), "document: busy; nothing changed. Call again.", true),
        (A::Write, "", O::Refused(R::Unread), "document: the document could not be read; nothing changed.", true),
        (
            A::Write,
            "",
            O::Refused(R::Denied {
                family: NoteFamily::Note,
                answer: jig_domain_authority::Answer::Refuse,
                findings: Box::new([jig_domain_authority::Finding::Tool]),
            }),
            "document: Authority refused: tool family note",
            true,
        ),
        (
            A::Write,
            "",
            O::Refused(R::Denied {
                family: NoteFamily::Recall,
                answer: jig_domain_authority::Answer::Refuse,
                findings: Box::new([jig_domain_authority::Finding::Tool]),
            }),
            "document: Authority refused: tool family recall",
            true,
        ),
        (A::Write, "", O::Refused(R::Note(jig_domain_notes::Refusal::Full)), "document: Note refused: full", true),
    ];
    for (action, heading, outcome, expected, error) in rows {
        let answer = CallAnswer::Document(Box::new(Documented { action, heading: heading.as_bytes().into(), outcome }));
        let (body, failed) = crate::answer(&answer).into_parts();
        assert_eq!(body.as_ref(), expected.as_bytes(), "{expected}");
        assert_eq!(failed, error, "{expected}");
    }
}
