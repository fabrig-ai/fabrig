//! Exhaustive clients' boundary translations (client/domain/client/README.md, 2, and client/domain/client/small-store.md, 1).
use jig_client_test_protocol_wire::codec;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use jig_test_domain as root;
use skein_lib::Token;

pub(crate) fn to_component(held: core::Held) -> Option<kit::Event> {
    if let core::Held::PeopleReply { to, reply } = held {
        return Some(kit::Event::Reply { to, reply });
    }
    if let core::Held::PublicRead { reply, answer } = held {
        return Some(kit::Event::Public { reply, answer });
    }
    if let core::Held::Read { reply, answer } = held {
        return Some(kit::Event::Read { reply, answer });
    }
    if let core::Held::Delivery { watcher, document } = held {
        return Some(kit::Event::Delivery { watcher, document });
    }
    if let core::Held::WatchEnded { watcher, why } = held {
        return Some(kit::Event::WatchEnded { watcher, why });
    }
    if let core::Held::CarriedUnused { number } = held {
        return Some(kit::Event::Application(kit::FromApplication::Unused { number }));
    }
    None
}

pub(crate) fn immediate(now: core::Now, admission: &mut Option<Token>) -> Option<kit::Event> {
    match now {
        core::Now::SignInRefused { to } => Some(kit::Event::SignInRefused { to }),
        core::Now::WatchRefused { watcher, why } => {
            if *admission == Some(watcher) {
                *admission = None;
            }
            Some(kit::Event::WatchRefused { watcher, why })
        }
        core::Now::RunText { watcher, turn, from, to, text, cut, call, writing } => {
            Some(kit::Event::RunText { watcher, turn, from, to, text, cut, call, writing })
        }
        core::Now::RelayBusy { .. }
        | core::Now::CallBusy { .. }
        | core::Now::Link(_)
        | core::Now::EffectAnswer { .. }
        | core::Now::TaskRead { .. }
        | core::Now::ProposalRead { .. }
        | core::Now::SettingsAnswer { .. }
        | core::Now::PersonAnswer { .. }
        | core::Now::SettledCallRefused { .. }
        | core::Now::Ask { .. }
        | core::Now::Call { .. }
        | core::Now::DropCall { .. }
        | core::Now::Undelivered { .. }
        | core::Now::Account(..)
        | core::Now::NotesIndexed { .. }
        | core::Now::NotesRecalled { .. }
        | core::Now::NotesRefused { .. }
        | core::Now::NoteBusy { .. }
        | core::Now::DropAssignment { .. }
        | core::Now::Activate { .. }
        | core::Now::CompleteBrief { .. }
        | core::Now::BriefCorePlanned { .. }
        | core::Now::WorkspaceRequest { .. }
        | core::Now::RunPreparationFailed { .. }
        | core::Now::BriefFailed { .. }
        | core::Now::RunPrepared { .. }
        | core::Now::PartBusy { .. }
        | core::Now::RestoreRefused => None,
    }
}

pub(crate) fn application(value: root::Delivery) -> Option<kit::Event> {
    let value = match value {
        root::Delivery::ApplicationDocument { watcher, number, document } => {
            kit::FromApplication::Delivery { watcher, family: number, record: codec::document(&document) }
        }
        root::Delivery::ApplicationRead { reply, number, answer } => {
            kit::FromApplication::Read { reply, family: number, record: Some(codec::read_answer(&answer)) }
        }
        root::Delivery::Application { to, number, outcome } => {
            kit::FromApplication::Answered { to, family: number, record: codec::outcome(&outcome) }
        }
        root::Delivery::Link { .. }
        | root::Delivery::ProjectMade { .. }
        | root::Delivery::CallBusy { .. }
        | root::Delivery::Projection { .. }
        | root::Delivery::Message { .. }
        | root::Delivery::Core(..)
        | root::Delivery::CallAnswer { .. }
        | root::Delivery::AssignedJob { .. }
        | root::Delivery::Assigned { .. }
        | root::Delivery::Fleet(..)
        | root::Delivery::System { .. }
        | root::Delivery::LostRead { .. }
        | root::Delivery::Store(_)
        | root::Delivery::Connector(..)
        | root::Delivery::Resume(_)
        | root::Delivery::ConnectorLoad { .. }
        | root::Delivery::Procedure { .. } => return None,
    };
    Some(kit::Event::Application(value))
}

pub(crate) fn watcher(value: &core::Event) -> Option<Token> {
    if let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = value {
        return Some(*watcher);
    }
    None
}

pub(crate) fn starting(value: core::Event) -> Option<kit::Event> {
    if let core::Event::ProposeSettings { reply_to, .. }
    | core::Event::ChangeModel { reply_to, .. }
    | core::Event::ChangeSettings { reply_to, .. }
    | core::Event::SignIn { reply_to, .. }
    | core::Event::People(people::Event::Ask { reply_to, .. } | people::Event::SignOut { reply_to, .. }) = value
    {
        return Some(kit::Event::Reply { to: reply_to, reply: people::Reply::Again(people::Again::NotReady) });
    }
    if let core::Event::PublicRead { reply, .. } = value {
        return Some(kit::Event::Public { reply, answer: core::PublicAnswer::Busy });
    }
    if let core::Event::Read { reply, .. } = value {
        return Some(kit::Event::Read { reply, answer: core::reads::Answer::None(core::reads::Why::Busy) });
    }
    if let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = value {
        return Some(kit::Event::WatchRefused { watcher, why: core::WatchRefusal::Busy });
    }
    None
}
