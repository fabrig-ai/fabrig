//! Ask temper's first request (temper/docs/design/domain/agent/ask-temper.md).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::print_stderr,
    reason = "the step fixture inspects stock settings and reports each measured request part"
)]

use super::{fixtures, stock};
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use smith_domain_run as run;
use smith_protocol_program::mcp;
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, protocol, store::Store, walking};

/// What temper gives the first request (temper/docs/design/domain/agent/ask-temper.md).
const FIRST_REQUEST_BYTES: u64 = 16_384;
const ASKING_CHARTER: u32 = 11;
const ASKING_TEMPLATE: u32 = 5;
const DRAFTER_CHARTER: u32 = 12;
const DRAFTING_TEMPLATE: u32 = 6;
const PROJECT: u32 = 1;
const CHANNEL: Token = Token::new(7);
// The stock agent profile's declared payload and endpoint output (temper-init/template/agent.json).
const TOOL_PAYLOAD: u32 = 10_662;
const OUTPUT_TOKENS: u32 = 8192;
// The stock-brief admission fixture's eight-turn window and maximum turn body.
const WINDOW: run::Window = run::Window { turns: 8, bytes: 16_777_216, largest_turn: 2_097_152 };
const FIRST_WORDS: &[u8] =
    b"How does temper work in this project, and how can I propose settings that suit the way our team works?";

fn stock_config() -> (engine::Config, engine::Limits, jig_protocol_config::Engine) {
    let document = stock::generated();
    let parsed = stock::engine(&document);
    let application = fixtures::value(&document, b"application");
    let application = jig_protocol_config::document(
        &application,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .expect("stock application document");
    let application = crate::startup::application::parse(
        &application,
        0,
        &parsed,
        &jig_protocol_config::Directories::working(),
        &mut crate::startup::Problems::empty(),
    )
    .expect("stock application limits");
    let mut limits = crate::limits::engine_candidate(&parsed.limits, &application.limits);
    limits.memory_bytes = parsed.memory;
    let mut config = walking::config(7);
    let owner = config.core.owners.first().expect("fixture owner").clone();
    config.core = jig_protocol_config::core(&parsed, [31; 16], 7);
    let mut deployment_owner = owner.clone();
    deployment_owner.project = people::DEPLOYMENT;
    config.core.owners = Box::new([deployment_owner, owner]);
    config.core.seed_projects = Box::new([core::SeedProject {
        number: PROJECT,
        name: "Project".into(),
        policy: config.core.defaults.policies.get(&1).expect("stock policy").clone(),
    }]);
    config.core.kinds = crate::stock::kinds(&limits, 1);
    config.kinds_stock = crate::stock::forge(&limits);
    config.forges = Box::default();
    config.period_span = crate::stock::PERIOD_SPAN;
    (config, limits, parsed)
}

fn first_assignment(charter: u32, template: u32) -> engine::Assignment {
    let (config, limits, parsed) = stock_config();
    let budget = config.core.defaults.policies.get(&1).expect("stock policy").policy.period_spend;
    let template = config
        .core
        .defaults
        .templates
        .iter()
        .find(|candidate| candidate.number == template)
        .expect("stock chat template")
        .clone();
    let restart = temper_engine_domain_world::oracle::config::copy(&config);
    let accounts = temper_engine_domain_world::fixtures::account_rows(&config.core.settings.llm_providers);
    let mut bootstrap = Driver::cold(Store::with_limits(&limits), config, &limits);
    bootstrap.period_sent = true;
    bootstrap.settle();
    let mut store = bootstrap.take_store();
    store.seed(accounts, Box::default());
    let mut driver = Driver::cold(store, restart, &limits);
    driver.period_sent = true;
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Period { project: PROJECT, period: 1, budget, next: None }));
    connect(&mut driver, &parsed.llm.models);
    driver.sign_in();
    driver.settle();
    let mut ask = driver.chat_ask(PROJECT, FIRST_WORDS);
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat creation") };
    let member = batch.first_mut().expect("one chat");
    member.executor = people::TaskExecutor::Agent { charter };
    let jig_domain_tasks::Contract::Report { words } = template.contract else { panic!("chat report") };
    member.contract = people::Contract::Report { words };
    member.flags.ended_by = people::EndedBy::Requester;
    member.flags.conversation = template.flags.conversation;
    member.flags.listed = template.flags.listed;
    member.label = template.label;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::default(),
        reply_to: ReplyTo::new(Token::new(1001)),
        sign_in: driver.session(),
        key: [31; 16],
        ask,
    }));
    driver.settle();
    let assignment = driver.last_assignment();
    driver.answer(
        CHANNEL,
        assignment.task,
        assignment.attempt,
        0,
        jig_domain_tasks::End::Parked,
        engine::Parts::none(),
    );
    driver.settle();
    assignment
}

