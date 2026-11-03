//! Calls answered at decode, as errors or successes, leave no commit and
//! no record (domain/root.md, 5.2; protocol/documentation.md, 5).
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::Token;
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_world::direct::Driver;

#[test]
fn an_undecodable_call_is_answered_as_an_error_with_no_commit() {
    let (mut driver, run) = Driver::chat();
    let (applied, rows) = (driver.store.applied, driver.store.rows.clone());
    let event = driver.protocol.undecodable(Token::new(7), run.task, run.attempt, 31);
    driver.send(event);
    driver.settle();
    assert_eq!(driver.store.applied, applied);
    assert_eq!(driver.store.rows, rows);
    let relayed: Vec<&host::Relayed> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::Link { down: host::LinkDown::Relayed { call, answer, .. }, .. } = delivery
                && call.as_ref() == 31_u64.to_be_bytes()
            {
                Some(answer)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(relayed, [&host::Relayed::Answer { error: true, text: Box::from(&b"unknown tool"[..]) }]);
}

#[test]
fn an_undecodable_call_leaves_no_record_for_the_resumed_run() {
    let (mut driver, run) = Driver::chat();
    let event = driver.protocol.undecodable(Token::new(7), run.task, run.attempt, 31);
    driver.send(event);
    driver.settle();
    driver.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(run.task, 93);
    let resumed = driver.last_assignment();
    assert_ne!(resumed.attempt, run.attempt);
    assert!(resumed.settled.is_empty());
    assert!(resumed.answered.is_empty());
}

#[test]
fn a_corrected_call_under_a_new_name_is_decided_once() {
    let (mut driver, run) = Driver::chat();
    let event = driver.protocol.undecodable(Token::new(7), run.task, run.attempt, 31);
    driver.send(event);
    driver.settle();
    driver.call(
        Token::new(7),
        run.task,
        run.attempt,
        32,
        engine::Call { completion: 2, position: 0, tool: engine::Tool::Unavailable },
    );
    driver.settle();
    assert_eq!(driver.answered(32), Some(&engine::CallAnswer::Unavailable));
    let name = 32_u64.to_be_bytes();
    assert_eq!(driver.protocol.answers.iter().filter(|(named, _)| named.as_ref() == name).count(), 1);
}

#[test]
fn a_call_answered_at_decode_is_relayed_as_a_success_with_no_commit() {
    let (mut driver, run) = Driver::chat();
    let (applied, rows) = (driver.store.applied, driver.store.rows.clone());
    let event = driver.protocol.answered_at_decode(Token::new(7), run.task, run.attempt, 31, b"answer made at decode");
    driver.send(event);
    driver.settle();
    assert_eq!(driver.store.applied, applied);
    assert_eq!(driver.store.rows, rows);
    let relayed: Vec<&host::Relayed> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::Link { down: host::LinkDown::Relayed { call, answer, .. }, .. } = delivery
                && call.as_ref() == 31_u64.to_be_bytes()
            {
                Some(answer)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(relayed, [&host::Relayed::Answer { error: false, text: Box::from(&b"answer made at decode"[..]) }]);
}

#[test]
fn a_call_answered_at_decode_leaves_no_record_for_the_resumed_run() {
    let (mut driver, run) = Driver::chat();
    let event = driver.protocol.answered_at_decode(Token::new(7), run.task, run.attempt, 31, b"answer made at decode");
    driver.send(event);
    driver.settle();
    driver.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(run.task, 93);
    let resumed = driver.last_assignment();
    assert_ne!(resumed.attempt, run.attempt);
    assert!(resumed.settled.is_empty());
    assert!(resumed.answered.is_empty());
}

fn decoded_task_call(
    driver: &mut Driver,
    run: &engine::Assignment,
    call: u64,
    tool: &[u8],
    input: &[u8],
) -> engine::CallAnswer {
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded task read");
    let temper_engine_smith::Decoded::Call(body) = temper_engine_smith::call(
        smith_domain_run::CallName {
            activation: run.attempt,
            completion: 1,
            position: u32::try_from(call).expect("call"),
        },
        tool,
        &input,
    )
    .expect("coordinator can decode its task tool") else {
        panic!("engine task call")
    };
    driver.call(Token::new(7), run.task, run.attempt, call, body);
    if driver.protocol.defer_render {
        driver.pump();
    } else {
        driver.settle();
    }
    driver.answered(call).expect("task call answered").clone()
}

fn coordinator() -> (Driver, engine::Assignment) {
    use temper_engine_domain_world::{protocol, store::Store};
    let bounds = super::added_deps::limits();
    let mut driver = Driver::configured(
        Store::new(),
        [super::added_deps::configuration(), super::added_deps::configuration()],
        &bounds,
    );
    driver.send(protocol::hello_slots(Token::new(7), bounds.fleet.slots, Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut ask = driver.chat_ask(1, b"G coordinates S and designers");
    let jig_domain_people::Ask::Create { batch, .. } = &mut ask else { panic!("creation") };
    batch[0].authority.tools = 519;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(1001)),
        sign_in: driver.session(),
        key: [31; 16],
        ask,
    }));
    driver.settle();
    let run = driver.last_assignment();
    (driver, run)
}

