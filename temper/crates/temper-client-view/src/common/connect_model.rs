//! Shared setup items (temper/docs/design/client/view-crates.md, 3.3).
use crate::{
    accounts::words::{self, Word as W},
    look,
    words::Words,
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Domain, WayKind,
    pages::{accounts::Way, connect_model as model},
};

pub(crate) fn say(w: &Words, word: W) -> Box<[u8]> {
    words::say(w, word, &[])
}

pub(crate) fn named(w: &Words, word: W, name: &[u8]) -> Box<[u8]> {
    words::say(w, word, &[Hole::Name(crate::words::shown(name, w))])
}

pub(crate) fn names(me: &c::Me, way: Way) -> (&[u8], &[u8]) {
    for offer in &me.providers {
        if offer.provider == way.provider() {
            return words::provider_words(offer.name.bytes());
        }
    }
    (b"", b"")
}

pub(crate) fn label(me: &c::Me, way: Way, ways: &skein_lib::List<Way>, w: &Words) -> Box<[u8]> {
    let (brand, plan) = names(me, way);
    match way {
        Way::SignIn { plan: true, .. } => named(w, W::Continue, plan),
        Way::SignIn { plan: false, .. } => named(w, W::SignInBrand, brand),
        Way::Program { .. } => named(w, W::SubscriptionWay, plan),
        Way::Key { .. } => {
            let mut keys = 0_u32;
            for way in ways {
                if model::model_way(*way).kind == WayKind::Key {
                    keys = keys.checked_add(1).expect("provider ways fit");
                }
            }
            if keys == 1 {
                say(w, W::KeyWayOnly)
            } else {
                let word = match brand.first() {
                    Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => W::KeyWayAn,
                    _ => W::KeyWay,
                };
                named(w, word, brand)
            }
        }
    }
}

pub(crate) fn description(b: &mut Builder, words: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::MODEL_DESCRIPTION);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}

/// The whole command for NAME (client/forge/records.md, *The printed command*).
pub(crate) fn printed_command(root: &Domain, w: &Words, name: &[u8]) -> Box<[u8]> {
    let (environment, answered) = match root.pages().connect_model() {
        Some(page) => (page.program_environment(), page.program_environment_answered()),
        None => (b"".as_slice(), false),
    };
    if answered {
        if environment.is_empty() {
            named(w, W::EnvironmentCommand, name)
        } else {
            words::join(&[environment, b" ", &named(w, W::ProgramLogin, name)])
        }
    } else if let Some(answer) = root.forge().deployment()
        && let Some(line) = &answer.program_login
    {
        words::join(&[line, b" ", &crate::words::shown(name, w)])
    } else {
        named(w, W::Command, name)
    }
}

pub(crate) fn command_prompt(root: &Domain, w: &Words) -> Box<[u8]> {
    let word = match root.forge().deployment() {
        Some(answer) if answer.program_login.is_some() => W::ProgramCommandHere,
        Some(_) | None => W::ProgramCommand,
    };
    named(w, word, b"")
}

pub(crate) fn command(b: &mut Builder, root: &Domain, w: &Words, name: &[u8], welcome: bool) {
    description(b, &command_prompt(root, w));
    b.open(Role::Code);
    b.style(if welcome { look::WELCOME_CMD } else { look::FLOW_CODE });
    b.text(Run::Plain, &printed_command(root, w, name));
    b.close();
}