fn connect(driver: &mut Driver, models: &[core::settings::CatalogueModel]) {
    let limits = driver.env.limits;
    let sources: Box<[_]> = driver
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let engine::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Source {
                number, ..
            })) = row
            {
                Some(*number)
            } else {
                None
            }
        })
        .collect();
    for source in sources {
        driver.send(engine::Event::Account(jig_domain_accounts::Event::Credential {
            number: source,
            generation: 1,
            valid: Duration::from_secs(3600),
        }));
    }
    let mut greeting = protocol::hello(CHANNEL, Box::default());
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) =
        &mut greeting
    else {
        panic!("worker greeting")
    };
    hello.bounds = jig_domain_link::Bounds {
        endpoints: models
            .iter()
            .map(|model| jig_domain_link::EndpointBound {
                endpoint: model.model.endpoint,
                window: model.model.window,
                output: model.model.output,
            })
            .collect(),
        turns: 800,
        spend: u64::MAX,
        time: Duration::from_secs(28_800),
        message_bytes: limits.tasks.message_bytes,
        inbox: limits.tasks.inbox_messages,
        unacknowledged: limits.fleet.turns,
        ..protocol::bounds()
    };
    driver.send(greeting);
    driver.settle();
}

fn opening(charter: run::Charter, limits: &run::Limits) -> run::Opening {
    let mut domain = run::Domain::new(limits, 7);
    let mut out = Queue::uncapped();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    run::step(
        &mut domain,
        &env,
        run::Event::Start {
            reply_to: ReplyTo::new(Token::new(1)),
            host_run: Token::new(1),
            activation: 1,
            identity: None,
            window: WINDOW,
            messages: Box::default(),
            charter,
            workspace: None,
            transcript: None,
            resumed: false,
        },
        &mut out,
    );
    while domain.is_ready() {
        run::resume(&mut domain, &env, &mut out);
    }
    while let Some(request) = out.pop() {
        if let run::Request::Open { opening, .. } = request {
            return opening;
        }
    }
    panic!("stock charter admitted and opened without a workspace")
}

fn declarations(
    opened: run::Opening,
    outcome: &run::outcome::OutcomeSpec,
    limits: &smith_protocol_llm::ComponentLimits,
) -> (u64, u64) {
    let prompt = smith_domain::llm::Prompt {
        asker: smith_domain::llm::Asker::Main,
        endpoint: smith_domain::llm::Endpoint(opened.llm.endpoint.0),
        affinity: smith_domain::llm::Affinity { key: [0; 16], thread: 0 },
        model: opened.llm.model,
        effort: opened.llm.effort,
        system: opened.system,
        tools: smith_domain::tools::Grants {
            inspect: opened.tools.inspect,
            modify: opened.tools.modify,
            shell: opened.tools.shell,
        },
        namespace: smith_domain::tools::Namespace::One,
        served: if opened.wait { Box::new([smith_domain::llm::Served::Wait]) } else { Box::default() },
        choice: smith_domain::llm::ToolChoice::Auto,
        messages: Box::default(),
        max_tokens: OUTPUT_TOKENS,
        purpose: smith_domain::llm::Purpose::Ordinary { guard: None },
    };
    let schemas = smith_protocol_llm::schemas_for_contract(
        &prompt,
        &limits.translation.stated,
        outcome,
        None,
        u32::try_from(limits.contract_bytes).unwrap(),
    )
    .expect("smith declarations");
    let fixed_declarations: u64 = schemas
        .iter()
        .flat_map(|schema| [&schema.name, &schema.description, &schema.schema])
        .map(|part| u64::try_from(part.len()).unwrap())
        .sum();
    let tools = schemas
        .into_iter()
        .map(|schema| mcp::Tool { name: schema.name, description: schema.description, input_schema: schema.schema })
        .chain(opened.host_tools.into_iter().map(|tool| mcp::Tool {
            name: tool.name,
            description: tool.description,
            input_schema: tool.schema,
        }))
        .collect();
    let mut below = Queue::uncapped();
    let server = mcp::Server::new(
        Token::new(1),
        0,
        &[0; 32],
        tools,
        mcp::Limits {
            message_bytes: limits.translation.client.request,
            answer_bytes: limits.translation.client.request,
            schema_bytes: u32::try_from(limits.contract_bytes).unwrap(),
            calls: 1,
            unauthenticated: Time::from_nanos(1),
        },
        &mut below,
    )
    .expect("MCP declarations render without running IO");
    (fixed_declarations, u64::try_from(server.tool_list().len()).unwrap())
}

