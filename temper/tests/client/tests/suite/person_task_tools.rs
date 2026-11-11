//! Revisioned ctl answers wake the real requesting goal (temper/docs/design/client/ctl/areas.md;
//! jig/docs/design/client/domain/contract/requests.md, 2).
use jig_domain_tasks as tasks;
use temper_client_tools::jig::{
    Printed,
    call::{Parsed, Value, Values},
};
use temper_client_world::{
    app::{Event, Root},
    fixtures,
    front::{Joint, account_acts},
};
use temper_engine_domain as engine;

const WORKER: skein_lib::Token = skein_lib::Token::new(7);
const BLUE: u32 = 1;
const RESULT_WORDS: u32 = 128;

struct Ctl {
    front: Joint,
    root: Root,
    number: u32,
}
impl Ctl {
    fn new(front: Joint, root: Root) -> Self {
        Self { front, root, number: 0 }
    }
    fn call(&mut self, words: &[&[u8]], file: Option<&[u8]>) -> Printed {
        let Parsed::Call { mut call, json } = temper_client_tools::table::parse(words).expect("ctl argv") else {
            panic!("typed call");
        };
        if let Some(bytes) = file {
            for value in &mut call.values {
                if matches!(value, Some(Values::One(Value::File(_)))) {
                    *value = Some(Values::One(Value::Text(bytes.into())));
                }
            }
        }
        self.number += 1;
        account_acts::event(
            &mut self.front,
            &mut self.root,
            Event::Call { number: self.number, call, json, value: None },
        );
        let printed = self.root.agent().printed(self.number).expect("ctl answered");
        Printed {
            status: printed.status,
            stdout: printed.stdout.clone(),
            stderr: printed.stderr.clone(),
            file: printed.file,
        }
    }
    fn text(&mut self, words: &[&[u8]]) -> String {
        let printed = self.call(words, None);
        assert_eq!(printed.status, 0, "call {words:?}: {printed:?}");
        String::from_utf8(printed.stdout.into_vec()).expect("ctl UTF-8")
    }
}
fn delegate(ctl: &mut Ctl, goal: u64, completion: u32) -> u64 {
    delegate_choices(ctl, goal, completion, &[BLUE])
}
fn delegate_choices(ctl: &mut Ctl, goal: u64, completion: u32, codes: &[u32]) -> u64 {
    let driver = &mut ctl.front.engine;
    let attempt = driver.task(goal).expect("running goal").attempt;
    let mut child = temper_engine_domain_world::fixtures::report_delegate(b"Choose a colour: blue", Box::new([]));
    child.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    child.authority.tools = tasks::Tools(0);
    child.authority.budget.spend = 0;
    child.contract = tasks::Contract::Verdict {
        choices: codes
            .iter()
            .map(|code| tasks::Verdict { code: *code, words: RESULT_WORDS, followups: 0, fields: Box::new([]) })
            .collect(),
    };
    driver.call(
        WORKER,
        goal,
        attempt,
        u64::from(completion),
        engine::Call { completion, position: 1, tool: engine::Tool::Delegate { batch: Box::new([child]) } },
    );
    driver.settle();
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(u64::from(completion)) else {
        panic!("person delegation: {:?}", driver.answered(u64::from(completion)));
    };
    let child = children[0].task;
    driver.answer(WORKER, goal, attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    child
}
fn discover(ctl: &mut Ctl, task: u64) -> String {
    let number = task.to_string();
    let inbox = ctl.text(&[b"inbox", b"--kind", b"person-tasks"]);
    assert!(inbox.contains(&format!("task show {task}")), "person inbox continuation: {inbox}");
    let shown = ctl.text(&[b"task", b"show", number.as_bytes()]);
    let revision = ctl.front.engine.task(task).expect("person task").revisions.person.to_string();
    assert!(shown.contains(&format!("task answer {task} --rev {revision}")), "answer instructions: {shown}");
    assert!(shown.contains("Choose a colour: blue"), "person specification: {shown}");
    assert!(shown.contains("1 (reason bytes 128)"), "contract choice: {shown}");
    assert!(shown.contains("task take"), "role claim offered: {shown}");
    revision
}
fn claim_and_return(ctl: &mut Ctl, task: u64) {
    let number = task.to_string();
    let take = format!("take-{task}");
    ctl.text(&[b"task", b"take", number.as_bytes(), b"--key", take.as_bytes()]);
    let shown = ctl.text(&[b"task", b"show", number.as_bytes()]);
    assert!(shown.contains("taken by: you"), "claimed task: {shown}");
    assert!(shown.contains("task hand-back"), "return offered: {shown}");
    let back = format!("back-{task}");
    ctl.text(&[b"task", b"hand-back", number.as_bytes(), b"--key", back.as_bytes()]);
    assert_eq!(ctl.front.engine.task(task).expect("returned task").taken_by, None);
    let take = format!("retake-{task}");
    ctl.text(&[b"task", b"take", number.as_bytes(), b"--key", take.as_bytes()]);
}
fn stale_answer(ctl: &mut Ctl, task: u64, revision: &str) {
    let number = task.to_string();
    let spec = ctl.front.engine.task(task).expect("person task").revisions.spec.to_string();
    let key = format!("amend-{task}");
    let amended = ctl.call(
        &[
            b"task",
            b"amend",
            number.as_bytes(),
            b"--rev",
            spec.as_bytes(),
            b"--spec",
            b"question.txt",
            b"Clarify the question",
            b"--key",
            key.as_bytes(),
        ],
        Some(b"Choose a colour: blue (new question)"),
    );
    assert_eq!(amended.status, 0, "{amended:?}");
    let key = format!("stale-{task}");
    let stale = ctl.call(
        &[
            b"task",
            b"answer",
            number.as_bytes(),
            b"--rev",
            revision.as_bytes(),
            b"1",
            b"blue",
            b"--key",
            key.as_bytes(),
        ],
        None,
    );
    assert_eq!(stale.status, 2, "stale person revision must be refused: {stale:?}");
    assert!(String::from_utf8_lossy(&stale.stderr).contains("changed"), "revision refusal: {stale:?}");
    assert!(ctl.front.engine.task(task).expect("stale answer keeps task live").person_answer.is_none());
}
fn answer_choice(ctl: &mut Ctl, task: u64, key: &[u8]) {
    let revision = discover(ctl, task);
    claim_and_return(ctl, task);
    stale_answer(ctl, task, &revision);
    let number = task.to_string();
    let shown = ctl.text(&[b"task", b"show", number.as_bytes(), b"--json"]);
    assert!(shown.contains("\"person_task\""), "person contract in JSON: {shown}");
    let shown = ctl.text(&[b"task", b"show", number.as_bytes()]);
    let revision = ctl.front.engine.task(task).expect("changed task").revisions.person.to_string();
    assert!(shown.contains(&format!("task answer {task} --rev {revision}")), "new revision: {shown}");
    let words: &[&[u8]] =
        &[b"task", b"answer", number.as_bytes(), b"--rev", revision.as_bytes(), b"1", b"blue", b"--key", key];
    ctl.text(words);
    let assignments = ctl.front.assignments.len();
    ctl.text(words);
    assert_eq!(ctl.front.assignments.len(), assignments, "keyed replay starts no new goal run");
    completed(ctl, task);
    let shown = ctl.text(&[b"task", b"show", number.as_bytes()]);
    assert!(shown.contains("answered by: you"), "answered task: {shown}");
    assert!(!shown.contains("taken by:"), "answered task: {shown}");
}
fn completed(ctl: &mut Ctl, task: u64) {
    let driver = &ctl.front.engine;
    let stored = driver.store.rows.get(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = stored else {
        panic!("completed person task: {stored:?}");
    };
    assert!(
        matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Verdict { code: BLUE, words, .. })) if words.as_ref() == b"blue")
    );
    let goal = ctl.front.assignments.last().expect("goal resumed");
    assert!(
        goal.messages.iter().any(|entry| matches!(
            entry.kind,
            tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: BLUE, .. })
        ) && entry.words.as_ref() == b"blue"
            && entry.from == tasks::Party::Task(task)),
        "goal wake carries result: {:?}",
        goal.messages
    );
}
fn numbered_question() {
    let mut front = Joint::new(fixtures::SEED, fixtures::domain_limits());
    let chat = fixtures::question(&mut front, b"Which colour?");
    let asker = front.engine.task(chat).expect("chat").delegates[0];
    let question = front.engine.task(asker).expect("asking delegate").questions[0].number;
    let (front, root) = fixtures::tools_on(front, fixtures::SEED);
    let mut ctl = Ctl::new(front, root);
    ctl.text(&[b"inbox", b"--kind", b"questions"]);
    ctl.text(&[b"task", b"answer", chat.to_string().as_bytes(), question.to_string().as_bytes(), b"blue"]);
    assert!(
        ctl.front
            .engine
            .task(asker)
            .expect("asking delegate")
            .inbox
            .iter()
            .any(|entry| matches!(entry.kind, tasks::MessageKind::Answer { question: q } if q == question)
                && entry.words.as_ref() == b"blue")
    );
}
fn start_goal(spec: &[u8]) -> (Ctl, u64) {
    let (front, root) = fixtures::person_task_tools(fixtures::SEED);
    let mut ctl = Ctl::new(front, root);
    let started =
        ctl.call(&[b"goal", b"start", b"--project", b"1", b"--template", b"4", spec, b"--key", b"goal"], None);
    assert_eq!(started.status, 0, "{started:?}");
    let goal = fixtures::done_task(&started);
    (ctl, goal)
}
fn role_configuration() -> engine::Config {
    use jig_domain_authority::{Executor, Scopes};
    let mut config = temper_engine_domain_world::walking::config(fixtures::SEED);
    let models = config
        .core
        .settings
        .models
        .iter()
        .enumerate()
        .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
            first_read: None,
            number: u32::try_from(at).expect("bounded fixture model count") + 1,
            model: model.clone(),
            shown: None,
            description: None,
            picker: true,
            withdrawn: false,
            source: jig_domain_core::settings::ModelSource::Stock,
            estimated: false,
        })
        .collect();
    let stock = temper_service::stock::with_models(models, RESULT_WORDS);
    config.core.defaults.charters = stock.charters.into_iter().filter(|charter| charter.number == 13).collect();
    config.core.defaults.templates = stock.templates.into_iter().filter(|template| template.number == 7).collect();
    let ceiling = &mut config.core.defaults.rules.ceiling;
    ceiling.delegation.kinds = Box::new([Executor::Charter(13)]);
    ceiling.notes = Scopes::GOAL;
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds.clone_from(&ceiling.delegation.kinds);
    policy.ceiling.notes = Scopes::GOAL;
    policy.roles[0].authority.delegation.kinds.clone_from(&ceiling.delegation.kinds);
    policy.roles[0].authority.notes = Scopes::GOAL;
    config
}

