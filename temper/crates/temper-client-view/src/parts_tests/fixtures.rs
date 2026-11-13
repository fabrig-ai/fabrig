pub(crate) use crate::view_kit::{
    FORGE, INBOX_TASK, NOW, PROJECT, SEED, account_root, asking_me, browser_words, cards, chat, chat_helpers, choice,
    confirm, connect_forge, deliver_me, document, domain_limits, find, first_run, forge_root, forge_root_accounts,
    hidden_playground_root, inbox, limits, needs, node_text, path, person_find, public_records, root_at, root_event,
    root_in_project, task, task_helpers, text, tree_limits, welcome, welcome_account, words,
};
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
