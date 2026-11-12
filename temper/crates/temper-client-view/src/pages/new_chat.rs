//! A new chat on the session layout (temper/docs/design/client/pages/README.md and temper/docs/design/client/forge/family.md, 7 and temper/docs/design/client/pages/welcome.md, 2).
use crate::{
    frame::Bindings,
    look,
    words::{Times, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, home::Repository},
};

/// Enter and Send share the model, playground and in-flight start readiness.
pub(crate) fn ready_to_start(root: &Domain) -> bool {
    let Some(chats) = root.pages().chats() else { return false };
    let Some(me) = root.pages().me(root.client()) else { return false };
    root.pages().choice().valid(me)
        && crate::choice::can_send(root)
        && temper_client_domain::pages::composer::over(&chats.words, chats.beyond, me.words_bytes) == 0
        && chats.start.is_none()
        && !root.forge().playground_starting()
        && root.forge().playground_failure().is_none()
}

#[expect(clippy::too_many_arguments, reason = "new chat passes the Home fallback its trial retry bindings")]
#[expect(clippy::too_many_lines, reason = "the page branches between its browser bar and terminal frame")]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    _forge: &mut crate::forge::bindings::Bindings,
    text_limit: u32,
    times: &mut Times,
) {
    let project = root.pages().project();
    if let Some(project) = project
        && root.pages().me(root.client()).is_some()
        && super::project_name(root.pages().me(root.client()), project).is_none()
    {
        super::home::main(b, root, w, bindings, accounts, project, times, 0, &crate::forge_sign_in::Forges::new());
        return;
    }
    let terminal = super::terminal::shown(root, w);
    b.open(Role::Main);
    if terminal {
        super::terminal::style(b, root.pages().look());
    }
    let draft = root.pages().address() == &(Address::NewChat { draft: true });
    b.style(if draft { look::MODE_SPLIT } else { look::MAIN });
    b.style(look::SESSION);
    if terminal {
        let heading = w.say(Word::NewChat, &[]);
        let draft = if draft {
            super::terminal::Draft::Shown
        } else if match project {
            Some(project) => temper_client_domain::pages::drafts_offered(root, project),
            None => false,
        } {
            super::terminal::Draft::Add
        } else {
            super::terminal::Draft::None
        };
        super::terminal::top(
            b,
            root,
            w,
            bindings,
            super::terminal::Top { project, heading: &heading, name: &heading, hint: None, number: None, draft },
        );
    } else {
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
        if !draft
            && let Some(project) = project
            && temper_client_domain::pages::drafts_offered(root, project)
        {
            super::document::add(b, w, bindings, Act::Go { address: Address::NewChat { draft: true } });
        }
        b.close();
        b.close();
    }
    b.open(Role::Region);
    b.name(&w.say_card(crate::cards::P::ConversationLog, &[]));
    b.style(look::CONVO);
    b.open(Role::Slot);
    b.style(look::THREAD);
    let playground = playground_start(root);
    let show_start =
        playground.is_some() || (root.forge().playground().is_none() && root.forge().playground_starting());
    if show_start {
        b.open(Role::Slot);
        b.style(look::STARTER);
        playground_status(b, root, w);
        if let Some(playground) = playground {
            context(b, root, w);
            failure(b, root, playground, w, bindings);
        }
    }
    let repository = match project {
        Some(project) => temper_client_domain::pages::home::repository(root, project),
        None => Repository::None,
    };
    let line = if draft {
        w.say(Word::ChatWrites, &[Hole::Name(w.say(Word::DraftName, &[]))])
    } else {
        match repository {
            Repository::In { owner, name } => w.say(Word::NewChatIn, &[Hole::Name(owner), Hole::Name(name)]),
            Repository::Unknown
            | Repository::Copying { .. }
            | Repository::CopiesIn { .. }
            | Repository::CopyOffer { .. }
            | Repository::Add { .. }
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
        }
    };
    text(b, &line, look::META_LINE);
    if let Some(playground) = playground {
        suggestions(b, root, playground, w, bindings);
    }
    if show_start {
        b.close();
    }
    if terminal {
        super::terminal::notes(b, root, w);
    }
    b.close();
    if !terminal || !super::terminal::commands::typing(root) {
        crate::choice::above_composer(b, root, None, w, bindings, times);
    }
    composer(b, root, w, bindings, text_limit);
    b.close();
    if draft {
        super::document::empty(b, w);
    }
    b.close();
}