#[test]
fn a_person_starts_a_standing_role_that_writes_now_and_reads_it_next_run() {
    use temper_client_world::front::engine_limits;
    use temper_engine_domain_world::{protocol, store::Store};
    const NOW: &str = "Tests are green.\nWaiting for a change, then check its tests.";
    let mut bounds = engine_limits();
    bounds.tasks.charters = 13;
    bounds.journal.run_bytes = 16_384;
    let front = Joint::with_engine(
        fixtures::SEED,
        &fixtures::domain_limits(),
        Store::with_limits(&bounds),
        [role_configuration(), role_configuration()],
        &bounds,
        1,
    );
    let (mut front, root) = fixtures::tools_on(front, fixtures::SEED);
    front.engine.send(engine::Event::Worker(engine::WorkerInput::Link { channel: WORKER, link: engine::Link::Lost }));
    front.engine.settle();
    let mut hello = protocol::hello_slots(WORKER, 1, Box::new([]));
    if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut hello
    {
        hello.bounds.time = skein_lib::Duration::from_secs(28_800);
        hello.bounds.turns = 800;
        hello.bounds.message_bytes = bounds.tasks.message_bytes;
    }
    front.engine.send(hello);
    front.engine.settle();
    let mut ctl = Ctl::new(front, root);
    let started = ctl.call(
        &[b"role", b"start", b"--project", b"1", b"--budget", b"50", b"Owns the tests", b"--key", b"tests-role"],
        None,
    );
    assert_eq!(started.status, 0, "{started:?}");
    let task = fixtures::done_task(&started);
    let row = ctl.front.engine.task(task).expect("standing task");
    assert_eq!(row.label, 3);
    assert_eq!(row.flags.ended_by, tasks::EndedBy::Standing);
    assert_eq!(row.requester, tasks::Party::Person(fixtures::PARTY));
    let run = ctl.front.assignments.last().expect("standing role assigned").clone();
    assert_eq!(run.task, task);
    let input = format!(
        r#"{{"scope":{{"kind":"goal","project":1,"goal":{task}}},"description":"now","body":"Tests are green.\nWaiting for a change, then check its tests."}}"#
    );
    let (answer, failed) =
        temper_engine_domain_world::host_tool::host_tool(&mut ctl.front.engine, &run, 1, b"note", input.as_bytes());
    assert!(!failed, "{}", String::from_utf8_lossy(&answer));
    ctl.front.engine.answer(WORKER, task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    ctl.front.engine.settle();
    assert!(ctl.front.engine.task(task).is_some(), "parking does not finish the role");
    ctl.text(&[b"task", b"message", task.to_string().as_bytes(), b"A change landed", b"--key", b"wake-role"]);
    let next = ctl.front.assignments.last().expect("standing role woken").clone();
    assert_eq!(next.task, task);
    assert!(next.attempt > run.attempt);
    assert!(next.sections.iter().any(|section| {
        matches!(&section.body, engine::BriefBody::Text(text) if skein_lib::bytes::find(text, NOW.as_bytes()).is_some())
    }), "next brief: {:?}", next.sections);
    let shown = ctl.text(&[b"role", b"show", task.to_string().as_bytes()]);
    assert!(shown.contains(NOW));
}

fn sign_in_stranger(ctl: &mut Ctl) -> u64 {
    const REPLY: skein_lib::Token = skein_lib::Token::new(2003);
    let driver = &mut ctl.front.engine;
    // A held person without a project role reaches executor permission admission.
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: skein_lib::ReplyTo::new(REPLY),
        identity: jig_domain_people::Identity {
            key: temper_engine_domain_forge::user_key(1, 8_u64),
            login: b"stranger".as_slice().into(),
            name: b"Stranger".as_slice().into(),
        },
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            engine::Delivery::WebReply { to, reply: jig_domain_people::Reply::SignedIn { person, .. } }
                if *to == skein_lib::ReplyTo::new(REPLY) =>
            {
                Some(*person)
            }
            _ => None,
        })
        .expect("stranger signed in")
}
#[test]
fn a_person_answers_a_delegated_choice_and_wakes_its_goal() {
    let (mut ctl, goal) = start_goal(b"Ask me a colour");
    for (completion, key) in [(28, b"colour-28".as_slice()), (37, b"colour-37".as_slice())] {
        let child = delegate(&mut ctl, goal, completion);
        answer_choice(&mut ctl, child, key);
    }
    numbered_question();
}

