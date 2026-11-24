//! Activation reasons and the messages an attempt is offered
//! (jig/docs/design/domain/tasks/messages.md, 2, 3 and 6 and jig/docs/design/domain/tasks/proposals.md).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "step tests script admitted messages and inspect selected outputs"
)]
use super::revisions::{Harness, task, to};
use crate::{
    Active, End, EscalationHolder, Event, MessageKind as M, Party, Phase, Reasons as R, Request, ResultsWake,
    TaskResult, WakePolicy, WakeRule, Why, Word,
};
use skein_lib::{Duration, Queue, Time, Wall};

const TASK: u64 = 1;
const SOURCE: u64 = 2;
const PERSON: u64 = 9;
const MESSAGE: u64 = 10;
const FENCE: u64 = 20;
// The fixture has 100 to spend; this attempt reserves ten and spends none.
const RUN_BUDGET: u64 = 10;
const PACE: Duration = Duration::from_secs(60);
// A batch of two holds the single test message until its age expires.
const BATCH_AGE: Duration = Duration::from_secs(30);
const BATCH: WakeRule = WakeRule::Batch { count: 2, age: BATCH_AGE };

fn fixture() -> Harness {
    holder(
        WakePolicy {
            questions: true,
            answers: true,
            timers: true,
            results: ResultsWake::Each,
            words: WakeRule::Immediate,
            notices: WakeRule::Immediate,
            news: WakeRule::Immediate,
        },
        crate::EndedBy::Result,
    )
}

fn holder(wake: WakePolicy, ended_by: crate::EndedBy) -> Harness {
    let mut h = Harness::new();
    let mut new = task(TASK, &[]);
    new.wake = wake;
    new.flags.ended_by = ended_by;
    if let crate::EndedBy::Closer(closer) = ended_by {
        new.authority.delegation.kinds =
            Box::new([crate::AuthorityExecutor::Charter(1), crate::AuthorityExecutor::Procedure(closer.code)]);
    }
    h.make(vec![new]);
    h
}

fn word(number: u64, kind: M, from: Party) -> Word {
    Word { number, from, kind, words: Box::new([]), at: Wall::EPOCH, hits: 1, eligible: false }
}

// Admission has already kept room and assigned the number; exercise the wake
// child with that committed inbox, including kinds numbered by its parent.
fn arrive(h: &mut Harness, word: Word) -> Vec<Request> {
    let row = &mut crate::domain::task_mut(&mut h.domain, TASK).expect("fixture task").record;
    let mut inbox = row.inbox.to_vec();
    inbox.push(word.clone());
    row.inbox = inbox.into();
    row.last_message = word.number;
    let mut out = Queue::uncapped();
    crate::wake::after_message(&mut h.domain, &h.env, TASK, None, word, &mut out);
    core::iter::from_fn(|| out.pop()).collect()
}

fn start(h: &mut Harness, offered: R) {
    let attempt = h
        .domain
        .run_context(TASK)
        .expect("activation context")
        .previous_attempt
        .checked_add(1)
        .expect("fixture attempts");
    start_task(h, TASK, attempt, offered);
}

fn start_task(h: &mut Harness, number: u64, attempt: u64, offered: R) {
    claim_task(h, number, attempt, offered);
    h.send(Event::Started { task: number, attempt });
}

