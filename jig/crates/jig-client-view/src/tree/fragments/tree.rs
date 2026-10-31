//! Whole supplied tree presentation (client/domain/pages.md, 5.2; view.md, 10).
//! Local folds and filters hide presentation only. Source depth, parentage,
//! count folds and continuation are never reconstructed from displayed rows.
use crate::JigBindings;
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::words::tree_words::TreePhrase as P;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run, State};

use crate::{
    Borrowed, Config,
    state::{action::Act, ui::borrowed::task::PlanView},
};
use jig_client::contract::{Chip, ObjectKey, PhaseCounts, Proposer, Tree, TreeRow};
use skein_lib::{Set, Stack};
#[derive(Clone, Copy, Debug)]
enum Visit {
    Enter(u32),
    Leave,
}
pub(crate) fn max_nodes_with_card(l: &Config, client: &jig_client::DomainLimits, card: u32) -> Option<u32> {
    let chip = crate::chip_nodes(l, client)?.max(crate::tree::fragments::chip::max_nodes(l, client)?);
    let row = card
        .checked_mul(client.rows.checked_add(1)?)?
        .checked_add(chip.checked_add(2)?.checked_mul(client.rows)?)?
        .checked_add(chip)?
        .checked_add(client.rows.checked_mul(2)?)?
        .checked_add(12)?;
    row.checked_mul(client.rows)?.checked_add(chip)?.checked_add(client.rows.checked_mul(14)?)?.checked_add(8)
}