#[test]
fn asks_first_request_fits_both_shapes() {
    let assignment = first_assignment(ASKING_CHARTER, ASKING_TEMPLATE);
    let configuration = fixtures::stock_configuration(TOOL_PAYLOAD, OUTPUT_TOKENS).expect("stock agent bounds");
    let limits = configuration.service.limits.domain.run;
    let charter = temper_engine_smith::charter(&assignment);
    let non_brief = limits
        .host_part
        .checked_sub(u64::from(run::brief_room(&charter, None, &limits).expect("stock host part")))
        .unwrap();
    assert_eq!(
        charter.grants.host_tools.iter().map(|tool| tool.name.as_ref()).collect::<Box<[_]>>().as_ref(),
        [b"see".as_slice(), b"document", b"remember", b"propose_settings"].as_slice()
    );
    let host_declarations: u64 = charter
        .grants
        .host_tools
        .iter()
        .flat_map(|tool| [&tool.name, &tool.description, &tool.schema])
        .map(|part| u64::try_from(part.len()).unwrap())
        .sum();
    let instructions = non_brief.checked_sub(host_declarations).unwrap();
    // A fresh Open run carries its gathered brief in the first user message;
    // charge those rendered sections once beside the system text.
    let brief = run::brief_bytes(&charter.brief).expect("gathered first-message brief");
    let outcome = charter.outcome.clone();
    let opened = opening(charter, &limits);
    let mechanics = u64::try_from(opened.system.len()).unwrap().checked_sub(instructions).unwrap();
    let (fixed_declarations, mcp_declarations) = declarations(opened, &outcome, &configuration.service.limits.llm);
    let mut within_bound = true;
    // The direct provider serializer is private. Its row uses smith's public
    // host-part measure and fixed schemas, without provider JSON framing.
    for (name, declarations) in
        [("chatgpt", host_declarations.checked_add(fixed_declarations).unwrap()), ("claude_code", mcp_declarations)]
    {
        let total =
            instructions.checked_add(mechanics).unwrap().checked_add(declarations).unwrap().checked_add(brief).unwrap();
        eprintln!(
            "{name}: instructions={instructions} mechanics={mechanics} declarations={declarations} brief={brief} total={total}"
        );
        within_bound &= total <= FIRST_REQUEST_BYTES;
    }
    assert!(within_bound, "first request exceeds the {FIRST_REQUEST_BYTES}-byte bound; parts printed for each shape");
}

#[test]
fn a_design_session_runs_the_drafter_with_no_change_or_goal() {
    let assignment = first_assignment(DRAFTER_CHARTER, DRAFTING_TEMPLATE);
    assert_eq!(assignment.run.policy.instructions.as_ref(), include_bytes!("../stock/drafter.txt"));
    assert!(assignment.workspace.repositories.is_empty());
    let charter = temper_engine_smith::charter(&assignment);
    for (name, offered) in [
        (b"goal".as_slice(), false),
        (b"change", false),
        (b"delegate", false),
        (b"note", true),
        (b"recall", true),
        (b"read_branch", true),
        (b"read_file", true),
    ] {
        assert_eq!(charter.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name), offered, "{name:?}");
    }
}
