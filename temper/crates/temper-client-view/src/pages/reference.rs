//! The stock reference beside Configure (temper/docs/design/client/README.md, 3.5).
use crate::{
    look,
    words::{Shell, Word, Words},
};
use skein_ui::{Builder, Role, Run};
use temper_client_domain::pages::chat::document;

pub(crate) fn pane(b: &mut Builder, w: &Words) {
    let page = temper_docs::HOW_TEMPER_BUILDS_TEMPER;
    b.open(Role::Region);
    b.name(page.title.as_bytes());
    b.style(look::DOC);
    b.open(Role::Slot);
    b.style(look::DOC_INNER);
    b.open(Role::Paragraph);
    b.style(look::DOC_PATH);
    b.text(Run::Plain, &w.say(Word::ReferencePath, &[]));
    if w.shell == Shell::Terminal {
        b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
    }
    b.text(Run::Plain, &w.say(Word::ReferenceUnchanged, &[]));
    b.close();
    b.open(Role::Heading { level: 2 });
    b.name(page.title.as_bytes());
    b.text(Run::Plain, page.title.as_bytes());
    b.close();
    b.open(Role::Paragraph);
    b.style(look::DOC_SUB);
    b.text(Run::Plain, page.summary.as_bytes());
    b.close();
    if let Some(first) = skein_lib::bytes::find(page.text.as_bytes(), b"## ") {
        for section in document::sections(page.text.as_bytes().get(first..).expect("sections in stock page")) {
            b.open(Role::Slot);
            b.style(look::DOC_SEC);
            if section.heading.starts_with(b"1. ") {
                b.open(Role::Slot);
                b.style(look::DOC_FLOW);
                skein_ui::markdown::read(section.bytes, 1, b);
                b.close();
            } else if section.heading.starts_with(b"5. ") || section.heading.starts_with(b"6. ") {
                closing(b, section.bytes, section.heading.starts_with(b"6. "));
            } else {
                skein_ui::markdown::read(section.bytes, 1, b);
            }
            b.close();
        }
    }
    b.close();
    b.close();
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared document reader splits the compiled UTF-8 page into byte sections"
)]
pub(crate) fn closing(b: &mut Builder, bytes: &[u8], numbers: bool) {
    let text = core::str::from_utf8(bytes).expect("stock page is UTF-8").trim_end();
    if let Some((body, note)) = text.rsplit_once("\n\n")
        && !note.is_empty()
        && !note.starts_with(['#', '-', '|', '`'])
    {
        if numbers && !figures(b, body) {
            skein_ui::markdown::read(bytes, 1, b);
            return;
        }
        if !numbers {
            skein_ui::markdown::read(body.as_bytes(), 1, b);
        }
        b.open(Role::Paragraph);
        b.style(look::DOC_MUTED);
        b.text(Run::Plain, note.as_bytes());
        b.close();
    } else {
        skein_ui::markdown::read(bytes, 1, b);
    }
}

fn figures(b: &mut Builder, text: &str) -> bool {
    let Some((heading, bullets)) = text.split_once("\n\n") else { return false };
    if bullets.is_empty() {
        return false;
    }
    for line in bullets.lines() {
        let Some(line) = line.strip_prefix("- ") else { return false };
        let Some((term, value)) = line.split_once(": ") else { return false };
        if term.is_empty() || value.is_empty() {
            return false;
        }
    }
    skein_ui::markdown::read(heading.as_bytes(), 1, b);
    b.open(Role::Slot);
    b.style(look::DOC_FIGURES);
    for line in bullets.lines() {
        let (term, value) =
            line.strip_prefix("- ").expect("validated bullet").split_once(": ").expect("validated figure");
        b.open(Role::Slot);
        b.open(Role::Term);
        b.text(Run::Plain, term.as_bytes());
        b.close();
        b.open(Role::Description);
        b.text(Run::Plain, value.as_bytes());
        b.close();
        b.close();
    }
    b.close();
    true
}