fn text(b: &mut Builder, words: &Words, p: P, holes: &[Hole]) {
    let saying = words.book().say(words.language(), p.id(), holes).expect("checked subtree words and typed holes");
    b.text(Run::Plain, &saying.text);
}
fn fold(b: &mut Builder, parent: u64, counts: PhaseCounts, held: u32, words: &Words) {
    b.open(Role::Region);
    let name = words.book().say(words.language(), P::Fold.id(), &[]).expect("checked fold caption");
    b.name(&name.text);
    text(b, words, P::Parent, &[Hole::Number(parent)]);
    for (p, value) in [
        (P::Waiting, counts.waiting),
        (P::Idle, counts.idle),
        (P::Starting, counts.starting),
        (P::Running, counts.running),
        (P::BackingOff, counts.backing_off),
        (P::Closing, counts.closing),
        (P::HeldCount, counts.held),
        (P::Done, counts.done),
        (P::Failed, counts.failed),
        (P::Cancelled, counts.cancelled),
    ] {
        text(b, words, p, &[Hole::Number(u64::from(value))]);
    }
    text(b, words, P::Held, &[Hole::Number(u64::from(held))]);
    b.close();
}
fn control(
    b: &mut Builder,
    client: &Borrowed<'_>,
    chip: &Chip,
    expanded: bool,
    words: &Words,
    bindings: &mut JigBindings,
) {
    if let Some(id) = client.objects().find(ObjectKey::Task(chip.task)) {
        crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            if expanded { JigPhrase::FoldSubtree } else { JigPhrase::OpenSubtree },
            &[],
            Act::Fold { object: id.token(), open: !expanded },
        );
    }
}
fn row(
    b: &mut Builder,
    value: &TreeRow,
    expanded: bool,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::TreeItem);
    b.key(crate::tree::pages::object_key(ObjectKey::Task(value.chip.task)));
    b.name(
        &words
            .say(JigPhrase::ChipNumber, &[Hole::Number(value.chip.task), crate::words::name(value.chip.title.bytes())]),
    );
    if expanded {
        b.state(State::Expanded);
    }
    crate::tree::cards::chip::chip(b, client, &value.chip, words, bindings);
    text(b, words, P::Parent, &[Hole::Number(value.parent)]);
    text(b, words, P::Depth, &[Hole::Number(u64::from(value.depth))]);
    text(b, words, P::Spent, &[Hole::Amount(value.spent)]);
    if value.cut {
        text(b, words, P::RowCut, &[]);
    }
    control(b, client, &value.chip, expanded, words, bindings);
    if !crate::state::page::conversations::live(value.chip.phase) {
        crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::ReadEarlier,
            &[],
            Act::OpenEnded { task: value.chip.task },
        );
    }
    if expanded {
        text(b, words, P::Proposals, &[]);
        for key in &value.proposals {
            let phrase = match key.proposer {
                Proposer::Task(_) => crate::words::result_words::ResultPhrase::ProposalTask,
                Proposer::Party(_) => crate::words::result_words::ResultPhrase::ProposalParty,
            };
            let maker = match key.proposer {
                Proposer::Task(n) | Proposer::Party(n) => n,
            };
            let saying = words
                .book()
                .say(words.language(), phrase.id(), &[Hole::Number(maker), Hole::Number(key.number)])
                .expect("proposal identity words");
            b.text(Run::Plain, &saying.text);
            if let Some(id) = client.objects().find(ObjectKey::Proposal(*key)) {
                crate::tree::cards::card(b, client, id, &crate::tree::cards::Place::TaskPage, words, bindings);
            }
        }
        text(b, words, P::Dependencies, &[]);
        for link in &value.dependencies {
            crate::tree::fragments::result::link(b, client, link, words, bindings);
        }
    }
}
/// Render the retained tree without interpreting its source continuation.
pub fn render(
    b: &mut Builder,
    value: &Tree,
    plan: &PlanView<'_>,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::Tree);
    let name = words.book().say(words.language(), P::Tree.id(), &[]).expect("checked tree caption");
    b.name(&name.text);
    b.open(Role::TreeItem);
    b.key(crate::tree::pages::object_key(ObjectKey::Task(value.root.task)));
    b.name(
        &words
            .say(JigPhrase::ChipNumber, &[Hole::Number(value.root.task), crate::words::name(value.root.title.bytes())]),
    );
    let expanded = plan.owner_expanded(&value.root);
    if expanded {
        b.state(State::Expanded);
    }
    text(b, words, P::Root, &[Hole::Number(value.root.task)]);
    crate::tree::cards::chip::chip(b, client, &value.root, words, bindings);
    control(b, client, &value.root, expanded, words, bindings);
    let count = value.rows.len();
    let mut seen = Set::with_capacity(count);
    let mut stack = Stack::with_capacity(count.checked_mul(2).expect("bounded tree visits"));
    if expanded {
        // Each component starts in source order. The seen set makes synthetic
        // cycles bounded and prevents a source row being emitted twice.
        for index in 0..count {
            if seen.contains(&index) {
                continue;
            }
            let first = value.rows.get(index).expect("source row exists");
            if !plan.owner_visible(first) || hidden(first, value, plan) {
                continue;
            }
            seen.insert(index).expect("source indices fit");
            stack.push(Visit::Enter(index)).expect("component fits");
            for _ in 0..count.checked_mul(2).expect("bounded tree visits") {
                let Some(visit) = stack.pop() else {
                    break;
                };
                match visit {
                    Visit::Leave => b.close(),
                    Visit::Enter(index) => {
                        let value_row = value.rows.get(index).expect("source row exists");
                        let open = plan.owner_expanded(&value_row.chip);
                        row(b, value_row, open, client, words, bindings);
                        stack.push(Visit::Leave).expect("paired close fits");
                        if open {
                            for offset in 0..count {
                                let child = count
                                    .checked_sub(1)
                                    .expect("row exists")
                                    .checked_sub(offset)
                                    .expect("reverse index fits");
                                let next = value.rows.get(child).expect("source row exists");
                                if next.parent == value_row.chip.task
                                    && !seen.contains(&child)
                                    && plan.owner_visible(next)
                                {
                                    seen.insert(child).expect("source indices fit");
                                    stack.push(Visit::Enter(child)).expect("child visit fits");
                                }
                            }
                        }
                    }
                }
            }
            assert!(stack.is_empty(), "all tree items close");
        }
    }
    b.close();
    b.close();
    // Folds remain visible independently of local row filters and expansion.
    for supplied in &value.folded {
        fold(b, supplied.parent, supplied.phases, supplied.held_below, words);
    }
    text(b, words, P::Held, &[Hole::Number(u64::from(value.held_below))]);
    text(b, words, if value.next.is_some() { P::Next } else { P::End }, &[]);
}
fn hidden(row: &TreeRow, tree: &Tree, plan: &PlanView<'_>) -> bool {
    let mut parent = row.parent;
    for _ in 0..tree.rows.len() {
        let mut found = None;
        for candidate in &tree.rows {
            if candidate.chip.task == parent {
                found = Some(candidate);
                break;
            }
        }
        match found {
            Some(candidate) => {
                if !plan.owner_expanded(&candidate.chip) {
                    return true;
                }
                parent = candidate.parent;
            }
            None => return false,
        }
    }
    false
}