fn composer(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_limit: u32) {
    let Some(chats) = root.pages().chats() else { return };
    let ready = ready_to_start(root);
    let name = w.say(
        if root.pages().address() == &(Address::NewChat { draft: true }) {
            Word::DraftWords
        } else {
            Word::NewChatWords
        },
        &[],
    );
    let terminal = super::terminal::shown(root, w);
    super::session::foot(b, &name);
    if terminal {
        super::terminal::prompt(b, w);
    }
    b.open(Role::TextArea);
    b.name(&name);
    b.placeholder(&name);
    b.value(&chats.words, temper_client_domain::pages::shown(root.pages()));
    if root.pages().switcher().renaming().is_none() {
        b.focus(if terminal { temper_client_domain::pages::shown(root.pages()) } else { 0 });
    }
    if ready || (terminal && super::terminal::commands::typing(root)) {
        b.state(State::SubmitsOnEnter);
    } else {
        bindings.block_submissions();
    }
    b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
    b.close();
    if !terminal {
        b.open(Role::Button);
        b.name(&w.say(Word::Send, &[]));
        if chats.words.is_empty() || !ready {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert(Act::StartChat));
        b.close();
    }
    if terminal {
        b.close();
        super::terminal::under(b, root, None, w, bindings, super::terminal::Hint::Commands);
    } else {
        super::session::foot_row(b, root, None, w, bindings, Word::ChatSendHint);
    }
    if let Some(me) = root.pages().me(root.client()) {
        super::session::over(
            b,
            w,
            temper_client_domain::pages::composer::over(&chats.words, chats.beyond, me.words_bytes),
        );
    }
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

fn playground_status(b: &mut Builder, root: &Domain, words: &Words) {
    let status = if root.forge().playground_failure().is_some() {
        Word::PlaygroundNotStarted
    } else if root.forge().playground_starting() {
        Word::PlaygroundStarting
    } else {
        Word::Ready
    };
    let status = words.say(status, &[]);
    b.open(Role::Status);
    b.style(look::STATUS_QUIET);
    b.name(&status);
    b.text(Run::Plain, &status);
    b.close();
}

fn context(b: &mut Builder, root: &Domain, words: &Words) {
    let Some(playground) = root.forge().playground() else { return };
    if playground.project.is_none()
        || playground.project != root.pages().project()
        || playground.state != temper_client_domain::forge::PlaygroundState::Running
        || root.pages().me(root.client()).is_none()
    {
        return;
    }
    b.open(Role::Slot);
    b.style(look::CONTEXT);
    b.style(look::META_LINE);
    b.open(Role::Paragraph);
    b.open(Role::Text);
    b.style(look::REPOSITORY);
    b.run(
        Run::Plain,
        &words.say(
            Word::PlaygroundRepository,
            &[Hole::Name(playground.owner.clone()), Hole::Name(playground.name.clone())],
        ),
    );
    b.close();
    b.close();
    b.close();
}

/// The playground whose start `NewChat` shows (temper/docs/design/client/forge/family.md, 7).
pub(crate) fn playground_start(root: &Domain) -> Option<&temper_client_domain::forge::Playground> {
    let Address::NewChat { .. } = root.pages().address() else { return None };
    let playground = root.forge().playground()?;
    let selected = match playground.project {
        Some(project) => root.pages().project() == Some(project),
        None => true,
    };
    if playground.state != temper_client_domain::forge::PlaygroundState::Resetting && selected {
        Some(playground)
    } else {
        None
    }
}

fn failure(
    b: &mut Builder,
    root: &Domain,
    playground: &temper_client_domain::forge::Playground,
    words: &Words,
    bindings: &mut Bindings,
) {
    if let Some(failure) = root.forge().playground_failure() {
        use temper_client_domain::forge::playground::StartFailure;
        let message = match failure {
            StartFailure::Group(why) => {
                words.say(Word::PlaygroundGroupFailed, &[Hole::Name(temper_client_domain::forge::words::lossy(why))])
            }
            StartFailure::Account => words.say(Word::PlaygroundAccountFailed, &[]),
            StartFailure::Adoption => words.say(
                Word::PlaygroundAdoptionFailed,
                &[Hole::Name(playground.owner.clone()), Hole::Name(playground.name.clone())],
            ),
            StartFailure::NotConnected => words.say(Word::PlaygroundNotConnected, &[]),
        };
        b.open(Role::Alert);
        b.name(&message);
        b.text(Run::Plain, &message);
        b.close();
        b.open(Role::Button);
        b.name(&words.say(Word::PlaygroundTryAgain, &[]));
        b.bind(bindings.insert(Act::RetryPlayground));
        b.close();
    }
}

fn suggestions(
    b: &mut Builder,
    root: &Domain,
    playground: &temper_client_domain::forge::Playground,
    words: &Words,
    bindings: &mut Bindings,
) {
    let Some(chats) = root.pages().chats() else { return };
    b.open(Role::List);
    b.style(look::STARTERS);
    b.name(&words.say(Word::SuggestedRequests, &[]));
    for (cue, value) in playground.cues.iter().enumerate() {
        b.open(Role::Item);
        b.open(Role::Button);
        b.name(value);
        if chats.start.is_some() || root.forge().playground_starting() || root.forge().playground_failure().is_some() {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert(Act::Suggest { cue: u8::try_from(cue).expect("eight cue indices fit") }));
        b.close();
        b.close();
    }
    b.close();
}
