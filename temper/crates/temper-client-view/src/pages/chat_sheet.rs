//! A chat's entry to the shared goal sheet and its start fact (client/pages/README.md).
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use jig_client::{contract as c, thread::Line};
use skein_lib::Wall;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, chat::document, home},
};

pub(crate) fn bar(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings) {
    if super::chat::asking(root) {
        return;
    }
    let Some(chat) = root.pages().chat() else { return };
    if chat.goal_from(root).is_some() {
        if let Some(chip) = chat.goal_chip(root) {
            let name = words.say(Word::ChatGoalLink, &[Hole::Name(skein_lib::bytes::copy_of(chip.title.bytes()))]);
            crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Task { task: chip.task } }));
            b.style(look::CHAT_GOAL);
            b.text(Run::Plain, &name);
            b.close();
        }
        return;
    }
    let Some(task) = chat.task(root.client()) else { return };
    let Some(draft) = &chat.draft else { return };
    if draft.body.trim_ascii().is_empty() || !temper_client_domain::pages::drafts_offered(root, task.project) {
        return;
    }
    let home::Repository::In { .. } = home::repository(root, task.project) else { return };
    b.open(Role::Button);
    b.name(&words.say(Word::ChatStartGoal, &[]));
    b.style(look::CHAT_GOAL);
    b.style(look::GOAL_PRIMARY);
    b.bind(bindings.insert(Act::OpenStart {
        draft: Some(temper_client_domain::pages::chat::sheet::path(&document::slug(&draft.title))),
        chat: Some(chat.task),
    }));
    b.close();
}

pub(crate) fn started(root: Option<&Domain>) -> Option<(u64, &c::Chip)> {
    let root = root?;
    if super::chat::asking(root) {
        return None;
    }
    let chat = root.pages().chat()?;
    Some((chat.goal_from(root)?.1, chat.goal_chip(root)?))
}

/// A fact follows every line at the same time and precedes the first later line.
pub(crate) fn before(
    b: &mut Builder,
    started: &mut Option<(u64, &c::Chip)>,
    line: Option<&Line<'_>>,
    words: &Words,
    bindings: &mut Bindings,
) {
    let Some((at, chip)) = *started else { return };
    if let Some(line) = line {
        let time = match line {
            Line::Words { at, .. } => *at,
            Line::Turn { at, .. } | Line::Undecoded { at, .. } => Some(*at),
            Line::Activity { line } => Some(line.source.at),
            Line::Provisional { .. } => None,
        };
        let Some(time) = time else { return };
        if time <= Wall::from_nanos(at) {
            return;
        }
    }
    b.open(Role::Slot);
    b.style(look::TURN_TOOL);
    crate::cards::text(b, Role::Paragraph, &words.say(Word::DelegationLineGoal, &[]));
    b.open(Role::Paragraph);
    b.style(look::TOOL_FILE);
    b.text(Run::Plain, chip.title.bytes());
    b.close();
    let state = words.say_word(jig_client::words::Word::Phase(chip.phase), &[]);
    b.open(Role::Status);
    if chip.phase == c::Phase::Held || chip.phase == c::Phase::Failed {
        b.style(look::STATUS_HELD);
    }
    b.name(&state);
    b.text(Run::Plain, &state);
    b.close();
    let name = words.say(Word::DelegationOpen, &[]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Task { task: chip.task } }));
    b.text(Run::Plain, &name);
    b.close();
    b.close();
    *started = None;
}
