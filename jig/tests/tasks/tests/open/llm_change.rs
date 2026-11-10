//! A requester's new LLM choice belongs to the next turn (domain/tasks/changes.md).
use super::*;
use jig_domain_tasks::{Change, LlmChoice, ModelEffort, Why};

const OPEN: Flags = Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN };

fn choice(model: u8, effort: Option<u8>) -> LlmChoice {
    LlmChoice { later: None, account: None, main: ModelEffort { model, effort }, subagents: Box::new([]) }
}

fn listening(flags: Flags) -> World {
    let mut chat = task(1, &[]);
    chat.flags = flags;
    chat.authority.models = Models::Named(0b11);
    chat.llm = Some(choice(0, Some(0)));
    let mut w = claimed_task(chat);
    commit_turn(&mut w, 1, None);
    w
}

fn amendment(llm: LlmChoice, instructions: Option<Box<[u8]>>) -> Amendment {
    Amendment {
        tracked: None,
        addressed: None,
        instructions,
        release: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: Some(llm),
        reason: b"change next run".as_slice().into(),
    }
}

fn amend(w: &mut World, llm: LlmChoice, instructions: Option<Box<[u8]>>) {
    let reply_to = w.to();
    let call = reply_to.into_token();
    w.send(Event::Amend {
        operation: tasks::Amend {
            task: 1,
            message: w.record(1).last_message + 1,
            stop_run: false,
            amendment: Box::new(amendment(llm, instructions)),
        },
        places: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(call),
        by: Party::Person(1),
        funder: None,
    });
    assert_eq!(w.replies[&call.raw()], Reply::Done);
}

