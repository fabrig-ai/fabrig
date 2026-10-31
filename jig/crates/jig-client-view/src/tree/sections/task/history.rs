//! Historical pages retain before/after facts and opaque edges (pages.md, 5.2).
use crate::JigBindings;
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{Borrowed, state::action::Act};
use jig_client::contract::HistoryRow;
pub(super) fn history(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    for (subtree, phrase) in [(false, JigPhrase::OwnHistory), (true, JigPhrase::SubtreeHistory)] {
        crate::tree::pages::work::control(
            b,
            words,
            bindings,
            phrase,
            Act::HistoryScope { subtree },
            client.task_page().subtree_history == subtree,
        );
    }
    b.open(Role::List);
    for row in &client.history_rows().rows {
        b.open(Role::Item);
        match row {
            HistoryRow::Change(row) => {
                b.key(skein_ui::Key { owner: crate::Owner::Jig.opaque(), kind: 41, a: row.task, b: row.revision });
                crate::tree::fragments::history::render(b, row, words, bindings, client.client.domain_limits().rows);
                task_link(b, row.task, words, bindings);
            }
            HistoryRow::Milestone(row) => {
                b.key(skein_ui::Key { owner: crate::Owner::Jig.opaque(), kind: 42, a: row.task, b: row.position });
                crate::tree::fragments::milestone::render(b, row, client, words, bindings);
                task_link(b, row.task, words, bindings);
            }
        }
        b.close();
    }
    b.close();
    crate::tree::pages::cut(b, words, client.history_rows().cut);
    crate::tree::pages::edges(
        b,
        words,
        bindings,
        &client.history_rows().edges,
        crate::state::page::task::HISTORY_WINDOW,
    );
}

fn task_link(b: &mut Builder, task: u64, words: &Words, bindings: &mut JigBindings) {
    crate::tree::frame::control(
        b,
        words,
        bindings,
        Role::Link,
        JigPhrase::TaskPage,
        &[Hole::Number(task)],
        Act::Go { address: crate::state::address::Address::task(task) },
    );
}
