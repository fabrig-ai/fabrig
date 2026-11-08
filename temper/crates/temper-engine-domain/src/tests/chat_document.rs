//! Section writes follow temper/docs/design/domain/agent.md, 8.4.3.
use crate::route::document::{heading, made};
use crate::{DocumentAction as A, DocumentRefusal as R};

fn limits() -> jig_domain_notes::Limits {
    jig_domain_notes::Limits {
        scopes: 1,
        entries_per_scope: 1,
        pattern_bytes: 32,
        description_bytes: 16,
        body_bytes: 128,
        references: 1,
        load_rows: 1,
        lines: 1,
        recalled: 1,
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    clippy::disallowed_macros,
    reason = "one table pins the section rules; ordinary tests build byte-bound fixtures"
)]
fn a_document_is_made_by_section() {
    let read = Some((b"Title".as_slice(), b"# Title\n\n## One\nOld\n\n\n## Two\nEnd\n".as_slice()));
    let long_title = b"# abcdefghijklmnopq";
    let bound_body = format!("# Title\n{}", "x".repeat(119));
    let long_body = format!("# Title\n{}", "x".repeat(120));
    let cut_heading = "x".repeat(1021);
    let cut_body = format!("# Title\n## {cut_heading}\n## Left\n");
    let mut roomy = limits();
    roomy.body_bytes = 4096;
    let rows = [
        (
            "first write",
            None,
            A::Write,
            b"#  New  \nBody\n\n".as_slice(),
            limits(),
            Ok((b"New".as_slice().into(), b"#  New  \nBody\n".as_slice().into())),
        ),
        (
            "write again",
            read,
            A::Write,
            b"# New\n",
            limits(),
            Ok((b"New".as_slice().into(), b"# New\n".as_slice().into())),
        ),
        (
            "append",
            read,
            A::Append,
            b"## Three\nNext\n\n",
            limits(),
            Ok((
                b"Title".as_slice().into(),
                b"# Title\n\n## One\nOld\n\n\n## Two\nEnd\n\n## Three\nNext\n".as_slice().into(),
            )),
        ),
        (
            "append no final newline",
            Some((b"Title".as_slice(), b"# Title\n## One\nOld".as_slice())),
            A::Append,
            b"## Two\nEnd",
            limits(),
            Ok((b"Title".as_slice().into(), b"# Title\n## One\nOld\n\n## Two\nEnd\n".as_slice().into())),
        ),
        (
            "replace case and spaces keeps blanks",
            read,
            A::Replace,
            b"##  oNE  \nNew\n",
            limits(),
            Ok((b"Title".as_slice().into(), b"# Title\n\n##  oNE  \nNew\n\n\n## Two\nEnd\n".as_slice().into())),
        ),
        (
            "replace last",
            read,
            A::Replace,
            b"## Two\nNew",
            limits(),
            Ok((b"Title".as_slice().into(), b"# Title\n\n## One\nOld\n\n\n## Two\nNew\n".as_slice().into())),
        ),
        (
            "backtick fence",
            read,
            A::Append,
            b"## Three\n```md\n## Hidden\n```\n",
            limits(),
            Ok((
                b"Title".as_slice().into(),
                b"# Title\n\n## One\nOld\n\n\n## Two\nEnd\n\n## Three\n```md\n## Hidden\n```\n".as_slice().into(),
            )),
        ),
        (
            "tilde fence",
            read,
            A::Append,
            b"## Three\n~~~md\n## Hidden\n~~~\n",
            limits(),
            Ok((
                b"Title".as_slice().into(),
                b"# Title\n\n## One\nOld\n\n\n## Two\nEnd\n\n## Three\n~~~md\n## Hidden\n~~~\n".as_slice().into(),
            )),
        ),
        ("no title", None, A::Write, b"Body", limits(), Err(R::NoTitle)),
        ("empty title", None, A::Write, b"#   \nBody", limits(), Err(R::NoTitle)),
        ("work title", None, A::Write, b"# work: private", limits(), Err(R::NoTitle)),
        (
            "title at bound",
            None,
            A::Write,
            b"# abcdefghijklmnop",
            limits(),
            Ok((b"abcdefghijklmnop".as_slice().into(), b"# abcdefghijklmnop\n".as_slice().into())),
        ),
        ("title past bound", None, A::Write, long_title, limits(), Err(R::TitlePastBound { bytes: 17, bound: 16 })),
        ("no heading", read, A::Append, b"text", limits(), Err(R::NotOneSection)),
        ("two headings", read, A::Replace, b"## One\n## Two", limits(), Err(R::NotOneSection)),
        ("no document append", None, A::Append, b"## One", limits(), Err(R::NoDocument)),
        ("no document replace", None, A::Replace, b"## One", limits(), Err(R::NoDocument)),
        (
            "missing section",
            read,
            A::Replace,
            b"## Missing",
            limits(),
            Err(R::NoSection { headings: Box::new([b"One".as_slice().into(), b"Two".as_slice().into()]), more: false }),
        ),
        (
            "no sections",
            Some((b"Title".as_slice(), b"# Title\n```\n## Hidden\n```\n".as_slice())),
            A::Replace,
            b"## Missing",
            limits(),
            Err(R::NoSection { headings: Box::new([]), more: false }),
        ),
        (
            "headings cut",
            Some((b"Title".as_slice(), cut_body.as_bytes())),
            A::Replace,
            b"## Missing",
            roomy,
            Err(R::NoSection { headings: Box::new([cut_heading.as_bytes().into()]), more: true }),
        ),
        ("section there", read, A::Append, b"##  ONE  \nOther", limits(), Err(R::SectionThere)),
        (
            "body at bound",
            None,
            A::Write,
            bound_body.as_bytes(),
            limits(),
            Ok((b"Title".as_slice().into(), format!("{bound_body}\n").as_bytes().into())),
        ),
        (
            "body past bound",
            None,
            A::Write,
            long_body.as_bytes(),
            limits(),
            Err(R::PastBound { bytes: 129, bound: 128 }),
        ),
    ];
    for (rule, read, action, body, limits, expected) in rows {
        let result = match made(read, action, body, &limits) {
            Ok(m) => Ok((m.description, m.body)),
            Err(why) => Err(why),
        };
        assert_eq!(result, expected, "{rule}");
    }
    assert_eq!(heading(A::Write, b"#  Title  \nBody").as_ref(), b" Title  ");
    assert_eq!(heading(A::Append, b"##  One \nBody").as_ref(), b" One ");
    assert_eq!(heading(A::Replace, b"text").as_ref(), b"");
}