fn claim_task(h: &mut Harness, number: u64, attempt: u64, offered: R) {
    let reasons = h.domain.run_context(number).expect("activation context").reasons;
    h.send(Event::Prepare { reply_to: to(), task: number });
    let out = h.send(Event::Claim {
        fresh: None,
        session_end: None,
        reply_to: to(),
        task: number,
        attempt,
        resumed: false,
        budget: RUN_BUDGET,
        account: None,
        writes: Box::new([]),
        reasons,
        offered,
    });
    assert!(
        out.iter().any(|r| match r {
            Request::Done { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
}

fn end(h: &mut Harness, end: End) {
    let attempt = h.domain.task(TASK).expect("running fixture").attempt;
    h.send(Event::Activation { reply_to: to(), task: TASK, attempt, end, saved: None, cause: crate::Cause::Unpriced });
}

fn escalation(h: &mut Harness) -> Vec<Request> {
    h.make(vec![task(SOURCE, &[])]);
    h.send(Event::Hold { task: SOURCE, why: crate::Hold::Stopped });
    let revision = h.domain.task(SOURCE).expect("held fixture").revisions.hold;
    h.send(Event::RoutedEscalation { task: SOURCE, revision, holder: EscalationHolder::Task(TASK), entry: MESSAGE })
}

fn proposal(h: &mut Harness) -> Vec<Request> {
    h.make(vec![task(SOURCE, &[])]);
    h.send(Event::Propose {
        reply_to: to(),
        proposal: crate::Proposal {
            number: MESSAGE,
            proposer: SOURCE,
            project: 1,
            action: crate::ProposalAction::Amend {
                task: SOURCE,
                revision: 1,
                amendment: super::revisions::amendment(),
            },
            reason: Box::new([]),
            as_holder: false,
            state: crate::ProposalState::Pending { holder: crate::ProposalHolder::Task(TASK), since: h.env.wall },
        },
    })
}

#[derive(Clone)]
enum Setup {
    Start,
    End(End),
    Arrive(Word),
    Policy(WakePolicy),
    Delegates(bool),
    Dependency(bool),
    Question(bool),
    Narrow,
    Retry,
    Escalation,
    Proposal,
    Read,
}

fn setup(h: &mut Harness, actions: &[Setup]) {
    for action in actions {
        match action {
            Setup::Start => start(h, R::ALL),
            Setup::Read => {
                let row = h.domain.task(TASK).expect("reading fixture");
                h.send(Event::Turn {
                    reply_to: to(),
                    task: TASK,
                    attempt: row.attempt,
                    turn: 1,
                    read: Some(MESSAGE),
                    refused: Box::new([]),
                    offered: Some(MESSAGE),
                    outstanding: Box::new([MESSAGE]),
                    cumulative: 0,
                });
            }
            Setup::End(ending) => end(h, ending.clone()),
            Setup::Arrive(word) => {
                arrive(h, word.clone());
            }
            Setup::Policy(policy) => {
                crate::domain::task_mut(&mut h.domain, TASK).expect("fixture").record.wake = *policy;
            }
            Setup::Delegates(pending) => {
                if *pending {
                    h.make(vec![task(SOURCE, &[])]);
                }
                crate::domain::task_mut(&mut h.domain, TASK).expect("fixture").record.delegates =
                    if *pending { Box::new([SOURCE]) } else { Box::new([]) };
            }
            Setup::Dependency(pending) => {
                crate::domain::task_mut(&mut h.domain, TASK).expect("fixture").record.waiting_on =
                    if *pending { Box::new([SOURCE]) } else { Box::new([]) };
            }
            Setup::Question(pending) => {
                crate::domain::task_mut(&mut h.domain, TASK).expect("fixture").record.questions = if *pending {
                    Box::new([crate::QuestionCredit {
                        number: MESSAGE,
                        answerer: Party::Person(PERSON),
                        asked: Party::Person(PERSON),
                        line: 1,
                    }])
                } else {
                    Box::new([])
                };
            }
            Setup::Narrow => {
                crate::domain::task_mut(&mut h.domain, TASK).expect("fixture").record.narrowing = true;
            }
            Setup::Retry => {
                h.env.now = h.env.now.saturating_add(Duration::from_secs(1));
                h.env.wall = Wall::from_nanos(Duration::from_secs(1).as_nanos());
                let mut out = Queue::uncapped();
                crate::fire(&mut h.domain, &h.env, &mut out);
            }
            Setup::Escalation => {
                escalation(h);
            }
            Setup::Proposal => {
                proposal(h);
            }
        }
    }
}

#[test]
fn activations_keep_only_the_reasons_that_wake_them() {
    use Setup::{Arrive, Delegates, Dependency, End as Ending, Policy, Question, Start};
    let question = word(MESSAGE, M::Question, Party::Person(PERSON));
    let words = word(MESSAGE, M::Words, Party::Person(PERSON));
    let result = word(MESSAGE, M::Result(crate::ResultKind::Report), Party::Task(SOURCE));
    let timer = word(FENCE, M::Timer { subscription: SOURCE }, Party::Task(SOURCE));
    let last = WakePolicy { results: ResultsWake::LastOrFailure, ..WakePolicy::DEFAULT };
    let news = word(MESSAGE, M::News { subscription: SOURCE, class: crate::NewsClass::Wakes }, Party::Task(SOURCE));
    let paced = WakePolicy { news: WakeRule::Paced { span: PACE }, ..WakePolicy::DEFAULT };
    let batch = WakePolicy { news: WakeRule::Batch { count: 2, age: PACE }, ..WakePolicy::DEFAULT };
    let mut eligible_news = news.clone();
    eligible_news.eligible = true;
    let finished = End::Finished { result: TaskResult::Report { words: Box::new([]) }, cancel_delegates: false };
    let cases: &[(&str, Vec<Setup>, R, bool)] = &[
        ("made", vec![], R::FIRST, true),
        ("person words", vec![Arrive(words.clone())], R::FIRST.union(R::WORDS), true),
        ("idle question", vec![Start, Ending(End::Parked), Arrive(question)], R::QUESTION, true),
        (
            "nonlast result",
            vec![Start, Ending(End::Parked), Policy(last), Delegates(true), Arrive(result.clone())],
            R::EMPTY,
            false,
        ),
        ("last result", vec![Start, Ending(End::Parked), Policy(last), Arrive(result)], R::RESULT, true),
        (
            "retry",
            vec![
                Arrive(words),
                Start,
                Setup::Read,
                Ending(End::Failed(crate::Class::Transient)),
                Arrive(timer),
                Setup::Retry,
            ],
            R::FIRST.union(R::WORDS).union(R::TIMER),
            true,
        ),
        (
            "own question",
            vec![
                Start,
                Question(true),
                Ending(End::Parked),
                Question(false),
                Arrive(word(FENCE, M::Answer { question: MESSAGE }, Party::Person(PERSON))),
            ],
            R::FIRST,
            true,
        ),
        (
            "kept done",
            vec![
                Start,
                Dependency(true),
                Ending(finished.clone()),
                Dependency(false),
                Arrive(word(FENCE, M::Met { dependency: SOURCE, removed: false }, Party::Person(PERSON))),
            ],
            R::FIRST,
            true,
        ),
        ("finished", vec![Start, Ending(finished)], R::EMPTY, false),
        ("narrowed park", vec![Start, Setup::Narrow, Ending(End::Parked)], R::FIRST, true),
        (
            "exceeded",
            vec![Start, Ending(End::Exceeded(crate::Exceeded { bound: crate::Bound::Answer, value: 1 }))],
            R::FIRST,
            false,
        ),
        ("refused", vec![Start, Ending(End::Refused)], R::FIRST, false),
        ("recheck", vec![Start, Ending(End::Recheck)], R::FIRST, true),
        ("pace held", vec![Start, Policy(paced), Arrive(news.clone())], R::FIRST, false),
        ("batch held", vec![Policy(batch), Arrive(news)], R::FIRST, true),
        ("pace eligible", vec![Start, Policy(paced), Arrive(eligible_news.clone())], R::FIRST.union(R::NEWS), false),
        ("batch eligible", vec![Policy(batch), Arrive(eligible_news)], R::FIRST.union(R::NEWS), true),
        (
            "never",
            vec![Arrive(word(
                MESSAGE,
                M::News { subscription: SOURCE, class: crate::NewsClass::Kept },
                Party::Task(SOURCE),
            ))],
            R::FIRST,
            true,
        ),
        ("escalation", vec![Setup::Escalation], R::FIRST.union(R::ESCALATION), true),
        ("proposal", vec![Setup::Proposal], R::FIRST.union(R::PROPOSAL), true),
    ];
    let mut results = Vec::new();
    let mut expected = Vec::new();
    for (name, actions, reasons, due) in cases {
        let mut h = fixture();
        setup(&mut h, actions);
        results.push((
            *name,
            h.domain.run_context(TASK).expect("context").reasons,
            h.domain.task(TASK).expect("task").phase == Phase::Active(Active::Due),
        ));
        expected.push((*name, *reasons, *due));
    }
    assert_eq!(results, expected);
}

enum Arrival {
    Message(Word),
    Escalation,
    Proposal,
}

struct Offering {
    name: &'static str,
    offered: R,
    arrivals: Vec<Arrival>,
    paced: bool,
    refused: &'static [u64],
    park: bool,
    relays: &'static [u64],
    inbox: &'static [u64],
    next: u64,
    due: bool,
    reasons: R,
}

#[test]
#[expect(
    clippy::too_many_lines,
    clippy::match_like_matches_macro,
    reason = "one table checks live offers, exact fence evidence and the next wake through selected requests"
)]
fn runs_hear_and_take_only_what_their_parts_offer() {
    let question = word(MESSAGE, M::Question, Party::Task(SOURCE));
    let mut cases = vec![
        Offering {
            name: "question",
            offered: R::FIRST,
            arrivals: vec![Arrival::Message(question.clone())],
            paced: false,
            refused: &[],
            park: false,
            relays: &[FENCE],
            inbox: &[MESSAGE],
            next: FENCE,
            due: false,
            reasons: R::FIRST.union(R::QUESTION),
        },
        Offering {
            name: "always offered",
            offered: R::FIRST,
            arrivals: vec![
                Arrival::Message(word(MESSAGE, M::Answer { question: SOURCE }, Party::Person(PERSON))),
                Arrival::Message(word(MESSAGE + 1, M::Amendment { revision: 1 }, Party::Person(PERSON))),
                Arrival::Message(word(MESSAGE + 2, M::Words, Party::Person(PERSON))),
            ],
            paced: false,
            refused: &[],
            park: false,
            relays: &[MESSAGE, MESSAGE + 1, MESSAGE + 2, FENCE],
            inbox: &[],
            next: MESSAGE,
            due: false,
            reasons: R::FIRST,
        },
        Offering {
            name: "task words",
            offered: R::FIRST,
            arrivals: vec![Arrival::Message(word(MESSAGE, M::Words, Party::Task(SOURCE)))],
            paced: false,
            refused: &[],
            park: false,
            relays: &[FENCE],
            inbox: &[MESSAGE],
            next: FENCE,
            due: false,
            reasons: R::FIRST.union(R::WORDS),
        },
        Offering {
            name: "entry",
            offered: R::FIRST,
            arrivals: vec![Arrival::Escalation],
            paced: false,
            refused: &[],
            park: false,
            relays: &[MESSAGE, FENCE],
            inbox: &[],
            next: MESSAGE,
            due: false,
            reasons: R::FIRST.union(R::ESCALATION),
        },
        Offering {
            name: "all",
            offered: R::ALL,
            arrivals: vec![
                Arrival::Message(question.clone()),
                Arrival::Message(word(MESSAGE + 1, M::Words, Party::Task(SOURCE))),
            ],
            paced: false,
            refused: &[],
            park: false,
            relays: &[MESSAGE, MESSAGE + 1, FENCE],
            inbox: &[],
            next: MESSAGE,
            due: false,
            reasons: R::FIRST,
        },
        Offering {
            name: "pace",
            offered: R::ALL,
            arrivals: vec![Arrival::Message(word(
                MESSAGE,
                M::News { subscription: SOURCE, class: crate::NewsClass::Wakes },
                Party::Task(SOURCE),
            ))],
            paced: true,
            refused: &[],
            park: false,
            relays: &[FENCE],
            inbox: &[MESSAGE],
            next: MESSAGE,
            due: false,
            reasons: R::FIRST,
        },
        Offering {
            name: "refused",
            offered: R::ALL,
            arrivals: vec![Arrival::Message(question.clone())],
            paced: false,
            refused: &[MESSAGE],
            park: false,
            relays: &[MESSAGE, FENCE],
            inbox: &[MESSAGE],
            next: MESSAGE,
            due: false,
            reasons: R::FIRST.union(R::QUESTION),
        },
        Offering {
            name: "question wakes next",
            offered: R::FIRST,
            arrivals: vec![Arrival::Message(question)],
            paced: false,
            refused: &[],
            park: true,
            relays: &[FENCE],
            inbox: &[MESSAGE],
            next: FENCE,
            due: true,
            reasons: R::QUESTION,
        },
        Offering {
            name: "entry wakes next",
            offered: R::FIRST,
            arrivals: vec![Arrival::Escalation],
            paced: false,
            refused: &[],
            park: true,
            relays: &[MESSAGE, FENCE],
            inbox: &[],
            next: MESSAGE,
            due: true,
            reasons: R::ESCALATION,
        },
    ];
    cases.push(Offering {
        name: "proposal wakes next",
        offered: R::FIRST,
        arrivals: vec![Arrival::Proposal],
        paced: false,
        refused: &[],
        park: true,
        relays: &[MESSAGE, FENCE],
        inbox: &[],
        next: MESSAGE,
        due: true,
        reasons: R::PROPOSAL,
    });
    for (name, offered, arrival, refused, due, reasons) in [
        ("covered escalation", R::ESCALATION, Arrival::Escalation, &[][..], false, R::ESCALATION),
        ("covered proposal", R::PROPOSAL, Arrival::Proposal, &[][..], false, R::PROPOSAL),
        ("refused covered escalation", R::ESCALATION, Arrival::Escalation, &[MESSAGE][..], true, R::ESCALATION),
        ("refused covered proposal", R::PROPOSAL, Arrival::Proposal, &[MESSAGE][..], true, R::PROPOSAL),
        ("uncovered escalation", R::QUESTION, Arrival::Escalation, &[][..], true, R::ESCALATION),
    ] {
        cases.push(Offering {
            name,
            offered,
            arrivals: vec![arrival],
            paced: false,
            refused,
            park: true,
            relays: &[MESSAGE, FENCE],
            inbox: &[],
            next: MESSAGE,
            due,
            reasons,
        });
    }
    // One arrival per row leaves the run's inbox bound unchanged while every
    // kind exercises the unpartitioned charter's relay path.
    for (name, kind) in [
        ("all answer", M::Answer { question: SOURCE }),
        ("all amendment", M::Amendment { revision: 1 }),
        ("all person stalled", M::PersonStalled { task: SOURCE }),
        ("all result", M::Result(crate::ResultKind::Report)),
        ("all notice", M::Notice { subscription: SOURCE, target: SOURCE, state: crate::NoticeState::Done }),
        ("all nothing awaited", M::NothingAwaited),
        ("all news", M::News { subscription: SOURCE, class: crate::NewsClass::Wakes }),
        ("all timer", M::Timer { subscription: SOURCE }),
        ("all question answered", M::QuestionAnswered { question: SOURCE }),
        ("all question ended", M::QuestionEnded { question: SOURCE }),
        ("all question owed", M::QuestionOwed),
        ("all proposal decision", M::ProposalDecision { proposal: SOURCE, accepted: true }),
        ("all settings decision", M::SettingsDecision { proposal: SOURCE, outcome: crate::SettingsOutcome::Rejected }),
        ("all effect not made", M::EffectNotMade { entry: SOURCE }),
        ("all met", M::Met { dependency: SOURCE, removed: false }),
        ("all finish requested", M::FinishRequested { revision: 1 }),
    ] {
        cases.push(Offering {
            name,
            offered: R::ALL,
            arrivals: vec![Arrival::Message(word(MESSAGE, kind, Party::Task(SOURCE)))],
            paced: false,
            refused: &[],
            park: false,
            relays: &[MESSAGE, FENCE],
            inbox: &[],
            next: MESSAGE,
            due: false,
            reasons: R::FIRST,
        });
    }
    let mut results = Vec::new();
    let mut expected = Vec::new();
    for case in cases {
        let mut h = fixture();
        start(&mut h, case.offered);
        if case.paced {
            crate::domain::task_mut(&mut h.domain, TASK).expect("fixture").record.wake.news =
                WakeRule::Paced { span: PACE };
        }
        let mut relays = Vec::new();
        for arrival in case.arrivals {
            let out = match arrival {
                Arrival::Message(word) => arrive(&mut h, word),
                Arrival::Escalation => escalation(&mut h),
                Arrival::Proposal => proposal(&mut h),
            };
            relays.extend(out.into_iter().filter_map(|r| match r {
                Request::Relay { word, .. } => Some(word.number),
                _ => None,
            }));
        }
        let out = arrive(&mut h, word(FENCE, M::Answer { question: SOURCE }, Party::Person(PERSON)));
        relays.extend(out.into_iter().filter_map(|r| match r {
            Request::Relay { word, .. } => Some(word.number),
            _ => None,
        }));
        let next = h.domain.next_waiting(TASK, None).expect("fence waiting").number;
        let out = h.send(Event::Turn {
            reply_to: to(),
            task: TASK,
            attempt: 1,
            turn: 1,
            read: Some(FENCE),
            refused: case.refused.into(),
            offered: Some(FENCE),
            outstanding: relays.clone().into(),
            cumulative: 0,
        });
        let accepted = out.iter().any(|r| match r {
            Request::TurnAcknowledged { .. } => true,
            _ => false,
        });
        // Offered words handled by this fixture are recorded; this table pins offering,
        // not the extra wake for leaving directing words unrecorded.
        let messages = h.domain.task(TASK).expect("task").unrecorded.iter().map(|word| word.number).collect();
        h.send(Event::RecordWords { reply_to: to(), task: TASK, messages });
        if case.park {
            end(&mut h, End::Parked);
        }
        let row = h.domain.task(TASK).expect("task");
        results.push((
            case.name,
            relays,
            row.inbox.iter().map(|w| w.number).collect::<Vec<_>>(),
            next,
            accepted,
            row.phase == Phase::Active(Active::Due),
            h.domain.run_context(TASK).expect("context").reasons,
            (case.park && case.due).then_some(row.due_reason),
        ));
        expected.push((
            case.name,
            case.relays.to_vec(),
            case.inbox.to_vec(),
            case.next,
            true,
            case.due,
            case.reasons,
            (case.park && case.due).then_some(Why::Message { message: MESSAGE }),
        ));
    }
    assert_eq!(results, expected);
}

// Results arrive after the ending delegate has left the live-delegate set.
fn result_arrival(h: &mut Harness, number: u64, source: u64, kind: crate::ResultKind, live: &[u64]) -> Vec<Request> {
    crate::domain::task_mut(&mut h.domain, TASK).expect("parent").record.delegates = live.into();
    arrive(h, word(number, M::Result(kind), Party::Task(source)))
}

#[test]
fn live_results_follow_wake_policy_not_parts() {
    use crate::ResultKind::{Cancelled, Failed, Report};
    for (name, policy, first, closer, claimed, expected) in [
        ("first success waits", ResultsWake::LastOrFailure, Report, false, false, vec![MESSAGE + 1]),
        ("first failure", ResultsWake::LastOrFailure, Failed, false, false, vec![MESSAGE, MESSAGE + 1]),
        ("first cancel", ResultsWake::LastOrFailure, Cancelled, false, false, vec![MESSAGE, MESSAGE + 1]),
        ("closer live", ResultsWake::LastOrFailure, Report, true, false, vec![MESSAGE + 1]),
        ("closer claimed", ResultsWake::LastOrFailure, Report, true, true, vec![MESSAGE + 1]),
        ("each", ResultsWake::Each, Report, false, false, vec![MESSAGE, MESSAGE + 1]),
        ("never", ResultsWake::Never, Failed, false, false, vec![]),
    ] {
        let mut h = fixture();
        let offered = if policy == ResultsWake::Never { R::ALL } else { R::FIRST };
        start(&mut h, offered);
        let row = &mut crate::domain::task_mut(&mut h.domain, TASK).expect("parent").record;
        row.wake.results = policy;
        if closer {
            row.flags.ended_by = crate::EndedBy::Closer(crate::Closer { connector: 1, code: 6 });
        }
        if claimed {
            row.phase = Phase::Active(Active::Claimed { attempt: 1 });
        }
        row.delegates = Box::new([7, 8]);
        let mut out = result_arrival(&mut h, MESSAGE, 7, first, &[8]);
        out.extend(result_arrival(&mut h, MESSAGE + 1, 8, Report, &[]));
        let relayed: Vec<_> = out
            .iter()
            .filter_map(|request| match request {
                Request::Relay { task, attempt, word, .. } => {
                    assert_eq!((*task, *attempt), (TASK, 1));
                    Some(word.number)
                }
                _ => None,
            })
            .collect();
        assert_eq!(relayed, expected, "{name}");
        let row = h.domain.task(TASK).expect("parent");
        assert_eq!(row.offered, offered, "results choose no new part: {name}");
        assert_eq!(row.reasons, R::FIRST, "results keep the run's reasons: {name}");
    }
}

#[test]
fn waiting_results_use_the_live_wake_rule_when_relay_room_frees() {
    for (policy, expected) in [
        (ResultsWake::LastOrFailure, vec![MESSAGE + 1, MESSAGE + 2]),
        (ResultsWake::Each, vec![MESSAGE, MESSAGE + 1, MESSAGE + 2]),
        (ResultsWake::Never, vec![]),
    ] {
        let mut h = fixture();
        start(&mut h, R::FIRST);
        let row = &mut crate::domain::task_mut(&mut h.domain, TASK).expect("parent").record;
        row.wake.results = policy;
        row.flags.ended_by = crate::EndedBy::Closer(crate::Closer { connector: 1, code: 6 });
        result_arrival(&mut h, MESSAGE, 7, crate::ResultKind::Report, &[8, 9]);
        result_arrival(&mut h, MESSAGE + 1, 8, crate::ResultKind::Failed, &[9]);
        // The relay sink held these results back; refill asks for them in order.
        let mut taken = Vec::new();
        let mut after = None;
        while let Some(word) = h.domain.next_waiting(TASK, after) {
            after = Some(word.number);
            taken.push(word.number);
        }
        let before_last = match policy {
            ResultsWake::LastOrFailure => vec![MESSAGE + 1],
            ResultsWake::Each => vec![MESSAGE, MESSAGE + 1],
            ResultsWake::Never => vec![],
        };
        assert_eq!(taken, before_last, "non-last success still waits: {policy:?}");
        result_arrival(&mut h, MESSAGE + 2, 9, crate::ResultKind::Report, &[]);
        if policy == ResultsWake::LastOrFailure {
            assert_eq!(h.domain.next_waiting(TASK, None).expect("success judged again").number, MESSAGE);
        }
        while let Some(word) = h.domain.next_waiting(TASK, after) {
            after = Some(word.number);
            taken.push(word.number);
        }
        assert_eq!(taken, expected, "{policy:?}");
        commit_read(&mut h, taken.last().copied(), &[], &taken);
        let row = h.domain.task(TASK).expect("parent");
        assert!(taken.iter().all(|number| !row.inbox.iter().any(|word| word.number == *number)));
    }
}

// The lead holds the landing's repair decision and asks its person in words.
const LANDING: u64 = 2;
const REPAIR: u64 = 9;
const PERSON_TASK: u64 = 10;
const ENTRY: u64 = 22;
const LEAD_ATTEMPT: u64 = 12;
const FOLLOW_UP: u64 = 11;
const RESULT: u64 = 23;
const FOLLOW_UP_ATTEMPT: u64 = 1;
const FOLLOW_UP_MESSAGE: u64 = 24;
const REPAIR_QUESTION: u64 = 25;
const REPAIR_ANSWER: u64 = 26;
const CLOSER: crate::EndedBy = crate::EndedBy::Closer(crate::Closer { connector: 1, code: 6 });
const LAST: WakePolicy = WakePolicy { results: ResultsWake::LastOrFailure, ..WakePolicy::DEFAULT };

fn delegate(h: &mut Harness, parent: u64, number: u64, person: bool) {
    let mut new = task(number, &[]);
    new.funder = crate::Funder::Task(parent);
    new.numbers.budget = 20;
    new.authority.budget.spend = 20;
    if person {
        new.executor = crate::Executor::Person(crate::PersonAddress::Person(PERSON));
    }
    let out = h.send(Event::Make {
        maker: None,
        places: Box::new([]),
        reply_to: to(),
        creator: Party::Task(parent),
        batch: Box::new([new]),
    });
    assert!(
        out.iter().any(|r| match r {
            Request::Made { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
}

fn repair_attempt(h: &mut Harness, attempt: u64) {
    start_task(h, REPAIR, attempt, R::ALL);
    let out = h.send(Event::Activation {
        reply_to: to(),
        task: REPAIR,
        attempt,
        end: End::Finished { result: TaskResult::NoChange { words: Box::new([]) }, cancel_delegates: false },
        saved: None,
        cause: crate::Cause::Unpriced,
    });
    assert!(
        out.iter().any(|r| match r {
            Request::Acknowledged { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
}

fn held_repair(offered: R) -> Harness {
    let mut h = fixture();
    hold_repair(&mut h, offered);
    h
}

fn hold_repair(h: &mut Harness, offered: R) {
    exhaust_repair(h);
    start_task(h, TASK, LEAD_ATTEMPT, offered);
    route_repair(h);
}

fn exhaust_repair(h: &mut Harness) {
    delegate(h, TASK, LANDING, false);
    delegate(h, LANDING, REPAIR, false);
    for attempt in 1..=3 {
        repair_attempt(h, attempt);
        h.env.now = h.env.now.saturating_add(Duration::from_secs(1));
        h.env.wall = Wall::from_nanos(h.env.now.as_nanos());
        let mut out = Queue::uncapped();
        crate::fire(&mut h.domain, &h.env, &mut out);
    }
    assert!(match h.domain.task(REPAIR).expect("repair").phase {
        Phase::Held { why: crate::Hold::Failures(crate::Class::Invalid), .. } => true,
        _ => false,
    });
}

fn route_repair(h: &mut Harness) {
    let revision = h.domain.task(REPAIR).expect("held repair").revisions.hold;
    let out =
        h.send(Event::RoutedEscalation { task: REPAIR, revision, holder: EscalationHolder::Task(TASK), entry: ENTRY });
    assert!(out.iter().any(|r| match r {
        Request::Relay { word, .. } if word.number == ENTRY => true,
        _ => false,
    }));
}

fn commit_read(h: &mut Harness, read: Option<u64>, refused: &[u64], entries: &[u64]) {
    let row = h.domain.task(TASK).expect("lead");
    let out = h.send(Event::Turn {
        reply_to: to(),
        task: TASK,
        attempt: row.attempt,
        turn: row.turn.checked_add(1).expect("fixture turns"),
        read,
        refused: refused.into(),
        offered: entries.last().copied(),
        outstanding: entries.into(),
        cumulative: 0,
    });
    assert!(
        out.iter().any(|r| match r {
            Request::TurnAcknowledged { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
}

fn finish_follow_up(h: &mut Harness) -> Vec<Request> {
    delegate(h, TASK, FOLLOW_UP, false);
    start_task(h, FOLLOW_UP, FOLLOW_UP_ATTEMPT, R::ALL);
    h.send(Event::Activation {
        reply_to: to(),
        task: FOLLOW_UP,
        attempt: FOLLOW_UP_ATTEMPT,
        end: End::Finished { result: TaskResult::Report { words: Box::new([]) }, cancel_delegates: false },
        saved: None,
        cause: crate::Cause::Unpriced,
    });
    h.send(Event::EffectsSettled { task: FOLLOW_UP });
    h.send(Event::Settled { task: FOLLOW_UP });
    h.send(Event::DelegateResult {
        task: TASK,
        word: word(RESULT, M::Result(crate::ResultKind::Report), Party::Task(FOLLOW_UP)),
    })
}

fn park_undecided_repair(h: &mut Harness) {
    hold_repair(h, R::ALL);
    park_covered_repair(h);
}

fn park_covered_repair(h: &mut Harness) {
    commit_read(h, Some(ENTRY), &[], &[ENTRY]);
    end(h, End::Parked);
}

#[test]
fn a_nonlast_success_wakes_a_parked_escalation_holder() {
    for ended_by in [crate::EndedBy::Result, CLOSER] {
        let mut h = holder(LAST, ended_by);
        park_undecided_repair(&mut h);
        finish_follow_up(&mut h);
        let row = h.domain.task(TASK).expect("lead");
        assert_eq!(row.phase, Phase::Active(Active::Due), "{ended_by:?}");
        assert_eq!(row.delegates.as_ref(), &[LANDING], "the result was not last");
        assert_eq!(row.due_reason, Why::Message { message: RESULT });
        assert_eq!(h.domain.run_context(TASK).expect("next run").reasons, R::ESCALATION.union(R::RESULT));
    }
}

const QUIET: WakePolicy = WakePolicy {
    questions: false,
    answers: false,
    timers: false,
    results: ResultsWake::Never,
    words: WakeRule::Never,
    notices: WakeRule::Never,
    news: WakeRule::Paced { span: PACE },
};

fn arrive_news(h: &mut Harness, class: crate::NewsClass) {
    let mut news = word(FOLLOW_UP_MESSAGE, M::News { subscription: SOURCE, class }, Party::Task(LANDING));
    news.at = h.env.wall;
    arrive(h, news);
}

fn fire_at(h: &mut Harness, due: Time) {
    h.env.now = due;
    h.env.wall = Wall::from_nanos(due.as_nanos());
    let mut out = Queue::uncapped();
    crate::fire(&mut h.domain, &h.env, &mut out);
}

fn parked_batched_holder() -> Harness {
    let mut h = holder(WakePolicy { news: BATCH, ..QUIET }, crate::EndedBy::Result);
    hold_repair(&mut h, R::ESCALATION);
    park_covered_repair(&mut h);
    h
}

#[test]
fn a_single_batched_news_message_wakes_the_escalation_holder() {
    let mut h = parked_batched_holder();
    arrive_news(&mut h, crate::NewsClass::Wakes);
    assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Due));
    assert_eq!(h.domain.run_context(TASK).expect("next run").reasons, R::ESCALATION.union(R::NEWS));
}

#[test]
fn without_an_escalation_single_batched_news_waits_for_age() {
    let mut h = parked_batched_holder();
    reject_repair(&mut h);
    arrive_news(&mut h, crate::NewsClass::Wakes);
    let due = h.env.now.saturating_add(BATCH_AGE);
    assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Idle));
    assert_eq!(h.domain.next_deadline(), Some(due));
    fire_at(&mut h, due);
    assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Due));
}

fn send_spoken(h: &mut Harness, task: u64, mut word: Word, words: &[u8]) {
    word.words = words.into();
    word.at = h.env.wall;
    h.send(Event::Message { reply_to: to(), project: 1, operation: crate::Message { task, word } });
}

#[test]
fn a_policy_kept_answer_wakes_the_escalation_holder() {
    let mut h = holder(QUIET, crate::EndedBy::Result);
    hold_repair(&mut h, R::ESCALATION);
    // The question is to a delegate, not the lead's requester, so its answer has no mandatory wake.
    send_spoken(&mut h, LANDING, word(REPAIR_QUESTION, M::Question, Party::Task(TASK)), b"Can you help?");
    park_covered_repair(&mut h);
    send_spoken(
        &mut h,
        TASK,
        word(REPAIR_ANSWER, M::Answer { question: REPAIR_QUESTION }, Party::Task(LANDING)),
        b"Yes.",
    );
    let row = h.domain.task(TASK).expect("lead");
    assert_eq!(row.phase, Phase::Active(Active::Due));
    assert_eq!(row.due_reason, Why::Message { message: REPAIR_ANSWER });
}

#[test]
fn unoffered_kept_news_wakes_an_escalation_holder_after_park() {
    let mut h = holder(QUIET, crate::EndedBy::Result);
    hold_repair(&mut h, R::ESCALATION);
    arrive_news(&mut h, crate::NewsClass::Kept);
    park_covered_repair(&mut h);
    // This pin checks the wake, not whether a park has to pause before it.
    if let Phase::Active(Active::BackingOff { until }) = h.domain.task(TASK).expect("lead").phase {
        fire_at(&mut h, Time::from_nanos(until.as_nanos()));
    }
    let row = h.domain.task(TASK).expect("lead");
    assert_eq!(row.phase, Phase::Active(Active::Due));
    assert_eq!(row.due_reason, Why::Message { message: FOLLOW_UP_MESSAGE });
}

fn kept_arrivals() -> [(M, R); 7] {
    [
        (M::Result(crate::ResultKind::Report), R::RESULT),
        (M::News { subscription: SOURCE, class: crate::NewsClass::Kept }, R::NEWS),
        (M::News { subscription: SOURCE, class: crate::NewsClass::Wakes }, R::NEWS),
        (M::Question, R::QUESTION),
        (M::Timer { subscription: SOURCE }, R::TIMER),
        (M::Notice { subscription: SOURCE, target: SOURCE, state: crate::NoticeState::Done }, R::NOTICE),
        (M::Words, R::WORDS),
    ]
}

#[test]
fn otherwise_kept_messages_wake_a_covered_escalation_holder() {
    for (kind, reason) in kept_arrivals() {
        let mut h = holder(QUIET, crate::EndedBy::Result);
        park_undecided_repair(&mut h);
        arrive(&mut h, word(FOLLOW_UP_MESSAGE, kind.clone(), Party::Task(LANDING)));
        let row = h.domain.task(TASK).expect("lead");
        assert_eq!(row.phase, Phase::Active(Active::Due), "{kind:?}");
        assert_eq!(row.due_reason, Why::Message { message: FOLLOW_UP_MESSAGE }, "{kind:?}");
        assert_eq!(h.domain.run_context(TASK).expect("next run").reasons, R::ESCALATION.union(reason), "{kind:?}");
    }
}

#[test]
fn a_restored_covered_escalation_wakes_on_the_next_message() {
    let mut h = holder(LAST, crate::EndedBy::Result);
    park_undecided_repair(&mut h);
    h.restart();
    finish_follow_up(&mut h);
    assert_eq!(h.domain.run_context(TASK).expect("next run").reasons, R::ESCALATION.union(R::RESULT));
    assert_eq!(h.domain.task(TASK).expect("lead").due_reason, Why::Message { message: RESULT });
}

#[test]
fn live_escalation_holders_hear_nonlast_successes_without_new_parts() {
    for started in [false, true] {
        let mut h = holder(LAST, crate::EndedBy::Result);
        exhaust_repair(&mut h);
        claim_task(&mut h, TASK, LEAD_ATTEMPT, R::FIRST);
        if started {
            h.send(Event::Started { task: TASK, attempt: LEAD_ATTEMPT });
        }
        route_repair(&mut h);
        let out = finish_follow_up(&mut h);
        let row = h.domain.task(TASK).expect("lead");
        assert!(
            out.iter().any(|request| match request {
                Request::Relay { task: TASK, attempt: LEAD_ATTEMPT, word, .. } => word.number == RESULT,
                _ => false,
            }),
            "started={started}: {out:?}"
        );
        assert_eq!(h.domain.next_waiting(TASK, Some(ENTRY)).map(|word| word.number), Some(RESULT));
        assert_eq!((row.offered, row.reasons), (R::FIRST, R::FIRST));
    }
}

fn reject_repair(h: &mut Harness) {
    let revision = h.domain.task(REPAIR).expect("repair").revisions.hold;
    h.send(Event::DecideEscalation {
        reply_to: to(),
        task: REPAIR,
        revision,
        by: crate::EscalationDecider::Task(TASK),
        entry: None,
        decision: crate::EscalationDecision::Reject { reason: Box::new([]) },
    });
}

#[test]
fn suppressed_arrivals_stay_kept_without_a_waiting_escalation() {
    for decided in [false, true] {
        for (kind, _) in kept_arrivals() {
            let mut h = holder(QUIET, crate::EndedBy::Result);
            if decided {
                park_undecided_repair(&mut h);
                reject_repair(&mut h);
            } else {
                delegate(&mut h, TASK, LANDING, false);
                start(&mut h, R::ALL);
                end(&mut h, End::Parked);
            }
            arrive(&mut h, word(FOLLOW_UP_MESSAGE, kind.clone(), Party::Task(LANDING)));
            assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Idle), "{decided}: {kind:?}");
        }
    }
}

#[test]
fn a_covered_proposal_does_not_wake_on_nonlast_success() {
    let mut h = holder(LAST, crate::EndedBy::Result);
    delegate(&mut h, TASK, REPAIR, false);
    start(&mut h, R::ALL);
    proposal(&mut h);
    commit_read(&mut h, Some(MESSAGE), &[], &[MESSAGE]);
    end(&mut h, End::Parked);
    finish_follow_up(&mut h);
    assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Idle));
}

#[test]
fn ending_escalation_wait_restores_batch_and_pace_deadlines() {
    for (rule, span) in [(BATCH, BATCH_AGE), (WakeRule::Paced { span: PACE }, PACE)] {
        let mut h = holder(WakePolicy { news: rule, ..QUIET }, crate::EndedBy::Result);
        hold_repair(&mut h, R::ALL);
        arrive_news(&mut h, crate::NewsClass::Wakes);
        reject_repair(&mut h);
        let due = h.env.now.saturating_add(span);
        assert_eq!(h.domain.next_deadline(), Some(due), "{rule:?}");
        end(&mut h, End::Parked);
        assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Idle), "{rule:?}");
        fire_at(&mut h, due);
        assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Due), "{rule:?}");
        assert_eq!(h.domain.run_context(TASK).expect("next run").reasons, R::NEWS, "{rule:?}");
    }
}

