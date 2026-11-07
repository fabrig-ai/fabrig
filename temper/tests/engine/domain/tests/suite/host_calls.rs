//! Calls answered at decode, as errors or successes, leave no commit and
//! no record (domain/root.md, 5.2; protocol/documentation.md, 5).
use jig_domain_host as host;
use jig_domain_link::EndpointBound;
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
    coordinator_within(super::added_deps::limits())
}

#[expect(clippy::large_types_passed_by_value, reason = "the fixture takes Copy limits for one configured coordinator")]
fn coordinator_within(bounds: engine::Limits) -> (Driver, engine::Assignment) {
    use temper_engine_domain_world::{protocol, store::Store};
    let mut driver = Driver::configured(
        Store::with_limits(&bounds),
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
    fixtures::tool_call(
        driver,
        resumed,
        call + 1,
        engine::Tool::Amend { keep_parameters: false, target: task, revision, amendment },
    );
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

// Designer changes and the charter-7 close-out share delegation admission
// (temper/docs/design/domain/agent.md, 4.3.1).
#[test]
fn a_change_authority_refusal_names_producer_and_settings() {
    let (mut world, chat) = producer_refusal_world();
    for (number, input, member) in [
        (71, br#"{"words":"design the next cut","producer":7}"#.as_slice(), "member 1"),
        (72, br#"{"words":"close out the plan","producer":7}"#.as_slice(), "member 1"),
        (73, br#"{"changes":[{"words":"code","producer":2},{"words":"design","producer":7}]}"#.as_slice(), "member 2"),
    ] {
        let text = producer_refusal(&mut world, &chat, number, input);
        assert!(text.contains("charter 7"), "producer missing: {text}");
        assert!(text.contains(member), "batch member missing: {text}");
        assert!(text.contains("executor permission"), "dimension missing: {text}");
        for setting in [
            "task authority.delegation.kinds",
            "project policy.ceiling.delegation.kinds",
            "rules.ceiling.delegation.kinds",
        ] {
            assert!(text.contains(setting), "setting {setting} missing: {text}");
        }
        let before = world.store.applied;
        let replay = producer_refusal(&mut world, &chat, number, input);
        assert_eq!(replay, text, "settled refusal replays its words");
        assert_eq!(world.store.applied, before, "replay makes no decision");
    }
}

fn producer_refusal_world() -> (temper_engine_domain_world::forge_routes::World, engine::Assignment) {
    use temper_engine_domain_world::{
        forge_routes::{Until, World},
        protocol,
    };
    let model = crate::stock::box_model();
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        providers: Box::new([]),
        models: (Box::new([model.clone()]))
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),
    };
    let mut defaults = temper_service::stock::with_models(llm.models.clone(), 384);
    let mut designer = defaults.charters[1].clone();
    designer.number = 7;
    defaults.charters = [defaults.charters.as_ref(), &[designer]].concat().into();
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    limits.tasks.batch = 2;
    let mut world = World::configured_stock_limits(&defaults, &model, &limits);
    world.adopt();
    let mut hello = protocol::hello_slots(Token::new(7), 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Hello { hello: ref mut worker, .. },
        ..
    }) = hello
    else {
        panic!("hello")
    };
    worker.bounds = jig_test_profile::fixtures::bounds();
    worker.bounds.time = skein_lib::Duration::from_secs(28_800);
    worker.bounds.endpoints =
        Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    world.send(hello);
    let ask = world.chat(1, b"coordinate designers and close out the plan");
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(1001)),
        sign_in: world.signed_in.expect("owner"),
        key: [31; 16],
        ask,
    }));
    world.until(Until::Assigned);
    let chat = world.assigned.last().expect("chat authorized for plan fields").clone();
    assert_ne!(chat.run.authority.tools.0 & jig_domain_core::ToolFamilies::JIG.control.0, 0);
    world.protocol.defer_render = true;
    (world, chat)
}

fn producer_refusal(
    world: &mut temper_engine_domain_world::forge_routes::World,
    chat: &engine::Assignment,
    number: u64,
    input: &[u8],
) -> String {
    let call = person_tool(chat, u32::try_from(number).expect("call"), b"change", input);
    settled_host_refusal(world, chat, number, b"change", call)
}