// temper/docs/design/domain/agent/agents.md, 2.
#[test]
fn a_person_started_goal_asks_its_person_and_no_stranger() {
    // start_goal creates one root task; a refusal adds no live or ended child.
    const ROOT_TASKS: usize = 1;
    const COMPLETION: u32 = 28;
    const REPORT_WORDS: u32 = 4096;
    const QUESTION: &[u8] = b"Which colour?\n\nChoose the colour.\nSaw: Both fit.\nChoice: Blue (recommended)";
    const ANSWER: &[u8] = b"Use blue";
    // A missing fixed address uses the signed-in stranger's allocated person number.
    for (fixed_address, refusal) in [
        (Some(tasks::PersonAddress::Person(fixtures::PARTY)), None),
        (Some(tasks::PersonAddress::Role(1)), Some("missing executor permission")),
        (None, Some("Refused: executor")),
    ] {
        let (mut ctl, goal) = start_goal(b"Ask me a colour");
        let no_role = sign_in_stranger(&mut ctl);
        let address = fixed_address.unwrap_or(tasks::PersonAddress::Person(no_role));
        let driver = &mut ctl.front.engine;
        let attempt = driver.task(goal).expect("running goal").attempt;
        let mut child = temper_engine_domain_world::fixtures::report_delegate(QUESTION, Box::new([]));
        child.executor = tasks::Executor::Person(address);
        child.contract = tasks::Contract::Report { words: REPORT_WORDS };
        child.authority.tools = tasks::Tools(0);
        child.authority.budget.spend = 0;
        child.wake = None;
        driver.call(
            WORKER,
            goal,
            attempt,
            u64::from(COMPLETION),
            engine::Call {
                completion: COMPLETION,
                position: 1,
                tool: engine::Tool::Delegate { batch: Box::new([child]) },
            },
        );
        driver.settle();
        let answer = driver.answered(u64::from(COMPLETION)).expect("delegation answered");
        if let Some(refusal) = refusal {
            let rendered = temper_engine_smith::answer_for_tool(
                answer,
                b"delegate",
                REPORT_WORDS,
                temper_engine_smith::ForgeReadBounds::STOCK,
            );
            assert!(rendered.error());
            assert!(String::from_utf8_lossy(rendered.text()).contains(refusal), "{address:?}: {rendered:?}");
            assert!(driver.task(goal).expect("goal").delegates.is_empty(), "refusal makes no task");
            assert_eq!(
                driver
                    .store
                    .rows
                    .values()
                    .filter(|record| matches!(
                        record,
                        engine::Record::Core(jig_domain_core::Record::Tasks(
                            tasks::Stored::Live(_) | tasks::Stored::Ended(_)
                        ))
                    ))
                    .count(),
                ROOT_TASKS
            );
            continue;
        }
        let engine::CallAnswer::Delegated(children) = answer else { panic!("person delegation: {answer:?}") };
        let child = children[0].task;
        assert_eq!(driver.task(child).expect("person task").requester, tasks::Party::Task(goal));
        driver.answer(WORKER, goal, attempt, 0, tasks::End::Parked, engine::Parts::none());
        driver.settle();
        let revision = driver.task(child).expect("person task").revisions.person.to_string();
        ctl.text(&[
            b"task",
            b"answer",
            child.to_string().as_bytes(),
            b"--rev",
            revision.as_bytes(),
            ANSWER,
            b"--key",
            b"colour",
        ]);
        let stored =
            ctl.front.engine.store.rows.get(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(child))));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = stored else {
            panic!("completed person task: {stored:?}");
        };
        assert!(
            matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) if words.as_ref() == ANSWER)
        );
        let resumed = ctl.front.assignments.last().expect("goal resumed");
        assert_eq!(resumed.task, goal);
        assert!(resumed.attempt > attempt);
        assert!(
            resumed.messages.iter().any(|entry| matches!(
                entry.kind,
                tasks::MessageKind::Result(tasks::ResultKind::Report)
            ) && entry.words.as_ref() == ANSWER
                && entry.from == tasks::Party::Task(child)),
            "goal wake: {:?}",
            resumed.messages
        );
    }
}

