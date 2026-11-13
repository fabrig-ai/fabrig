//! The Chats page (temper/docs/design/client/pages/README.md and temper/docs/design/client/view.md, 1 and temper/docs/design/client/pages/welcome.md, 2).
pub(crate) use crate::common::chats::{active, outcome_text, status_words, words_not_begun, working};
use crate::{
    bindings::Bindings,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::{EndedGroup, TaskLine};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
pub(crate) use temper_client_domain::pages::chats::ended;
use temper_client_domain::{Address, ChatsFilter, Domain, pages::Act};
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    _forge: &mut crate::forge::bindings::Bindings,
    _text_limit: u32,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    b.style(crate::look::CHATS_PAGE);
    intro(b, root, words);
    repository_context(b, root, words, bindings);
    list(b, root, words, bindings, times);
    b.close();
}
fn project_name(root: &Domain, words: &Words) -> Box<[u8]> {
    match root.pages().project() {
        Some(project) => match super::project_name(root.pages().me(root.client()), project) {
            Some(name) => name,
            None => words.say(Word::Project, &[Hole::Number(u64::from(project))]),
        },
        None => words.say(Word::ProjectPane, &[]),
    }
}
fn intro(b: &mut Builder, root: &Domain, words: &Words) {
    use temper_client_domain::pages::home::{Repository, repository};
    b.open(Role::Slot);
    b.style(crate::look::CHATS_INTRO);
    b.open(Role::Paragraph);
    b.style(crate::look::CHATS_KICKER);
    let kicker = words.say(Word::ChatsKicker, &[Hole::Name(project_name(root, words))]);
    b.name(&kicker);
    b.text(Run::Plain, &kicker);
    b.close();
    let repository = match root.pages().project() {
        Some(project) => Some(repository(root, project)),
        None => None,
    };
    let headline = match repository {
        Some(Repository::In { owner, name }) => {
            words.say(Word::ChatsHeadlineIn, &[Hole::Name(owner), Hole::Name(name)])
        }
        Some(
            Repository::Unknown
            | Repository::Copying { .. }
            | Repository::CopiesIn { .. }
            | Repository::CopyOffer { .. }
            | Repository::Add { .. }
            | Repository::Adopting(_)
            | Repository::Offer { .. }
            | Repository::GitHub { .. }
            | Repository::None,
        )
        | None => words.say(Word::ChatsHeadline, &[]),
    };
    text(b, Role::Heading { level: 1 }, &headline);
    b.open(Role::Paragraph);
    b.style(crate::look::CHATS_LEDE);
    let lede = words.say(if offers_drafts(root) { Word::ChatsDraftLede } else { Word::ChatsLede }, &[]);
    b.name(&lede);
    b.text(Run::Plain, &lede);
    b.close();
    b.close();
}
fn repository_context(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings) {
    if let Some(project) = root.pages().project()
        && temper_client_domain::forge::listed::without_repository(root.forges(), root.forge().playground(), project)
            == Some(true)
    {
        b.open(Role::Slot);
        b.style(crate::look::CHATS_BAND);
        b.open(Role::Paragraph);
        b.open(Role::Text);
        b.style(crate::look::CHATS_TAG);
        b.run(Run::Plain, &words.say(Word::NoRepositoryYet, &[]));
        b.close();
        if words.shell == crate::words::Shell::Terminal {
            b.text(Run::Plain, b" ");
        }
        words.text(b, Word::ChatsNoRepositoryLine, &[Hole::Name(project_name(root, words))]);
        b.close();
        crate::bindings::link(
            b,
            &words.say(Word::ForgeAdopt, &[]),
            bindings.insert(Act::Go { address: Address::Adopt { project } }),
        );
        b.style(crate::look::CHATS_ACT);
        words.text(b, Word::ForgeAdopt, &[]);
        b.close();
        b.close();
    }
}

