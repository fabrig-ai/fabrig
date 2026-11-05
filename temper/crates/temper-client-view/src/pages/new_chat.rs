//! A new chat on the session layout (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Times, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{Act, home::Repository},
};

/// Enter and Send share the model, playground and in-flight start readiness.
pub(crate) fn ready_to_start(root: &Domain) -> bool {
    let Some(chats) = root.pages().chats() else { return false };
    let Some(me) = root.pages().me(root.client()) else { return false };
    root.pages().choice().valid(me)
        && crate::choice::can_send(root)
        && chats.start.is_none()
        && !root.forge().playground_starting()
        && root.forge().playground_failure().is_none()
}

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    _forge: &mut crate::forge::bindings::Bindings,
    text_limit: u32,
    times: &mut Times,
) {
    let project = root.pages().project();
    if let Some(project) = project
        && root.pages().me(root.client()).is_some()
        && super::project_name(root.pages().me(root.client()), project).is_none()
    {
        super::home::main(b, root, w, bindings, project, times);
        return;
    }
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SESSION);
    super::session::top(
        b,
        root,
        w,
        bindings,
        project,
        super::session::Path::Chats,
        super::session::Heading::Words(&w.say(Word::NewChat, &[])),
    );
    super::session::right(b);
    text(b, &crate::choice::new_chat_model(root, w), look::BYLINE);
    b.close();
    b.close();
    b.open(Role::Region);
    b.name(&w.say_card(crate::cards::P::ConversationLog, &[]));
    b.style(look::CONVO);
    b.open(Role::Slot);
    b.style(look::THREAD);
    let repository = match project {
        Some(project) => temper_client_domain::pages::home::repository(root, project),
        None => Repository::None,
    };
    let line = match repository {
        Repository::In { owner, name } => w.say(Word::NewChatIn, &[Hole::Name(owner), Hole::Name(name)]),
        Repository::Unknown
        | Repository::Adopting(_)
        | Repository::Offer { .. }
        | Repository::GitHub { .. }
        | Repository::None => {
            let name = match project {
                Some(project) => match super::project_name(root.pages().me(root.client()), project) {
                    Some(name) => name,
                    None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
                },
                None => w.say(Word::ProjectPane, &[]),
            };
            w.say(Word::NewChatNoRepository, &[Hole::Name(name)])
        }
    };
    text(b, &line, look::META_LINE);
    b.close();
    crate::choice::composer(b, root, None, w, bindings, times);
    composer(b, root, w, bindings, text_limit);
    b.close();
    b.close();
}

fn composer(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_limit: u32) {
    let Some(chats) = root.pages().chats() else { return };
    let ready = ready_to_start(root);
    let name = w.say(Word::NewChatWords, &[]);
    super::session::foot(b, &name);
    b.open(Role::TextArea);
    b.name(&name);
    b.placeholder(&name);
    b.value(&chats.words, temper_client_domain::pages::shown(root.pages()));
    if root.pages().switcher().renaming().is_none() {
        b.focus(0);
    }
    if ready {
        b.state(State::SubmitsOnEnter);
    } else {
        bindings.block_submissions();
    }
    b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
    b.close();
    b.open(Role::Button);
    b.name(&w.say(Word::Send, &[]));
    if chats.words.is_empty() || !ready {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::StartChat));
    b.close();
    super::session::hint(b, w);
    if chats.refusal.is_some() {
        w.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit);
    }
    if let Some(outcome) = &chats.outcome
        && !super::chats::accepted(outcome)
    {
        super::chats::outcome_text(b, w, outcome, text_limit);
    }
    b.close();
}
fn text(b: &mut Builder, words: &[u8], style: skein_ui::Style) {
    b.open(Role::Paragraph);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
