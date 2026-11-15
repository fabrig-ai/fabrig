//! The two-line header (temper/docs/design/client/tui/pages/README.md, 3).
use super::text;
use crate::{
    look,
    words::{Word, Words},
};
use skein_lib::bytes::concat;
use skein_ui::{Builder, Role, Style};
use temper_client_view_words::words::Word as Shared;

fn mark(b: &mut Builder, upper: [Option<Style>; 6], lower: [Option<Style>; 6]) {
    for (upper, lower) in upper.into_iter().zip(lower) {
        let (glyph, style) = match (upper, lower) {
            (None, None) => (" ", None),
            (Some(style), None) => ("▀", Some(style)),
            (None, Some(style)) => ("▄", Some(style)),
            (Some(upper), Some(lower)) => {
                assert_eq!(upper, lower, "the mark has no cell of two colours");
                ("█", Some(upper))
            }
        };
        text(b, Role::Text, glyph.as_bytes(), style);
    }
    text(b, Role::Text, b"  ", None);
}

pub(crate) fn draw(b: &mut Builder, front: bool, plain: bool, version: &[u8], address: &[u8], words: &Words) {
    let c = Some(look::MARK_CYAN);
    let blue = Some(look::MARK_BLUE);
    let v = Some(look::MARK_VIOLET);
    b.open(Role::Region);
    b.region(look::HEADER);
    text(b, Role::Paragraph, b"", None);
    // A spread region with one child keeps the line's start and cuts its end.
    b.open(Role::Region);
    b.region(look::HEADER_LINE);
    b.open(Role::Paragraph);
    if !plain {
        mark(b, [None, None, c, c, c, None], [None, blue, c, None, None, None]);
    }
    text(b, Role::Text, &words.say(Word::Title, &[]), Some(look::NAME));
    if !version.is_empty() {
        text(b, Role::Text, &concat(&[b" ", version]), Some(look::HINT));
    }
    b.close();
    b.close();
    b.open(Role::Region);
    b.region(look::HEADER_LINE);
    b.open(Role::Paragraph);
    if !plain {
        mark(b, [None, blue, None, v, None, v], [blue, blue, v, v, v, v]);
    }
    if front {
        text(b, Role::Text, &words.view().say(Shared::WelcomeBoxName, &[]), Some(look::NAME));
        text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::FAINT));
        text(b, Role::Text, &words.say(Word::BoxPurpose, &[]), Some(look::HINT));
    } else {
        text(b, Role::Text, address, Some(look::HINT));
    }
    b.close();
    b.close();
    b.close();
}
