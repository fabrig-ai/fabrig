//! Inbox groups reuse the same cards and their context (client/domain/pages.md, 3).
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::{JigBindings, tree::cards};
use jig_client::thread::window::End;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::{
    Borrowed,
    state::{
        action::Act,
        page::inbox::{Filter, needs},
    },
};
use jig_client::contract::{
    InboxDocument, InboxGroup, InboxReadGroup, InboxRow, OwnerInboxEntry, OwnerInboxObject, Proposer,
};
/// The inbox's filters and counts while either group is shown.
pub fn header(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) -> bool {
    let Some(project) = client.shown_inbox() else {
        return false;
    };
    if !client.shows(crate::state::page::Section::Inbox { group: InboxReadGroup::Needs, project })
        && !client.shows(crate::state::page::Section::Inbox { group: InboxReadGroup::Updates, project })
    {
        return false;
    }
    b.open(Role::Region);
    b.region(crate::regions::JigRegion::InboxHeader.region());
    b.name(&words.say(JigPhrase::Inbox, &[]));
    filters(b, client, words, bindings);
    let page = client.inbox();
    let (count, updates) = match &page.document {
        Some(InboxDocument::Owner(value)) => (u64::from(value.cached_needs), value.cached_updates),
        None => {
            let count = u64::from(page.needs.questions)
                .checked_add(u64::from(page.needs.proposals))
                .expect("inbox count fits")
                .checked_add(u64::from(page.needs.escalations))
                .expect("inbox count fits")
                .checked_add(u64::from(page.needs.person_tasks))
                .expect("inbox count fits");
            (count, page.updates)
        }
    };
    words.text(b, JigPhrase::Needs, &[Hole::Number(count)]);
    words.text(
        b,
        if updates.limited { JigPhrase::MoreUpdates } else { JigPhrase::Updates },
        &[Hole::Number(u64::from(updates.value))],
    );
    b.close();
    true
}
pub(crate) fn group(
    b: &mut Builder,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
    parent: InboxReadGroup,
    project: Option<u32>,
) -> bool {
    if !client.shows(crate::state::page::Section::Inbox { group: parent, project }) {
        return false;
    }
    let group = parent == InboxReadGroup::Needs;
    crate::tree::pages::work::section(
        b,
        words,
        if group { JigPhrase::NeedsYou } else { JigPhrase::InboxUpdates },
        if group { crate::regions::JigRegion::InboxNeeds } else { crate::regions::JigRegion::InboxUpdates },
    );
    let page = client.inbox();
    super::status(b, client, words, crate::state::page::Section::Inbox { group: parent, project });
    b.open(Role::List);
    for entry in &page.rows {
        if needs(entry.group()) == group {
            b.open(Role::Item);
            match entry {
                InboxRow::Owner(entry) => owner_entry(b, client, words, bindings, entry, group),
            }
            b.close();
        }
    }
    b.close();

    window(b, client, words, bindings, parent);
    b.close();
    true
}
pub(crate) fn status(b: &mut Builder, client: &Borrowed<'_>, words: &Words) {
    let text_limit = client.text_limit;
    let page = client.inbox();
    if page.goal_context_pending {
        owner_text(b, words, crate::words::inbox_words::InboxPhrase::GoalContextPending, &[]);
    }
    if page.live_more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
    if let Some(refusal) = &page.refusal {
        words.refusal(b, refusal, text_limit);
    }
}
fn window(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings, parent: InboxReadGroup) {
    let page = client.inbox();
    if let Some(kind) = page.filter.kind
        && InboxReadGroup::of(kind) != parent
    {
        return;
    }
    let window = page.window(parent);
    crate::tree::pages::cut(b, words, window.cut);
    if window.reading.is_some() {
        words.status(b, JigPhrase::Looking, &[]);
    }
    if window.more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
    for end in [End::Before, End::After] {
        let more = match end {
            End::Before => window.before.is_some(),
            End::After => window.after.is_some() || window.more,
        };
        if more {
            crate::tree::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                match end {
                    End::Before => JigPhrase::ReadEarlier,
                    End::After => JigPhrase::ReadLater,
                },
                &[],
                Act::More {
                    window: skein_lib::Token::new(match parent {
                        InboxReadGroup::Needs => crate::state::page::conversations::INBOX_WINDOW,
                        InboxReadGroup::Updates => crate::state::page::conversations::INBOX_UPDATES_WINDOW,
                    }),
                    end,
                },
            );
        } else {
            words.text(
                b,
                match end {
                    End::Before => JigPhrase::WindowStart,
                    End::After => JigPhrase::WindowEnd,
                },
                &[],
            );
        }
    }
}
fn filter(
    b: &mut Builder,
    words: &Words,
    bindings: &mut JigBindings,
    phrase: JigPhrase,
    holes: &[Hole],
    value: Filter,
) {
    crate::tree::frame::control(b, words, bindings, Role::Button, phrase, holes, Act::FilterInbox { filter: value });
}
fn filters(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    let inbox = client.inbox();
    let current = inbox.filter;
    filter(b, words, bindings, JigPhrase::AllProjects, &[], Filter { project: None, ..current });
    if let Some(frame) = &client.frame().document {
        for project in &frame.projects {
            let phrase = JigPhrase::Project;
            let hole = crate::words::name(project.name.bytes());
            filter(b, words, bindings, phrase, &[hole], Filter { project: Some(project.project), ..current });
            crate::tree::fragments::frame_facts::project_loss(b, words, project);
        }
    }
    filter(b, words, bindings, JigPhrase::AllKinds, &[], Filter { kind: None, ..current });
    for kind in [
        InboxGroup::Questions,
        InboxGroup::Proposals,
        InboxGroup::Escalations,
        InboxGroup::PersonTasks,
        InboxGroup::Results,
        InboxGroup::Replies,
    ] {
        let phrase = match kind {
            InboxGroup::Questions => JigPhrase::Question,
            InboxGroup::Proposals => JigPhrase::Proposal,
            InboxGroup::Escalations => JigPhrase::Escalation,
            InboxGroup::PersonTasks => JigPhrase::PersonTask,
            InboxGroup::Results => JigPhrase::Result,
            InboxGroup::Replies => JigPhrase::Reply,
        };
        filter(b, words, bindings, phrase, &[], Filter { kind: Some(kind), ..current });
    }
    filter(b, words, bindings, JigPhrase::AllGoals, &[], Filter { goal: None, ..current });
    for entry in &inbox.rows {
        if let Some(goal) = entry.goal() {
            filter(
                b,
                words,
                bindings,
                JigPhrase::OnlyGoal,
                &[Hole::Number(goal)],
                Filter { goal: Some(goal), ..current },
            );
        }
    }
}

