//! Run reads join the core's settings documents with the forge's values
//! (temper/docs/design/domain/agent.md, 8.1).
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

const SEED: u64 = 9761;

fn landing_value() -> temper_engine_domain_forge::settings::Value {
    use temper_engine_domain_forge::{GateLast, GatePattern, settings};
    settings::Value::Rule(settings::LandingRule {
        pattern: GatePattern {
            segments: Box::new([Box::from(b"forge".as_slice())]),
            last: GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: true,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    })
}

fn settings_agent_config(allowed: bool) -> engine::Config {
    let mut config = walking::config(SEED);
    if allowed {
        config.core.defaults.rules.ceiling.tools.0 |= core::ToolFamilies::JIG.settings;
        for project in &mut config.core.seed_projects {
            project.policy.policy.ceiling.tools.0 |= core::ToolFamilies::JIG.settings;
            for role in &mut project.policy.policy.roles {
                role.authority.tools.0 |= core::ToolFamilies::JIG.settings;
            }
        }
    }
    config.core.kinds = temper_service::stock::kinds(&walking::limits(), config.core.connectors[0]);
    config.kinds_stock = engine::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 1);
    let mut stock = config.kinds_stock.into_vec();
    stock.push(temper_engine_domain_forge::settings::Stock {
        kind: 1,
        scope: 0,
        number: 1,
        value: temper_engine_domain_forge::settings::SizedValue { size: 0, value: landing_value() },
    });
    config.kinds_stock = stock.into_boxed_slice();
    // The deployment owner creates the project through the root, acquiring
    // its project role within the fixture's existing owner and holding bounds.
    config.core.owners[0].project = people::DEPLOYMENT;
    let mut policy = config.core.seed_projects[0].policy.clone();
    policy.permissions = Box::new([]);
    config.core.seed_projects = Box::new([]);
    config.core.defaults.policies.insert(1, policy).expect("one default policy");
    config
}

fn settings_agent(allowed: bool) -> (Driver, engine::Assignment) {
    let limits = walking::limits();
    let config = settings_agent_config(allowed);
    let accounts = temper_engine_domain_world::fixtures::account_rows(&config.core.settings.llm_providers);
    let mut bootstrap = Driver::cold(Store::new(), config, &limits);
    bootstrap.period_sent = true;
    bootstrap.settle();
    let mut store = bootstrap.take_store();
    store.seed(accounts, Box::new([]));
    let mut driver = Driver::cold(store, settings_agent_config(allowed), &limits);
    driver.settle();
    driver.hello(Token::new(7));
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(991)),
        sign_in: driver.session(),
        key: [91; 16],
        ask: people::Ask::MakeProject { name: "Settings project".into(), policy: 1 },
    }));
    driver.settle();
    driver.period();
    driver.settle();
    driver.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(992)),
        sign_in: driver.session(),
        key: [92; 16],
        ask: walking::configured_chat(
            &jig_test_profile::fixtures::Creating::new(&walking::config(SEED).core, walking::limits().tasks.tree_tasks),
            &driver.store.rows,
            driver.env.limits.tasks.tree_tasks,
            1,
            b"Read settings",
        ),
    }));
    driver.settle();
    let run = driver.last_assignment();
    (driver, run)
}

fn host_tool(
    driver: &mut Driver,
    run: &engine::Assignment,
    completion: u32,
    tool: &[u8],
    input: &[u8],
) -> (Box<[u8]>, bool) {
    use jig_domain_host as host;
    use smith_domain_run as smith;
    use temper_engine_domain_world::protocol;
    let mut name = skein_lib::Writer::new(16);
    name.put(&run.attempt.to_be_bytes()).expect("activation");
    name.put(&completion.to_be_bytes()).expect("completion");
    name.put(&0_u32.to_be_bytes()).expect("position");
    let name = name.finish();
    driver.protocol.defer_render = true;
    driver.send(protocol::up(
        Token::new(7),
        host::LinkUp::Call {
            run: Token::new(run.task),
            attempt: Token::new(run.attempt),
            call: name.clone(),
            tool: tool.into(),
            writes: tool != b"settings",
            input: input.into(),
            deadline: Duration::from_secs(10),
        },
        engine::Parts::none(),
    ));
    driver.pump();
    let request = driver.host_requests.pop().expect("host decoder called");
    let engine::HostRequest::Decode { to, call, .. } = *request else { panic!("decode") };
    assert_eq!(call.name, name);
    let input = smith::HostInput::attested(call.input).expect("host input");
    let decoded = temper_engine_smith::call_configured(
        smith::CallName { activation: run.attempt, completion, position: 0 },
        &call.tool,
        &input,
        driver.root.models(),
    )
    .expect("valid host call");
    let temper_engine_smith::Decoded::Call(body) = decoded else { panic!("valid settings tool enters the root") };
    driver.send(Event::Worker(engine::WorkerInput::DecodedHostCall { to, body }));
    driver.pump();
    let mut settings_delivery = None;
    for (index, delivery) in driver.delivered.iter().enumerate() {
        if let Delivery::Host(delivery) = delivery
            && matches!(delivery.as_ref(), engine::HostDelivery::Settings { .. })
        {
            settings_delivery = Some(index);
            break;
        }
    }
    let (to, answer) = if let Some(index) = settings_delivery {
        let Delivery::Host(delivery) = driver.delivered.remove(index) else { panic!("settings delivery") };
        let engine::HostDelivery::Settings { to, answer, application, from, requested } = *delivery else {
            panic!("settings read")
        };
        (to, temper_engine_smith::settings_answer(&answer, from, requested, driver.root.models(), application.as_ref()))
    } else {
        let (to, rendered_name) = driver.protocol.pending_renders.pop().expect("host answer to render");
        assert_eq!(rendered_name, name);
        let (_, answer) = driver.protocol.answers.last().expect("typed answer");
        (to, temper_engine_smith::answer(answer))
    };
    let (body, error) = answer.into_parts();
    driver.send(Event::Worker(engine::WorkerInput::RenderedHostCall {
        to,
        answer: core::SettledAnswer::Host { error, body },
    }));
    driver.pump();
    for delivery in driver.delivered.iter().rev() {
        if let Delivery::Link {
            down: host::LinkDown::Relayed { call, answer: host::Relayed::Answer { text, error }, .. },
            ..
        } = delivery
            && *call == name
        {
            return (text.clone(), *error);
        }
    }
    panic!("settings tool answer reached the run");
}