#[test]
fn task_report_selectors_render_complete_retained_continuations() {
    const REPORT: &[u8] = b"Left: document retained reports; disposition: accepted.";
    const PAGE_BYTES: u32 = 65_536; // walking's read room holds three result texts, an inbox and their carrier.
    // Both continuation rows measured 44 ms CPU together.
    // The larger report exceeds 3,584 bytes and this world's message room.
    for (report_bytes, answer_bytes) in [(REPORT.len(), 100), (3_585, 512)] {
        let mut expected = REPORT.to_vec();
        expected.resize(report_bytes, b'r');
        let mut bounds = super::added_deps::limits();
        bounds.tasks.result_bytes = bounds.tasks.result_bytes.max(u32::try_from(report_bytes).expect("bounded report"));
        bounds.tasks.inbox_bytes = bounds.tasks.inbox_bytes.max(bounds.tasks.result_bytes);
        bounds.forge.client.op_bytes = bounds.forge.client.op_bytes.max(bounds.tasks.result_bytes);
        // The retained result and its inbox copy fit the paged task row at restart and read.
        bounds.journal.result_bytes = bounds.journal.result_bytes.max(bounds.tasks.result_bytes);
        bounds.journal.transcript_bytes = PAGE_BYTES;
        bounds.loads.transcript_bytes = PAGE_BYTES;
        bounds.loads.bytes = PAGE_BYTES;
        bounds.loads.reply_bytes = PAGE_BYTES;
        let (mut driver, run) = coordinator_within(bounds);
        let mut member =
            temper_engine_domain_world::fixtures::report_delegate(b"specification is not a report", Box::new([]));
        member.authority.tools = tasks::Tools(519);
        member.contract = tasks::Contract::Report { words: bounds.tasks.result_bytes };
        temper_engine_domain_world::fixtures::tool_call(
            &mut driver,
            &run,
            10,
            engine::Tool::Delegate { batch: Box::new([member]) },
        );
        let Some(engine::CallAnswer::Delegated(children)) = driver.answered(10) else { panic!("report delegate") };
        let child = children[0].task;
        let producer = super::added_deps::assignment(&driver, child);
        driver.answer(
            Token::new(7),
            child,
            producer.attempt,
            0,
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: expected.clone().into() },
                cancel_delegates: false,
            },
            engine::Parts::none(),
        );
        driver.settle();
        let applied = driver.store.applied;
        let mut report = String::new();
        let mut from = 0;
        // Each continuation advances at least one byte.
        for call in 20..20 + u64::try_from(report_bytes).expect("bounded report") {
            let input = format!("{{\"task\":{child},\"field\":\"result\",\"from\":{from}}}");
            let answer = decoded_task_call(&mut driver, &run, call, b"read_task", input.as_bytes());
            let engine::CallAnswer::TaskRead(read) = &answer else { panic!("task read") };
            assert_eq!(
                read.as_ref().as_ref().expect("retained report").words.field,
                jig_domain_core::reads::TaskTextField::Result
            );
            let rendered = temper_engine_smith::answer_bounded(&answer, answer_bytes);
            assert!(!rendered.error(), "{}", String::from_utf8_lossy(rendered.text()));
            assert!(rendered.text().len() <= usize::try_from(answer_bytes).expect("answer room"));
            let text = String::from_utf8(rendered.text().to_vec()).expect("report text");
            let (_, body) = text.split_once(&format!("Result from {from}:\n")).expect("selected report label");
            if let Some((words, next)) = body.rsplit_once("\nnext: ") {
                report.push_str(words);
                let next = next.parse().expect("byte offset");
                assert!(next > from);
                from = next;
            } else {
                report.push_str(body);
                break;
            }
        }
        assert_eq!(report.as_bytes(), expected, "complete report rather than specification words");
        assert_eq!(driver.store.applied, applied, "reading records no calls");
        assert!(from > 0, "host answer bound forced a continuation");
    }
}

// temper/docs/design/draft/reference-code-production.md, 8;
// temper/docs/design/protocol/configuration.md, 13.9; jig/docs/design/domain/people.md, 8.
type PersonWorld = temper_engine_domain_world::forge_routes::World;

