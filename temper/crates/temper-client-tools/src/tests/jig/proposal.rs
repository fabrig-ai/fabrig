//! Party proposals cross the ordinary ctl runner and fake client boundary.
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect ctl's UTF-8 output")]
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;

const SHOW: &[&str] = &["proposal", "show", "p73/17"];

fn state(spend: u64) -> c::AmendmentState {
    let task = f::goal(f::TASK, f::waiting_on_me());
    let mut authority = task.authority;
    authority.authority.spend = spend;
    c::AmendmentState {
        addressed: None,
        taken_by: None,
        instructions: None,
        spec: task.spec,
        authority,
        wake: task.wake,
        dependencies: List::with_capacity(0),
        cut: false,
    }
}
fn amendment(proposer: c::Proposer) -> c::ReadValue {
    let c::ReadValue::Proposal(mut proposal) = f::proposal(proposer, &[c::ActKind::Accept]) else {
        panic!("proposal fixture")
    };
    proposal.action = c::ProposalAction::Amend {
        task: c::TaskLink::Stored { number: f::TASK, status: None },
        addressed: None,
        before: Box::new(state(100)),
        after: Box::new(state(200)),
        reason: f::document_text(b"more room"),
        release: None,
    };
    c::ReadValue::Proposal(proposal)
}
fn retained(action: c::RetainedAction) -> c::ReadValue {
    c::ReadValue::Ended(Box::new(c::Ended {
        key: c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Party(f::PARTY), number: f::PROPOSAL }),
        value: c::EndedValue::Proposal(c::ProposalHistory {
            reason: f::document_text(b"more room"),
            action,
            number: f::PROPOSAL,
            proposer: f::PARTY,
            as_holder: false,
            state: c::ProposalHistoryState::Withdrawn,
        }),
        at: None,
        by: None,
    }))
}
fn retained_amendment() -> c::ReadValue {
    retained(c::RetainedAction::Amend {
        task: f::TASK,
        change: Box::new(c::RetainedAmendment {
            addressed: None,
            instructions: None,
            release: None,
            spec: None,
            authority: Some(state(200).authority),
            wake: None,
            dependencies: None,
            reason: f::document_text(b"more room"),
            cut: false,
        }),
    })
}
fn show(value: c::ReadValue, json: bool) -> crate::jig::Printed {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit_prepared(f::call(SHOW), json, None);
    root.serve(call, c::ReadResult::Value(value));
    root.answer(call).expect("shown proposal")
}
fn decided() -> c::Outcome {
    c::Outcome::Document(c::DocumentOutcome::ProposalDecided {
        proposal: c::ProposalKey { proposer: c::Proposer::Party(f::PARTY), number: f::PROPOSAL },
        by: f::PARTY,
        at: f::DECIDED_AT,
        choice: c::ProposalChoice::Withdrawn,
        repeated: true,
    })
}
#[test]
fn pending_amendments_show_target_and_both_authorities_for_either_maker() {
    for proposer in [c::Proposer::Party(f::PARTY), c::Proposer::Task(f::TASK)] {
        // The request and returned document must name the same typed maker.
        let argv = if proposer == c::Proposer::Party(f::PARTY) { SHOW } else { &["proposal", "show", "t41/17"] };
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(argv), json, None);
            root.serve(call, c::ReadResult::Value(amendment(proposer)));
            let answer = root.answer(call).expect("amendment shown");
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            if json {
                let value: serde_json::Value = serde_json::from_slice(&answer.stdout).unwrap();
                assert_eq!(value["action"]["amend"]["task"], f::TASK);
                assert_eq!(value["action"]["amend"]["before"]["authority"]["spend"], 100u64);
                assert_eq!(value["action"]["amend"]["after"]["authority"]["spend"], 200u64);
            } else {
                assert!(text.contains("amend task 41"), "{text}");
                assert!(text.contains("before: authority {"), "{text}");
                assert!(text.contains("after: authority {"), "{text}");
                assert!(text.contains("spend 1.00;"), "{text}");
                assert!(text.contains("spend 2.00;"), "{text}");
            }
        }
    }
}
#[test]
fn pending_party_amendment_decides_with_its_document_project() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(f::DECIDE_PARTY_CALL);
    assert_eq!(
        root.reads.get(&call).unwrap().1,
        c::Read::Proposal { proposal: c::ProposalKey { proposer: c::Proposer::Party(f::PARTY), number: f::PROPOSAL } }
    );
    root.serve(call, c::ReadResult::Value(amendment(c::Proposer::Party(f::PARTY))));
    assert_eq!(
        f::act_ask(&root, call),
        &c::Ask::Decide {
            project: f::PROJECT,
            on: c::DecideOn::Proposal {
                proposal: c::ProposalKey { proposer: c::Proposer::Party(f::PARTY), number: f::PROPOSAL },
                decision: c::Decision::Accepted,
                running: None,
            },
        }
    );
    assert!(root.reads.is_empty());
}
#[test]
fn withdrawal_sends_unread_and_replays_the_same_ask_key_and_answer() {
    for project in ["1", "shop"] {
        let mut root = f::signed_in(f::PARTY);
        let argv = ["proposal", "withdraw", "p73/17", "--project", project, "--key", "withdraw"];
        let first = root.submit(&argv);
        assert!(root.reads.is_empty(), "no proposal read precedes withdrawal");
        assert_eq!(root.sent_name(first), Some(f::named("withdraw")));
        assert_eq!(f::act_ask(&root, first), &c::Ask::Withdraw { project: f::PROJECT, proposal: f::PROPOSAL });
        let encoded = f::encoded(&root.sends.get(&first).unwrap().2);
        root.done(first, c::Outcome::Document(c::DocumentOutcome::Withdrawn { proposal: f::PROPOSAL }));
        let first_answer = root.answer(first).expect("withdrawn");
        assert_eq!((first_answer.status, first_answer.stdout.as_ref()), (0, b"done: withdraw\n".as_slice()));
        let second = root.submit(&argv);
        assert!(root.reads.is_empty());
        assert_eq!(f::encoded(&root.sends.get(&second).expect("replay sent").2), encoded);
        assert_eq!(root.sent_name(second), root.sent_name(first));
        root.done(second, c::Outcome::Document(c::DocumentOutcome::Withdrawn { proposal: f::PROPOSAL }));
        let second_answer = root.answer(second).expect("first engine answer printed again");
        assert_eq!(
            (second_answer.status, second_answer.stdout, second_answer.stderr),
            (first_answer.status, first_answer.stdout, first_answer.stderr)
        );
    }
}
#[test]
fn withdrawal_refuses_a_task_or_another_party_without_sending() {
    for proposal in ["t41/17", "p74/17"] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["proposal", "withdraw", proposal, "--project", "1"]);
        let answer = root.answer(call).expect("local usage");
        assert_eq!(answer.status, 1);
        assert_eq!(answer.stderr.as_ref(), b"proposal withdraw needs your party's pN/K\n");
        assert!(root.sends.is_empty());
        assert!(root.reads.is_empty());
    }
    assert!(f::parse(&["proposal", "withdraw", "p73/17"]).is_err(), "project is required");
}
#[test]
fn withdrawal_is_a_copyable_call_only_when_offered() {
    for offered in [false, true] {
        let acts = if offered { &[c::ActKind::Withdraw][..] } else { &[] };
        let answer = show(f::proposal(c::Proposer::Party(f::PARTY), acts), false);
        assert_eq!(answer.status, 0);
        let text = core::str::from_utf8(&answer.stdout).unwrap();
        assert_eq!(text.contains("acts:\n  proposal withdraw p73/17 --project 1 --key KEY\n"), offered, "{text}");
        assert_eq!(text.contains("withdraw"), offered, "{text}");
    }
}
#[test]
fn withdrawal_keeps_retained_decisions_and_unknown_words() {
    for json in [false, true] {
        for unknown in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let first = root.submit(f::WITHDRAW_CALL);
            root.done(first, c::Outcome::Document(c::DocumentOutcome::Withdrawn { proposal: f::PROPOSAL }));
            assert_eq!(root.answer(first).expect("first withdrawal").status, 0);
            let mut fresh = f::call(f::WITHDRAW_CALL);
            fresh.key = Some(f::name(b"withdraw-again"));
            let call = root.submit_prepared(fresh, json, None);
            assert_ne!(root.sent_name(call), root.sent_name(first));
            if unknown {
                root.answered(call, c::Answer::Refused { refusal: c::Refusal::Unknown, at: None });
            } else {
                root.done(call, decided());
            }
            let answer = root.answer(call).expect("engine answer");
            assert_eq!(answer.status, if unknown { 2 } else { 0 });
            if !json {
                if unknown {
                    assert_eq!(
                        answer.stderr.as_ref(),
                        b"proposal withdraw: this proposal isn't waiting; it may have been decided or replaced\n"
                    );
                } else {
                    assert_eq!(answer.stdout.as_ref(), b"decided first: by you, withdrawn, 14:02\n");
                }
            }
        }
    }
}
#[test]
fn retained_party_amendment_shows_replacement_and_decides_via_target_project() {
    for json in [false, true] {
        let answer = show(retained_amendment(), json);
        assert_eq!(answer.status, 0);
        let text = core::str::from_utf8(&answer.stdout).unwrap();
        if json {
            let value: serde_json::Value = serde_json::from_slice(&answer.stdout).unwrap();
            assert_eq!(value["action"]["amend"]["task"], f::TASK);
            assert_eq!(value["action"]["amend"]["authority"]["authority"]["spend"], 200u64);
            assert!(value["state"]["withdrawn"].is_object());
        } else {
            assert!(text.contains("amend · withdrawn\ntask 41\nreplacement: authority {"), "{text}");
            assert!(text.contains("spend 2.00;"), "{text}");
            assert!(!text.contains("before"), "no before reconstructed");
        }
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(f::DECIDE_PARTY_CALL), json, None);
        root.serve(call, c::ReadResult::Value(retained_amendment()));
        assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Task { task: f::TASK });
        assert!(root.sends.is_empty());
        root.serve(call, f::task_result(f::goal(f::TASK, f::waiting_on_me())));
        let c::Ask::Decide { project, .. } = f::act_ask(&root, call) else { panic!("decide") };
        assert_eq!(*project, f::PROJECT);
        root.done(call, decided());
        let answer = root.answer(call).expect("retained decision");
        assert_eq!(answer.status, 0);
        if !json {
            assert!(core::str::from_utf8(&answer.stdout).unwrap().starts_with("decided first:"));
        }
    }
}
#[test]
fn retained_party_batch_shows_state_but_cannot_guess_a_decision_project() {
    let history = retained(c::RetainedAction::Batch(List::with_capacity(0)));
    let answer = show(history.clone(), false);
    assert_eq!(answer.status, 0);
    assert_eq!(answer.stdout.as_ref(), b"proposal p73/17 \xc2\xb7 batch \xc2\xb7 withdrawn\n");
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(f::DECIDE_PARTY_CALL);
    root.serve(call, c::ReadResult::Value(history));
    let answer = root.answer(call).expect("project unavailable");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, b"project unavailable\n".as_slice()));
    assert!(root.sends.is_empty());
    assert!(root.reads.is_empty());
}