#[test]
fn a_reason_past_its_choice_names_the_bound() {
    let (mut ctl, goal) = start_goal(b"Ask me a colour");
    let child = delegate(&mut ctl, goal, 28);
    let number = child.to_string();
    ctl.text(&[b"task", b"take", number.as_bytes(), b"--key", b"take-colour"]);
    let revision = ctl.front.engine.task(child).expect("person task").revisions.person.to_string();
    let reason = vec![b'x'; usize::try_from(RESULT_WORDS).expect("reason bound") + 1];
    let refused =
        ctl.call(&[b"task", b"answer", number.as_bytes(), b"--rev", revision.as_bytes(), b"1", &reason], None);
    assert_eq!(refused.status, 2);
    assert_eq!(
        refused.stderr.as_ref(),
        format!("task answer: Limit reached: task {child}'s reason for choice 1 is at most 128 bytes\n").as_bytes()
    );
    assert!(ctl.front.engine.task(child).expect("refused task stays live").person_answer.is_none());
    let reason = &reason[..usize::try_from(RESULT_WORDS).expect("reason bound")];
    ctl.text(&[b"task", b"answer", number.as_bytes(), b"--rev", revision.as_bytes(), b"1", reason]);
    let stored =
        ctl.front.engine.store.rows.get(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(child))));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = stored else {
        panic!("completed person task: {stored:?}");
    };
    assert!(
        matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Verdict { code: BLUE, words, .. })) if words.as_ref() == reason)
    );
}