#[test]
fn reference_goal_person_question_round_trip() {
    for missing in ["rules", "project", "creator", "none", "existing"] {
        let (mut world, goal) = reference_person_world(missing);
        if missing == "none" || missing == "existing" {
            person_question_round_trip(&mut world, &goal);
        } else {
            person_delegation_refusal(&mut world, &goal, missing);
        }
        world.close();
    }
}

fn reference_person_settings() -> Vec<jig_domain_core::settings::Item> {
    use jig_client::contract as c;
    let limits = temper_client_tools::limits::tools();
    let mut file: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../../docs/examples/reference/settings.json"))
            .expect("reference settings");
    let entries = file["items"].as_array_mut().expect("items");
    entries.retain(|item| item["setting"]["kind"] != "landing-rule");
    for item in entries {
        if item["setting"]["kind"] == "template" {
            item["set"]["llm"] = serde_json::Value::Null;
            item["set"]["contract"]["report"]["words"] =
                temper_engine_domain_world::forge_routes::box_limits().tasks.result_bytes.into();
        }
    }
    let items =
        temper_client_tools::jig::settings_file::decode(&serde_json::to_vec(&file).expect("settings JSON"), &limits)
            .expect("reference settings decode");
    let ask = c::Ask::ChangeSettings(c::ChangeSettings { against: 1, items, running: c::Running::Stays });
    let jig_client_protocol_wire::Ask::ChangeSettings(change) =
        jig_client_protocol_link::translate::asks::ask(&ask, &limits.settings).expect("settings wire")
    else {
        panic!("settings change")
    };
    jig_protocol_engine_clients::translate::settings::items(change.items().into(), &[], &limits.settings)
        .expect("reference core settings")
        .into_vec()
}

fn reference_person_defaults() -> jig_domain_core::settings::Defaults {
    use jig_domain_core::settings::{Edit, Value};
    let model = crate::stock::box_model();
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        providers: Box::new([]),
        models: (Box::new([model.clone()]))
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),
    };
    let mut defaults = temper_service::stock::with_models(llm.models.clone(), 10_660);
    let mut charters = defaults.charters.to_vec();
    for item in reference_person_settings() {
        match item.edit {
            Edit::Set(Value::Charter(mut charter)) => {
                // Scripted turns use the fake worker's model and no checkout document reads.
                charter.policy.model = model.clone();
                charter.spend = defaults.charters[0].spend;
                if charter.policy.waiting == skein_lib::Duration::ZERO {
                    charter.policy.waiting = defaults.charters[0].policy.waiting;
                }
                charter.documents = Box::new([]);
                charters.retain(|old| old.number != charter.number);
                charters.push(*charter);
            }
            Edit::Set(Value::Template(template)) => {
                defaults.templates = [defaults.templates.as_ref(), &[*template]].concat().into();
            }
            other => panic!("reference task settings: {other:?}"),
        }
    }
    defaults.charters = charters.into();
    reference_person_authority(&mut defaults.rules.ceiling);
    let policy = defaults.policies.get_mut(&1).expect("policy 1");
    reference_person_authority(&mut policy.policy.ceiling);
    reference_person_authority(&mut policy.policy.roles[0].authority);
    defaults
}

fn reference_person_authority(authority: &mut jig_domain_authority::Authority) {
    use jig_domain_authority::Executor;
    let recipe = include_str!("../../../../../docs/examples/reference/README.md")
        .split_once("cat > reference-authority.json <<'JSON'\n")
        .expect("recipe")
        .1
        .split_once("\nJSON")
        .expect("recipe end")
        .0;
    let required: serde_json::Value = serde_json::from_str(recipe).expect("authority recipe");
    for bit in required["tools"].as_array().expect("tools") {
        authority.tools.0 |= bit.as_u64().expect("tool bit");
    }
    for bit in required["notes"].as_array().expect("notes") {
        authority.notes.0 |= u8::try_from(bit.as_u64().expect("note bit")).expect("scope");
    }
    let mut kinds = authority.delegation.kinds.to_vec();
    for kind in required["kinds"].as_array().expect("kinds") {
        let (name, number) = kind.as_object().expect("executor").iter().next().expect("executor name");
        let number = u32::try_from(number.as_u64().expect("number")).expect("executor number");
        let executor = match name.as_str() {
            "charter" => Executor::Charter(number),
            "job" => Executor::Job(number),
            "procedure" => Executor::Procedure(number),
            "role" => Executor::Role(number),
            other => panic!("recipe executor {other}"),
        };
        if !kinds.contains(&executor) {
            kinds.push(executor);
        }
    }
    authority.delegation.kinds = kinds.into();
    assert!(authority.delegation.kinds.contains(&Executor::Role(0)), "recipe permits asking the person");
}

