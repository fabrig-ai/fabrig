// Fixture parsing without a deployment keeps written paths.
//! What the root's construction asserts and the worker's rules against the
//! agent, each refused at start naming its path (protocol/configuration/limits.md).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary test code edits UTF-8 configuration fixtures and reads their diagnostics"
)]
use super::fixtures;
use crate::startup::{Problems, checks, lines};

// The stock catalogue holds ten charters and six templates.
// Authority and task executor places hold its eighteen delegation kinds.
const ENGINE_OVERRIDES: &str = r#""tasks.charters":10,"core.templates":6,"authority.executors":18,"tasks.executor_kinds":18,"tasks.result_bytes":32,"tasks.run_inbox":2,"hub.turns":1,"people.amendment_bytes":64,"core.policy_bytes":8192,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1"#;
const WORKER_OVERRIDES: &str = r#""hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048"#;

fn started(document: &[u8]) -> Box<[u8]> {
    let found = match fixtures::start(
        document,
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    ) {
        Ok(service) => {
            drop(service);
            Problems::empty()
        }
        Err(problems) => problems,
    };
    lines(&found)
}
fn overridden(document: &[u8], path: &[u8], shipped: &str, extra: &str) -> Box<[u8]> {
    fixtures::with(document, path, format!("{{{shipped},{extra}}}").as_bytes())
}
fn engine_with(extra: &str) -> Box<[u8]> {
    overridden(&fixtures::document(), b"engine.limits.overrides", ENGINE_OVERRIDES, extra)
}
/// The fixture engine's root as the start makes it, held to `limits`.
fn root(limits: &temper_engine_domain::Limits) -> Box<[u8]> {
    let engine = fixtures::guard_engine();
    let mut found = Problems::empty();
    let seeds = fixtures::seeds();
    let core = jig_protocol_config::core(&engine.jig, seeds.deployment, seeds.seed);
    checks::root(core, &engine, &seeds, &[], limits, &mut found);
    lines(&found)
}
fn root_limits() -> temper_engine_domain::Limits {
    let engine = fixtures::guard_engine();
    crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap()
}