#[test]
fn the_document_flight_joins_the_note_result() {
    use crate::route::host_route::DocumentFlight;
    use crate::{CallAnswer as C, DocumentOutcome as O, Documented, NoteFamily};
    use jig_domain_notes::Refusal as N;
    let written = O::Written { name: 1, revision: 2, lines: 3, added: 1, removed: 1, first: 3 };
    let rows = [
        (None, C::NoteWritten { name: 1, revision: 2, lines: 3, added: 1, removed: 1, first: 3 }, written),
        (None, C::NoteRefused(N::Moved { revision: 3 }), O::Refused(R::Moved)),
        (None, C::NoteRefused(N::Missing), O::Refused(R::Moved)),
        (None, C::NoteRefused(N::Busy), O::Refused(R::Busy)),
        (
            None,
            C::NoteRefused(N::PastBound { bytes: 129, bound: 128 }),
            O::Refused(R::PastBound { bytes: 129, bound: 128 }),
        ),
        (None, C::NoteRefused(N::Full), O::Refused(R::Note(N::Full))),
        (
            Some(NoteFamily::Note),
            C::ToolDenied {
                answer: jig_domain_authority::Answer::Propose,
                findings: Box::new([jig_domain_authority::Finding::Tool]),
            },
            O::Refused(R::Denied {
                family: NoteFamily::Note,
                answer: jig_domain_authority::Answer::Propose,
                findings: Box::new([jig_domain_authority::Finding::Tool]),
            }),
        ),
        (
            Some(NoteFamily::Recall),
            C::ToolDenied {
                answer: jig_domain_authority::Answer::Propose,
                findings: Box::new([jig_domain_authority::Finding::Tool]),
            },
            O::Refused(R::Denied {
                family: NoteFamily::Recall,
                answer: jig_domain_authority::Answer::Propose,
                findings: Box::new([jig_domain_authority::Finding::Tool]),
            }),
        ),
    ];
    for (checked, answer, outcome) in rows {
        let flight = DocumentFlight { action: A::Replace, heading: b"One".as_slice().into(), checked };
        assert_eq!(
            flight.join(answer),
            C::Document(Box::new(Documented { action: A::Replace, heading: b"One".as_slice().into(), outcome }))
        );
    }
    let flight = DocumentFlight { action: A::Write, heading: b"Title".as_slice().into(), checked: None };
    let denied = C::ToolDenied {
        answer: jig_domain_authority::Answer::Refuse,
        findings: Box::new([jig_domain_authority::Finding::Tool]),
    };
    assert_eq!(flight.join(denied.clone()), denied, "the asking family's denial passes unchanged");
}
