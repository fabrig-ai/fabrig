//! Requester news reaches the model once (temper/docs/design/domain/agent/README.md, 7).
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Part, Script};
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, fixtures, store::Store};
use temper_system_world::{composed::call, world};

// Director, owner and two reporters; only the requesters run again.
const TASKS: u32 = 4;
const ATTEMPTS: u32 = 8; // Four initial runs and room for both requester wakes.
const TURNS: u32 = 16; // The system goal fixture's bounded transcript envelope.
const INBOX_MESSAGES: u32 = 7; // The initial notice, both endings and their acknowledgements.
const WORD_BYTES: u32 = 1024;
const BRIEF_SECTIONS: u32 = 8; // Task, policy, resources, results and their joined sections.
const FORGE_BRIEF_SECTIONS: u32 = 80; // The system goal fixture's brief-fetch envelope.
const JOURNAL_WRITES: u32 = 4735; // The system goal fixture's complete four-task decision.
const DELIVERIES: u32 = 512;
const HELD: u32 = 1536;
const TRANSCRIPT_BYTES: u32 = 32768;

fn delegate(
    driver: &mut Driver,
    parent: &engine::Assignment,
    batch: Box<[engine::Delegate]>,
    number: u64,
) -> Vec<engine::Assignment> {
    driver.call(
        world::CHANNEL,
        parent.task,
        parent.attempt,
        number,
        engine::Call { completion: 1, position: 1, tool: engine::Tool::Delegate { batch } },
    );
    driver.settle();
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(number) else {
        panic!("delegated: {:?}", driver.answered(number))
    };
    children
        .iter()
        .map(|child| {
            driver
                .delivered
                .iter()
                .find_map(|delivery| {
                    if let engine::Delivery::Assigned { assignment, .. } = delivery
                        && assignment.task == child.task
                    {
                        Some(assignment.clone())
                    } else {
                        None
                    }
                })
                .expect("child assigned")
        })
        .collect()
}

fn park(driver: &mut Driver, run: &engine::Assignment) {
    driver.answer(world::CHANNEL, run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
}

fn hear(driver: &mut Driver, task: u64, results: &[(u64, &[u8])], report: &[u8]) {
    let assignment = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let engine::Delivery::Assigned { assignment, .. } = delivery
                && assignment.task == task
            {
                Some(assignment.clone())
            } else {
                None
            }
        })
        .expect("requester resumed");
    assert!(assignment.attempt > 1);
    let input = format!("{{\"report\":\"{}\"}}", std::str::from_utf8(report).expect("report"));
    let mut agent = world::scripted_agent_for(
        &assignment,
        Script { cue: Box::new([]), turns: Box::new([call(b"finish", input.as_bytes())]) },
    );
    world::run_assignment(driver, &assignment, &mut agent);
    assert_ne!(agent.prompts(), []);
    for (sender, words) in results {
        let expected = format!("Result from task {sender}:\n{}", std::str::from_utf8(words).expect("result"));
        let count: usize = agent
            .prompts()
            .iter()
            .flat_map(|query| &query.messages)
            .flat_map(|message| &message.parts)
            .map(|part| {
                if let Part::Text { text } = part {
                    text.windows(expected.len()).filter(|window| *window == expected.as_bytes()).count()
                } else {
                    0
                }
            })
            .sum();
        assert_eq!(count, 1, "each numbered ending reaches its requester once: {expected}");
    }
}

#[test]
fn each_requester_hears_each_delegates_ending_once() {
    let mut bounds = temper_engine_domain_world::walking::limits();
    bounds.fleet.slots = TASKS;
    bounds.fleet.attempts = ATTEMPTS;
    bounds.fleet.turns = TURNS;
    bounds.tasks.tasks = TASKS;
    bounds.tasks.project_tasks = TASKS;
    bounds.tasks.tree_tasks = TASKS;
    bounds.tasks.depth = 3;
    bounds.tasks.delegates = 3;
    bounds.tasks.batch = 2;
    bounds.tasks.inbox_messages = INBOX_MESSAGES;
    bounds.tasks.inbox_bytes = WORD_BYTES;
    bounds.people.inbox_tasks = TASKS * 2;
    bounds.people.inbox_recipients = INBOX_MESSAGES * 3;
    bounds.authority.batch = 2;
    bounds.brief.briefs = TASKS;
    bounds.brief.sections = BRIEF_SECTIONS;
    bounds.forge.brief_sections = FORGE_BRIEF_SECTIONS;
    bounds.forge.tasks = TASKS;
    bounds.brief.brief_bytes = WORD_BYTES;
    bounds.journal.writes = JOURNAL_WRITES;
    bounds.journal.deliveries = DELIVERIES;
    bounds.journal.held = HELD;
    bounds.journal.transcript_bytes = TRANSCRIPT_BYTES;
    bounds.journal.result_bytes = WORD_BYTES;
    let configs = std::array::from_fn(|_| {
        let mut config = world::config(966);
        config.core.defaults.rules.maximum_run_spend = 10;
        config.core.defaults.rules.ceiling.delegation.tasks = 5;
        config.core.defaults.rules.ceiling.delegation.depth = 3;
        config.core.defaults.rules.ceiling.tools = authority::Tools(3);
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling = config.core.defaults.rules.ceiling.clone();
        policy.roles[0].authority = policy.ceiling.clone();
        config
    });
    let mut driver = Driver::configured(Store::with_limits(&bounds), configs, &bounds);
    driver.send(world::hello(4));
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut request = driver.chat_ask(1, b"Report both endings");
    let jig_domain_people::Ask::Create { batch, .. } = &mut request else { panic!("chat batch") };
    batch[0].budget = jig_domain_people::Budget::AsAllowed { cap: Some(50) };
    batch[0].contract = jig_domain_people::Contract::Report { words: 128 };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(91)),
        sign_in: driver.session(),
        key: [91; 16],
        ask: request,
    }));
    driver.settle();
    let director = driver.last_assignment();
    let mut owner = fixtures::report_delegate(b"Own both reports", Box::new([]));
    owner.authority.tools = tasks::Tools(1);
    owner.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 2, depth: 1 };
    owner.authority.budget.spend = 30;
    owner.budget = jig_domain_people::Budget::AsAllowed { cap: Some(30) };
    let owner = delegate(&mut driver, &director, Box::new([owner]), 92).remove(0);
    park(&mut driver, &director);
    let children = [b"Kit result".as_slice(), b"App result"].map(|words| {
        let mut child = fixtures::report_delegate(words, Box::new([]));
        child.authority.tools = tasks::Tools(0);
        child
    });
    let children = delegate(&mut driver, &owner, Box::new(children), 93);
    park(&mut driver, &owner);
    let reports = [(children[0].task, b"Kit landed".as_slice()), (children[1].task, b"App landed".as_slice())];
    for (child, (_, words)) in children.iter().zip(reports) {
        driver.answer(
            world::CHANNEL,
            child.task,
            child.attempt,
            0,
            tasks::End::Finished { result: tasks::TaskResult::Report { words: words.into() }, cancel_delegates: false },
            engine::Parts::none(),
        );
        driver.settle();
    }
    hear(&mut driver, owner.task, &reports, b"Both landed");
    hear(&mut driver, director.task, &[(owner.task, b"Both landed")], b"Done");
}