fn coordinator_delegates(
    driver: &mut Driver,
    run: &engine::Assignment,
    call: u64,
    words: &[&[u8]],
    after: Option<u64>,
) -> Vec<u64> {
    use temper_engine_domain_world::fixtures;
    let batch = words
        .iter()
        .map(|words| {
            let dependencies =
                after.map_or_else(|| Box::new([]) as Box<[_]>, |task| Box::new([engine::Dependency::Existing(task)]));
            let mut member = fixtures::report_delegate(words, dependencies);
            member.authority.tools = tasks::Tools(519);
            member
        })
        .collect();
    fixtures::tool_call(driver, run, call, engine::Tool::Delegate { batch });
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(call) else {
        panic!("delegation: {:?}", driver.answered(call));
    };
    children.iter().map(|child| child.task).collect()
}

fn task_read(driver: &mut Driver, run: &engine::Assignment, call: u64, task: u64) -> String {
    let input = format!("{{\"task\":{task}}}");
    let before = driver.store.applied;
    let answer = decoded_task_call(driver, run, call, b"read_task", input.as_bytes());
    assert_eq!(driver.store.applied, before, "task reads retain no call record");
    let rendered = temper_engine_smith::answer_bounded(&answer, 4096);
    assert!(!rendered.error(), "{}", String::from_utf8_lossy(rendered.text()));
    String::from_utf8(rendered.text().to_vec()).expect("task text")
}

fn cancel_prerequisite(driver: &mut Driver, run: &engine::Assignment, prerequisite: u64) -> engine::Assignment {
    let child = super::added_deps::assignment(driver, prerequisite);
    for assignment in [&child, run] {
        driver.answer(Token::new(7), assignment.task, assignment.attempt, 0, tasks::End::Parked, engine::Parts::none());
        driver.settle();
    }
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(1002)),
        sign_in: driver.session(),
        key: [32; 16],
        ask: jig_domain_people::Ask::Cancel {
            project: 1,
            task: prerequisite,
            reason: b"remake S".as_slice().into(),
            grace: None,
        },
    }));
    driver.settle();
    let cancelled = super::added_deps::assignment(driver, prerequisite);
    driver.answer(Token::new(7), cancelled.task, cancelled.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let resumed = super::added_deps::assignment(driver, run.task);
    assert_ne!(
        resumed.attempt,
        run.attempt,
        "dependency holds wake G; parent={:?}, source={:?}",
        driver.task(run.task).map(|r| &r.phase),
        driver.task(prerequisite).map(|r| &r.phase)
    );
    resumed
}

#[test]
fn coordinator_reads_held_and_cancelled_delegates_to_repair_dependencies() {
    let (mut driver, run) = coordinator();
    let prerequisite = coordinator_delegates(&mut driver, &run, 10, &[b"S: shared source specification"], None)[0];
    let designers = coordinator_delegates(&mut driver, &run, 11, &[b"designer A", b"designer B"], Some(prerequisite));
    let resumed = cancel_prerequisite(&mut driver, &run, prerequisite);
    let source = task_read(&mut driver, &resumed, 24, prerequisite);
    assert!(source.contains("State: cancelled"));
    let words = source.split_once("Words from 0:\n").expect("retained specification").1;
    assert_eq!(words, "S: shared source specification");
    let replacement = coordinator_delegates(&mut driver, &resumed, 25, &[words.as_bytes()], None)[0];
    for (index, task) in designers.iter().enumerate() {
        let call = 40 + u64::try_from(index).expect("two designers") * 2;
        repair_designer(&mut driver, &resumed, call, *task, prerequisite, replacement);
    }
    coordinator_refusals(&mut driver, &resumed, prerequisite);
    unrelated_task_read(&mut driver, &resumed, prerequisite);
}

