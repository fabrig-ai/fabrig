//! Exhaustive clients' boundary translations (client/domain/client/README.md, 2 and 8).
use jig_client_test_protocol_wire::codec;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use jig_test_domain as root;
use skein_lib::Token;

pub(crate) fn to_component(held: core::Held) -> Option<kit::Event> {
    match held {
        core::Held::PeopleReply { to, reply } => Some(kit::Event::Reply { to, reply }),
        core::Held::PublicRead { reply, answer } => Some(kit::Event::Public { reply, answer }),
        core::Held::Read { reply, answer } => Some(kit::Event::Read { reply, answer }),
        core::Held::Delivery { watcher, document } => Some(kit::Event::Delivery { watcher, document }),
        core::Held::WatchEnded { watcher, why } => Some(kit::Event::WatchEnded { watcher, why }),
        core::Held::CarriedUnused { number } => Some(kit::Event::Application(kit::FromApplication::Unused { number })),
        core::Held::Acted { .. }
        | core::Held::Drained
        | core::Held::ApplicationDelivery { .. }
        | core::Held::ApplicationWatchEnded { .. }
        | core::Held::ReadLoad { .. }
        | core::Held::PublicWait { .. }
        | core::Held::Applied { .. }
        | core::Held::MakeEffect { .. }
        | core::Held::SettledCall { .. }
        | core::Held::Assign { .. }
        | core::Held::Grant { .. }
        | core::Held::Relayed { .. }
        | core::Held::Inbound { .. }
        | core::Held::Relay { .. }
        | core::Held::CallAnswer { .. }
        | core::Held::NeedArrived { .. }
        | core::Held::Result { .. }
        | core::Held::Acknowledge { .. }
        | core::Held::TaskTerminalAcknowledged { .. }
        | core::Held::TaskTurnKept { .. }
        | core::Held::AcknowledgeTurn { .. }
        | core::Held::Cancel { .. }
        | core::Held::ReadList { .. }
        | core::Held::Sources { .. }
        | core::Held::CheckSource { .. }
        | core::Held::SignOutSource { .. }
        | core::Held::SourceSignedOut { .. }
        | core::Held::Refuse { .. }
        | core::Held::TurnBusy { .. }
        | core::Held::StopRun { .. }
        | core::Held::Load(..)
        | core::Held::NotesLoad { .. }
        | core::Held::NotesWritten { .. }
        | core::Held::NotesDeleted { .. } => None,
    }
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
        | core::Now::StartPreparation { .. }
        | core::Now::SeenConversation { .. }
        | core::Now::LoadEndedRead { .. }
        | core::Now::EndedProposal { .. }
        | core::Now::DecidedProposal { .. }
        | core::Now::EndedPersonTask { .. }
        | core::Now::EndedTask { .. }
        | core::Now::PartyInputs { .. }
        | core::Now::CompleteBrief { .. }
        | core::Now::BriefCorePlanned { .. }
        | core::Now::WorkspaceRequest { .. }
        | core::Now::RunPreparationFailed { .. }
        | core::Now::BriefFailed { .. }
        | core::Now::RunPrepared { .. }
        | core::Now::EndedEscalation { .. }
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
        | root::Delivery::Start(_)
        | root::Delivery::Procedure { .. } => return None,
    };
    Some(kit::Event::Application(value))
}

pub(crate) fn watcher(value: &core::Event) -> Option<Token> {
    match value {
        core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } => Some(*watcher),
        core::Event::SettingsCommit { .. }
        | core::Event::Fill { .. }
        | core::Event::Settling
        | core::Event::Stopping
        | core::Event::Marked { .. }
        | core::Event::ReadLoaded { .. }
        | core::Event::PublicRead { .. }
        | core::Event::PublicAnswered { .. }
        | core::Event::PublicDurable { .. }
        | core::Event::Read { .. }
        | core::Event::Act { .. }
        | core::Event::ProposeSettings { .. }
        | core::Event::ChangeModel { .. }
        | core::Event::ChangeSettings { .. }
        | core::Event::Key { .. }
        | core::Event::Application { .. }
        | core::Event::ApplicationPart { .. }
        | core::Event::ApplicationFinished { .. }
        | core::Event::ProjectionSettled { .. }
        | core::Event::SettledCall { .. }
        | core::Event::EffectStart { .. }
        | core::Event::EffectConnector(..)
        | core::Event::EffectDeadline
        | core::Event::SignIn { .. }
        | core::Event::Unwatch { .. }
        | core::Event::Delivered { .. }
        | core::Event::StreamDelivered { .. }
        | core::Event::RunCall { .. }
        | core::Event::RunInput { .. }
        | core::Event::RunText { .. }
        | core::Event::Tasks(..)
        | core::Event::StartRecurring { .. }
        | core::Event::BriefAssembled { .. }
        | core::Event::WorkspacePrepared { .. }
        | core::Event::ClaimPrepared { .. }
        | core::Event::ClaimRefused { .. }
        | core::Event::ConnectorRepair { .. }
        | core::Event::PersonTask { .. }
        | core::Event::Period { .. }
        | core::Event::TranscriptLoaded { .. }
        | core::Event::People(..)
        | core::Event::Fleet(..)
        | core::Event::TurnPayload { .. }
        | core::Event::AnswerPayload { .. }
        | core::Event::AcceptedTurn { .. }
        | core::Event::RefusedPayload { .. }
        | core::Event::Activate { .. }
        | core::Event::EndedReadLoaded { .. }
        | core::Event::SeenConversationLoaded { .. }
        | core::Event::EndedProposal { .. }
        | core::Event::DecidedProposal { .. }
        | core::Event::EndedPersonTask { .. }
        | core::Event::EndedTaskLoaded { .. }
        | core::Event::PartyInputsLoaded { .. }
        | core::Event::EndedEscalation { .. }
        | core::Event::NamedAnswer { .. }
        | core::Event::SubscribeTopic { .. }
        | core::Event::NamedAction { .. }
        | core::Event::DelegateValidated { .. }
        | core::Event::DelegateInputFailed { .. }
        | core::Event::DelegateHoldings { .. }
        | core::Event::ProcedureHoldings { .. }
        | core::Event::ProcedureStep { .. }
        | core::Event::Brief(..)
        | core::Event::Holdings { .. }
        | core::Event::ProjectReleased { .. }
        | core::Event::Account(..)
        | core::Event::View(..)
        | core::Event::Notes(..) => None,
    }
}

