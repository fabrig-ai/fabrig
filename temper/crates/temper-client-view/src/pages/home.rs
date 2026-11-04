//! A project's first page, and its settings (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{Address, ChatsFilter, Domain, pages::Act};

/// A project's first page (temper/docs/design/client/README.md, 3.5).
pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    page(b, root, w, bindings, project, false);
}

/// A project's settings (temper/docs/design/client/README.md, 3.5).
pub(crate) fn settings(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    page(b, root, w, bindings, project, true);
}

fn page(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, settings: bool) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    if settings {
        b.style(crate::look::SETTINGS);
    }
    let me = root.pages().me(root.client());
    let name = crate::pages::project_name(me, project);
    let fallback = w.say(Word::Project, &[Hole::Number(u64::from(project))]);
    let missing = me.is_some() && name.is_none();
    let name = name.as_deref().unwrap_or(&fallback);
    let heading = if settings { w.say(Word::SettingsFor, &[Hole::Name(name.into())]) } else { name.into() };
    b.open(Role::Heading { level: 1 });
    b.name(&heading);
    b.text(Run::Plain, &heading);
    b.close();
    if missing {
        b.open(Role::Paragraph);
        w.text(b, Word::ProjectNotYours, &[Hole::Number(u64::from(project))]);
        b.close();
        let chats = w.say(Word::Chats, &[]);
        crate::link(
            b,
            &chats,
            bindings.insert(Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } }),
        );
        b.text(Run::Plain, &chats);
        b.close();
    } else if settings {
        settings_controls(b, root, w, bindings, project);
    }
    b.close();
}

fn settings_controls(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::List);
    b.open(Role::Item);
    crate::frame::forges_link(b, root, w, bindings);
    b.close();
    b.open(Role::Item);
    crate::frame::accounts_link(b, root, w, bindings);
    b.close();
    b.open(Role::Item);
    let name = w.say(Word::Adoption, &[]);
    crate::link(b, &name, bindings.insert(Act::Go { address: Address::Adopt { project } }));
    b.text(Run::Plain, &name);
    b.close();
    b.close();
    b.close();
    if root.client().link().sign_out_lost {
        b.open(Role::Alert);
        b.name(&w.say(Word::SignOutLost, &[]));
        w.text(b, Word::SignOutLost, &[]);
        b.close();
    }
    if root.pages().me(root.client()).is_some() || root.client().link().sign_out_lost {
        b.open(Role::Button);
        b.name(&w.say(Word::SignOut, &[]));
        b.bind(bindings.insert(Act::SignOut));
        b.close();
    }
}