#[test]
fn a_new_choice_on_an_idle_chat_adds_no_word_and_wakes_nothing() {
    let mut w = listening(OPEN);
    assert_eq!(w.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    let inbox = w.record(1).inbox.clone();
    let messages = w
        .records
        .values()
        .filter(|row| matches!(row, Stored::Activity(row) if matches!(row.line, Line::Words { .. } | Line::Heard(_))))
        .count();
    amend(&mut w, choice(1, None), None);
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert!(!w.record(1).llm_changed);
    assert_eq!(w.activations(1), 1);
    assert!(w.activations.is_empty());
    assert_eq!(w.record(1).inbox, inbox);
    assert_eq!(w.record(1).last_message, 1);
    assert_eq!(
        w.records
            .values()
            .filter(
                |row| matches!(row, Stored::Activity(row) if matches!(row.line, Line::Words { .. } | Line::Heard(_)))
            )
            .count(),
        messages
    );
    assert!(
        w.records
            .values()
            .any(|row| matches!(row, Stored::History(row) if row.task == 1 && row.change == Change::Amended))
    );
}

#[test]
fn a_new_choice_on_a_listening_chat_reaches_no_run() {
    let mut w = listening(OPEN);
    let next = choice(1, None);
    amend(&mut w, next.clone(), None);
    assert!(w.relays.is_empty(), "the amendment says nothing to the live run");
    assert!(w.stops.is_empty(), "the choice alone leaves the run listening");
    assert_eq!(w.record(1).llm, Some(next));
    assert!(w.record(1).llm_changed);
    assert!(w.record(1).inbox.is_empty());
    assert_eq!(w.record(1).phase, Phase::Active(Active::Running { attempt: 1 }));
}

#[test]
fn the_words_after_a_new_choice_stop_the_run_and_wait_for_the_next() {
    let mut w = listening(OPEN);
    amend(&mut w, choice(1, None), None);
    assert_eq!(say(&mut w, 2), Reply::Done);
    assert!(w.stops.contains(&(1, 1)), "the next words stop the old choice");
    assert!(w.relays.is_empty());
    assert_eq!(w.record(1).inbox.iter().map(|word| word.number).collect::<Vec<_>>(), [2]);
    assert_eq!(w.terminal(1, raw_finish()), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(w.record(1).due_reason, Why::Message { message: 2 });
    assert_eq!(w.record(1).tries, Tries::NONE);
    assert!(!w.record(1).llm_changed);
    run_ended(&w, 1, EndKind::Parked);
    assert_eq!(w.activations(1), 2);
    assert_eq!(w.contexts[&1].inbox.iter().map(|word| word.number).collect::<Vec<_>>(), [2]);
    assert_eq!(w.contexts[&1].llm, Some(choice(1, None)));
    w.claim(1, 2);
    assert_eq!(w.record(1).llm, Some(choice(1, None)));
    commit_turn(&mut w, 2, Some(2));
    assert_eq!(w.terminal(1, raw_finish()), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert!(!w.record(1).llm_changed);
}

#[test]
fn more_words_while_the_run_stops_ask_no_second_stop() {
    let mut w = listening(OPEN);
    amend(&mut w, choice(1, None), None);
    assert_eq!(say(&mut w, 2), Reply::Done);
    assert!(w.stops.remove(&(1, 1)), "first words ask for a stop");
    for number in [3, 4] {
        assert_eq!(say(&mut w, number), Reply::Done);
        assert!(w.stops.is_empty(), "only one stop per attempt");
    }
    assert!(w.relays.is_empty());
    assert_eq!(w.record(1).due_reason, Why::Message { message: 2 });
    assert_eq!(w.record(1).inbox.iter().map(|word| word.number).collect::<Vec<_>>(), [2, 3, 4]);
}

#[test]
fn a_run_left_alone_after_a_new_choice_parks_and_the_next_words_wake_it() {
    let mut w = listening(OPEN);
    amend(&mut w, choice(1, None), None);
    assert_eq!(w.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert!(!w.record(1).llm_changed);
    assert!(w.activations.is_empty());
    assert_eq!(w.activations(1), 1);
    run_ended(&w, 1, EndKind::Parked);
    assert_eq!(say(&mut w, 2), Reply::Done);
    assert_eq!(w.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(w.record(1).due_reason, Why::Message { message: 2 });
}

#[test]
fn a_new_effort_alone_is_a_new_choice() {
    changed_choice(choice(0, Some(1)));
}

#[test]
fn a_subagents_model_is_part_of_the_choice() {
    let mut next = choice(0, Some(0));
    next.subagents = Box::new([ModelEffort { model: 1, effort: None }]);
    changed_choice(next);
}

#[test]
fn an_account_preference_is_part_of_the_choice() {
    let mut next = choice(0, Some(0));
    next.account = Some(7);
    changed_choice(next);
}

fn changed_choice(next: LlmChoice) {
    let mut w = listening(OPEN);
    amend(&mut w, next, None);
    assert!(w.record(1).llm_changed);
    assert_eq!(say(&mut w, 2), Reply::Done);
    assert!(w.stops.contains(&(1, 1)));
    assert!(w.relays.is_empty());
}

#[test]
fn an_amendment_naming_the_held_choice_marks_nothing() {
    let mut w = listening(OPEN);
    amend(&mut w, choice(0, Some(0)), None);
    assert!(!w.record(1).llm_changed);
    assert!(w.relays.is_empty());
    assert_eq!(say(&mut w, 2), Reply::Done);
    assert!(w.stops.is_empty());
    assert_eq!(w.relays.iter().copied().collect::<Vec<_>>(), [(1, 1, 2)]);
}

#[test]
fn a_new_choice_with_new_instructions_is_heard_by_the_next_run() {
    let mut w = listening(OPEN);
    amend(&mut w, choice(1, None), Some(b"new instructions".as_slice().into()));
    assert!(w.stops.contains(&(1, 1)));
    assert!(w.relays.is_empty());
    assert_eq!(w.record(1).inbox.len(), 1);
    assert_eq!(w.record(1).inbox[0].words.as_ref(), b"new instructions");
    assert_eq!(w.record(1).inbox[0].kind, MessageKind::Amendment { revision: 1 });
    assert_eq!(w.terminal(1, raw_finish()), Reply::Acknowledged(Accepted::New));
    assert_eq!(w.record(1).phase, Phase::Active(Active::Due));
    run_ended(&w, 1, EndKind::Parked);
    assert_eq!(w.contexts[&1].inbox[0].words.as_ref(), b"new instructions");
}

#[test]
fn a_finite_tasks_new_choice_is_heard_as_ever() {
    let mut w = listening(Flags::PLAIN);
    amend(&mut w, choice(1, None), None);
    assert!(!w.record(1).llm_changed);
    assert_eq!(w.relays.iter().copied().collect::<Vec<_>>(), [(1, 1, 1)]);
    assert!(w.stops.is_empty());
    assert_eq!(w.record(1).inbox.len(), 1);
    assert_eq!(w.record(1).inbox[0].kind, MessageKind::Amendment { revision: 1 });

    let mut idle = listening(Flags::PLAIN);
    let reply_to = idle.to();
    idle.send(Event::Subscribe {
        reply_to,
        task: 1,
        subscription: tasks::Subscription {
            number: 1,
            kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
        },
    });
    assert_eq!(idle.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    assert_eq!(idle.record(1).phase, Phase::Active(Active::Idle));
    amend(&mut idle, choice(1, None), None);
    assert_eq!(idle.record(1).phase, Phase::Active(Active::Due));
    assert_eq!(idle.record(1).inbox.len(), 1);
    assert!(!idle.record(1).llm_changed);
    assert_eq!(idle.activations(1), 2);
}

#[test]
fn a_run_behind_its_choice_is_still_behind_after_a_restart() {
    let mut w = listening(OPEN);
    amend(&mut w, choice(1, None), None);
    w.restart();
    assert!(w.record(1).llm_changed);
    assert_eq!(say(&mut w, 2), Reply::Done);
    assert!(w.stops.contains(&(1, 1)));
    assert!(!w.relays.contains(&(1, 1, 2)));
}

#[test]
fn a_run_behind_its_choice_is_offered_no_waiting_word() {
    let mut w = listening(OPEN);
    amend(&mut w, choice(1, None), None);
    assert_eq!(say(&mut w, 2), Reply::Done);
    let mut restored = jig_tasks_world::domain(&LIMITS, 327);
    let mut out = skein_lib::Queue::uncapped();
    tasks::step(
        &mut restored,
        &w.env,
        Event::Restore { record: Stored::Live(Box::new(w.record(1).clone())) },
        &mut out,
    );
    assert_eq!(restored.next_waiting(1, None), None, "relay polling must not offer the old run a word");
}

#[test]
fn compounded_llm_only_amendments_also_say_nothing() {
    let mut w = listening(OPEN);
    assert_eq!(w.terminal(1, End::Parked), Reply::Acknowledged(Accepted::New));
    let operations = [choice(1, None), choice(0, Some(1))]
        .into_iter()
        .enumerate()
        .map(|(at, llm)| {
            Operation::Amend(tasks::Amend {
                task: 1,
                message: u64::try_from(at).expect("two amendments") + 1,
                stop_run: false,
                amendment: Box::new(amendment(llm, None)),
            })
        })
        .collect::<Vec<_>>();
    let reply_to = w.to();
    let call = reply_to.into_token();
    w.send(Event::Compound {
        reply_to: skein_lib::ReplyTo::new(call),
        by: Party::Person(1),
        project: 1,
        operations: skein_lib::List::from_boxed(operations.into_boxed_slice()),
        period: 0,
        pool_budget: 1_000,
        period_budget: 10_000,
    });
    assert_eq!(w.replies[&call.raw()], Reply::Done);
    assert_eq!(w.record(1).phase, Phase::Active(Active::Idle));
    assert!(!w.record(1).llm_changed);
    assert!(w.record(1).inbox.is_empty());
    assert_eq!(w.activations(1), 1);
    assert_eq!(w.record(1).last_message, 2);
    assert_eq!(w.record(1).llm, Some(choice(0, Some(1))));
}
