pub use temper_client_view_kit::*;
pub mod chat;
use jig_client::contract::{ApplicationPart, Part, PartKey, PartKind};

pub fn owner_part_text(words: &crate::words::Words, segments: &[&[u8]]) -> Box<[u8]> {
    let mut builder = skein_ui::Builder::new(&tree_limits());
    crate::forge::owner_part(
        &mut builder,
        &Part { connector: FORGE, kind: PartKind::Resource, key: PartKey::Path(path(segments)) },
        words,
    );
    text(&builder.finish())
}

pub fn part_tree(words: &crate::words::Words, part: ApplicationPart) -> skein_ui::Tree {
    let mut builder = skein_ui::Builder::new(&tree_limits());
    crate::forge::drawn(&mut builder, &part, words);
    builder.finish()
}

pub fn part_text(words: &crate::words::Words, part: ApplicationPart) -> Box<[u8]> {
    text(&part_tree(words, part))
}
