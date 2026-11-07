//! The configuring chat before its first words (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{Domain, pages::Act};

#[expect(clippy::too_many_lines, reason = "one tree keeps the configuring opening, composer and reference in order")]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    project: u32,
    text_limit: u32,
) {
    if root.pages().me(root.client()).is_some()
        && super::project_name(root.pages().me(root.client()), project).is_none()
    {
        let mut times = crate::words::Times::new(skein_lib::Wall::EPOCH);
        super::home::main(b, root, w, bindings, accounts, project, &mut times, 0);
        return;
    }
    let terminal = super::terminal::shown(root, w);
    b.open(Role::Main);
    if terminal {
        super::terminal::style(b, root.pages().look());
    }
    b.style(look::MAIN);
    b.style(look::SESSION);
    let reference = reference(root);
    if let Some(mode) = reference {
        b.style(super::document::style(mode));
    }
    if terminal {
        let heading = w.say(Word::ConfigureTitle, &[]);
        super::terminal::top(
            b,
            root,
            w,
            bindings,
            super::terminal::Top {
                project: Some(project),
                heading: &heading,
                name: &heading,
                hint: None,
                number: None,
                draft: super::terminal::Draft::None,
            },
        );
    } else {
        super::session::top(
            b,
            root,
            w,
            bindings,
            Some(project),
            super::session::Path::Settings,
            super::session::Heading::Words(&w.say(Word::ConfigureTitle, &[])),
        );
        super::session::right(b);
        if let Some(mode) = reference {
            super::document::modes(b, Word::ReferenceGroup, mode, w, bindings);
        }
        super::session::to_terminal(b, w, bindings);
        b.close();
        b.close();
    }
    b.open(Role::Region);
    b.name(&w.say_card(crate::cards::P::ConversationLog, &[]));
    b.style(look::CONVO);
    b.open(Role::Slot);
    b.style(look::THREAD);
    b.open(Role::Slot);
    b.style(look::STARTER);
    let name = match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    };
    let question = w.say(Word::ConfigureQuestion, &[Hole::Name(name.clone())]);
    text(b, Role::Heading { level: 2 }, &question, None);
    paragraph(b, &w.say(Word::ConfigureLede, &[]), look::STARTER_LEDE);
    b.open(Role::List);
    b.name(&w.say(Word::ConfigureSets, &[]));
    b.style(look::STARTER_SETS);
    for (term, line) in [
        (Word::ConfigureWorkTerm, Word::ConfigureWorkLine),
        (Word::ConfigureModelsTerm, Word::ConfigureModelsLine),
        (Word::ConfigureApprovalsTerm, Word::ConfigureApprovalsLine),
    ] {
        b.open(Role::Item);
        pair(b, w, term, line, &[]);
        b.close();
    }
    b.close();
    paragraph(b, &w.say(Word::ConfigureNote, &[]), look::STARTER_NOTE);
    b.open(Role::Region);
    let ways = w.say(Word::ConfigureWays, &[]);
    b.name(&ways);
    b.style(look::STARTER_WAYS);
    let pending = match root.pages().chats() {
        Some(chats) => chats.start.is_some(),
        None => false,
    };
    b.open(Role::Slot);
    b.style(look::STARTER_HEAD);
    if w.shell == Shell::Browser {
        text(b, Role::Heading { level: 3 }, &ways, None);
    }
    b.open(Role::Button);
    b.name(&w.say(Word::ConfigureUnsureTitle, &[]));
    b.style(look::STARTER_UNSURE);
    starter(b, w, bindings, Word::ConfigureUnsure, &[], pending);
    b.close();
    b.close();
    b.open(Role::List);
    b.style(look::STARTERS);
    for (title, line, sent) in [
        (Word::ConfigureAloneTitle, Word::ConfigureAloneLine, Word::ConfigureAlone),
        (Word::ConfigureDesignTitle, Word::ConfigureDesignLine, Word::ConfigureDesign),
    ] {
        b.open(Role::Item);
        b.open(Role::Button);
        pair(b, w, title, line, &[]);
        starter(b, w, bindings, sent, &[], pending);
        b.close();
        b.close();
    }
    if let Some(holes) = model_accounts(root, w) {
        b.open(Role::Item);
        b.open(Role::Button);
        pair(b, w, Word::ConfigureModelsTitle, Word::ConfigureModelsCardLine, &holes);
        starter(b, w, bindings, Word::ConfigureModels, &holes, pending);
        b.close();
        b.close();
    }
    b.close();
    own_way(b, w, bindings, name, pending);
    b.close();
    b.close();
    b.close();
    composer(b, root, w, bindings, pending, text_limit);
    b.close();
    if let Some(mode) = reference
        && mode != temper_client_domain::pages::chat::document::Mode::Chat
    {
        super::reference::pane(b, w);
    }
    b.close();
}

fn paragraph(b: &mut Builder, words: &[u8], style: skein_ui::Style) {
    b.open(Role::Paragraph);
    b.style(style);
    b.text(Run::Plain, words);
    b.close();
}

fn pair(b: &mut Builder, w: &Words, title: Word, line: Word, holes: &[Hole]) {
    let title = w.say(title, &[]);
    let line = w.say(line, holes);
    match w.shell {
        Shell::Browser => {
            b.text(Run::Plain, &title);
            b.text(Run::Plain, &line);
        }
        Shell::Terminal => {
            let words = w.say(Word::HomeAskLine, &[Hole::Name(title), Hole::Name(line)]);
            b.text(Run::Plain, &words);
        }
    }
}

