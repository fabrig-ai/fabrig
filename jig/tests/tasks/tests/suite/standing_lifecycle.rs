//! A standing agent waits and only cancellation ends it (tasks/standing.md, section 2).
use jig_domain_tasks::{
    self as tasks, Accepted, Active, Cause, End, EndKind, Ending, Event, Funder, Key, Line, MessageKind, Party, Phase,
    Reasons, Request, ResultFollowups, Stored, TaskResult, Tries, Word,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::Duration;

pub(super) fn standing() -> tasks::New {
    let mut member = task(1, &[]);
    member.flags.ended_by = tasks::EndedBy::Standing;
    member
}

pub(super) fn world(seed: u64) -> World {
    let mut world = World::new(seed, LIMITS);
    assert_eq!(world.make(Party::Person(1), vec![standing()]), Reply::Made(vec![1]));
    world
}

pub(super) fn wake(world: &mut World) {
    let number = world.record(1).last_message + 1;
    let reply_to = world.to();
    world.send(Event::Message {
        reply_to,
        project: 1,
        operation: tasks::Message {
            task: 1,
            word: Word {
                number,
                from: Party::Person(1),
                kind: MessageKind::Words,
                words: b"again".as_slice().into(),
                at: world.env.wall,
                hits: 1,
                eligible: true,
            },
        },
    });
}

pub(super) fn read(world: &mut World) {
    let row = world.record(1);
    if row.inbox.is_empty() {
        return;
    }
    let attempt = row.attempt;
    let turn = row.turn + 1;
    let read = Some(row.last_message);
    let outstanding = row.inbox.iter().map(|word| word.number).collect();
    let reply_to = world.to();
    world.send(Event::Turn {
        reply_to,
        task: 1,
        attempt,
        turn,
        read,
        offered: read,
        outstanding,
        refused: Box::new([]),
        cumulative: 0,
    });
}

pub(super) fn cancel(world: &mut World) {
    let reply_to = world.to();
    world.send(Event::Cancel {
        reply_to,
        by: Party::Person(1),
        operation: tasks::Cancel { task: 1, reason: Box::new([]), grace: None, message: None },
    });
    world.complete_cancel();
    assert!(matches!(world.results.get(&1), Some(Ending::Cancelled { .. })));
}

#[test]
fn standing_parks_without_nothing_awaited_or_tries_across_restart() {
    let mut world = world(864);
    for attempt in 1..=3 {
        if attempt > 1 {
            wake(&mut world);
        }
        world.claim(1, attempt);
        read(&mut world);
        assert_eq!(world.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
        let row = world.record(1);
        assert_eq!(row.phase, Phase::Active(Active::Idle));
        assert_eq!(row.tries, Tries::NONE);
        assert!(row.inbox.is_empty());
        assert_eq!(row.last_message, attempt - 1, "no NothingAwaited was numbered");
        assert!(world.results.is_empty());
        world.restart();
        assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
        assert_eq!(world.record(1).tries, Tries::NONE);
    }
    cancel(&mut world);
}

#[test]
fn standing_finishes_park_with_bounded_words_and_no_followups() {
    let straddling = format!("{}界{}", "x".repeat(511), "y".repeat(86)).into_bytes();
    assert_eq!(straddling.len(), 600);
    let mut followup = task(2, &[]);
    followup.funder = Funder::Task(1);
    followup.authority.budget.spend = 5;
    followup.numbers.budget = 5;
    let cases = [
        (
            End::Finished { result: TaskResult::Report { words: b"done".as_slice().into() }, cancel_delegates: false },
            b"done".to_vec(),
        ),
        (
            End::Finished { result: TaskResult::Report { words: vec![b'x'; 600].into() }, cancel_delegates: false },
            vec![b'x'; 512],
        ),
        (
            End::Finished { result: TaskResult::Report { words: straddling.into() }, cancel_delegates: false },
            vec![b'x'; 511],
        ),
        (
            End::Finished {
                result: TaskResult::Failure { reason: b"failed".as_slice().into() },
                cancel_delegates: false,
            },
            b"failed".to_vec(),
        ),
        (
            End::FinishedWithFollowups {
                result: TaskResult::Verdict { code: 0, fields: Box::new([]), words: b"follow".as_slice().into() },
                followups: ResultFollowups::Delegates(Box::new([followup])),
            },
            b"follow".to_vec(),
        ),
    ];
    for (end, words) in cases {
        let mut world = World::new(865, LIMITS);
        let mut member = standing();
        if matches!(end, End::FinishedWithFollowups { .. }) {
            member.contract = tasks::Contract::Verdict {
                choices: Box::new([tasks::Verdict { code: 0, words: 32, fields: Box::new([]), followups: 1 }]),
            };
        }
        assert_eq!(world.make(Party::Person(1), vec![member]), Reply::Made(vec![1]));
        world.claim(1, 1);
        let reply_to = world.to();
        world.stage(Event::Activation {
            reply_to,
            task: 1,
            attempt: 1,
            end,
            saved: Some(Box::new([])),
            cause: Cause::Priced { cumulative: 30 },
        });
        assert!(world.staged_requests().iter().any(|request| matches!(
            request,
            Request::Acknowledged { task: 1, attempt: 1, accepted: Accepted::New, parked: true, .. }
        )));
        world.durable();
        world.deliver();
        assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
        assert_eq!(world.record(1).tries, Tries::NONE);
        assert_eq!(world.record(1).numbers.spent, 30);
        assert!(world.results.is_empty());
        assert_eq!(world.live(), 1, "follow-ups made nowhere");
        assert!(world.record(1).delegates.is_empty());
        assert!(world.record(1).proposal.is_none());
        let stood = world
            .records
            .values()
            .filter_map(|row| match row {
                Stored::Activity(row) if row.task == 1 => match &row.line {
                    Line::Stood { attempt, words } => Some((*attempt, words.as_ref())),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(stood, [(1, words.as_slice())]);
        assert!(world.records.values().any(|row| matches!(row,
            Stored::Activity(row) if row.task == 1 && row.line == Line::RunEnded { attempt: 1, end: EndKind::Parked, spent: 30 }
        )));
        world.restart();
        wake(&mut world);
        assert!(!world.contexts[&1].reasons.meets(Reasons::FIRST), "a finish parked ends the first activation");
        cancel(&mut world);
    }
}

#[test]
fn standing_idle_with_a_live_delegate_has_no_stall_bound() {
    let mut world = world(866);
    let mut child = task(2, &[]);
    child.funder = Funder::Task(1);
    child.authority.budget.spend = 20;
    child.numbers.budget = 20;
    assert_eq!(world.make(Party::Task(1), vec![child]), Reply::Made(vec![2]));
    world.claim(1, 1);
    world.terminal(1, End::Parked);
    // Limits have no agent-idle stall; pass both the person and proposal stalls.
    let beyond = LIMITS.person_stall.saturating_add(LIMITS.proposal_stall).saturating_add(Duration::from_secs(86_400));
    world.elapse(beyond);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    assert_eq!(world.record(1).delegates.as_ref(), [2]);
    world.restart();
    world.elapse(beyond);
    assert_eq!(world.record(1).phase, Phase::Active(Active::Idle));
    assert!(world.records.contains_key(&Key::Live(2)));
    cancel(&mut world);
}
