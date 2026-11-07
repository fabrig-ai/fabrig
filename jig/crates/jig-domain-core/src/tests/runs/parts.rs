//! Run preparation chooses instructions and messages for its reasons (jig/docs/design/domain/engine.md, 7.1).
use crate::routing::run::{
    parts::{offered_kinds, run_instructions},
    preparation,
};
use crate::tests::*;
use jig_domain_link::{Form, Scheme};
use tasks::Reasons;

fn part(on: Reasons, instructions: &[u8]) -> CharterPart {
    CharterPart { on, instructions: instructions.into() }
}

#[test]
fn run_parts_follow_reasons_in_order_and_own_instructions_replace_them() {
    let mut charter = config(1).defaults.charters[0].clone();
    charter.policy.instructions = b"S".as_slice().into();
    let parts = Box::new([part(Reasons::FIRST, b"F"), part(Reasons::QUESTION, b"Q")]);
    let none: Box<[CharterPart]> = Box::new([]);
    for (name, parts, own, reasons, instructions, offered) in [
        ("none", none, None, Reasons::FIRST, b"S".as_slice(), Reasons::ALL),
        ("first", parts.clone(), None, Reasons::FIRST, b"S\n\nF", Reasons::EMPTY),
        (
            "first and question",
            parts.clone(),
            None,
            Reasons::FIRST.union(Reasons::QUESTION),
            b"S\n\nF\n\nQ",
            Reasons::QUESTION,
        ),
        ("question", parts.clone(), None, Reasons::QUESTION, b"S\n\nQ", Reasons::QUESTION),
        ("unmatched", parts.clone(), None, Reasons::RESULT, b"S", Reasons::EMPTY),
        (
            "both read once",
            Box::new([part(Reasons::QUESTION.union(Reasons::RESULT), b"QR")]),
            None,
            Reasons::QUESTION.union(Reasons::RESULT),
            b"S\n\nQR",
            Reasons::QUESTION.union(Reasons::RESULT),
        ),
        (
            "whole selected on",
            Box::new([part(Reasons::QUESTION.union(Reasons::RESULT), b"QR")]),
            None,
            Reasons::QUESTION,
            b"S\n\nQR",
            Reasons::QUESTION.union(Reasons::RESULT),
        ),
        ("own", parts.clone(), Some(b"own".as_slice()), Reasons::FIRST, b"own", Reasons::ALL),
        ("empty own", parts, Some(b"".as_slice()), Reasons::FIRST, b"", Reasons::ALL),
    ] {
        charter.parts = parts;
        assert_eq!(run_instructions(&charter, own, reasons).as_ref(), instructions, "{name}: instructions");
        assert_eq!(offered_kinds(&charter, own.is_some(), reasons), offered, "{name}: offered");
    }
}

#[test]
fn preparation_offers_person_words_and_keeps_an_uncovered_question() {
    let (mut core, mut limits) = make_tree();
    let charter = core.current.test_charter_mut(1);
    charter.policy.resume = false;
    charter.policy.instructions = b"S".as_slice().into();
    charter.parts = Box::new([part(Reasons::FIRST, b"F")]);
    // A skipped question comes first, so the run-inbox cap is applied after filtering.
    limits.tasks.run_inbox = 1;
    for (number, from, kind) in [
        (1, tasks::Party::Task(2), tasks::MessageKind::Question),
        (2, tasks::Party::Person(1), tasks::MessageKind::Words),
    ] {
        send_task(
            &mut core,
            &limits,
            tasks::Event::Message {
                reply_to: ReplyTo::new(Token::new(number)),
                project: 1,
                operation: tasks::Message {
                    task: 1,
                    word: tasks::Word {
                        number,
                        from,
                        kind,
                        words: b"person".as_slice().into(),
                        at: Wall::EPOCH,
                        hits: 1,
                        eligible: true,
                    },
                },
            },
        );
    }
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 1 });
    core.contexts.insert(1, core.tasks.run_context(1).unwrap()).unwrap();
    core.begin_transcript(1, 0, None);
    watch_outputs(&mut core, &limits, Event::BriefAssembled { task: 1, failure: None });
    let attempt = core.workspace_flight(1).unwrap().1;
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            task: 1,
            attempt,
            holds: Box::new([]),
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: None,
        },
    );
    let mut prepared = None;
    for request in &out {
        if let Request::Now(now) = request
            && let Now::RunPrepared { run, messages, .. } = now.as_ref()
        {
            prepared = Some((run, messages));
        }
    }
    let (run, messages) = prepared.expect("prepared run");
    assert_eq!(run.policy.instructions.as_ref(), b"S\n\nF");
    let mut numbers = List::with_capacity(limits.tasks.run_inbox);
    for word in messages {
        numbers.push(word.number).unwrap();
    }
    assert_eq!(numbers.as_slice(), [2]);
    let proof = core.proofs.get(&1).unwrap();
    assert_eq!(proof.outstanding.as_ref(), [2]);
    assert_eq!(proof.offered, Some(2));
    let mut work = Queue::uncapped();
    let reasons = core.contexts.get(&1).unwrap().reasons;
    preparation::route_claim_prepared(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        &mut work,
        1,
        attempt,
        run.budget,
        Box::new([]),
    );
    let Some(Work::Event(Event::Tasks(tasks::Event::Claim { offered, reasons: claimed, .. }))) = work.pop() else {
        panic!("prepared claim");
    };
    assert_eq!(claimed, reasons);
    assert_eq!(offered, Reasons::EMPTY);
}