fn result_projections(ctl: &mut Ctl, goal: u64, child: u64) {
    let assignment = ctl.front.assignments.last().expect("goal resumed");
    let word = assignment.messages.iter().find(|word| word.from == tasks::Party::Task(child)).expect("result wake");
    let wake = String::from_utf8(engine::inbox_words(goal, word).into_vec()).expect("wake UTF-8");
    assert!(wake.contains("verdict 2") && wake.contains("bold"), "rendered wake: {wake}");
    let attempt = ctl.front.engine.task(goal).expect("resumed goal").attempt;
    let completion = 39;
    ctl.front.engine.call(
        WORKER,
        goal,
        attempt,
        u64::from(completion),
        engine::Call {
            completion,
            position: 1,
            tool: engine::Tool::ReadTask { task: child, field: jig_domain_core::reads::TaskTextField::Result, from: 0 },
        },
    );
    ctl.front.engine.settle();
    let read = ctl.front.engine.answered(u64::from(completion)).expect("retained result read");
    let answer = temper_engine_smith::answer(read);
    let text = String::from_utf8_lossy(answer.text());
    assert!(text.contains("Verdict: 2") && text.contains("Result from 0:\nbold"), "host read: {text}");
    let activity = ctl.text(&[b"task", b"activity", goal.to_string().as_bytes()]);
    assert!(activity.contains("verdict 2") && activity.contains("bold"), "activity: {activity}");
    let shown = ctl.text(&[b"task", b"show", child.to_string().as_bytes()]);
    assert!(shown.contains("verdict 2") && shown.contains("bold"), "completed task: {shown}");
}
#[test]
fn a_person_choice_code_reaches_the_asker_and_ctl() {
    let (mut ctl, goal) = start_goal(b"Choose a style");
    let child = delegate_choices(&mut ctl, goal, 28, &[2, 3]);
    let number = child.to_string();
    ctl.text(&[b"task", b"take", number.as_bytes(), b"--key", b"take-style"]);
    let words: &[&[u8]] = &[b"task", b"answer", number.as_bytes(), b"--rev", b"0", b"2", b"bold", b"--key", b"style"];
    ctl.text(words);
    let assignments = ctl.front.assignments.len();
    ctl.text(words);
    assert_eq!(ctl.front.assignments.len(), assignments, "keyed replay starts no new goal run");
    let stored =
        ctl.front.engine.store.rows.get(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(child))));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = stored else {
        panic!("completed person task: {stored:?}");
    };
    assert!(
        matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Verdict { code: 2, words, .. })) if words.as_ref() == b"bold")
    );
    assert!(ctl.front.engine.task(child).is_none(), "result read uses retained row");
    result_projections(&mut ctl, goal, child);
}