fn offers_drafts(root: &Domain) -> bool {
    match root.pages().project() {
        Some(project) => temper_client_domain::pages::drafts_offered(root, project),
        None => false,
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "the list filters and merges two borrowed row windows without scratch storage"
)]
fn list(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, times: &mut Times) {
    let Some(chats) = root.pages().chats() else { return };
    b.open(Role::Slot);
    b.style(crate::look::CHATS_SECTION);
    b.open(Role::Slot);
    b.style(crate::look::CHATS_SECTION_HEAD);
    b.open(Role::Slot);
    b.style(crate::look::FILTERS);
    for (filter, word) in [
        (ChatsFilter::All, Word::AllChats),
        (ChatsFilter::Live, Word::LiveChats),
        (ChatsFilter::Drafts, Word::DraftsChats),
        (ChatsFilter::Closed, Word::ClosedChats),
    ] {
        if filter == ChatsFilter::Drafts && !offers_drafts(root) {
            continue;
        }
        b.open(Role::Button);
        b.name(&words.say(word, &[]));
        if (if chats.filter == ChatsFilter::Drafts && !offers_drafts(root) { ChatsFilter::All } else { chats.filter })
            == filter
        {
            b.state(State::Pressed);
        }
        b.bind(bindings.insert(Act::ChatFilter { filter }));
        b.close();
    }
    b.close();
    crate::bindings::link(
        b,
        &words.say(Word::NewChatLink, &[]),
        bindings.insert(Act::Go { address: Address::NewChat { draft: false } }),
    );
    b.style(crate::look::CHATS_ACT);
    words.text(b, Word::NewChatLink, &[]);
    b.close();
    b.close();
    let live = match chats.live(root.client()) {
        Some(live) if chats.filter != ChatsFilter::Closed => live.rows.as_slice(),
        _ => &[],
    };
    let (closed, more) = match &chats.page {
        Some(page) if chats.filter != ChatsFilter::Live => match &page.ended {
            EndedGroup::Rows { rows, after, .. } => (rows.as_slice(), after.is_some()),
            EndedGroup::None => (&[][..], false),
        },
        _ => (&[][..], false),
    };
    let only_drafts = chats.filter == ChatsFilter::Drafts && offers_drafts(root);
    let mut any = false;
    for rows in [live, closed] {
        for line in rows {
            any |= !only_drafts || chats.documents.contains_key(&line.chip.task);
        }
    }
    if any {
        b.open(Role::List);
        b.name(&words.say(Word::YourChats, &[]));
        b.style(crate::look::CHATS_ROWS);
        // Select from the two bounded windows without allocating scratch or
        // cloning documents. Only the previous key survives each selection.
        let mut previous = None;
        for _ in 0..live.len().checked_add(closed.len()).expect("two bounded windows fit") {
            let mut next: Option<&TaskLine> = None;
            for rows in [live, closed] {
                for line in rows {
                    if only_drafts && !chats.documents.contains_key(&line.chip.task) {
                        continue;
                    }
                    let key = (line.ended_at.unwrap_or(line.last_activity), line.chip.task);
                    if let Some(previous) = previous
                        && !before(previous, key)
                    {
                        continue;
                    }
                    if let Some(chosen) = next
                        && !before(key, (chosen.ended_at.unwrap_or(chosen.last_activity), chosen.chip.task))
                    {
                        continue;
                    }
                    next = Some(line);
                }
            }
            let Some(chosen) = next else { break };
            row(b, root, chosen, words, bindings, times);
            previous = Some((chosen.ended_at.unwrap_or(chosen.last_activity), chosen.chip.task));
        }
        b.close();
    } else if chats.known(root.client()) && (chats.filter != ChatsFilter::Drafts || chats.documents_known()) {
        let empty = words.say(Word::ChatsNone, &[]);
        b.open(Role::Paragraph);
        b.name(&empty);
        b.style(crate::look::EMPTY);
        b.text(Run::Plain, &empty);
        b.close();
    }
    if more {
        b.open(Role::Button);
        b.style(crate::look::CHATS_MORE);
        b.name(&words.say(Word::MoreClosedChats, &[]));
        b.bind(bindings.insert(Act::ClosedChats));
        b.close();
    }
    b.close();
}

fn before(a: (skein_lib::Wall, u64), b: (skein_lib::Wall, u64)) -> bool {
    a.0 > b.0 || (a.0 == b.0 && a.1 < b.1)
}

