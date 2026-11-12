//! Kept facts and requester authority (domain/tasks/lifecycle.md, 7–9).
use super::revisions::{Harness, LIMITS, task};
use crate::{Domain, Event, Party, Stored};
use alloc::boxed::Box;

#[test]
fn shared_kept_rule_separates_claims_from_finish_authority() {
    use crate::{End, EndedBy, Executor, TaskResult};
    let done = End::Finished { result: TaskResult::Report { words: Box::new([]) }, cancel_delegates: false };
    let failed = End::Finished { result: TaskResult::Failure { reason: Box::new([]) }, cancel_delegates: false };
    let malformed = End::Finished {
        result: TaskResult::Verdict { fields: Box::new([]), code: 0, words: Box::new([]) },
        cancel_delegates: false,
    };
    let non_finish = End::Failed(crate::Class::Invalid);
    for (dependencies, ended_by, blocks, discards, parks_done, parks_failure, parks_invalid) in [
        (&[][..], EndedBy::Result, false, false, false, false, false),
        (&[2][..], EndedBy::Result, true, false, true, false, true),
        (&[][..], EndedBy::Requester, false, true, false, false, false),
        (&[2][..], EndedBy::Requester, true, true, true, false, true),
    ] {
        let kept = crate::run::ends::pending(dependencies, ended_by);
        assert_eq!(kept.blocks_claims(), blocks);
        assert_eq!(kept.may_be_due(Executor::Agent { charter: 1 }), !blocks);
        assert_eq!(kept.discards(&done), discards);
        assert_eq!(kept.discards(&failed), discards);
        assert_eq!(kept.discards(&malformed), discards);
        assert!(!kept.discards(&non_finish));
        assert_eq!(kept.parks(&done), parks_done);
        assert_eq!(kept.parks(&failed), parks_failure);
        assert_eq!(kept.parks(&malformed), parks_invalid);
        assert!(!kept.parks(&non_finish));
    }
}

#[test]
fn an_own_question_parks_only_a_runs_finished_failure() {
    use crate::{End, EndedBy, ResultFollowups, TaskResult};
    let failure = TaskResult::Failure { reason: Box::new([]) };
    let finished = End::Finished { result: failure.clone(), cancel_delegates: false };
    let question = crate::run::ends::pending(&[], EndedBy::Result).with_question(Some(1));
    for (name, kept, end, parks) in [
        ("own question", question, finished.clone(), true),
        (
            "own question with followups",
            question,
            End::FinishedWithFollowups { result: failure, followups: ResultFollowups::Delegates(Box::new([])) },
            true,
        ),
        ("dependency only", crate::run::ends::pending(&[2], EndedBy::Result), finished.clone(), false),
        ("owing only", crate::run::ends::pending(&[], EndedBy::Result).with_owing(true), finished, false),
        ("run failed", question, End::Failed(crate::Class::Run), false),
    ] {
        assert_eq!(kept.run_parks(&end), parks, "{name}");
        assert!(!kept.parks(&end), "{name}: procedure failure must not park");
    }
}

#[test]
fn requester_ending_restores_without_rejudging_earlier_authority() {
    let mut h = Harness::new();
    let mut open = task(1, &[]);
    open.flags.conversation = true;
    open.flags.ended_by = crate::EndedBy::Requester;
    h.make([open].into());
    h.restart();
    let original = h.domain.task(1).expect("restored open task").clone();
    assert_eq!(original.flags.ended_by, crate::EndedBy::Requester);
    for requester in [Party::Person(0), Party::Task(1), Party::Deployment { project: 1 }] {
        let mut broken = original.clone();
        broken.requester = requester;
        let mut cold = Harness::new();
        cold.domain = Domain::new(&LIMITS, crate::tests::KEPT, 91, 1, crate::tests::charters(), Box::new([0]));

        let out = cold.send(Event::Restore { record: Stored::Live(Box::new(broken)) });
        assert_eq!(out.len(), 0);
        assert_eq!(cold.domain.task(1).unwrap().requester, requester);
    }
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::match_like_matches_macro,
    reason = "unit test inspects selected terminal and activity events with explicit patterns"
)]
fn an_agent_finish_waits_for_live_delegates_unless_it_cancels_them() {
    use super::revisions::to;
    use crate::{Accepted, Active, Cause, End, EndKind, Key, Line, Phase, Request, SavedResource, TaskResult, Tries};
    for cancel_delegates in [false, true] {
        let mut h = Harness::new();
        h.make([task(1, &[])].into());
        let made = h.send(Event::Make { reply_to: to(), creator: Party::Task(1), batch: Box::new([task(2, &[])]) });
        assert!(made.iter().any(|request| if let Request::Made { .. } = request { true } else { false }));
        h.send(Event::Prepare { reply_to: to(), task: 1 });
        h.send(Event::Claim {
            session_end: None,
            reply_to: to(),
            task: 1,
            attempt: 1,
            resumed: false,
            budget: 10,
            account: None,
            writes: Box::new([]),
            offered: crate::Reasons::ALL,
            reasons: crate::Reasons::FIRST,
        });
        h.send(Event::Started { task: 1, attempt: 1 });
        let turn = h.send(Event::Turn {
            reply_to: to(),
            task: 1,
            attempt: 1,
            turn: 1,
            read: None,
            offered: None,
            refused: Box::new([]),
            outstanding: Box::new([]),
            cumulative: 1,
        });
        assert!(turn.iter().any(|request| if let Request::TurnAcknowledged { .. } = request { true } else { false }));
        let saved = Box::new([SavedResource { connector: 1, path: Box::new([b"work".as_slice().into()]) }]);
        let out = h.send(Event::Activation {
            reply_to: to(),
            task: 1,
            attempt: 1,
            cause: Cause::Priced { cumulative: 2 },
            end: End::Finished { result: TaskResult::Report { words: b"ready".as_slice().into() }, cancel_delegates },
            saved: Some(saved.clone()),
        });
        assert!(
            out.iter().any(|request| {
                if let Request::Acknowledged { accepted: Accepted::New, parked, .. } = request {
                    *parked != cancel_delegates
                } else {
                    false
                }
            }),
            "{out:?}"
        );
        let row = h.domain.task(1).expect("parent still live");
        assert_eq!(row.tries, Tries::NONE);
        assert_eq!(row.turn, 1);
        assert_eq!(row.saved.as_ref(), saved.as_ref());
        assert_eq!(row.numbers.spent, 2);
        assert_eq!(row.run_reserved, 0);
        assert_eq!(row.last_answer, Some(1));
        assert_eq!(row.reasons, crate::Reasons::EMPTY);
        assert!(!h.stored.contains_key(&Key::Ended(1)));
        let end_kind = if cancel_delegates {
            let Phase::Closing(closing) = &row.phase else { unreachable!("cancelling finish closes") };
            assert_eq!(closing.ending, crate::Ending::Done(TaskResult::Report { words: b"ready".as_slice().into() }));
            assert!(if let Phase::Closing(_) = h.domain.task(2).expect("delegate closing").phase {
                true
            } else {
                false
            });
            EndKind::Finished
        } else {
            assert_eq!(row.phase, Phase::Active(Active::Idle));
            assert_eq!(row.delegates.as_ref(), [2]);
            assert_eq!(h.domain.task(2).expect("delegate live").phase, Phase::Active(Active::Due));
            EndKind::Parked
        };
        assert!(h.stored.values().any(|stored| {
            if let Stored::Activity(activity) = stored
                && activity.task == 1
                && let Line::RunEnded { attempt: 1, end, .. } = activity.line
            {
                end == end_kind
            } else {
                false
            }
        }));
    }
}
