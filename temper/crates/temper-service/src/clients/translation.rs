//! Clients' events and the root's party vocabulary, without retained state.
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use temper_engine_domain as root;

/// Translate the core events the clients' component issues. An internal core
/// event on this boundary aborts the component instead of deciding root work.
pub fn to_root(event: core::Event) -> Result<root::PartyInput, Box<kit::Event>> {
    let value = match event {
        core::Event::ProposeSettings { reply_to, sign_in, key, digest, change } => {
            let kinds = match kind_values(&change.items) {
                Ok(kinds) => kinds,
                Err(why) => {
                    return Err(Box::new(kit::Event::Reply { to: reply_to, reply: people::Reply::Refused(why) }));
                }
            };
            root::PartyInput::ProposeSettings { reply_to, sign_in, key, digest, change, kinds }
        }
        core::Event::ChangeSettings { reply_to, sign_in, key, digest, change } => {
            let kinds = match kind_values(&change.items) {
                Ok(kinds) => kinds,
                Err(why) => {
                    return Err(Box::new(kit::Event::Reply { to: reply_to, reply: people::Reply::Refused(why) }));
                }
            };
            root::PartyInput::ChangeSettings { reply_to, sign_in, key, digest, change, kinds }
        }
        core::Event::SignIn { reply_to, identity, held } => root::PartyInput::SignedIn { reply_to, identity, held },
        core::Event::People(people::Event::Ask { reply_to, sign_in, key, ask, models }) => {
            root::PartyInput::Ask { reply_to, sign_in, key, ask, models }
        }
        core::Event::People(people::Event::Whose { reply_to, sign_in }) => {
            root::PartyInput::Whose { reply_to, sign_in }
        }
        core::Event::People(people::Event::SignOut { reply_to, sign_in }) => {
            root::PartyInput::SignOut { reply_to, sign_in }
        }
        core::Event::Key { reply_to, sign_in, key, ask, carried } => {
            root::PartyInput::Key { reply_to, sign_in, key, ask, carried }
        }
        core::Event::Read { reply, sign_in, read } => root::PartyInput::Read { reply_to: reply, sign_in, read },
        core::Event::Watch { watcher, sign_in, subject, window } => {
            root::PartyInput::Watch { watcher, sign_in, subject, window }
        }
        core::Event::WatchRun { watcher, sign_in, task, turn, offset } => {
            root::PartyInput::WatchRun { watcher, sign_in, task, turn, offset }
        }
        core::Event::Unwatch { watcher } => root::PartyInput::Unwatch { watcher },
        core::Event::Delivered { watcher } => root::PartyInput::Delivered { watcher },
        core::Event::StreamDelivered { watcher } => root::PartyInput::StreamDelivered { watcher },
        core::Event::SettingsCommit { .. }
        | core::Event::SubscribeTopic { .. }
        | core::Event::Fill { .. }
        | core::Event::Stopping
        | core::Event::Marked { .. }
        | core::Event::ReadLoaded { .. }
        | core::Event::Application { .. }
        | core::Event::ApplicationPart { .. }
        | core::Event::ApplicationFinished { .. }
        | core::Event::ProjectionSettled { .. }
        | core::Event::SettledCall { .. }
        | core::Event::EffectStart { .. }
        | core::Event::EffectConnector(..)
        | core::Event::EffectDeadline
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
        | core::Event::Period { .. }
        | core::Event::TranscriptLoaded { .. }
        | core::Event::People(..)
        | core::Event::Fleet(..)
        | core::Event::TurnPayload { .. }
        | core::Event::AnswerPayload { .. }
        | core::Event::AcceptedTurn { .. }
        | core::Event::RefusedPayload { .. }
        | core::Event::Activate { .. }
        | core::Event::PreparedAgent { .. }
        | core::Event::EndedTaskLoaded { .. }
        | core::Event::SeenConversationLoaded { .. }
        | core::Event::EndedReadLoaded { .. }
        | core::Event::PartyInputsLoaded { .. }
        | core::Event::EndedProposal { .. }
        | core::Event::DecidedProposal { .. }
        | core::Event::EndedPersonTask { .. }
        | core::Event::EndedEscalation { .. }
        | core::Event::NamedAnswer { .. }
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
        | core::Event::Notes(..)
        | core::Event::Act { .. } => return Err(Box::new(kit::Event::Abort)),
    };
    Ok(value)
}

/// Decode complete values at the schema ceilings before the root routes them
/// (temper/docs/design/domain/root.md, 5.6).
fn kind_values(items: &[core::settings::Item]) -> Result<Box<[root::KindValue]>, people::Refusal> {
    use core::settings as s;
    let ceilings = temper_client_protocol_wire::v1::CEILINGS;
    let bounds = temper_engine_smith::kinds::Bounds {
        segments: ceilings.landing_pattern_segments,
        segment_bytes: ceilings.landing_pattern_segments_item,
        gates: ceilings.landing_rule_gates,
        approvals: ceilings.landing_rule_approvals,
    };
    let mut values = skein_lib::List::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for (index, item) in items.iter().enumerate() {
        let s::Setting::Application { kind: kind @ 1..=3, scope, number } = item.setting else { continue };
        let value = match &item.edit {
            s::Edit::Removed | s::Edit::Reset => continue,
            s::Edit::Set(s::Value::Application(bytes)) => temper_engine_smith::kinds::decode(kind, bytes, &bounds),
            s::Edit::Set(_) => None,
        };
        let Some(value) = value else {
            return Err(people::Refusal::NotWellFormed {
                at: u32::try_from(index).expect("bounded item index").checked_add(1).expect("item count fits"),
                why: people::Malformed::Settings { setting: item.setting, fault: people::SettingFault::Shape },
            });
        };
        values.push(root::KindValue { kind, scope, number, value }).expect("one value per set item");
    }
    Ok(values.into_boxed())
}