pub(crate) fn row(
    b: &mut Builder,
    root: &Domain,
    line: &TaskLine,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Item);
    b.style(crate::look::CHATS_ROW);
    if line.ended_at.is_some() || ended(&line.chip) {
        b.style(crate::look::CHATS_ROW_ENDED);
    }
    b.key(skein_ui::Key { owner: crate::look::owner(), kind: 10, a: line.chip.task, b: 0 });
    b.open(Role::Slot);
    b.style(crate::look::CHATS_TOP);
    // A slot keeps the number's command available in the terminal (client/pages/README.md).
    b.open(Role::Slot);
    let asking = match root.pages().me(root.client()) {
        Some(me) => temper_client_domain::pages::chats::asking(me, line.project, &line.chip.executor),
        None => false,
    };
    let number = words.say(Word::Chat, &[Hole::Number(line.chip.task)]);
    crate::bindings::link(b, &number, bindings.insert(Act::Go { address: Address::Chat { task: line.chip.task } }));
    b.text(Run::Plain, &number);
    if asking && words.shell == crate::words::Shell::Terminal {
        words.text(b, Word::ToolSeparator, &[]);
        words.text(b, Word::Asking, &[]);
    }
    b.close();
    if asking && words.shell == crate::words::Shell::Browser {
        words.text(b, Word::ToolSeparator, &[]);
        words.text(b, Word::Asking, &[]);
    }
    if let Some(chats) = root.pages().chats() {
        if let Some(title) = chats.documents.get(&line.chip.task) {
            words.text(b, Word::ToolSeparator, &[]);
            words.text(
                b,
                Word::ChatRowWrites,
                &[Hole::Name(temper_client_domain::pages::chat::document::slug(title.bytes()))],
            );
        }
        if let Some(goal) =
            temper_client_domain::pages::chats::Chats::goal_from(root.forge(), line.project, line.chip.task)
            && let Some(chip) = chats.goal_chip(root.client(), goal)
        {
            words.text(b, Word::ToolSeparator, &[]);
            words.text(b, Word::ChatRowGoal, &[]);
            crate::bindings::link(
                b,
                chip.title.bytes(),
                bindings.insert(Act::Go { address: Address::Task { task: goal } }),
            );
            b.text(Run::Plain, chip.title.bytes());
            b.close();
        }
    }
    b.close();
    b.open(match words.shell {
        crate::words::Shell::Terminal => Role::Paragraph,
        crate::words::Shell::Browser => Role::Slot,
    });
    if let Some(me) = root.pages().me(root.client()) {
        let (token, name) = if line.ended_at.is_some() {
            (crate::look::STATUS_QUIET, words.say(Word::ChatClosed, &[]))
        } else {
            status_words(&line.chip, (&line.waits_on, None, None), (line.project, line.requester), None, me, words)
        };
        b.open(Role::Status);
        b.style(token);
        if token == crate::look::STATUS_YOURS || token == crate::look::STATUS_WAITING {
            b.style(crate::look::STATE_NEEDS);
        }
        b.name(&name);
        b.text(Run::Plain, &name);
        b.close();
    }
    words.text(b, Word::ToolSeparator, &[]);
    b.open(Role::Text);
    b.style(crate::look::TIME);
    b.run(Run::Plain, &times.say(words, TimeForm::Span, line.ended_at.unwrap_or(line.last_activity)));
    b.close();
    b.close();
    b.close();
    b.open(Role::Paragraph);
    b.style(crate::look::CHATS_LINE);
    b.name(line.chip.title.bytes());
    b.text(Run::Plain, line.chip.title.bytes());
    b.close();
    if (line.ended_at.is_some() || ended(&line.chip)) && line.flags.retention == jig_client::contract::Retention::Kept {
        b.open(Role::Paragraph);
        b.style(crate::look::DETAILS);
        let kept = words.say(Word::ChatKept, &[]);
        b.name(&kept);
        b.text(Run::Plain, &kept);
        b.close();
    }
    b.open(Role::Paragraph);
    b.style(crate::look::DETAILS);
    let spend = words.say(Word::ChatRowSpend, &[Hole::Number(line.budget.numbers.spent)]);
    b.name(&spend);
    b.text(Run::Plain, &spend);
    if line.ended_at.is_none() && !ended(&line.chip) && line.chip.details.details.live_delegates > 0 {
        words.text(b, Word::ToolSeparator, &[]);
        words.text(b, Word::ChatDelegates, &[Hole::Number(u64::from(line.chip.details.details.live_delegates))]);
    }
    b.close();
    b.close();
}

/// A successful start/message is already represented by the conversation.
pub(crate) fn accepted(outcome: &jig_client::Outcome) -> bool {
    let jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Done(_))) = outcome
    else {
        return false;
    };
    true
}

#[cfg(test)]
pub(crate) fn status(
    b: &mut Builder,
    chip: &jig_client::contract::Chip,
    (waits_on, delegates, run_wait): (
        &jig_client::contract::WaitsOn,
        Option<u32>,
        Option<&jig_client::contract::TaskRunWait>,
    ),
    (project, requester): (u32, jig_client::contract::Requester),
    wake: Option<&jig_client::contract::WakePolicy>,
    me: &jig_client::contract::Me,
    words: &Words,
) {
    let (token, name) = status_words(chip, (waits_on, delegates, run_wait), (project, requester), wake, me, words);
    b.open(Role::Status);
    b.style(token);
    b.name(&name);
    b.text(Run::Plain, &name);
    b.close();
}

/// Whether chats in this project can write drafts (temper/docs/design/client/pages/settings.md, 2).
#[must_use]
pub fn drafts_offered(root: &Domain, project: u32) -> bool {
    temper_client_domain::pages::drafts_offered(root, project)
}
