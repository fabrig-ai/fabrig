//! Budget routing and advisory work in the queries supplied to Smith.
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use serde_json::json;
use skein_fake_llm_domain::api::{Part, Query, Script, Turn};
use skein_lib::Wall;
use smith_agent_world::World as Agent;
use temper_engine_domain as engine;
use temper_engine_domain_world::{budget as b, direct::Driver, fixtures};
use temper_system_world::{composed::call, world};

fn fixture() -> (Driver, engine::Assignment) {
    b::configured([world::config(965), world::config(965)])
}

fn tool(name: &[u8], args: &serde_json::Value) -> Turn {
    call(name, &serde_json::to_vec(args).expect("tool JSON"))
}

fn waiting(mut turns: Vec<Turn>) -> Script {
    turns.push(call(b"subscribe", br#"{"kind":"timer","at":18446744073709551615}"#));
    turns.push(call(b"wait", b"{}"));
    Script { cue: b"Complete the task within its authority.".as_slice().into(), turns: turns.into() }
}

fn finish() -> Script {
    Script {
        cue: b"Complete the task within its authority.".as_slice().into(),
        turns: Box::new([tool(b"finish", &json!({"report":"Done."}))]),
    }
}

fn run(root: &mut Driver, assignment: &engine::Assignment, script: Script) -> Agent {
    let mut agent = world::scripted_waiting_agent_for(assignment, None, script);
    world::run_assignment(root, assignment, &mut agent);
    assert!(
        matches!(agent.answer(), smith_domain_run::Answer::Parked { .. } | smith_domain_run::Answer::Accepted { .. }),
        "{:?}",
        agent.answer()
    );
    agent
}

fn park(root: &mut Driver, assignment: &engine::Assignment, call: u64) {
    fixtures::tool_call(
        root,
        assignment,
        call,
        engine::Tool::Subscribe {
            kind: tasks::SubscriptionKind::Timer { at: Wall::from_nanos(u64::MAX), period: None },
        },
    );
    b::end(root, assignment, tasks::End::Parked);
}

fn occurrences(query: &Query, words: &[u8]) -> usize {
    let count = |text: &[u8]| text.windows(words.len()).filter(|bytes| *bytes == words).count();
    count(&query.system)
        + query
            .messages
            .iter()
            .flat_map(|message| &message.parts)
            .map(|part| match part {
                Part::Text { text } => count(text),
                Part::ToolCall { .. } | Part::ToolOutput { .. } | Part::Opaque { .. } => 0,
            })
            .sum::<usize>()
}

fn each_query(agent: &Agent, words: &[u8], count: usize) {
    assert_ne!(agent.prompts(), []);
    for query in agent.prompts() {
        assert_eq!(occurrences(query, words), count, "{}", String::from_utf8_lossy(words));
    }
}

#[test]
fn budget_questions_and_answers_reach_each_owners_queries_once() {
    let (mut root, director) = fixture();
    let [a, a1] = b::owners(&mut root, &director);
    park(&mut root, &director, 110);
    park(&mut root, &a, 111);
    let question = b::message(&mut root, &a1, a.task, 112, engine::MessageForm::Question, b::QUESTION);
    park(&mut root, &a1, 113);
    let a = b::latest(&root, a.task);
    let agent = run(
        &mut root,
        &a,
        waiting(vec![tool(
            b"message",
            &json!({
                "target":director.task,"form":"question","words":String::from_utf8_lossy(b::UP)
            }),
        )]),
    );
    each_query(&agent, b::QUESTION, 1);
    let director = b::latest(&root, director.task);
    let up = root.task(a.task).expect("owner").questions[0].number;
    let agent = run(&mut root, &director, waiting(Vec::new()));
    each_query(&agent, b::UP, 1);
    b::ask_person(
        &mut root,
        32,
        people::Ask::AnswerQuestion { project: 1, task: director.task, question: up, words: b::ANSWER.into() },
    );
    let a = b::latest(&root, a.task);
    let agent = run(
        &mut root,
        &a,
        waiting(vec![tool(
            b"message",
            &json!({
                "target":a1.task,"form":"answer","question":question,"words":String::from_utf8_lossy(b::ANSWER)
            }),
        )]),
    );
    each_query(&agent, b::ANSWER, 1);
    let a1 = b::latest(&root, a1.task);
    let agent = run(&mut root, &a1, finish());
    each_query(&agent, b::ANSWER, 1);
    let a = b::latest(&root, a.task);
    let agent = run(&mut root, &a, finish());
    each_query(&agent, b::ANSWER, 0);
    let director = b::latest(&root, director.task);
    let agent = run(&mut root, &director, finish());
    // The chat is notified; it never relays the person's answer with a model tool.
    each_query(&agent, b::ANSWER, 1);
    b::healthy(&root);
}

#[test]
fn the_next_investigation_runs_with_its_amended_effort() {
    let (mut root, owner) = fixture();
    let mut member = b::delegate(b"Investigate the budget.", 10, 0);
    member.llm = Some(b::choice(0, 0));
    let first = b::make(&mut root, &owner, 101, member);
    park(&mut root, &owner, 110);
    let agent = run(
        &mut root,
        &first,
        waiting(vec![tool(
            b"message",
            &json!({
                "target":owner.task,"form":"words","words":String::from_utf8_lossy(b::FINDING)
            }),
        )]),
    );
    assert_eq!(first.run.policy.model.effort.as_deref(), Some(b"medium".as_slice()));
    assert_eq!(agent.turns()[0].effort.as_deref(), Some(b"medium".as_slice()));
    assert!(agent.prompts().iter().all(|query| query.model.as_ref() == b"fake-1"));
    let owner = b::latest(&root, owner.task);
    let agent = run(
        &mut root,
        &owner,
        waiting(vec![tool(
            b"amend",
            &json!({
                "target":first.task,"revision":0,"amendment":{
                    "llm":{"main":{"model":0,"effort":1}},"reason":String::from_utf8_lossy(b::REASON)
                }
            }),
        )]),
    );
    each_query(&agent, b::FINDING, 1);
    let next = b::latest(&root, first.task);
    assert!(next.attempt > first.attempt);
    let agent = run(&mut root, &next, finish());
    assert_eq!(next.run.policy.model.effort.as_deref(), Some(b"high".as_slice()));
    assert_eq!(agent.turns()[0].effort.as_deref(), Some(b"high".as_slice()));
    assert_eq!(agent.prompts().len(), 1);
    assert_eq!(agent.prompts()[0].model.as_ref(), b"fake-1");
    each_query(&agent, b::REASON, 1);
    let owner = b::latest(&root, owner.task);
    run(&mut root, &owner, finish());
    b::healthy(&root);
}

#[test]
fn one_advisor_reads_the_report_and_returns_its_cause_once() {
    let (mut root, owner) = fixture();
    let investigator = b::make(&mut root, &owner, 101, b::delegate(b"Investigate.", 10, 0));
    park(&mut root, &owner, 102);
    b::report(&mut root, &investigator, b::REPORT);
    let owner = b::latest(&root, owner.task);
    let agent = run(
        &mut root,
        &owner,
        waiting(vec![tool(
            b"delegate",
            &json!({"batch":[{
                "executor":{"kind":"agent","charter":1},"spec":{"words":String::from_utf8_lossy(b::REPORT),"parameters":[]},
                "contract":{"kind":"report","words":128},
                "authority":{"tools":0,"grants":[],"notes":0,"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":10}},
                "llm":{"main":{"model":1,"effort":0}}
            }]}),
        )]),
    );
    each_query(&agent, b::REPORT, 1);
    let advisor = root.last_assignment();
    assert_ne!(advisor.task, investigator.task);
    assert_eq!(advisor.run.policy.model.name.as_ref(), b"owner");
    let agent = run(
        &mut root,
        &advisor,
        Script {
            cue: b::REPORT.into(),
            turns: Box::new([tool(b"finish", &json!({"report":String::from_utf8_lossy(b::ADVICE)}))]),
        },
    );
    assert_eq!(agent.prompts().len(), 1);
    assert_eq!(agent.turns().len(), 1);
    assert_eq!(agent.prompts()[0].model.as_ref(), b"owner");
    each_query(&agent, b::REPORT, 1);
    let document = b::document(&mut root, advisor.task);
    assert_eq!(document.llm, Some(b::choice(1, 0)));
    assert_eq!(document.requester, tasks::Party::Task(owner.task));
    assert!(matches!(document.result, Some(jig_domain_core::documents::Ending::Done(
        jig_domain_core::documents::ResultValue::Report(ref text)
    )) if text.bytes.as_ref() == b::ADVICE));
    let owner = b::latest(&root, owner.task);
    let agent = run(&mut root, &owner, waiting(Vec::new()));
    each_query(&agent, b::ADVICE, 1);
    each_query(&agent, b::REPORT, 0);
    root.say(owner.task, 33);
    root.settle();
    let owner = b::latest(&root, owner.task);
    let agent = run(&mut root, &owner, finish());
    each_query(&agent, b::ADVICE, 0);
    b::healthy(&root);
}
