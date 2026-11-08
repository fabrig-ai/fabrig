use crate::pages::chat::document::{self, NoteChange, NoteWrite};

#[test]
fn a_first_write_parses() {
    assert_eq!(
        document::note_write("Note 12 revision 1 written · 64 lines".as_bytes()),
        Some(NoteWrite { note: 12, revision: 1, change: NoteChange::First { lines: 64 } })
    );
}
#[test]
fn a_later_write_parses() {
    for words in
        ["Note 12 revision 3 written · +18 −6 · line 31", "Note 12 revision 3 written · +18 −6 · line 31 · 76 lines"]
    {
        assert_eq!(
            document::note_write(words.as_bytes()),
            Some(NoteWrite { note: 12, revision: 3, change: NoteChange::Later { added: 18, removed: 6, first: 31 } })
        );
    }
}
#[test]
fn a_recall_never_parses() {
    for words in [
        "Note 12 revision 1: A title\nBody\n",
        "Note 12 revision 1 · 64 lines",
        "Note 12 revision 1: A title written · 64 lines",
    ] {
        assert_eq!(document::note_write(words.as_bytes()), None);
    }
}
#[test]
fn a_refusal_never_parses() {
    for words in [
        "Note refused: busy",
        "Note 12 revision X written · 64 lines",
        "Note 12 revision 1 written · 64 lines trailing",
        "Note 12 revision 3 written · +18 −6 · line bad",
        "Note 12 revision 3 written · +18 −6 · line 31 · bad lines",
        "Note 12 revision 3 written · +18 −6 · line 31 · 76 lines trailing",
    ] {
        assert_eq!(document::note_write(words.as_bytes()), None);
    }
}
#[test]
fn slug_is_the_mocks() {
    assert_eq!(document::slug(b" Away MODE! 2026 ").as_ref(), b"away-mode-2026");
    assert_eq!(document::slug("Été / café".as_bytes()).as_ref(), b"t-caf");
}
#[test]
fn slug_cuts_at_28() {
    assert_eq!(document::slug(b"abcdefghijklmnopqrstuvwxyz-123456").as_ref(), b"abcdefghijklmnopqrstuvwxyz-1");
    assert_eq!(document::slug(b"abcdefghijklmnopqrstuvwxyz / 123456").as_ref(), b"abcdefghijklmnopqrstuvwxyz-1");
}
#[test]
fn an_empty_title_slugs_untitled() {
    for title in [b"".as_slice(), b" !!! ", "🍂".as_bytes()] {
        assert_eq!(document::slug(title).as_ref(), b"untitled");
    }
}
const BODY: &[u8] = b"# Title\nDraft\n## First\nOne\n## Second\nTwo\n";
#[test]
fn sections_split_at_each_heading() {
    let sections = collected(BODY);
    assert_eq!(sections.len(), 3);
    assert_eq!(
        (sections.get(0).unwrap().line, sections.get(0).unwrap().heading, sections.get(0).unwrap().bytes),
        (1, b"Title".as_slice(), b"# Title\nDraft\n".as_slice())
    );
    assert_eq!(
        (sections.get(1).unwrap().line, sections.get(1).unwrap().heading, sections.get(1).unwrap().bytes),
        (3, b"First".as_slice(), b"## First\nOne\n".as_slice())
    );
    assert_eq!(
        (sections.get(2).unwrap().line, sections.get(2).unwrap().heading, sections.get(2).unwrap().bytes),
        (5, b"Second".as_slice(), b"## Second\nTwo\n".as_slice())
    );
    let plain = collected(b"No heading\n");
    assert_eq!(
        (plain.get(0).unwrap().line, plain.get(0).unwrap().heading, plain.get(0).unwrap().bytes),
        (1, b"".as_slice(), b"No heading\n".as_slice())
    );
    assert_eq!(document::sections(b"").count(), 1);
}
#[test]
fn a_fence_hides_a_heading() {
    let sections = collected(b"# Title\n```md\n## Hidden\n```\n## Visible\n");
    assert_eq!(sections.len(), 2);
    assert_eq!((sections.get(1).unwrap().line, sections.get(1).unwrap().heading), (5, b"Visible".as_slice()));
}
#[test]
fn section_holding_finds_a_line() {
    for (line, index) in [(0, 0), (1, 0), (2, 0), (3, 1), (4, 1), (5, 2), (6, 2)] {
        assert_eq!(document::section_holding(BODY, line), index);
    }
}
#[test]
fn a_line_past_the_end_is_the_last_section() {
    assert_eq!(document::section_holding(BODY, u32::MAX), 2);
}
fn collected(body: &[u8]) -> skein_lib::List<document::Section<'_>> {
    let capacity = u32::try_from(body.len()).unwrap().checked_add(1).unwrap();
    let mut sections = skein_lib::List::with_capacity(capacity);
    for section in document::sections(body) {
        sections.push(section).unwrap();
    }
    sections
}

#[test]
fn document_calls_parse_their_first_answer_line() {
    for (tool, words, expected) in [
        (
            b"document".as_slice(),
            "Note 12 revision 1 written · 64 lines\ndocument: \"Title\" written, 64 lines.",
            Some(NoteWrite { note: 12, revision: 1, change: NoteChange::First { lines: 64 } }),
        ),
        (
            b"document",
            "Note 12 revision 3 written · +18 −6 · line 31 · 76 lines\ndocument: ## One replaced, 76 lines.",
            Some(NoteWrite { note: 12, revision: 3, change: NoteChange::Later { added: 18, removed: 6, first: 31 } }),
        ),
        (b"note", "Note 12 revision 1 written · 64 lines\nanother line", None),
        (b"document", "document: busy; nothing changed. Call again.", None),
        (b"recall", "Note 12 revision 1 written · 64 lines", None),
    ] {
        assert_eq!(document::tool_write(tool, words.as_bytes()), expected, "{words}");
    }
}
