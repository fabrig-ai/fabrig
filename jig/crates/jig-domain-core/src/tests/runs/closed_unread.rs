use crate::tests::*;

fn unread_window(transient: u32, closed: bool) -> (Core, Limits) {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let mut row = core.tasks.task(1).unwrap().clone();
    row.tries.transient = transient;
    let ledger = *core.tasks.funding(row.funder).unwrap();
    fixtures::restore(&mut core, &limits, Box::new([row]));
    send_task(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) });
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 1 });
    let mut proof = RunProof::claimed(1, 1, 1, None, jig_domain_fleet::HostKind::Worker);
    proof.window = Some(CurrentWindow { attempt: 1, turn: 1, closed });
    core.proofs.insert(1, proof).unwrap();
    core.begin_transcript(1, 1, Some((1, 1)));
    core.contexts.insert(1, core.tasks.run_context(1).unwrap()).unwrap();
    core.transcripts.get_mut(&1).unwrap().append_text(Some(b"partial text".as_slice().into()), 128);
    (core, limits)
}

fn answer_unread(core: &mut Core, limits: &Limits, work: &mut Queue<Work>) -> Queue<Request> {
    let load = fixture_window(core, 1, (1, 1));
    loads::loaded(
        core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
        work,
        load,
        engine_store::Answer::Failed(engine_store::Failure::Unread),
    )
}

fn assert_fresh(core: &Core, work: &mut Queue<Work>) {
    let Some(Work::StartBrief { task }) = work.pop() else { panic!("unread tail continues the brief") };
    assert_eq!(task, 1);
    assert!(work.is_empty(), "no preparation failure is counted");
    assert!(core.contexts.contains_key(&1));
    let transcript = core.transcripts.get(&1).unwrap();
    assert!(transcript.window.is_none());
    assert!(transcript.last.is_none());
    assert!(transcript.texts.is_empty());
}

#[test]
fn an_unread_closed_tail_continues_the_brief_without_counting() {
    let (mut core, limits) = unread_window(0, true);
    let mut work = Queue::with_capacity(2);
    let _requests = answer_unread(&mut core, &limits, &mut work);
    assert_fresh(&core, &mut work);
    assert!(core.transcripts.get(&1).unwrap().closed);
}

#[test]
fn an_unread_closed_tail_discards_a_loaded_prefix_and_ends_the_load() {
    use engine_store::{Answer, Failure, Rows};
    let (mut core, limits) = unread_window(0, true);
    let load = fixture_window(&mut core, 1, (1, 1));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut work = Queue::with_capacity(2);
    let turn = TurnRecord {
        task: 1,
        attempt: 1,
        turn: 1,
        spent: 0,
        read: None,
        opens_window: true,
        at: Wall::EPOCH,
        transcript: b"first turn".as_slice().into(),
    };
    let page =
        Answer::Rows(Rows::Turns { turns: Box::new([(turn, Some(b"first turn".as_slice().into()))]), more: true });
    assert!(loads::loaded(&mut core, &env, &mut work, load, page).is_empty());
    assert!(work.is_empty());
    assert!(loads::awaited(&core, load));
    assert_eq!(core.transcripts.get(&1).unwrap().last, Some((1, 1)));

    let terminal = Answer::Failed(Failure::Unread);
    assert!(loads::loaded(&mut core, &env, &mut work, load, terminal).is_empty());
    assert_fresh(&core, &mut work);
    assert!(!loads::awaited(&core, load));
}

#[test]
fn an_unread_open_window_after_a_transient_try_counts_once_and_closes() {
    assert_open_unread_counts(1);
}

#[test]
fn an_unread_open_window_without_a_transient_retry_counts_once() {
    assert_open_unread_counts(0);
}

fn assert_open_unread_counts(before: u32) {
    let (mut core, limits) = unread_window(before, false);
    let mut work = Queue::with_capacity(2);
    let requests = answer_unread(&mut core, &limits, &mut work);
    let mut saved = false;
    for request in &requests {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof)))) = request {
            saved = proof.window.expect("unread window retained").closed;
        }
    }
    assert!(saved, "closure is saved in the failure decision");
    assert!(core.proofs.get(&1).unwrap().window.unwrap().closed);
    let Some(Work::Event(Event::Tasks(
        event @ tasks::Event::PreparationCounted { task: 1, class: tasks::Class::Transient, .. },
    ))) = work.pop()
    else {
        panic!("an unread resume spends one transient try");
    };
    assert!(work.is_empty());
    assert!(!core.contexts.contains_key(&1));
    assert!(!core.transcripts.contains_key(&1));
    send_task(&mut core, &limits, event);
    assert_eq!(core.tasks.task(1).unwrap().tries.transient, before.checked_add(1).expect("one counted failure"));
}

#[test]
fn incompatible_window_answers_fail_and_never_close_the_window() {
    use engine_store::{Answer, Failure, Rows};
    for answer in [
        Answer::Rows(Rows::Records { records: Box::new([]), more: true }),
        Answer::Rows(Rows::Read { rows: Box::new([]), next: None }),
        Answer::Failed(Failure::Busy),
        Answer::Failed(Failure::Gone),
        Answer::Failed(Failure::TooLinked { joins: 1 }),
        Answer::Failed(Failure::TurnProblem {
            task: 1,
            attempt: 1,
            turn: 1,
            owner: Box::new(fixtures::ended_report(1, tasks::Party::Person(1))),
            why: crate::reads::TurnFailure::Malformed,
        }),
        Answer::Failed(Failure::Failed),
    ] {
        let (mut core, limits) = unread_window(0, false);
        let load = fixture_window(&mut core, 1, (1, 1));
        let requests = watch_store_outputs(&mut core, &limits, engine_store::In::Loaded { load, result: answer });
        let mut failed = false;
        for request in requests {
            if let Request::Now(now) = request
                && let Now::BriefFailed { task: 1, failure } = *now
            {
                assert_eq!(failure.words.as_ref(), b"its window could not be loaded");
                failed = true;
            }
        }
        assert!(failed);
        assert_eq!(core.tasks.task(1).unwrap().tries.transient, 1);
        assert!(!core.proofs.get(&1).unwrap().window.unwrap().closed);
        assert!(!core.transcripts.contains_key(&1));
        assert!(!loads::awaited(&core, load));
        assert!(
            watch_store_outputs(
                &mut core,
                &limits,
                engine_store::In::Loaded { load, result: Answer::Failed(Failure::Unread) }
            )
            .is_empty(),
            "a failed window has exactly one terminal"
        );
    }
}