fn repair_designer(
    driver: &mut Driver,
    resumed: &engine::Assignment,
    call: u64,
    task: u64,
    prerequisite: u64,
    replacement: u64,
) {
    use temper_engine_domain_world::fixtures;
    let held = task_read(driver, resumed, call, task);
    assert!(held.contains(&format!("Dependencies: {prerequisite}\n")));
    assert!(held.contains(&format!(": dependency {prerequisite}\n")));
    let revision = held
        .lines()
        .next()
        .expect("identity")
        .split_once(", spec revision ")
        .expect("revision")
        .1
        .parse()
        .expect("number");
    let notice = resumed
        .messages
        .iter()
        .find(|word| matches!(word.kind, tasks::MessageKind::Escalation { task: subject, .. } if subject == task))
        .expect("designer escalation delivered");
    let rendered = engine::inbox_words(resumed.task, notice);
    let rendered = String::from_utf8_lossy(&rendered);
    assert!(rendered.contains(&format!("Held task {task}, hold revision ")));
    assert!(rendered.contains("read_task"));
    let mut amendment = fixtures::task_amendment(None, None, b"use replacement S");
    amendment.dependencies = Some(Box::new([replacement]));
    fixtures::tool_call(driver, resumed, call + 1, engine::Tool::Amend { target: task, revision, amendment });
    assert!(matches!(driver.answered(call + 1), Some(engine::CallAnswer::Amended { .. })));
    assert_eq!(driver.task(task).expect("live designer").dependencies.as_ref(), &[replacement]);
    let fresh = task_read(driver, resumed, call, task);
    assert!(fresh.contains(&format!("Dependencies: {replacement}\n")), "a repeated read sees the amendment");
}

fn coordinator_refusals(driver: &mut Driver, run: &engine::Assignment, ended: u64) {
    let input = format!("{{\"target\":{ended},\"form\":\"words\",\"words\":\"read your words\"}}");
    assert!(matches!(
        decoded_task_call(driver, run, 47, b"message", input.as_bytes()),
        engine::CallAnswer::ToolDenied { .. }
    ));
    let input = format!("{{\"kind\":\"task\",\"target\":{ended}}}");
    assert!(matches!(
        decoded_task_call(driver, run, 48, b"subscribe", input.as_bytes()),
        engine::CallAnswer::ToolDenied { .. }
    ));
}

fn unrelated_task_read(driver: &mut Driver, run: &engine::Assignment, ended: u64) {
    let ask = driver.chat_ask(1, b"unrelated task words");
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(1003)),
        sign_in: driver.session(),
        key: [33; 16],
        ask,
    }));
    driver.settle();
    let other = driver.last_assignment();
    assert_ne!(other.task, run.task);
    let input = format!("{{\"task\":{}}}", other.task);
    let answer = decoded_task_call(driver, run, 49, b"read_task", input.as_bytes());
    let rendered = temper_engine_smith::answer_bounded(&answer, 4096);
    assert!(rendered.error());
    assert!(!String::from_utf8_lossy(rendered.text()).contains("unrelated task words"));
    let input = format!("{{\"task\":{ended}}}");
    let answer = decoded_task_call(driver, &other, 50, b"read_task", input.as_bytes());
    assert!(temper_engine_smith::answer_bounded(&answer, 4096).error(), "retention grants no unrelated reader");
}

#[test]
fn task_reads_do_not_settle_over_a_mutation_call() {
    let (mut driver, run) = coordinator();
    let child = coordinator_delegates(&mut driver, &run, 10, &[b"retained delegate specification"], None)[0];
    driver.protocol.defer_render = true;
    let input = format!("{{\"task\":{child}}}");
    let answer = decoded_task_call(&mut driver, &run, 10, b"read_task", input.as_bytes());
    let rendered = temper_engine_smith::answer_bounded(&answer, 4096);
    let (body, error) = rendered.into_parts();
    assert!(!error);
    let (to, name) = driver.protocol.pending_renders.pop().expect("fresh task read renders");
    let before = driver.store.applied;
    driver.send(engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
        to,
        answer: jig_domain_core::SettledAnswer::Host { error, body: body.clone() },
    }));
    driver.pump();
    assert_eq!(driver.store.applied, before, "a read cannot replace the mutation's retained answer");
    let reply = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Link { down: host::LinkDown::Relayed { call, answer, .. }, .. } = delivery
                && *call == name
            {
                Some(answer)
            } else {
                None
            }
        })
        .expect("task read delivered");
    assert_eq!(reply, &host::Relayed::Answer { error: false, text: body });
}