fn without_person(authority: &mut jig_domain_authority::Authority) {
    authority.delegation.kinds = authority
        .delegation
        .kinds
        .iter()
        .filter(|kind| **kind != jig_domain_authority::Executor::Role(0))
        .copied()
        .collect();
}

fn reference_missing_role(defaults: &mut jig_domain_core::settings::Defaults, missing: &str) {
    match missing {
        "rules" | "project" | "existing" => {
            if missing == "rules" {
                without_person(&mut defaults.rules.ceiling);
            }
            let policy = &mut defaults.policies.get_mut(&1).expect("policy").policy;
            without_person(&mut policy.ceiling);
            without_person(&mut policy.roles[0].authority);
        }
        "creator" => without_person(&mut defaults.policies.get_mut(&1).expect("policy").policy.roles[0].authority),
        "none" => {}
        other => panic!("row {other}"),
    }
}

fn reference_person_world(missing: &str) -> (PersonWorld, engine::Assignment) {
    use temper_engine_domain_world::{forge_routes::Until, protocol};
    let mut defaults = reference_person_defaults();
    let configured = defaults.policies.get(&1).expect("policy").clone();
    reference_missing_role(&mut defaults, missing);
    // The reference coordinator instruction text plus its assignment carriers and 25% headroom.
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    limits.journal.run_bytes = 26_000;
    let mut world = PersonWorld::configured_stock_limits(&defaults, &crate::stock::box_model(), &limits);
    world.adopt();
    if missing == "existing" {
        reference_existing_policy(&mut world, configured);
    }
    let mut event = protocol::hello_slots(Token::new(7), 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.bounds = jig_test_profile::fixtures::bounds();
    hello.bounds.time = skein_lib::Duration::from_secs(28_800);
    let model = crate::stock::box_model();
    hello.bounds.endpoints =
        Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    world.send(event);
    let jig_domain_people::Ask::Create { project, mut batch } = world.chat(1, b"Coordinate a person-only choice")
    else {
        panic!("goal")
    };
    let member = &mut batch[0];
    member.executor = jig_domain_people::TaskExecutor::Agent { charter: 5 };
    member.wake = None;
    if missing != "none" && missing != "existing" {
        member.authority.tools &= !jig_domain_core::ToolFamilies::JIG.propose.0;
    }
    member.contract = jig_domain_people::Contract::Report { words: 384 };
    member.flags = jig_domain_people::Flags { listed: true, ..jig_domain_people::Flags::PLAIN };
    member.tracked = Some(1);
    member.authority.deadline = Some(skein_lib::Wall::from_nanos(1_000_000_000_000_000));
    member.deadline = jig_domain_people::Deadline::Named;
    person_ask(&mut world, 201, jig_domain_people::Ask::Create { project, batch });
    assert!(
        world.web_replies.iter().any(|reply| matches!(
            reply,
            jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Created { .. })
        )),
        "goal creation: {:?}",
        world.web_replies
    );
    world.until(Until::Assigned);
    let goal = world.assigned.last().expect("coordinator assigned").clone();
    assert_eq!(goal.charter, 5);
    (world, goal)
}

fn reference_existing_policy(world: &mut PersonWorld, policy: jig_domain_core::ConfiguredPolicy) {
    use jig_domain_core::settings as s;
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: skein_lib::ReplyTo::new(Token::new(300)),
        sign_in: world.signed_in.expect("owner"),
        key: [30; 16],
        digest: [30; 32],
        kinds: Box::new([]),
        change: Box::new(s::Change {
            against: 1,
            running: s::Running::Stays,
            items: Box::new([s::Item {
                setting: s::Setting::ProjectPolicy { project: 1 },
                edit: s::Edit::Set(s::Value::Policy(Box::new(policy))),
            }]),
        }),
    }));
    world.until(temper_engine_domain_world::forge_routes::Until::Ready);
    assert!(
        world.web_replies.iter().any(|reply| matches!(
            reply,
            jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::SettingsChanged { revision: 2 })
        )),
        "existing project updated: {:?}",
        world.web_replies
    );
}