fn owner_text(b: &mut Builder, words: &Words, phrase: crate::words::inbox_words::InboxPhrase, holes: &[Hole]) {
    let saying =
        words.book().say(words.language(), phrase.id(), holes).expect("whole Inbox words and typed holes checked");
    b.text(Run::Plain, &saying.text);
}
fn owner_entry(
    b: &mut Builder,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
    entry: &OwnerInboxEntry,
    group: bool,
) {
    use crate::words::inbox_words::InboxPhrase;
    let task = entry.task_number();
    let (kind, a, value) = match entry.object {
        OwnerInboxObject::Question { message } => (50, task, message),
        OwnerInboxObject::Proposal { proposer, number } => match proposer {
            Proposer::Task(task) => (51, task, number),
            Proposer::Party(party) => (52, party, number),
        },
        OwnerInboxObject::Escalation { revision } => (53, task, revision),
        OwnerInboxObject::PersonTask => (54, task, 0),
        OwnerInboxObject::Reply { message } => (55, task, message),
        OwnerInboxObject::Result { position } => (56, task, position),
    };
    b.key(skein_ui::Key { owner: crate::Owner::Jig.opaque(), kind, a, b: value });
    match entry.object {
        OwnerInboxObject::Question { message } => owner_text(b, words, InboxPhrase::Question, &[Hole::Number(message)]),
        OwnerInboxObject::Proposal { proposer, number } => match proposer {
            Proposer::Task(task) => {
                owner_text(b, words, InboxPhrase::TaskProposal, &[Hole::Number(task), Hole::Number(number)]);
            }
            Proposer::Party(party) => {
                owner_text(b, words, InboxPhrase::PartyProposal, &[Hole::Number(party), Hole::Number(number)]);
            }
        },
        OwnerInboxObject::Escalation { revision } => {
            owner_text(b, words, InboxPhrase::Escalation, &[Hole::Number(revision)]);
        }
        OwnerInboxObject::PersonTask => owner_text(b, words, InboxPhrase::PersonTask, &[]),
        OwnerInboxObject::Reply { message } => owner_text(b, words, InboxPhrase::Reply, &[Hole::Number(message)]),
        OwnerInboxObject::Result { position } => owner_text(b, words, InboxPhrase::Result, &[Hole::Number(position)]),
    }
    if let Some(project) = entry.project {
        owner_text(b, words, InboxPhrase::Project, &[Hole::Number(u64::from(project))]);
    }
    if group {
        if let Some(at) = entry.at {
            words.waiting(b, client.wall(), at);
        }
    } else if entry.unread == Some(true) {
        words.text(b, JigPhrase::NewUpdate, &[]);
    }
    if let Some(activity) = entry.activity {
        owner_text(b, words, InboxPhrase::Activity, &[Hole::Number(activity)]);
    }
    crate::tree::fragments::result::link(b, client, &entry.task, words, bindings);
    if let Some(id) = client.objects().find(entry.card_key())
        && !client.owes(id)
    {
        cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
    }
}