#[test]
fn the_fixture_document_starts() {
    assert_eq!(&*started(&fixtures::document()), b"");
    assert_eq!(&*root(&root_limits()), b"");
}
#[test]
fn a_document_seeding_no_project_starts() {
    let document = fixtures::with(&fixtures::document(), b"engine.projects", b"[]");
    let document = fixtures::with(&document, b"application.landing.projects", b"[]");
    assert_eq!(&*started(&document), b"");
}
#[test]
fn engine_slots_above_0_refuse_the_start() {
    assert_eq!(
        &*started(&engine_with(r#""fleet.engine_slots":1"#)),
        b"engine.limits.overrides.fleet.engine_slots: the root has no hub, so the fleet must place every run on a worker\n"
    );
}
#[test]
fn the_hub_check_refuses_a_charter_the_engine_may_host() {
    for hosts in [jig_domain_fleet::Kinds::Engine, jig_domain_fleet::Kinds::Both] {
        let mut engine = fixtures::guard_engine().jig;
        engine.charters[0].hosts = hosts;
        let mut found = Problems::empty();
        checks::hub(&engine, &mut found);
        assert_eq!(
            lines(&found).as_ref(),
            b"charter 1.hosts: the root has no hub, so the fleet must place every run on a worker\n"
        );
    }
}

#[test]
fn forge_count_past_the_accounts_credentials_refuses_the_start() {
    for (credentials, forges, accepted) in [(3, 3, true), (2, 3, false)] {
        let mut engine = fixtures::engine().jig;
        engine.limits.core.accounts.credentials = credentials;
        let mut problems = Problems::empty();
        checks::credentials(&engine, &[], forges, &mut problems);
        assert_eq!(problems.is_empty(), accepted);
        if !accepted {
            assert_eq!(
                problems.problems.get(0).unwrap().path,
                jig_protocol_config::Path::root()
                    .member(b"application")
                    .member(b"limits")
                    .member(b"overrides")
                    .member(b"forge.forges")
            );
        }
    }
}
#[test]
fn watched_names_do_not_change_checkout_channel_capacity() {
    for watched in [1, 3, 8] {
        let overrides = format!(r#"{{"forge.resources_per_task":{watched}}}"#);
        assert_eq!(
            &*started(&fixtures::with(&fixtures::document(), b"application.limits.overrides", overrides.as_bytes())),
            b"",
            "watched names: {watched}"
        );
        let mut own = crate::limits::profile(crate::limits::Profile::Small);
        own.forge.resources_per_task = watched;
        assert_eq!(
            crate::translate::limits(&own).expect("channel limits").items_repositories,
            own.checkout.repositories
        );
    }
}

#[test]
fn the_hubs_accounts_past_the_agents_name_both_limits() {
    let document =
        overridden(&fixtures::document(), b"workers[0].limits.overrides", WORKER_OVERRIDES, r#""hub.accounts":4096"#);
    assert_eq!(
        &*started(&document),
        b"workers[0].limits.overrides.hub.accounts: must agree with the agent's accounts\n"
    );
}
#[test]
fn the_checkouts_repositories_past_the_agents_directories_name_both_limits() {
    let mut agent = fixtures::startup_agent();
    agent.host_limits.directories = 1;
    let worker = fixtures::worker_settings();
    let own = crate::limits::profile(crate::limits::Profile::Small);
    let mut found = Problems::empty();
    checks::worker(&crate::worker_settings::root_limits(&worker, &agent, &own), &mut found);
    assert_eq!(
        &*lines(&found),
        b"application.limits.overrides.checkout.repositories: must agree with the agent's directories\n"
    );
}
#[test]
fn a_resume_limit_past_a_rows_bytes_refuses_the_start() {
    assert_eq!(
        &*started(&fixtures::with(
            &engine_with(r#""core.resume_bytes":65541,"hub.transcript_bytes":262161"#),
            b"workers[0].limits.overrides",
            format!("{{{WORKER_OVERRIDES},\"hub.transcript_bytes\":262161}}").as_bytes(),
        )),
        b"application.limits.overrides.root.journal.transcript_bytes: 65540 bytes, short of core.resume_bytes, 65541 bytes\n"
    );
}
#[test]
fn judges_tables_smaller_than_the_authoritys_refuse_the_start() {
    let mut limits = root_limits();
    limits.forge.judge_projects = limits.authority.projects.checked_sub(1).unwrap();
    assert_eq!(
        &*root(&limits),
        b"application.limits.overrides.forge.judge_projects: must agree with authority.projects in engine.limits\n"
    );
    let mut limits = root_limits();
    limits.forge.judge_criteria = limits.authority.requirements.checked_sub(1).unwrap();
    assert_eq!(
        &*root(&limits),
        b"application.limits.overrides.forge.judge_criteria: must agree with authority.requirements in engine.limits\n"
    );
}
#[test]
fn an_owners_project_without_an_escalation_role_refuses_the_start() {
    let mut engine = fixtures::guard_engine();
    engine.jig.projects[0].policy.escalation_role = None;
    let seeds = fixtures::seeds();
    let core = jig_protocol_config::core(&engine.jig, seeds.deployment, seeds.seed);
    let limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let mut found = Problems::empty();
    checks::root(core, &engine, &seeds, &[], &limits, &mut found);
    assert!(String::from_utf8_lossy(&lines(&found)).lines().any(|line| line
        == "engine.projects[0].policy.escalation_role: an owner's project must name an escalation role from 0 to 3"));
}
#[test]
fn a_charters_run_past_its_bytes_refuses_the_start() {
    let mut limits = root_limits();
    limits.journal.run_bytes = 16;
    let found = root(&limits);
    assert!(found.starts_with(b"charter 1: "), "{}", String::from_utf8_lossy(&found));
    assert!(found.ends_with(b" bytes, past root.journal.run_bytes of 16\n"), "{}", String::from_utf8_lossy(&found));
}
#[test]
fn permission_mappings_past_a_rows_bytes_refuse_the_start() {
    let mut limits = root_limits();
    limits.journal.transcript_bytes = 1;
    limits.forge.judge_projects = limits.authority.projects;
    let found = root(&limits);
    assert!(
        String::from_utf8_lossy(&found).lines().any(|line| line.starts_with("engine.projects[0].roles: ")
            && line.ends_with(" bytes, past root.journal.transcript_bytes of 1")),
        "{}",
        String::from_utf8_lossy(&found)
    );
}
#[test]
fn forges_past_the_clients_op_bytes_refuse_the_start() {
    let mut limits = root_limits();
    limits.forge.client.op_bytes = 1;
    assert_eq!(&*root(&limits), b"application.forges: must agree with forge.repositories and forge.client.op_bytes\n");
}
#[test]
fn a_deployment_provider_0_refuses_the_start() {
    let document = fixtures::with(&fixtures::document(), b"engine.providers[1].number", b"0");
    assert_eq!(
        &*started(&document),
        b"engine.providers[1].number: the deployment's provider must not be 0, the forge's sign-in provider\n"
    );
}

#[test]
fn configured_worker_count_past_membership_refuses_startup() {
    let mut limits = root_limits();
    limits.fleet.configured_workers = 1;
    assert_eq!(&*root(&limits), b"engine.hosts.workers: past its bound\n");
}
#[test]
fn configured_worker_name_past_membership_names_its_startup_path() {
    let mut limits = root_limits();
    limits.fleet.worker_name_bytes = 1;
    assert_eq!(&*root(&limits), b"engine.hosts.workers[0].name: past its bound\n");
}

#[test]
fn landing_rules_do_not_enlarge_the_seed_policy() {
    // Project 7 is document entry 0. Its policy fits without landing criteria.
    let document =
        fixtures::with(&fixtures::document(), b"engine.limits.overrides", format!("{{{ENGINE_OVERRIDES}}}").as_bytes());
    let document = fixtures::with(&document, b"engine.projects[0].number", b"7");
    let document = fixtures::with(&document, b"application.landing.projects[0].project", b"7");
    // The old small inline form is jig's input only; this guard takes typed values.
    let mut engine = fixtures::guard_engine();
    engine.jig = fixtures::parse_engine(&fixtures::with(fixtures::ENGINE, b"projects", fixtures::INLINE_PROJECTS));
    engine.jig.projects[0].number = 7;
    engine.jig.owners[0].project = 7;
    let parsed = jig_protocol_config::document(
        &document,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let at = jig_protocol_config::read::Object::new(&parsed, 0).unwrap().member(&parsed, b"application").unwrap();
    let mut found = Problems::empty();
    let application = crate::startup::application::parse(
        &parsed,
        at,
        &engine.jig,
        &jig_protocol_config::Directories::working(),
        &mut found,
    )
    .unwrap();
    engine.landings = application.landings;
    let seeds = fixtures::seeds();
    let core = jig_protocol_config::core(&engine.jig, seeds.deployment, seeds.seed);
    let mut limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    limits.policy_bytes = 512;
    checks::root(core, &engine, &seeds, &application.landing_projects, &limits, &mut found);
    assert!(found.is_empty(), "{}", String::from_utf8_lossy(&lines(&found)));
}

#[test]
fn unaccountable_policy_bytes_use_the_existing_bound_formatter() {
    let mut found = Problems::empty();
    found.push(
        jig_protocol_config::Path::root()
            .member(b"engine")
            .member(b"projects")
            .child(jig_protocol_config::Segment::Index(0))
            .member(b"policy")
            .member(b"bytes"),
        crate::startup::What::Jig(jig_protocol_config::What::Bound),
    );
    assert_eq!(lines(&found).as_ref(), b"engine.projects[0].policy.bytes: past its bound\n");
}

#[test]
fn the_small_engine_watermark_counts_commits_and_door_outputs() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let own = crate::limits::profile(crate::limits::Profile::Small);
    let root = crate::limits::engine(&jig, &own).unwrap();
    assert_eq!(root.watermark, own.root.journal.commits + own.root.journal.deliveries);
    assert_eq!(root.watermark, 515);
}

#[test]
fn judge_projects_past_the_wires_dismissals_name_both() {
    let mut limits = crate::limits::profile(crate::limits::Profile::Small);
    for (projects, expected) in [
        (1024, b"".as_slice()),
        (1025, b"application.limits.overrides.forge.judge_projects: must agree with wire::DISMISSED / 8\n".as_slice()),
    ] {
        limits.forge.judge_projects = projects;
        let mut found = Problems::empty();
        checks::limits(&limits, &limits.git_component, &mut found);
        assert_eq!(lines(&found).as_ref(), expected);
    }
}

/// The same typed worker limits that startup checks before allocating a service.
fn worker_limits() -> temper_worker_domain::Limits {
    crate::worker_settings::root_limits(
        &fixtures::worker_settings(),
        &fixtures::startup_agent(),
        &crate::limits::profile(crate::limits::Profile::Small),
    )
}

#[test]
fn payload_event_refusals_name_the_configured_and_required_bytes() {
    for (configured, expected) in [
        (4096, "4096 bytes, short of agent.json profile.declared.tool_payload less the 2-byte separator, 8192 bytes"),
        (8193, "8193 bytes, past agent.json profile.declared.tool_payload less the 2-byte separator of 8192"),
    ] {
        let mut limits = worker_limits();
        limits.agent.message_bytes = 8194;
        limits.bounds.message_bytes = 8192;
        limits.host.event_bytes = configured;
        let mut found = Problems::empty();
        checks::worker(&limits, &mut found);
        let expected = format!("workers[0].limits.overrides.hub.event_bytes: {expected}\n");
        assert_eq!(lines(&found).as_ref(), expected.as_bytes());
    }
}

#[test]
fn payload_checkout_refusals_name_the_configured_and_required_bytes() {
    let mut limits = worker_limits();
    limits.agent.call_bytes = 8194;
    limits.checkout.message_bytes = 4098;
    let mut found = Problems::empty();
    checks::worker(&limits, &mut found);
    assert_eq!(lines(&found).as_ref(), b"application.limits.overrides.checkout.message_bytes: 4098 bytes, short of the agent's call_bytes (agent.json profile.declared.tool_payload), 8194 bytes\n");
}

#[test]
fn payload_turn_queue_refusals_name_the_configured_and_required_bytes() {
    let mut limits = worker_limits();
    limits.agent.turns = 2;
    limits.host.turns = 2;
    limits.host.turn_bytes = 100;
    limits.host.turn_queue_bytes = 199;
    let mut found = Problems::empty();
    checks::worker(&limits, &mut found);
    assert_eq!(lines(&found).as_ref(), b"workers[0].limits.overrides.hub.turn_queue_bytes: 199 bytes, short of the agent's turns * workers[0].limits.overrides.hub.turn_bytes, 200 bytes\n");
}

fn complete_profile_startup(document: &[u8], agent: &crate::settings::Agent) {
    let named = crate::named(document, &jig_protocol_config::Directories::working()).unwrap();
    let mut entries = skein_lib::List::with_capacity(u32::try_from(named.files.len()).unwrap());
    for file in named.files {
        entries.push((file.path, skein_lib::Secret::new(vec![b'0'; 64].into()))).unwrap();
    }
    let found = crate::startup::check(
        document,
        &jig_protocol_config::Directories::working(),
        &crate::settings::Files { entries },
        agent,
        &fixtures::seeds(),
        None,
        skein_lib::List::with_capacity(0),
        None,
    );
    if let Err(problems) = found {
        panic!("{}", String::from_utf8_lossy(&lines(&problems)));
    }
}

/// Merge every shipped overlay member as protocol/configuration/reference.md requires.
fn merge_profile(base: &[u8], overlay: &[u8]) -> Box<[u8]> {
    use skein_json::writer::{Encoder, Limits};
    let bounds = jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    let base = jig_protocol_config::document(base, &bounds).unwrap();
    let overlay = jig_protocol_config::document(overlay, &bounds).unwrap();
    let mut tokens = Vec::new();
    merge_profile_members((&base, 0), (&overlay, 0), &mut tokens);
    let merged =
        skein_json::Document::from_tokens(&tokens, &skein_json::document::Limits { tokens: 16_384, text: 1_048_576 })
            .unwrap();
    let limits = Limits { depth: 32, length: 1_048_576 };
    let mut measure = Encoder::measure(&limits);
    measure.document(&merged);
    let mut writer = Encoder::write(measure.measured().unwrap(), &limits);
    writer.document(&merged);
    writer.finish()
}

fn merge_profile_members(
    base: (&skein_json::Document, u32),
    overlay: (&skein_json::Document, u32),
    tokens: &mut Vec<skein_json::Token>,
) {
    use skein_json::{Kind, Token};
    if base.0.token(base.1).unwrap().kind != Kind::ObjectStart
        || overlay.0.token(overlay.1).unwrap().kind != Kind::ObjectStart
    {
        copy_profile_value(overlay, tokens);
        return;
    }
    tokens.push(Token::ObjectStart);
    for (document, start, retained) in [(base.0, base.1, true), (overlay.0, overlay.1, false)] {
        let mut at = start.checked_add(1).unwrap();
        while document.token(at).unwrap().kind != Kind::ObjectEnd {
            let key = document.text(document.token(at).unwrap()).unwrap();
            let value = at.checked_add(1).unwrap();
            let previous = base.0.member(base.1, key);
            if retained || previous.is_none() {
                tokens.push(Token::Key(key.into()));
                if let (Some(original), Some(replacement)) = (previous, overlay.0.member(overlay.1, key)) {
                    merge_profile_members((base.0, original), (overlay.0, replacement), tokens);
                } else {
                    copy_profile_value((document, value), tokens);
                }
            }
            at = jig_protocol_config::read::after(document, value).unwrap();
        }
    }
    tokens.push(Token::ObjectEnd);
}

fn copy_profile_value(source: (&skein_json::Document, u32), tokens: &mut Vec<skein_json::Token>) {
    use skein_json::{Kind, Token};
    let (document, start) = source;
    for at in start..jig_protocol_config::read::after(document, start).unwrap() {
        let record = document.token(at).unwrap();
        let text = document.text(record).unwrap();
        tokens.push(match record.kind {
            Kind::ObjectStart => Token::ObjectStart,
            Kind::ObjectEnd => Token::ObjectEnd,
            Kind::ArrayStart => Token::ArrayStart,
            Kind::ArrayEnd => Token::ArrayEnd,
            Kind::Key => Token::Key(text.into()),
            Kind::String => Token::String(text.into()),
            Kind::Number => Token::Number(text.into()),
            Kind::True => Token::True,
            Kind::False => Token::False,
            Kind::Null => Token::Null,
            Kind::Long => panic!("profile text is retained"),
        });
    }
}

fn profile_question_bound(bytes: &[u8]) -> u32 {
    let document = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let jig = fixtures::parse_engine(&fixtures::value(bytes, b"engine"));
    let mut problems = Problems::empty();
    let application = crate::startup::application::parse(
        &document,
        document.member(0, b"application").unwrap(),
        &jig,
        &jig_protocol_config::Directories::working(),
        &mut problems,
    )
    .unwrap();
    assert_eq!(lines(&problems).as_ref(), b"");
    crate::limits::engine(&jig.limits, &application.limits).unwrap().forge.change_policy.questions
}

/// The reference supplies four questions over init's standard eighty, before the first start.
#[test]
fn the_reference_profile_keeps_four_questions_per_change() {
    let generated = super::stock::generated();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(10_662, 8192).expect("init agent"));
    assert_eq!(profile_question_bound(&generated), 80, "init keeps standard's bound");
    complete_profile_startup(&generated, &agent);
    let reference = reference_document(&generated);
    complete_profile_startup(&reference, &reference_agent());
    assert_eq!(profile_question_bound(&reference), 4, "the reference overrides standard's bound");
}

#[test]
fn shipped_payload_profiles_pass_complete_startup_checks() {
    let generated = super::stock::generated();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(10_662, 8192).expect("stock agent"));
    complete_profile_startup(&generated, &agent);
    complete_profile_startup(&reference_document(&generated), &reference_agent());
}

#[test]
fn raised_checkout_profiles_pass_complete_startup_checks() {
    let mut stock: serde_json::Value = serde_json::from_slice(&super::stock::generated()).unwrap();
    stock["application"]["limits"]["overrides"]["checkout.conflicts"] = 1024_u32.into();
    stock["application"]["limits"]["overrides"]["git_component.stdout_bytes"] = 262_144_u32.into();
    let stock_agent = fixtures::parsed_agent(fixtures::stock_configuration(10_662, 8192).expect("stock agent"));
    complete_profile_startup(&serde_json::to_vec(&stock).unwrap(), &stock_agent);
    let generated = fixtures::document();
    let mut agent = fixtures::startup_agent();
    agent.host_limits.conflicts = 1024;
    let mut small: serde_json::Value = serde_json::from_slice(&generated).unwrap();
    let limits = &mut small["application"]["limits"]["overrides"];
    limits["checkout.repositories"] = 2_u32.into();
    limits["checkout.conflicts"] = 1024_u32.into();
    limits["git_component.stdout_bytes"] = 262_144_u32.into();
    complete_profile_startup(&serde_json::to_vec(&small).unwrap(), &agent);
}

#[test]
fn a_16386_byte_payload_requires_a_larger_model_answer() {
    let Err(refused) = fixtures::stock_configuration(16_386, 8192) else {
        panic!("the input must fit the model answer");
    };
    assert!(refused.contains("input-answer"), "{refused}");
    assert!(refused.contains("llm.input[endpoint=1]=99876"), "{refused}");
    assert!(refused.contains("llm.answer[endpoint=1]=65536"), "{refused}");
    let raised = fixtures::stock_configuration(16_386, 16_384).expect("a supported larger output admits the payload");
    assert_eq!(raised.profile.declared.tool_payload, 16_386);
}

fn reference_room() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/room.json")).unwrap()
}

fn reference_agent() -> crate::settings::Agent {
    let room = reference_room();
    fixtures::parsed_agent(
        fixtures::stock_configuration(
            u32::try_from(room["agent"]["tool_payload"].as_u64().unwrap()).unwrap(),
            u32::try_from(room["agent"]["output"].as_u64().unwrap()).unwrap(),
        )
        .expect("reference agent"),
    )
}

fn reference_document(generated: &[u8]) -> Box<[u8]> {
    let merged = merge_profile(generated, include_bytes!("../../../../docs/examples/reference/limits.json"));
    let mut document: serde_json::Value = serde_json::from_slice(&merged).unwrap();
    let room = reference_room();
    let overlay = serde_json::to_vec(&room["worker"]).unwrap();
    for worker in document["workers"].as_array_mut().unwrap() {
        *worker = serde_json::from_slice(&merge_profile(&serde_json::to_vec(worker).unwrap(), &overlay)).unwrap();
    }
    document["application"] = serde_json::from_slice(&merge_profile(
        &serde_json::to_vec(&document["application"]).unwrap(),
        &serde_json::to_vec(&room["application"]).unwrap(),
    ))
    .unwrap();
    serde_json::to_vec(&document).unwrap().into_boxed_slice()
}

#[test]
fn the_reference_room_starts_and_holds_its_relations() {
    // Measured CPU: 0.041 s for profile derivation, complete startup and the relations.
    let room = reference_room();
    let agent = reference_agent();
    let document = reference_document(&super::stock::generated());
    complete_profile_startup(&document, &agent);
    let document: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let engine = &document["engine"]["limits"]["overrides"];
    let own = &document["application"]["limits"]["overrides"];
    let payload = u64::from(agent.tool_payload);
    let output = room["agent"]["output"].as_u64().unwrap();
    let turn = u64::from(agent.limits.bodies.turn_body);
    let resume = turn + 4;
    let transcript = resume + 12 * (resume / 4);
    let loaded = turn + u64::try_from(size_of::<temper_engine_domain::Record>()).unwrap();
    assert_eq!((payload, output, turn), (63_740, 48_000, 18_431_842));
    assert!((payload + 260) * 6 <= output * 8);
    for (name, value) in [
        ("core.resume_bytes", resume),
        ("hub.transcript_bytes", transcript),
        ("hub.turn_bytes", turn),
        ("hub.turn_queue_bytes", 8 * turn),
        ("tasks.message_bytes", payload - 2),
        ("tasks.result_bytes", payload - 2),
        ("tasks.inbox_bytes", 82 * (payload - 2)),
    ] {
        assert_eq!(engine[name].as_u64(), Some(value), "engine {name}");
    }
    for worker in document["workers"].as_array().unwrap() {
        for (name, value) in [
            ("hub.event_bytes", payload - 2),
            ("hub.transcript_bytes", transcript),
            ("hub.turn_bytes", turn),
            ("hub.turn_queue_bytes", 8 * turn),
        ] {
            assert_eq!(worker["limits"]["overrides"][name].as_u64(), Some(value), "worker {name}");
        }
    }
    for (name, value) in [
        ("checkout.message_bytes", payload),
        ("forge.client.op_bytes", payload - 2),
        ("forge_component.body_bytes", 2 * (payload - 2) + 21),
        // A pull-file page keeps a patch and a previous name beside its path and row.
        (
            "forge.client.answer_bytes",
            2 * (2 * (payload - 2) + 21)
                + u64::from(crate::limits::profile(crate::limits::Profile::Standard).forge_component.string_bytes)
                + u64::try_from(skein_forge::measure::FILE_BYTES).unwrap(),
        ),
        ("root.journal.run_bytes", 65_536),
        ("root.journal.transcript_bytes", resume),
        ("root.loads.bytes", loaded),
        ("root.loads.reply_bytes", loaded),
        ("root.loads.read_bytes", 2 * turn),
        ("root.loads.transcript_bytes", turn),
    ] {
        assert_eq!(own[name].as_u64(), Some(value), "application {name}");
    }
    let settings: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/settings.json")).unwrap();
    for item in settings["items"].as_array().unwrap() {
        match item["setting"]["kind"].as_str().unwrap() {
            "charter" => {
                assert_eq!(item["set"]["model"]["output"].as_u64(), Some(output));
                for model in item["set"]["alternatives"].as_array().unwrap() {
                    assert_eq!(model["output"].as_u64(), Some(output));
                }
            }
            "template" if item["setting"]["number"] == 4_u32 => {
                assert_eq!(item["set"]["contract"]["report"]["words"].as_u64(), Some(payload - 2));
            }
            _ => {}
        }
    }
}