fn person_ask(world: &mut PersonWorld, number: u8, ask: jig_domain_people::Ask) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(u64::from(number))),
        sign_in: world.signed_in.expect("owner"),
        key: [number; 16],
        ask,
    }));
    world.until(temper_engine_domain_world::forge_routes::Until::Ready);
}

fn person_tool(run: &engine::Assignment, number: u32, tool: &[u8], input: &[u8]) -> engine::Call {
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded host input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: run.attempt, completion: 1, position: number },
        tool,
        &input,
    )
    .expect("decoded host call") else {
        panic!("engine call")
    };
    call
}

fn person_delegate_input() -> &'static [u8] {
    br#"{"batch":[{"executor":{"kind":"role","role":0},"spec":{"words":"Choose 1: keep the design, or 2: widen it. Recommend 1."},"contract":{"kind":"verdict","choices":[{"code":1,"words":128},{"code":2,"words":128}]},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":0},"notes":0}}]}"#
}

fn person_delegation_refusal(world: &mut PersonWorld, goal: &engine::Assignment, missing: &str) {
    let call = person_tool(goal, 71, b"delegate", person_delegate_input());
    world.protocol.defer_render = true;
    let text = settled_host_refusal(world, goal, 71, b"delegate", call.clone());
    assert!(text.len() <= usize::try_from(world.env.limits.call_answer_bytes).expect("answer bound"));
    assert!(text.contains("role 0"), "blocked person role: {text}");
    assert!(text.contains(r#"{"role":0}"#), "settings executor form: {text}");
    let setting = match missing {
        "rules" => "rules.ceiling.delegation.kinds",
        "project" => "project policy.ceiling.delegation.kinds",
        "creator" => "task authority.delegation.kinds",
        other => panic!("negative row {other}"),
    };
    assert!(text.contains(setting), "deficient {setting}: {text}");
    assert!(!text.contains("For a change to a repository"), "person needs no repository advice: {text}");
    let before = world.store.applied;
    assert_eq!(settled_host_refusal(world, goal, 71, b"delegate", call), text, "settled refusal replays");
    assert_eq!(world.store.applied, before);
    assert_eq!(person_needs(world), 0, "refused delegation creates no inbox question");
    person_fallback_refusals(world, goal);
}

fn settled_host_refusal(
    world: &mut PersonWorld,
    run: &engine::Assignment,
    number: u64,
    tool: &[u8],
    call: engine::Call,
) -> String {
    let before = world.call_errors.len();
    let event = world.protocol.call(Token::new(7), run.task, run.attempt, number, call);
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if !world.protocol.pending_renders.is_empty() || world.call_errors.len() > before {
            break;
        }
    }
    if let Some((to, _)) = world.protocol.pending_renders.pop() {
        let answer = &world.protocol.answers.last().expect("refusal").1;
        let (body, error) = temper_engine_smith::answer_for_tool(answer, tool, 4096).into_parts();
        assert!(error, "call is refused: {answer:?}");
        world.send(engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
            to,
            answer: jig_domain_core::SettledAnswer::Host { error, body },
        }));
        world.until(temper_engine_domain_world::forge_routes::Until::Ready);
    }
    assert_eq!(world.call_errors.len(), before + 1);
    String::from_utf8(world.call_errors.last().expect("worker feedback").text.to_vec()).expect("words")
}

fn person_read(world: &mut PersonWorld, read: jig_domain_core::reads::Read) -> jig_domain_core::reads::Answer {
    let before = world.reads.len();
    world.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: skein_lib::ReplyTo::new(Token::new(501)),
        sign_in: world.signed_in.expect("owner"),
        read,
    }));
    for _ in 0..200 {
        world.tick();
        if world.reads.len() > before {
            return world.reads.last().expect("read").1.clone();
        }
    }
    panic!("person read answered")
}

fn person_needs(world: &mut PersonWorld) -> u32 {
    let jig_domain_core::reads::Answer::Document(doc) = person_read(world, jig_domain_core::reads::Read::Frame) else {
        panic!("frame")
    };
    let jig_domain_core::documents::Document::Frame(frame) = *doc else { panic!("frame document") };
    frame.needs
}

