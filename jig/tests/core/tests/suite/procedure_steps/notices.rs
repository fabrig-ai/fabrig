use super::{decide, make_delegate, program_world_with_batch};
use jig_core_world::fixtures::live_row;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const BATCH: u32 = 1;
const NOTICE_BYTES: usize = tasks::ANSWER_NOTICE_BYTES;

fn step(messages: &[core::StepMessage]) -> core::ProcedureStep {
    core::ProcedureStep {
        messages: messages.iter().map(copy).collect(),
        ..core::ProcedureStep::new(core::StepEnd::Wait)
    }
}

fn copy(part: &core::StepMessage) -> core::StepMessage {
    core::StepMessage { target: part.target, kind: part.kind.clone(), words: part.words.clone(), notice: part.notice }
}

fn message(target: u64, kind: tasks::MessageKind, words: &[u8], notice: Option<u64>) -> core::StepMessage {
    core::StepMessage { target, kind, words: words.into(), notice }
}

fn wake(world: &mut World, task: u64) {
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task })));
}

fn question_world(seed: u64) -> (World, u64, u64, u64) {
    let (mut world, asker) = program_world_with_batch(seed, None, None, Some(BATCH));
    let recipient = make_delegate(&mut world, asker);
    wake(&mut world, asker);
    decide(&mut world, asker, step(&[message(recipient, tasks::MessageKind::Question, b"ready?", None)]));
    let question = live_row(&world, recipient).inbox.last().expect("sent word").number;
    (world, asker, recipient, question)
}

fn answer(world: &mut World, asker: u64, recipient: u64, question: u64) {
    let run = *world.assignments.iter().rev().find(|(task, _)| *task == recipient).expect("recipient agent's run");
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9003)),
        key: core::CallKey { task: run.0, attempt: run.1, completion: 1, position: 0 },
        action: core::NamedAction::Message {
            target: asker,
            kind: tasks::MessageKind::Answer { question },
            words: b"yes".as_slice().into(),
        },
    }));
    assert!(live_row(world, asker).inbox.iter().any(|word| word.kind == tasks::MessageKind::Answer { question }));
}

#[test]
fn a_procedure_notice_fits_beside_a_full_message_batch() {
    let (mut world, asker, recipient, question) = question_world(483);
    answer(&mut world, asker, recipient, question);
    let notice = [b'n'; NOTICE_BYTES];
    decide(
        &mut world,
        asker,
        step(&[
            message(recipient, tasks::MessageKind::Words, b"ordinary", None),
            message(recipient, tasks::MessageKind::Words, &notice, Some(question)),
        ]),
    );
    let got = &live_row(&world, recipient).inbox;
    // The agent has not read its question yet.
    assert_eq!(got.len(), BATCH as usize + 2);
    for (word, expected) in got.iter().skip(1).zip([b"ordinary".as_slice(), notice.as_slice()]) {
        assert_eq!(word.kind, tasks::MessageKind::Words);
        assert_eq!(word.from, tasks::Party::Task(asker));
        assert_eq!(word.words.as_ref(), expected);
    }
    assert!(live_row(&world, asker).inbox.is_empty());
}

fn refuses_whole(world: &mut World, asker: u64, recipient: u64, messages: &[core::StepMessage]) {
    let parent = live_row(world, asker).clone();
    let child = live_row(world, recipient).clone();
    decide(world, asker, core::ProcedureStep { words: Some(b"must not commit".as_slice().into()), ..step(messages) });
    assert_eq!(live_row(world, asker), &parent);
    assert_eq!(live_row(world, recipient), &child);
}

#[test]
fn a_bad_notice_refuses_the_whole_step() {
    let (mut world, asker, recipient, question) = question_world(484);
    let good = || message(recipient, tasks::MessageKind::Words, b"notice", Some(question));
    let ordinary = || message(recipient, tasks::MessageKind::Words, b"ordinary", None);
    // The answer arrives after this step's offer, so even its real question
    // number cannot be used until the next step.
    wake(&mut world, asker);
    answer(&mut world, asker, recipient, question);
    refuses_whole(&mut world, asker, recipient, &[ordinary(), good()]);
    decide(&mut world, asker, step(&[]));
    let other = world.assignments[0].0;
    for bad in [
        message(other, tasks::MessageKind::Words, b"wrong target", Some(question)),
        message(recipient, tasks::MessageKind::Question, b"wrong kind", Some(question)),
        message(recipient, tasks::MessageKind::Words, &[b'n'; NOTICE_BYTES + 1], Some(question)),
    ] {
        refuses_whole(&mut world, asker, recipient, &[ordinary(), bad]);
    }
    refuses_whole(&mut world, asker, recipient, &[ordinary(), good(), good()]);
    decide(&mut world, asker, step(&[ordinary(), good()]));
    assert_eq!(live_row(&world, recipient).inbox.last().expect("sent word").words.as_ref(), b"notice");
}