pub(crate) fn starting(value: core::Event) -> Option<kit::Event> {
    match value {
        core::Event::ProposeSettings { reply_to, .. }
        | core::Event::ChangeModel { reply_to, .. }
        | core::Event::ChangeSettings { reply_to, .. }
        | core::Event::SignIn { reply_to, .. }
        | core::Event::People(people::Event::Ask { reply_to, .. } | people::Event::SignOut { reply_to, .. }) => {
            Some(kit::Event::Reply { to: reply_to, reply: people::Reply::Again(people::Again::NotReady) })
        }
        core::Event::PublicRead { reply, .. } => Some(kit::Event::Public { reply, answer: core::PublicAnswer::Busy }),
        core::Event::Read { reply, .. } => {
            Some(kit::Event::Read { reply, answer: core::reads::Answer::None(core::reads::Why::Busy) })
        }
        core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } => {
            Some(kit::Event::WatchRefused { watcher, why: core::WatchRefusal::Busy })
        }
        core::Event::SettingsCommit { .. }
        | core::Event::Fill { .. }
        | core::Event::Settling
        | core::Event::Stopping
        | core::Event::Marked { .. }
        | core::Event::ReadLoaded { .. }
        | core::Event::PublicAnswered { .. }
        | core::Event::PublicDurable { .. }
        | core::Event::Act { .. }
        | core::Event::Key { .. }
        | core::Event::Application { .. }
        | core::Event::ApplicationPart { .. }
        | core::Event::ApplicationFinished { .. }
        | core::Event::ProjectionSettled { .. }
        | core::Event::SettledCall { .. }
        | core::Event::EffectStart { .. }
        | core::Event::EffectConnector(..)
        | core::Event::EffectDeadline
        | core::Event::Unwatch { .. }
        | core::Event::Delivered { .. }
        | core::Event::StreamDelivered { .. }
        | core::Event::RunCall { .. }
        | core::Event::RunInput { .. }
        | core::Event::RunText { .. }
        | core::Event::Tasks(..)
        | core::Event::StartRecurring { .. }
        | core::Event::BriefAssembled { .. }
        | core::Event::WorkspacePrepared { .. }
        | core::Event::ClaimPrepared { .. }
        | core::Event::ClaimRefused { .. }
        | core::Event::ConnectorRepair { .. }
        | core::Event::PersonTask { .. }
        | core::Event::Period { .. }
        | core::Event::TranscriptLoaded { .. }
        | core::Event::People(..)
        | core::Event::Fleet(..)
        | core::Event::TurnPayload { .. }
        | core::Event::AnswerPayload { .. }
        | core::Event::AcceptedTurn { .. }
        | core::Event::RefusedPayload { .. }
        | core::Event::Activate { .. }
        | core::Event::EndedReadLoaded { .. }
        | core::Event::SeenConversationLoaded { .. }
        | core::Event::EndedProposal { .. }
        | core::Event::DecidedProposal { .. }
        | core::Event::EndedPersonTask { .. }
        | core::Event::EndedTaskLoaded { .. }
        | core::Event::PartyInputsLoaded { .. }
        | core::Event::EndedEscalation { .. }
        | core::Event::NamedAnswer { .. }
        | core::Event::SubscribeTopic { .. }
        | core::Event::NamedAction { .. }
        | core::Event::DelegateValidated { .. }
        | core::Event::DelegateInputFailed { .. }
        | core::Event::DelegateHoldings { .. }
        | core::Event::ProcedureHoldings { .. }
        | core::Event::ProcedureStep { .. }
        | core::Event::Brief(..)
        | core::Event::Holdings { .. }
        | core::Event::ProjectReleased { .. }
        | core::Event::Account(..)
        | core::Event::View(..)
        | core::Event::Notes(..) => None,
    }
}
