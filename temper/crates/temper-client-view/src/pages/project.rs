//! A project's settings page (temper/docs/design/draft/playground-reset.md, 5).
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::Decimal;
use skein_ui::{Builder, Role, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{Act, project::Deletion},
};

pub(crate) fn settings(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    let mut listed = None;
    if let Some(me) = root.pages().me(root.client()) {
        for row in &me.projects {
            if row.project == project {
                listed = Some(row);
                break;
            }
        }
    }
    let (title, description) = if let Some(row) = listed {
        let number = Decimal::of(u64::from(project));
        (
            w.say(Word::ProjectSettingsTitle, &[Hole::Name(skein_lib::bytes::copy_of(row.name.bytes()))]),
            w.say(Word::ProjectSettingsNumber, &[Hole::Name(skein_lib::bytes::copy_of(number.as_bytes()))]),
        )
    } else {
        (w.say(Word::ProjectSettings, &[]), w.say(Word::ProjectGone, &[]))
    };
    crate::cards::text(b, Role::Heading { level: 1 }, &title);
    crate::cards::text(b, Role::Paragraph, &description);
    deletion(b, root, w, bindings, project, listed);
    b.close();
}

fn deletion(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    listed: Option<&c::MeProject>,
) {
    if let Some(playground) = root.demo()
        && playground.project == Some(project)
    {
        return;
    }
    let Some(settings) = root.pages().settings() else { return };
    if settings.project != project {
        return;
    }
    if settings.outcome == Some(Deletion::Deleted) {
        crate::cards::text(b, Role::Heading { level: 2 }, &w.say(Word::ProjectDelete, &[]));
        crate::cards::text(b, Role::Status, &w.say(Word::ProjectDeleted, &[Hole::Name(settings.typed.clone())]));
        return;
    }
    if settings.deleting.is_some() {
        crate::cards::text(b, Role::Heading { level: 2 }, &w.say(Word::ProjectDelete, &[]));
        crate::cards::text(b, Role::Paragraph, &w.say(Word::ProjectDeleteDescription, &[]));
        crate::cards::text(b, Role::Status, &w.say(Word::ProjectDeleting, &[Hole::Name(settings.typed.clone())]));
        return;
    }
    let Some(row) = listed else { return };
    crate::cards::text(b, Role::Heading { level: 2 }, &w.say(Word::ProjectDelete, &[]));
    if row.role != c::HeldRole::Owner {
        crate::cards::text(b, Role::Paragraph, &w.say(Word::ProjectDeleteOwners, &[]));
        return;
    }
    crate::cards::text(b, Role::Paragraph, &w.say(Word::ProjectDeleteDescription, &[]));
    let name = [Hole::Name(skein_lib::bytes::copy_of(row.name.bytes()))];
    if let Some(Deletion::Refused(refusal)) = &settings.outcome {
        let reason = refusal_words(w, refusal);
        crate::cards::text(b, Role::Status, &w.say(Word::ProjectDeleteRefused, &[Hole::Name(reason)]));
    }
    b.open(Role::TextField);
    b.name(&w.say(Word::ProjectDeleteName, &name));
    b.value(&settings.typed, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(Act::DeleteWords { words: Box::default() }));
    b.close();
    b.open(Role::Button);
    b.name(&w.say(Word::ProjectDeleteConfirm, &name));
    if settings.typed.as_ref() != row.name.bytes() {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::DeleteProject));
    b.close();
}

fn refusal_words(w: &Words, refusal: &c::Refusal) -> Box<[u8]> {
    let hole = match refusal {
        c::Refusal::Account { account, .. } => Some(Hole::Number(u64::from(*account))),
        c::Refusal::Taken { by } => Some(Hole::Number(*by)),
        c::Refusal::Llm { model, .. } => Some(Hole::Name(w.model_name(model.bytes()))),
        c::Refusal::Stale(_)
        | c::Refusal::Full(_)
        | c::Refusal::AccountMove { .. }
        | c::Refusal::SignInRedirect { .. }
        | c::Refusal::NoEscalationHolder
        | c::Refusal::NameTaken
        | c::Refusal::Malformed { .. }
        | c::Refusal::Role { .. }
        | c::Refusal::NotWaiting
        | c::Refusal::NoFurther
        | c::Refusal::Authority { .. }
        | c::Refusal::Limit { .. }
        | c::Refusal::Moved { .. }
        | c::Refusal::Ended { .. }
        | c::Refusal::Unknown
        | c::Refusal::NotOffered
        | c::Refusal::KeyConflict
        | c::Refusal::Circle { .. }
        | c::Refusal::Dependency { .. }
        | c::Refusal::Kept { .. }
        | c::Refusal::Provider => None,
    };
    w.say_word(jig_client::words::Word::refusal(refusal), hole.as_slice())
}