#[test]
fn a_run_reads_a_landing_rule_and_the_forges_charter() {
    let (mut driver, run) = settings_agent(true);
    let original = driver.store.rows.clone();
    for (completion, input, heading, value) in [
        (
            1,
            br#"{"setting":{"kind":"landing-rule","number":1}}"#.as_slice(),
            b"landing-rule 1, revision 1\n".as_slice(),
            br#"{"pattern":{"segments":["666f726765"],"last":{"open":""}},"enforce":true,"ci":true,"up_to_date":false,"gates":[],"approvals":[]}"#.as_slice(),
        ),
        (
            2,
            br#"{"setting":{"kind":"forge-charter","number":1}}"#.as_slice(),
            b"forge-charter 1, revision 1\n".as_slice(),
            b"1".as_slice(),
        ),
    ] {
        let (answer, error) = host_tool(&mut driver, &run, completion, b"settings", input);
        assert!(!error, "{}", String::from_utf8_lossy(&answer));
        assert!(answer.len() <= usize::try_from(temper_engine_smith::DOC_ANSWER).expect("host answer bound"));
        assert!(answer.starts_with(heading), "{}", String::from_utf8_lossy(&answer));
        let rest = answer.strip_prefix(heading).expect("document heading");
        assert!(rest.starts_with(value), "{}", String::from_utf8_lossy(&answer));
        assert!(rest[value.len()..].starts_with(b"\ndefault, "));
        assert!(answer.ends_with(b" bytes\nend\n"));
    }
    assert_eq!(driver.store.rows, original, "settings reads decide and save nothing");
}

#[test]
fn a_run_without_the_settings_family_cannot_read_a_forge_setting() {
    let (mut driver, run) = settings_agent(false);
    let original = driver.store.rows.clone();
    let (answer, error) =
        host_tool(&mut driver, &run, 1, b"settings", br#"{"setting":{"kind":"landing-rule","number":1}}"#);
    assert!(error, "{}", String::from_utf8_lossy(&answer));
    let (_, part) = driver.protocol.answers.last().expect("family denial");
    assert!(matches!(part, engine::CallAnswer::ToolDenied { .. }));
    assert_eq!(driver.store.rows, original, "a refused read saves no call record");
}

#[test]
fn a_runs_project_policy_and_unavailable_revision_reads_stay_unknown() {
    let (mut driver, run) = settings_agent(true);
    for (completion, input) in [
        (1, br#"{"setting":{"kind":"project-policy","project":1}}"#.as_slice()),
        (2, br#"{"setting":{"kind":"landing-rule","number":1},"revision":2}"#.as_slice()),
    ] {
        let (answer, error) = host_tool(&mut driver, &run, completion, b"settings", input);
        assert!(error);
        assert_eq!(answer.as_ref(), b"unknown\nend\n");
    }
}

#[test]
fn a_runs_held_revision_does_not_receive_the_newest_forge_value() {
    use temper_engine_domain_world::kinds;
    let (mut driver, run) = settings_agent(true);
    let temper_engine_domain_forge::settings::Value::Rule(mut rule) = landing_value() else { panic!("rule fixture") };
    rule.ci = false;
    driver.send(kinds::change(
        driver.session(),
        1,
        93,
        Box::new([kinds::item(1, 0, 1, &temper_engine_domain_forge::settings::Value::Rule(rule))]),
    ));
    driver.settle();
    assert_eq!(kinds::revision(&driver.store.rows), 2, "the new rule committed");
    let (answer, error) =
        host_tool(&mut driver, &run, 1, b"settings", br#"{"setting":{"kind":"landing-rule","number":1},"revision":1}"#);
    assert!(error);
    assert_eq!(answer.as_ref(), b"unknown\nend\n", "the held revision keeps the core's answer");
    let (answer, error) =
        host_tool(&mut driver, &run, 2, b"settings", br#"{"setting":{"kind":"landing-rule","number":1}}"#);
    assert!(!error);
    assert!(answer.starts_with(b"landing-rule 1, revision 2\n"));
    assert!(skein_lib::bytes::find(&answer, br#""ci":false"#).is_some());
    assert!(answer.ends_with(b" bytes\nend\n"));
}