fn person_question_round_trip(world: &mut PersonWorld, goal: &engine::Assignment) {
    let (producer, change, question) = reference_person_escalation(world, goal);
    let call = person_tool(goal, 72, b"delegate", person_delegate_input());
    let answer = world.scripted_call(goal, 72, call.tool);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("person delegated: {answer:?}") };
    let person = made[0].task;
    assert!(person_needs(world) > 0);
    assert_person_inbox(world, person);
    let jig_domain_core::reads::Answer::Document(doc) =
        person_read(world, jig_domain_core::reads::Read::PersonTask { task: person })
    else {
        panic!("person task")
    };
    let jig_domain_core::documents::Document::PersonTask(person_form) = *doc else { panic!("person document") };
    assert_eq!(person_form.spec.words.bytes.as_ref(), b"Choose 1: keep the design, or 2: widen it. Recommend 1.");
    let jig_domain_core::documents::ContractLine::Verdict { choices, cut: false } = person_form.contract else {
        panic!("choices")
    };
    assert_eq!(choices.iter().map(|choice| choice.code).collect::<Vec<_>>(), [1, 2]);
    person_park(world, goal);
    person_ask(world, 202, jig_domain_people::Ask::TakePerson { project: 1, task: person });
    person_ask(
        world,
        203,
        jig_domain_people::Ask::AnswerPerson {
            project: 1,
            task: person,
            revision: person_form.revision,
            result: jig_domain_people::PersonResult::Verdict { code: 1, words: b"Keep the design.".as_slice().into() },
        },
    );
    assert_eq!(person_needs(world), 0);
    reference_person_answer(world, goal, &producer, change, question, person);
}

fn reference_person_escalation(world: &mut PersonWorld, goal: &engine::Assignment) -> (engine::Assignment, u64, u64) {
    let call = person_tool(goal, 70, b"change", br#"{"words":"Resolve the person-only choice","questions":"foreman"}"#);
    let engine::CallAnswer::Delegated(made) = world.scripted_call(goal, 70, call.tool) else {
        panic!("change created")
    };
    let change = made[0].task;
    let producer = person_assignment(world, 2, None);
    let input = format!(
        r#"{{"target":{change},"form":"question","words":"Choose 1: keep the design, or 2: widen it. Recommend 1."}}"#
    );
    let call = person_tool(&producer, 71, b"message", input.as_bytes());
    let engine::CallAnswer::Sent { message } = world.scripted_call(&producer, 71, call.tool) else {
        panic!("producer asks")
    };
    person_park(world, &producer);
    let foreman = person_assignment(world, 6, None);
    world.foreman_verdict(&foreman, 3, b"Only the person can choose.");
    for _ in 0..200 {
        world.tick();
        if let Some(question) = world
            .root
            .tasks()
            .task(goal.task)
            .expect("goal")
            .inbox
            .iter()
            .find(|word| word.from == tasks::Party::Task(change) && word.kind == tasks::MessageKind::Question)
            && world.store.pending.is_empty()
        {
            assert!(String::from_utf8_lossy(&question.words).contains("Only the person can choose."));
            assert!(
                world.root.tasks().task(producer.task).expect("producer").questions.iter().any(|q| q.number == message)
            );
            return (producer, change, question.number);
        }
    }
    panic!("foreman escalates to coordinator: {:?}", world.root.tasks().task(goal.task).expect("goal").inbox);
}

fn person_assignment(world: &mut PersonWorld, charter: u32, after: Option<&engine::Assignment>) -> engine::Assignment {
    for _ in 0..200 {
        world.tick();
        if let Some(run) = world.assigned.iter().rev().find(|run| {
            run.charter == charter && after.is_none_or(|old| old.task == run.task && old.attempt < run.attempt)
        }) && world.store.pending.is_empty()
        {
            return run.clone();
        }
    }
    let row = after.and_then(|run| world.root.tasks().task(run.task));
    panic!(
        "charter {charter} assigned; phase/numbers {:?}; runs {:?}; replies {:?}",
        row.map(|row| (&row.phase, &row.numbers, &row.inbox)),
        world.assigned.iter().map(|run| (run.task, run.attempt, run.charter)).collect::<Vec<_>>(),
        world.web_replies
    );
}

fn person_park(world: &mut PersonWorld, run: &engine::Assignment) {
    world.scripted_read_inbox(run);
    let turns = world.root.tasks().task(run.task).expect("live run").turn;
    let mut event =
        world.protocol.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        panic!("wait ends run")
    };
    answer.turns = turns;
    world.send(event);
    world.until(temper_engine_domain_world::forge_routes::Until::Ready);
}

