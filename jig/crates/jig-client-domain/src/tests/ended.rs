use super::fixtures as f;
use crate::{Up, contract::*};
use skein_lib::List;

#[test]
fn retained_party_keys_match_only_party_actions_and_task_keys_keep_all_actions() {
    let Up::Watched { event: WatchEvent::Delivered(Delivery::Task(task)), .. } = f::delivery(37) else {
        panic!("task fixture")
    };
    let text = DocumentText { bytes: Text::new(b"why".as_slice().into(), 32).unwrap(), cut: false };
    let part = Part { connector: 7, kind: PartKind::Effect, key: PartKey::Number(13) };
    for (action, party_matches) in [
        (
            RetainedAction::Amend {
                task: 37,
                change: Box::new(RetainedAmendment {
                    addressed: None,
                    instructions: None,
                    release: None,
                    spec: None,
                    authority: None,
                    wake: None,
                    dependencies: None,
                    reason: text.clone(),
                    cut: false,
                }),
            },
            true,
        ),
        (RetainedAction::Batch(List::with_capacity(0)), true),
        (
            RetainedAction::Effect {
                part: part.clone(),
                authority: task.authority.clone(),
                attempt: 17,
                completion: 19,
                position: 23,
                purpose: 29,
            },
            false,
        ),
        (RetainedAction::ProcedureEffect { part, authority: task.authority.clone(), step: 31, purpose: 29 }, false),
        (RetainedAction::Widen { task: 37, authority: task.authority.clone() }, false),
        (RetainedAction::Release { task: 37 }, false),
    ] {
        for proposer in [Proposer::Party(11), Proposer::Task(11)] {
            let key = ObjectKey::Proposal(ProposalKey { proposer, number: 13 });
            let ended = Ended {
                key,
                at: None,
                by: None,
                value: EndedValue::Proposal(ProposalHistory {
                    proposer: 11,
                    number: 13,
                    reason: text.clone(),
                    as_holder: true,
                    action: action.clone(),
                    state: ProposalHistoryState::Withdrawn,
                }),
            };
            assert_eq!(ended.matches(key), proposer == Proposer::Task(11) || party_matches, "{proposer:?}: {action:?}");
            assert!(!ended.matches(ObjectKey::Proposal(ProposalKey { proposer, number: 17 })));
            let mut wrong_proposer = ended.clone();
            let EndedValue::Proposal(history) = &mut wrong_proposer.value else { unreachable!() };
            history.proposer = 19;
            assert!(!wrong_proposer.matches(key));
            let mut wrong_number = ended;
            let EndedValue::Proposal(history) = &mut wrong_number.value else { unreachable!() };
            history.number = 23;
            assert!(!wrong_number.matches(key));
        }
    }
}
