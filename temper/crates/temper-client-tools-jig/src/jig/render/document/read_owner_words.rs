//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
#[expect(clippy::too_many_lines, reason = "one exhaustive encoding of jig's owner message vocabulary")]
pub(super) fn encode_owner_message_kind(
    value: &jig_client::contract::MessageKind,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::MessageKind::SettingsDecision { proposal, outcome } => {
            out.object_start();
            out.key(b"settings_decision");
            out.object_start();
            out.key(b"proposal");
            out.unsigned(*proposal);
            out.key(b"outcome");
            match outcome {
                jig_client::contract::SettingsOutcome::Accepted { revision } => {
                    out.object_start();
                    out.key(b"accepted");
                    out.unsigned(u64::from(*revision));
                    out.object_end();
                }
                jig_client::contract::SettingsOutcome::Rejected => out.string(b"rejected"),
                jig_client::contract::SettingsOutcome::Stale { newest } => {
                    out.object_start();
                    out.key(b"stale");
                    out.unsigned(u64::from(*newest));
                    out.object_end();
                }
            }
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Escalation { task, revision } => {
            out.object_start();
            out.key(b"escalation");
            out.object_start();
            out.key(b"task");
            out.unsigned(*(task));
            out.key(b"revision");
            out.unsigned(*(revision));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Proposal { proposer, proposal, kind } => {
            encode_message_proposal(*proposer, *proposal, *kind, out, budget);
        }
        jig_client::contract::MessageKind::PersonStalled { task } => {
            out.object_start();
            out.key(b"person_stalled");
            out.object_start();
            out.key(b"task");
            out.unsigned(*(task));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::ProposalDecision { proposal, accepted } => {
            out.object_start();
            out.key(b"proposal_decision");
            out.object_start();
            out.key(b"proposal");
            out.unsigned(*(proposal));
            out.key(b"accepted");
            out.boolean(*(accepted));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Words => out.string(b"words"),
        jig_client::contract::MessageKind::EffectNotMade { entry } => {
            out.object_start();
            out.key(b"effect_not_made");
            out.object_start();
            out.key(b"entry");
            out.unsigned(*(entry));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Amendment { revision } => {
            out.object_start();
            out.key(b"amendment");
            out.object_start();
            out.key(b"revision");
            out.unsigned(*(revision));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Met { dependency, removed } => {
            out.object_start();
            out.key(b"met");
            out.object_start();
            out.key(b"dependency");
            out.unsigned(*dependency);
            out.key(b"removed");
            out.boolean(*removed);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Question => out.string(b"question"),
        jig_client::contract::MessageKind::QuestionOwed => out.string(b"question_owed"),
        jig_client::contract::MessageKind::NothingAwaited => out.string(b"nothing_awaited"),
        jig_client::contract::MessageKind::Answer { question } => {
            out.object_start();
            out.key(b"answer");
            out.object_start();
            out.key(b"question");
            out.unsigned(*(question));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::QuestionAnswered { question } => {
            out.object_start();
            out.key(b"question_answered");
            out.object_start();
            out.key(b"question");
            out.unsigned(*question);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::QuestionEnded { question } => {
            out.object_start();
            out.key(b"question_ended");
            out.object_start();
            out.key(b"question");
            out.unsigned(*question);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Notice { subscription, target, state } => {
            encode_message_notice(*subscription, *target, *state, out, budget);
        }
        jig_client::contract::MessageKind::Timer { subscription } => {
            out.object_start();
            out.key(b"timer");
            out.object_start();
            out.key(b"subscription");
            out.unsigned(*(subscription));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::News { subscription, class } => {
            out.object_start();
            out.key(b"news");
            out.object_start();
            out.key(b"subscription");
            out.unsigned(*(subscription));
            out.key(b"class");
            encode_owner_news_class(*class, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::MessageKind::Result(field_0) => {
            out.object_start();
            out.key(b"result");
            encode_owner_word_result(field_0, out, budget);
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_message_proposal_kind(
    value: jig_client::contract::MessageProposalKind,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::MessageProposalKind::Settings => out.string(b"settings"),
        jig_client::contract::MessageProposalKind::Batch => out.string(b"batch"),
        jig_client::contract::MessageProposalKind::Effect => out.string(b"effect"),
        jig_client::contract::MessageProposalKind::Amend => out.string(b"amend"),
        jig_client::contract::MessageProposalKind::Widen => out.string(b"widen"),
        jig_client::contract::MessageProposalKind::Release => out.string(b"release"),
    }
}

pub(super) fn encode_owner_news_class(
    value: jig_client::contract::NewsClass,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::NewsClass::Wakes => out.string(b"wakes"),
        jig_client::contract::NewsClass::Kept => out.string(b"kept"),
        jig_client::contract::NewsClass::Dropped => out.string(b"dropped"),
    }
}

pub(super) fn encode_owner_notice_state(
    value: jig_client::contract::NoticeState,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::NoticeState::Held => out.string(b"held"),
        jig_client::contract::NoticeState::Done => out.string(b"done"),
        jig_client::contract::NoticeState::Failed => out.string(b"failed"),
        jig_client::contract::NoticeState::Cancelled => out.string(b"cancelled"),
    }
}

pub(super) fn encode_owner_word_result(
    value: &jig_client::contract::WordResult,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::WordResult::Approved { state, words_present } => {
            out.object_start();
            out.key(b"approved");
            out.object_start();
            out.key(b"state");
            budget.words((state).bytes(), out);
            out.key(b"words_present");
            out.boolean(*(words_present));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::WordResult::Report => out.string(b"report"),
        jig_client::contract::WordResult::Verdict { code, fields } => {
            out.object_start();
            out.key(b"verdict");
            out.object_start();
            out.key(b"code");
            out.unsigned(u64::from(*(code)));
            out.key(b"fields");
            out.array_start();
            for item_1 in fields {
                super::encode_result_field(item_1, out, budget);
            }
            out.array_end();
            out.object_end();
            out.object_end();
        }
        jig_client::contract::WordResult::Change { connector, kind, resource } => {
            out.object_start();
            out.key(b"change");
            out.object_start();
            out.key(b"connector");
            out.unsigned(u64::from(*(connector)));
            out.key(b"kind");
            out.unsigned(u64::from(*(kind)));
            out.key(b"resource");
            out.unsigned(*(resource));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::WordResult::Failed => out.string(b"failed"),
        jig_client::contract::WordResult::Cancelled => out.string(b"cancelled"),
        jig_client::contract::WordResult::Job => out.string(b"job"),
    }
}

fn encode_message_proposal(
    proposer: u64,
    proposal: u64,
    kind: jig_client::contract::MessageProposalKind,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"proposal");
    out.object_start();
    out.key(b"proposer");
    out.unsigned(proposer);
    out.key(b"proposal");
    out.unsigned(proposal);
    out.key(b"kind");
    encode_owner_message_proposal_kind(kind, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_message_notice(
    subscription: u64,
    target: u64,
    state: jig_client::contract::NoticeState,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"notice");
    out.object_start();
    out.key(b"subscription");
    out.unsigned(subscription);
    out.key(b"target");
    out.unsigned(target);
    out.key(b"state");
    encode_owner_notice_state(state, out, budget);
    out.object_end();
    out.object_end();
}

#[cfg(test)]
mod tests {
    use super::encode_owner_message_kind;
    use jig_client::contract::{MessageKind, SettingsOutcome};
    use skein_json::writer::{Encoder, Limits};

    #[test]
    fn settings_decisions_encode_all_three_outcomes() {
        let limits = Limits { depth: 8, length: 256 };
        for (outcome, expected) in [
            (
                SettingsOutcome::Accepted { revision: 7 },
                br#"{"settings_decision":{"proposal":3,"outcome":{"accepted":7}}}"#.as_slice(),
            ),
            (SettingsOutcome::Rejected, br#"{"settings_decision":{"proposal":3,"outcome":"rejected"}}"#),
            (SettingsOutcome::Stale { newest: 8 }, br#"{"settings_decision":{"proposal":3,"outcome":{"stale":8}}}"#),
        ] {
            let kind = MessageKind::SettingsDecision { proposal: 3, outcome };
            let mut budget = super::super::Cut { room: 0, left: 0, turn: None, subject: false, digest: None };
            let mut measured = Encoder::measure(&limits);
            encode_owner_message_kind(&kind, &mut measured, &mut budget);
            let mut encoded = Encoder::write(measured.measured().expect("bounded decision"), &limits);
            encode_owner_message_kind(&kind, &mut encoded, &mut budget);
            assert_eq!(encoded.finish().as_ref(), expected);
        }
    }
}
