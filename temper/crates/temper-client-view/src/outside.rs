//! The frame and the column outside a project
//! (temper/docs/design/client/view.md, 1; temper/docs/design/client/web/look.md).
pub(crate) use crate::column::{column, words};
use crate::{
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run, Style};

/// The top bar outside a project (temper/docs/design/client/view.md, 1).
#[expect(clippy::collapsible_if, reason = "the welcome keeps its deployment content separate from the shared end")]
pub(crate) fn top(
    b: &mut Builder,
    root: &temper_client_domain::Domain,
    w: &Words,
    bindings: &mut crate::bindings::Bindings,
    welcome: bool,
) {
    use crate::words::Shell;
    use temper_client_domain::{Address, address::Filter, pages::Act};
    let me = root.pages().me(root.client());
    b.open(Role::Banner);
    b.style(look::TOP);
    crate::link(
        b,
        &w.say(Word::Brand, &[]),
        bindings.insert(Act::Go { address: Address::Inbox { filter: Filter::ALL } }),
    );
    b.style(look::BRAND);
    w.text(b, Word::Brand, &[]);
    b.close();
    if let Address::Welcome { step } = root.pages().address()
        && welcome
    {
        if let Some(deployment) = root.forge().deployment()
            && deployment.welcome
        {
            crate::pages::welcome::steps(b, root, w, bindings, *step);
        }
    }
    b.open(Role::Slot);
    b.style(look::TOP_END);
    b.open(Role::Region);
    b.region(crate::frame::link_region());
    let (word, style) = match root.client().link().state {
        jig_client::Link::SignedOut => (Word::SignedOut, None),
        jig_client::Link::OutOfDate => (Word::OutOfDate, None),
        jig_client::Link::Offline => (Word::Offline, Some(look::OFFLINE)),
        jig_client::Link::Live if me.is_some() => (Word::Live, Some(look::LIVE)),
        jig_client::Link::Starting | jig_client::Link::Live => (Word::Connecting, Some(look::CONNECTING)),
    };
    status(b, &w.say(word, &[]), style);
    let behind = crate::frame::behind(root);
    if behind && w.shell == Shell::Terminal {
        status(b, &w.say(Word::Behind, &[]), Some(look::BEHIND));
    }
    b.close();
    if !welcome && behind && w.shell == Shell::Browser {
        status(b, &w.say(Word::Behind, &[]), Some(look::BEHIND));
    }
    if !welcome {
        if let Some(me) = me {
            status(b, me.party.name.bytes(), None);
        }
    }
    if !welcome && root.client().link().sign_out_lost {
        b.open(Role::Alert);
        b.name(&w.say(Word::SignOutLost, &[]));
        w.text(b, Word::SignOutLost, &[]);
        b.close();
    }
    if !welcome && (me.is_some() || root.client().link().sign_out_lost) {
        b.open(Role::Button);
        b.name(&w.say(Word::SignOut, &[]));
        b.bind(bindings.insert(Act::SignOut));
        b.close();
    }
    b.close();
    b.close();
}

fn status(b: &mut Builder, said: &[u8], style: Option<Style>) {
    b.open(Role::Status);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(said);
    b.text(Run::Plain, said);
    b.close();
}

/// `me` is being read where the page needs it: the page draws the top bar alone
/// (temper/docs/design/client/view.md, 1).
pub(crate) fn reading_me(root: &temper_client_domain::Domain, words: &Words, ways: &crate::sign_in::Ways) -> bool {
    crate::column::unavailable(root).is_none()
        && root.authorization().is_none()
        && root.approval().is_none()
        && !crate::sign_in::offered(root, words, ways)
        && root.pages().me(root.client()).is_none()
}

/// The deployment has answered and the person has no project (client/pages/README.md).
pub(crate) fn no_project(root: &temper_client_domain::Domain) -> bool {
    if let temper_client_domain::Address::Welcome { .. } = root.pages().address() {
        return false;
    }
    root.pages().me(root.client()).is_some() && root.pages().project().is_none() && root.forge().deployment().is_some()
}

/// The first project's form or the welcome's acts (temper/docs/design/client/pages/README.md).
pub(crate) fn no_project_main(
    b: &mut Builder,
    root: &temper_client_domain::Domain,
    w: &Words,
    bindings: &mut crate::bindings::Bindings,
) {
    use alloc::boxed::Box;
    use temper_client_domain::{
        Address, Step,
        pages::{Act, switcher, welcome},
    };
    column(b, &w.say(Word::NoProjectYet, &[]), &[]);
    if root.forge().deployment().expect("no-project dispatch has deployment").welcome {
        words(b, w, Word::NoProjectSetUp, &[]);
        b.open(Role::Slot);
        b.style(look::OUTSIDE_ACTS);
        let step = if welcome::forge_kept(root) { welcome::go_on(root) } else { Step::Forge { forge: None } };
        crate::link(
            b,
            &w.say(Word::WelcomeSetup, &[]),
            bindings.insert(Act::Go { address: Address::Welcome { step } }),
        );
        b.style(look::OUTSIDE_PRIMARY);
        w.text(b, Word::WelcomeSetup, &[]);
        b.close();
        if welcome::playground_offered(root) == Some(true) && root.forge().playground().is_none() {
            b.open(Role::Button);
            b.style(look::OUTSIDE_SECONDARY);
            b.name(&w.say(Word::WelcomePlayground, &[]));
            b.bind(bindings.insert(Act::Playground));
            b.close();
        }
        b.close();
    } else {
        words(b, w, Word::NoProjectLine, &[]);
        let switcher = root.pages().switcher();
        let pending = switcher.pending().is_some();
        b.open(Role::Form);
        b.name(&w.say(Word::NewProject, &[]));
        b.open(Role::TextField);
        b.name(&w.say(Word::ProjectName, &[]));
        b.state(skein_ui::State::SubmitsOnEnter);
        b.value(switcher.new_words(), temper_client_domain::pages::shown(root.pages()));
        if pending {
            b.state(skein_ui::State::Disabled);
        }
        b.bind(bindings.insert(Act::Switcher(switcher::Act::NewWords { words: Box::default() })));
        b.close();
        b.open(Role::Button);
        b.style(look::OUTSIDE_PRIMARY);
        b.name(&w.say(Word::CreateProject, &[]));
        if pending {
            b.state(skein_ui::State::Disabled);
        }
        b.bind(bindings.insert(Act::Switcher(switcher::Act::Create)));
        b.close();
        if let Some((switcher::Form::New, outcome)) = switcher.refusal() {
            b.open(Role::Alert);
            crate::pages::chats::outcome_text(b, w, outcome, w.name_bytes);
            b.close();
        }
        b.close();
    }
    b.close();
}