#[test]
fn a_parked_escalation_waits_when_covered_regardless_of_person_delegates() {
    for (name, offered, person, read, refused, phase) in [
        ("covered unread without person", R::ALL, false, None, &[][..], Active::Idle),
        ("covered read with person", R::ALL, true, Some(ENTRY), &[][..], Active::Idle),
        ("uncovered with person", R::QUESTION, true, Some(ENTRY), &[][..], Active::Due),
        ("refused with person", R::ALL, true, Some(ENTRY), &[ENTRY][..], Active::Due),
    ] {
        let mut h = held_repair(offered);
        commit_read(&mut h, read, refused, &[ENTRY]);
        if person {
            delegate(&mut h, TASK, PERSON_TASK, true);
        }
        let held = h.domain.task(REPAIR).expect("repair").clone();
        end(&mut h, End::Parked);
        assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(phase), "{name}");
        let row = h.domain.task(REPAIR).expect("repair");
        assert_eq!(row.phase, held.phase, "{name}");
        assert_eq!(row.escalation, held.escalation, "{name}");
    }
}

#[test]
fn restored_entries_wake_only_uncovered_idle_holders() {
    for (name, arrival, offered, due) in [
        ("covered escalation", Arrival::Escalation, R::ESCALATION, false),
        ("unpartitioned escalation", Arrival::Escalation, R::ALL, false),
        ("uncovered escalation", Arrival::Escalation, R::QUESTION, true),
        ("covered proposal", Arrival::Proposal, R::PROPOSAL, false),
        ("unpartitioned proposal", Arrival::Proposal, R::ALL, false),
        ("uncovered proposal", Arrival::Proposal, R::QUESTION, true),
    ] {
        let mut h = fixture();
        start(&mut h, R::ALL);
        match arrival {
            Arrival::Escalation => {
                escalation(&mut h);
            }
            Arrival::Proposal => {
                proposal(&mut h);
            }
            Arrival::Message(_) => unreachable!("entry rows only"),
        }
        end(&mut h, End::Parked);
        crate::domain::task_mut(&mut h.domain, TASK).expect("idle holder").record.offered = offered;
        let mut out = Queue::uncapped();
        crate::domain::publish(&mut h.domain, &h.env, TASK, &mut out);
        // Persist the offered kinds as part of the task row before the restart.
        for request in core::iter::from_fn(|| out.pop()) {
            if let Request::Save { record } = request {
                h.stored.insert(record.key(), record);
            }
        }
        let before = h.domain.next_waiting(TASK, None);
        h.restart();
        assert_eq!(h.domain.task(TASK).expect("holder").phase == Phase::Active(Active::Due), due, "{name}");
        assert_eq!(h.domain.next_waiting(TASK, None), before, "{name}");
    }
}

#[test]
fn a_covered_escalation_keeps_its_stall_deadline() {
    let mut h = held_repair(R::ALL);
    commit_read(&mut h, Some(ENTRY), &[], &[ENTRY]);
    end(&mut h, End::Parked);
    h.env.now = h.env.now.saturating_add(h.env.limits.escalation_stall);
    h.env.wall = Wall::from_nanos(h.env.now.as_nanos());
    let mut out = Queue::uncapped();
    crate::fire(&mut h.domain, &h.env, &mut out);
    assert!(core::iter::from_fn(|| out.pop()).any(|r| match r {
        Request::EscalationStalled { task: REPAIR, holder: EscalationHolder::Task(TASK), .. } => true,
        _ => false,
    }));
    assert_eq!(h.domain.task(TASK).expect("lead").phase, Phase::Active(Active::Idle));
}
