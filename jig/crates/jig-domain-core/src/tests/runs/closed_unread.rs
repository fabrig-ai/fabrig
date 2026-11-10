use crate::{routing::run::preparation::transcript_unread, tests::*};

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
    let (mut core, _) = unread_window(0, true);
    let mut work = Queue::with_capacity(2);
    let _requests = transcript_unread(&mut core, &mut work, 1);
    assert_fresh(&core, &mut work);
    assert!(core.transcripts.get(&1).unwrap().closed);
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
    let requests = transcript_unread(&mut core, &mut work, 1);
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