/// One immediate or released output belonging to the clients' component.
#[must_use]
pub fn to_component(request: root::PartyRequest) -> Option<kit::Event> {
    match request {
        root::PartyRequest::SignInRefused { to } => Some(kit::Event::SignInRefused { to }),
        root::PartyRequest::WatchRefused { watcher, why } => Some(kit::Event::WatchRefused { watcher, why }),
        root::PartyRequest::RunText { watcher, turn, from, to, text, cut, call, writing } => {
            Some(kit::Event::RunText { watcher, turn, from, to, text, cut, call, writing })
        }
        root::PartyRequest::Deliver(delivery) => delivered(delivery),
    }
}
fn delivered(delivery: root::Delivery) -> Option<kit::Event> {
    match delivery {
        root::Delivery::WebReply { to, reply } => Some(kit::Event::Reply { to, reply }),
        root::Delivery::CarriedUnused { number } => {
            Some(kit::Event::Application(kit::FromApplication::Unused { number }))
        }
        root::Delivery::Read { reply, answer } => Some(kit::Event::Read { reply, answer }),
        root::Delivery::Document { watcher, document } => Some(kit::Event::Delivery { watcher, document }),
        root::Delivery::WatchEnded { watcher, why } => Some(kit::Event::WatchEnded { watcher, why }),

        // Durable results are read through the core's existing documents and
        // watches, as in the reference's Held::Result routing.
        root::Delivery::PlaygroundNote { .. }
        | root::Delivery::Forges { .. }
        | root::Delivery::Application { .. }
        | root::Delivery::ForgeDocument { .. }
        | root::Delivery::Panel { .. }
        | root::Delivery::SourceSignedOut { .. }
        | root::Delivery::Result { .. }
        | root::Delivery::Host(_)
        | root::Delivery::Link { .. }
        | root::Delivery::ProjectMade { .. }
        | root::Delivery::ProjectReleased { .. }
        | root::Delivery::ForgeCommitted { .. }
        | root::Delivery::ForgeCall { .. }
        | root::Delivery::Bound { .. }
        | root::Delivery::Procedure { .. }
        | root::Delivery::Relay { .. }
        | root::Delivery::ReadEscalationDecision { .. }
        | root::Delivery::Fleet(_)
        | root::Delivery::AssignedJob { .. }
        | root::Delivery::Assigned { .. }
        | root::Delivery::ForgeRead { .. }
        | root::Delivery::ReadLoad { .. }
        | root::Delivery::Drained
        | root::Delivery::Refuse { .. }
        | root::Delivery::TurnBusy { .. }
        | root::Delivery::NotesLoad { .. }
        | root::Delivery::Start(_)
        | root::Delivery::Load { .. } => None,
    }
}
/// Refuse decisions while restoration or the first periods are incomplete.
#[must_use]
#[expect(clippy::match_same_arms, reason = "Whose keeps its explicit not-ready answer beside the party refusals")]
pub fn not_ready(input: root::PartyInput) -> Option<kit::Event> {
    match input {
        root::PartyInput::Whose { reply_to, .. } => {
            Some(kit::Event::Reply { to: reply_to, reply: people::Reply::Again(people::Again::NotReady) })
        }
        root::PartyInput::SignedIn { reply_to, .. }
        | root::PartyInput::SignOut { reply_to, .. }
        | root::PartyInput::Ask { reply_to, .. }
        | root::PartyInput::ProposeSettings { reply_to, .. }
        | root::PartyInput::ChangeSettings { reply_to, .. }
        | root::PartyInput::Key { reply_to, .. }
        | root::PartyInput::Application { reply_to, .. } => {
            Some(kit::Event::Reply { to: reply_to, reply: people::Reply::Again(people::Again::NotReady) })
        }
        root::PartyInput::ForgeRead { reply: reply_to, .. } | root::PartyInput::Read { reply_to, .. } => {
            Some(kit::Event::Read { reply: reply_to, answer: core::reads::Answer::None(core::reads::Why::Busy) })
        }
        root::PartyInput::Watch { watcher, .. } | root::PartyInput::WatchRun { watcher, .. } => {
            Some(kit::Event::WatchRefused { watcher, why: core::WatchRefusal::Busy })
        }
        root::PartyInput::Unwatch { .. }
        | root::PartyInput::Delivered { .. }
        | root::PartyInput::StreamDelivered { .. }
        | root::PartyInput::StartRecurring { .. }
        | root::PartyInput::Period { .. }
        | root::PartyInput::ProcedureStep { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{ReplyTo, Token};

    #[test]
    fn an_owner_settings_request_crosses_the_clients_boundary() {
        let event = core::Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(1)),
            sign_in: 2,
            key: [3; 16],
            digest: [4; 32],
            change: Box::new(core::settings::Change {
                against: 5,
                items: Box::new([]),
                running: core::settings::Running::Stays,
            }),
        };
        let input = to_root(event).expect("client settings request");
        let root::PartyInput::ChangeSettings { sign_in, key, digest, change, .. } = &input else {
            panic!("settings request")
        };
        assert_eq!((*sign_in, *key, *digest, change.against), (2, [3; 16], [4; 32], 5));
        let Some(kit::Event::Reply { reply: people::Reply::Again(people::Again::NotReady), .. }) = not_ready(input)
        else {
            panic!("settings request must be retried until ready");
        };
    }

    #[test]
    fn a_client_cannot_send_the_internal_settings_commit() {
        let Err(event) = to_root(core::Event::SettingsCommit { room: 1 }) else { panic!("internal commit admitted") };
        let kit::Event::Abort = *event else {
            panic!("internal settings commit must abort the client");
        };
    }
}