fn assert_person_inbox(world: &mut PersonWorld, person: u64) {
    use jig_domain_core::{documents, reads};
    let reads::Answer::Page(page) = person_read(
        world,
        reads::Read::Inbox {
            group: reads::Group::PersonTasks,
            filter: reads::Filter { project: Some(1), goal: None },
            from: None,
        },
    ) else {
        panic!("owner inbox")
    };
    assert!(
        page.rows.iter().any(|row| matches!(row, reads::PageRow::Inbox(entry)
        if matches!(&entry.task, documents::TaskLink::Live(chip) if chip.number == person)
            && entry.object == documents::InboxObject::PersonTask)),
        "question appears in owner's inbox"
    );
}

fn reference_person_answer(
    world: &mut PersonWorld,
    goal: &engine::Assignment,
    producer: &engine::Assignment,
    change: u64,
    question: u64,
    person: u64,
) {
    let resumed = person_assignment(world, 5, Some(goal));
    assert!(
        resumed.messages.iter().any(|word| word.from == tasks::Party::Task(person)
            && matches!(word.kind, tasks::MessageKind::Result(_))
            && String::from_utf8_lossy(&word.words).contains("Keep the design.")),
        "person result wakes coordinator"
    );
    let input = format!(r#"{{"target":{change},"form":"answer","question":{question},"words":"Keep the design."}}"#);
    let call = person_tool(&resumed, 73, b"message", input.as_bytes());
    assert!(matches!(world.scripted_call(&resumed, 73, call.tool), engine::CallAnswer::Sent { .. }));
    let resumed_producer = person_assignment(world, 2, Some(producer));
    assert!(
        resumed_producer.messages.iter().any(|word| word.from == tasks::Party::Task(change)
            && matches!(word.kind, tasks::MessageKind::Answer { .. })
            && word.words.as_ref() == b"Keep the design."),
        "change relays the person's answer to producer"
    );
    assert!(world.root.tasks().task(producer.task).expect("producer").questions.is_empty());
}

fn person_fallback_refusals(world: &mut PersonWorld, goal: &engine::Assignment) {
    let mut input: serde_json::Value = serde_json::from_slice(person_delegate_input()).expect("delegate input");
    input["batch"][0]["executor"] = serde_json::json!({"kind":"agent","charter":2});
    input["action"] = "batch".into();
    input["reason"] = "Ask the person".into();
    let input = serde_json::to_vec(&input).expect("proposal input");
    let call = person_tool(goal, 74, b"propose", &input);
    let text = settled_host_refusal(world, goal, 74, b"propose", call);
    assert!(text.contains("tool family propose"), "proposal family refusal: {text}");
    let tasks::Party::Person(requester) = world.root.tasks().task(goal.task).expect("goal").requester else {
        panic!("person requester")
    };
    let input = format!(r#"{{"target":{requester},"form":"question","words":"Keep or widen?"}}"#);
    let call = person_tool(goal, 75, b"message", input.as_bytes());
    let text = settled_host_refusal(world, goal, 75, b"message", call);
    assert!(text.contains("task reference"), "requester person is not a task reference: {text}");
    assert_eq!(person_needs(world), 0, "neither refused fallback creates a person task");
}

// temper/docs/design/domain/root.md, 5.2.
const WINDOW: u32 = 512;
const REPLY: u32 = 2048;

fn read_room_bounds(reply: Option<u32>) -> engine::Limits {
    let mut bounds = super::added_deps::limits();
    bounds.documents.text_bytes = WINDOW;
    bounds.host_reply_bytes = reply;
    bounds.tasks.spec_bytes = 5000;
    bounds.journal.transcript_bytes = 32_768;
    bounds.brief.budgets.task = 6000;
    bounds.brief.brief_bytes = 7000;
    bounds.journal.result_bytes = 7000;
    bounds
}

fn read_whole(driver: &mut Driver, run: &engine::Assignment, child: u64) -> (Vec<u8>, u32) {
    let mut words = Vec::new();
    let mut calls = 0;
    loop {
        let from = words.len();
        let input = format!("{{\"task\":{child},\"from\":{from}}}");
        let answer = decoded_task_call(driver, run, 100 + u64::from(calls), b"read_task", input.as_bytes());
        let engine::CallAnswer::TaskRead(result) = &answer else { panic!("task read") };
        let read = result.as_ref().as_ref().expect("successful read through the journal door");
        let counted = u64::try_from(size_of_val(result.as_ref())).expect("task-read carrier fits u64")
            + jig_domain_core::reads::run_task::owned_bytes(read);
        assert!(counted <= u64::from(driver.env.limits.journal.transcript_bytes));
        let rendered = temper_engine_smith::answer_bounded(&answer, REPLY);
        assert!(!rendered.error(), "{}", String::from_utf8_lossy(rendered.text()));
        assert!(rendered.text().len() <= usize::try_from(REPLY).expect("reply fits usize"));
        let rendered = std::str::from_utf8(rendered.text()).expect("rendered UTF-8");
        let label = format!("Words from {from}:\n");
        let body = rendered.split_once(&label).expect("words offset").1;
        calls += 1;
        if let Some((piece, next)) = body.rsplit_once("\nnext: ") {
            words.extend_from_slice(piece.as_bytes());
            assert_eq!(next.parse::<usize>().expect("next byte offset"), words.len(), "follow the printed next");
        } else {
            words.extend_from_slice(body.as_bytes());
            return (words, calls);
        }
        assert!(calls < 20, "read makes progress");
    }
}

#[test]
fn a_run_reads_a_spec_past_the_clients_window_in_one_call() {
    let (mut driver, run) = coordinator_within(read_room_bounds(Some(REPLY)));
    let spec = vec![b's'; 1500];
    let child = coordinator_delegates(&mut driver, &run, 10, &[&spec], None)[0];
    let (words, calls) = read_whole(&mut driver, &run, child);
    assert_eq!(words, spec);
    assert_eq!(calls, 1, "a spec within the reply comes back whole");
}

#[test]
fn a_spec_past_the_room_continues_by_offset_and_comes_back_whole() {
    let (mut driver, run) = coordinator_within(read_room_bounds(Some(REPLY)));
    let spec = b"the specification ".repeat(278);
    let spec = &spec[..5000];
    let child = coordinator_delegates(&mut driver, &run, 10, &[spec], None)[0];
    let (words, calls) = read_whole(&mut driver, &run, child);
    assert_eq!(words, spec);
    assert_eq!(calls, 3);
    let (mut driver, run) = coordinator_within(read_room_bounds(None));
    let child = coordinator_delegates(&mut driver, &run, 10, &[spec], None)[0];
    let (words, calls) = read_whole(&mut driver, &run, child);
    assert_eq!(words, spec);
    assert_eq!(calls, 10);
}

#[test]
fn without_a_local_agents_reply_a_read_stays_in_the_documents_window() {
    let (mut driver, run) = coordinator_within(read_room_bounds(None));
    let spec = vec![b's'; 1500];
    let child = coordinator_delegates(&mut driver, &run, 10, &[&spec], None)[0];
    let input = format!("{{\"task\":{child}}}");
    let answer = decoded_task_call(&mut driver, &run, 100, b"read_task", input.as_bytes());
    let engine::CallAnswer::TaskRead(result) = answer else { panic!("task read") };
    let read = result.expect("task read succeeds");
    assert_eq!(read.words.bytes.len(), usize::try_from(WINDOW).unwrap());
    assert_eq!(read.words.next, Some(u64::from(WINDOW)));
}

#[test]
fn a_read_stays_within_the_journal_bound() {
    let mut bounds = read_room_bounds(Some(REPLY));
    // The local reply exceeds the journal; each typed answer must fit the door.
    bounds.host_reply_bytes = Some(65_536);
    assert!(bounds.journal.transcript_bytes < bounds.host_reply_bytes.unwrap());
    let (mut driver, run) = coordinator_within(bounds);
    let spec = vec![b's'; 5000];
    let child = coordinator_delegates(&mut driver, &run, 10, &[&spec], None)[0];
    let (words, _) = read_whole(&mut driver, &run, child);
    assert_eq!(words, spec);
}