fn starter(b: &mut Builder, w: &Words, bindings: &mut Bindings, sent: Word, holes: &[Hole], pending: bool) {
    let words = w.say(sent, holes);
    if pending {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::Starter { words }));
}

fn own_way(b: &mut Builder, w: &Words, bindings: &mut Bindings, name: Box<[u8]>, pending: bool) {
    b.open(Role::Group);
    b.name(&w.say(Word::ConfigureOwn, &[]));
    b.style(look::STARTER_OWN);
    b.open(Role::Slot);
    b.style(look::STARTER_OWN_TEXT);
    paragraph(b, &w.say(Word::ConfigureOwnTitle, &[]), look::STARTER_OWN_TITLE);
    b.open(Role::List);
    b.name(&w.say(Word::ConfigureOwnSteps, &[]));
    b.style(look::STARTER_OWN_STEPS);
    for step in [
        Word::ConfigureOwnDraft,
        Word::ConfigureOwnCut,
        Word::ConfigureOwnAgents,
        Word::ConfigureOwnBranch,
        Word::ConfigureOwnMain,
    ] {
        b.open(Role::Item);
        b.text(Run::Plain, &w.say(step, &[]));
        b.close();
    }
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::STARTER_OWN_ACTS);
    b.open(Role::Button);
    b.name(&w.say(Word::ConfigureOwnSee, &[]));
    b.bind(bindings.insert(Act::ShowReference));
    b.close();
    b.open(Role::Button);
    b.name(&w.say(Word::ConfigureOwnStart, &[]));
    starter(b, w, bindings, Word::ConfigureOwnSends, &[Hole::Name(name)], pending);
    b.close();
    b.close();
    b.close();
}

fn model_accounts(root: &Domain, w: &Words) -> Option<[Hole; 2]> {
    let me = root.pages().me(root.client())?;
    let models = temper_client_domain::pages::choice::offer(me, root.pages().project()?)?;
    let mut first = None;
    for model in models {
        for account in &model.accounts {
            if let Some((number, provider)) = first {
                if number != *account {
                    return Some([
                        Hole::Name(account_name(root, me, number, provider, w)),
                        Hole::Name(account_name(root, me, *account, model.provider, w)),
                    ]);
                }
            } else {
                first = Some((*account, model.provider));
            }
        }
    }
    None
}

fn account_name(
    root: &Domain,
    me: &jig_client::contract::Me,
    number: u32,
    provider: Option<u32>,
    w: &Words,
) -> Box<[u8]> {
    use jig_client::contract::AccountKind;
    let mut row = None;
    if let Some(document) = root.pages().accounts_document(root.client()) {
        for account in &document.accounts {
            if account.number == number {
                row = Some(account);
                break;
            }
        }
    }
    let provider = match row {
        Some(row) => row.provider,
        None => provider,
    };
    let mut name = b"".as_slice();
    for offer in &me.providers {
        if Some(offer.provider) == provider {
            name = offer.name.bytes();
            break;
        }
    }
    if let Some(row) = row
        && let Some(supplied) = &row.provider_name
        && !supplied.cut
        && !supplied.bytes().is_empty()
    {
        name = supplied.bytes();
    }
    let (brand, plan) = crate::accounts::words::provider_words(name);
    let Some(row) = row else { return crate::choice::shown(brand, w) };
    let mut program = false;
    let mut sign_in = false;
    let mut key = false;
    for source in &row.sources {
        program |= source.kind == AccountKind::Program;
        sign_in |= source.kind == AccountKind::SignIn;
        key |= source.kind == AccountKind::Key;
    }
    let name = if program {
        crate::choice::shown(plan, w)
    } else if row.plan {
        w.say(Word::ConfigurePlan, &[Hole::Name(crate::choice::shown(plan, w))])
    } else if key && !sign_in {
        w.say(Word::ConfigureKey, &[Hole::Name(crate::choice::shown(brand, w))])
    } else {
        crate::choice::shown(brand, w)
    };
    crate::choice::shown(&name, w)
}

fn composer(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, pending: bool, text_limit: u32) {
    let Some(chats) = root.pages().chats() else { return };
    let terminal = super::terminal::shown(root, w);
    super::session::foot(b, &w.say(Word::ConfigureTitle, &[]));
    if terminal {
        super::terminal::prompt(b, w);
    }
    b.open(Role::TextArea);
    let name = w.say(Word::ConfigureWords, &[]);
    b.name(&name);
    b.placeholder(&name);
    b.value(&chats.words, temper_client_domain::pages::shown(root.pages()));
    if root.pages().switcher().renaming().is_none() {
        b.focus(0);
    }
    if pending {
        bindings.block_submissions();
    } else {
        b.state(State::SubmitsOnEnter);
    }
    b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
    b.close();
    if !terminal {
        b.open(Role::Button);
        b.name(&w.say(Word::Send, &[]));
        if pending || chats.words.is_empty() {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert(Act::StartChat));
        b.close();
    }
    if terminal {
        b.close();
        super::terminal::under(b, root, None, w, bindings, super::terminal::Hint::Commands);
    } else {
        super::session::foot_row(
            b,
            root,
            None,
            w,
            bindings,
            if chats.reference.is_some() { Word::DraftSendHint } else { Word::ChatSendHint },
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

fn text(b: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}

pub(crate) fn reference(root: &Domain) -> Option<temper_client_domain::pages::chat::document::Mode> {
    match root.pages().chats() {
        Some(chats) => chats.reference,
        None => None,
    }
}
