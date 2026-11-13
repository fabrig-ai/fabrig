//! A stock goal's report crosses Smith's note calls and the person's scoped read.
//! Contract: temper/docs/design/domain/agent/agents.md, 3.
use super::*;
use jig_domain_core::{documents, reads};

// The first note in the empty world's scope, and its initial revision.
const NOTE: u64 = 1;
const FIRST_REVISION: u32 = 1;
// One creation followed by one revision leaves one report.
const REVISED: u32 = 2;
const ONE_REPORT: u32 = 1;
// The system world's configured owner and seeded project.
const OWNER: u64 = 7;
const PROJECT: u32 = 1;
// Fifty units fund the three scripted tool turns and their finishing reserve.
const BUDGET: u64 = 50;
const HEADLINE: &[u8] = b"report: The first task landed.";
const BODY: &[u8] = b"I landed the first task after its checks passed.\n\nLeft: the second task.";
const READ: Token = Token::new(504);

fn configured() -> (Driver, engine::Event) {
    let limits = temper_engine_domain_world::forge_routes::box_limits();
    let configs = std::array::from_fn(|_| {
        let mut config = world::config(904);
        let llm = jig_protocol_config::Llm {
            endpoints: Box::new([]),
            providers: Box::new([]),
            models: (config.core.settings.models.clone())
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
        config.core.defaults =
            temper_engine_service_stock::stock::with_models(llm.models.clone(), limits.tasks.result_bytes);
        config.core.seed_projects[0].policy = config.core.defaults.policies.get(&1).expect("stock policy").clone();
        config.core.connectors = Box::new([1]);
        config.kinds_stock = engine::landing_stock(&[], &skein_lib::Map::with_capacity(0), 2, 4);
        config
    });
    let mut hello = world::hello(1);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    for charter in &configs[0].core.defaults.charters {
        worker.bounds.turns = worker.bounds.turns.max(charter.policy.turns);
        worker.bounds.time = worker.bounds.time.max(charter.policy.time);
        worker.bounds.waiting = worker.bounds.waiting.max(charter.policy.waiting);
    }
    worker.bounds.message_bytes = limits.tasks.message_bytes;
    worker.bounds.inbox = limits.tasks.run_inbox;
    let budget = configs[0].core.defaults.rules.period_spend;
    let mut driver = Driver::configured(Store::with_limits(&limits), configs, &limits);
    driver.period_sent = true;
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Period { project: PROJECT, period: 1, budget, next: None }));
    driver.settle();
    (driver, hello)
}

fn script(task: u64) -> Script {
    let scope = format!(r#"{{"kind":"goal","project":{PROJECT},"goal":{task}}}"#);
    let first = format!(
        r#"{{"scope":{scope},"description":"report: I planned two tasks.","body":"I split the work into two tasks."}}"#
    );
    let revised = format!(
        r#"{{"scope":{scope},"name":{NOTE},"recalled":{FIRST_REVISION},"description":"report: The first task landed.","body":"I landed the first task after its checks passed.\n\nLeft: the second task."}}"#
    );
    let call = |name: &[u8], arguments: &[u8]| Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    Script {
        cue: b"@goalplan".as_slice().into(),
        turns: Box::new([
            call(b"note", first.as_bytes()),
            call(b"note", revised.as_bytes()),
            call(b"finish", br#"{"report":"The first task landed; the report is current."}"#),
        ]),
    }
}

#[test]
fn a_stock_goal_revises_the_report_its_person_reads() {
    let (mut driver, hello) = configured();
    driver.send(hello);
    driver.settle();
    let (_, session) = sign_in(&mut driver, OWNER, 501);
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(502)),
        sign_in: session,
        key: [52; 16],
        ask: goal(&driver, b"@goalplan Report the goal's progress", BUDGET, 1),
    }));
    driver.settle();
    let assignment = driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::Assigned { assignment, .. } => Some(assignment.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("goal assigned: {:?}", driver.delivered));
    let mut agent = world::scripted_agent_for(&assignment, script(assignment.task));
    world::run_assignment(&mut driver, &assignment, &mut agent);
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(READ),
        sign_in: session,
        // The core form of the client's Read::Notes in NoteScope::Task.
        read: reads::Read::ScopedNotes {
            scope: jig_domain_notes::Scope::Goal { project: PROJECT, goal: assignment.task },
            description: Some(b"report: ".as_slice().into()),
            from: None,
        },
    }));
    driver.settle();
    let page = driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::Read { reply, answer: reads::Answer::Page(page) } if *reply == ReplyTo::new(READ) => Some(page),
            _ => None,
        })
        .expect("person reads the goal's report");
    assert_eq!(page.rows.len(), ONE_REPORT);
    for row in &page.rows {
        let reads::PageRow::Note(note) = row else { panic!("report note") };
        assert_eq!(note.number, NOTE);
        assert_eq!(note.revision, REVISED);
        assert_eq!(note.scope, documents::NoteLocation::Goal { project: PROJECT, task: assignment.task });
        assert_eq!(note.description.bytes.as_ref(), HEADLINE);
        assert_eq!(note.body.bytes.as_ref(), BODY);
    }
}
