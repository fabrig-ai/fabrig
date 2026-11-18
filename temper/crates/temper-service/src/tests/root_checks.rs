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

// The stock catalogue holds twelve charters and seven templates.
// Authority and task executor places hold its eighteen delegation kinds.
const ENGINE_OVERRIDES: &str = r#""tasks.charters":13,"core.templates":7,"authority.executors":22,"tasks.executor_kinds":22,"tasks.result_bytes":32,"tasks.run_inbox":2,"hub.turns":1,"people.amendment_bytes":64,"core.policy_bytes":8192,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1"#;

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
    let core = temper_engine_service::startup::core(&engine.jig, seeds.deployment, seeds.seed);
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
    let document = fixtures::document();
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
        let mut own = temper_limits_fixtures::small();
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
        overridden(&fixtures::document(), b"engine.limits.overrides", ENGINE_OVERRIDES, r#""hub.accounts":4096"#);
    assert_eq!(
        &*started(&document),
        b"workers[0].limits.overrides.hub.accounts: must agree with the agent's accounts\n"
    );
}

#[test]
fn a_resume_limit_past_a_rows_bytes_refuses_the_start() {
    assert_eq!(
        &*started(&engine_with(r#""core.resume_bytes":65541,"hub.transcript_bytes":262161"#)),
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
    let core = temper_engine_service::startup::core(&engine.jig, seeds.deployment, seeds.seed);
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
    let mut engine = fixtures::guard_engine();
    engine.jig.providers[1].number = 0;
    let mut found = Problems::empty();
    checks::provider(&engine.jig, &mut found);
    assert_eq!(
        lines(&found).as_ref(),
        b"engine.providers[1].number: the deployment's provider must not be 0, the forge's sign-in provider\n"
    );
}

#[test]
fn configured_worker_count_past_membership_refuses_startup() {
    let mut limits = root_limits();
    limits.fleet.configured_workers = 0;
    assert_eq!(&*root(&limits), b"engine.limits.overrides.fleet.configured_workers: past its bound\n");
}
#[test]
fn configured_worker_name_past_membership_names_its_startup_path() {
    let mut limits = root_limits();
    limits.fleet.worker_name_bytes = 4;
    assert_eq!(&*root(&limits), b"engine.limits.overrides.fleet.worker_name_bytes: past its bound\n");
}

#[test]
fn landing_rules_do_not_enlarge_the_seed_policy() {
    // Project 7 is typed seed entry 0. Its policy fits without landing criteria.
    let document =
        fixtures::with(&fixtures::document(), b"engine.limits.overrides", format!("{{{ENGINE_OVERRIDES}}}").as_bytes());
    let mut engine = fixtures::guard_engine();
    let project = &mut engine.jig.projects[0];
    project.policy.ceiling.grants = Box::default();
    project.policy.ceiling.delegation.kinds = Box::new([jig_domain_authority::Executor::Charter(1)]);
    project.policy.ceiling.delegation.depth = 4;
    project.policy.ceiling.delegation.tasks = 8;
    project.policy.ceiling.tools = jig_domain_authority::Tools(1);
    project.policy.ceiling.notes = jig_domain_authority::Scopes(0);
    project.policy.ceiling.note_resources = Box::default();
    project.policy.ceiling.budget.spend = 100;
    let mut role = project.policy.roles[0].clone();
    role.authority = project.policy.ceiling.clone();
    role.period_spend = 100;
    project.policy.roles = Box::new([role]);
    project.policy.period_spend = 100;
    project.policy.projections = Box::default();
    project.policy.requirements = Box::default();
    project.roles = Box::new([jig_domain_people::PermissionRole { connector: 1, permission: 1, role: 0 }]);
    engine.jig.projects[0].number = 7;
    engine.jig.owners[0].project = 7;
    let parsed = jig_protocol_config::document(
        &document,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let at = jig_protocol_config::read::Object::new(&parsed, 0).unwrap().member(&parsed, b"application").unwrap();
    let mut found = Problems::empty();
    let _application = crate::startup::application::parse(
        &parsed,
        at,
        &engine.jig,
        &jig_protocol_config::Directories::working(),
        &mut found,
    )
    .unwrap();
    let seeds = fixtures::seeds();
    let core = temper_engine_service::startup::core(&engine.jig, seeds.deployment, seeds.seed);
    let mut limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    limits.policy_bytes = 512;
    checks::root(core, &engine, &seeds, &[], &limits, &mut found);
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
    let jig = jig_protocol_config_fixtures::small();
    let own = temper_limits_fixtures::small();
    let root = crate::limits::engine(&jig, &own).unwrap();
    assert_eq!(root.watermark, own.root.journal.commits + own.root.journal.deliveries);
    assert_eq!(root.watermark, 515);
}

#[test]
fn judge_projects_past_the_wires_dismissals_name_both() {
    let mut limits = temper_limits_fixtures::small();
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

fn complete_profile_startup(document: &[u8], agent: &crate::settings::Agent) {
    drop(checked_profile(document, agent));
}
fn checked_profile(document: &[u8], agent: &crate::settings::Agent) -> crate::settings::Engine {
    let named = fixtures::named(document, &jig_protocol_config::Directories::working()).unwrap();
    let mut entries = skein_lib::List::with_capacity(u32::try_from(named.files.len()).unwrap());
    for file in named.files {
        entries.push((file.path, skein_lib::Secret::new(vec![b'0'; 64].into()))).unwrap();
    }
    checked_profile_with_files(document, agent, &crate::settings::Files { entries })
}
fn checked_profile_with_files(
    document: &[u8],
    agent: &crate::settings::Agent,
    files: &crate::settings::Files,
) -> crate::settings::Engine {
    match crate::startup::checked_start_with(
        document,
        &jig_protocol_config::Directories::working(),
        files,
        Some(agent),
        &fixtures::seeds(),
        skein_lib::List::with_capacity(0),
        None,
        &fixtures::given(document, &jig_protocol_config::Directories::working()),
        None,
    ) {
        Ok(checked) => checked.engine,
        Err(problems) => panic!("{}", String::from_utf8_lossy(&lines(&problems))),
    }
}

/// Merge every shipped overlay member as protocol/configuration/reference.md requires.
fn merge_profile_members(base: &mut serde_json::Value, overlay: serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base), serde_json::Value::Object(overlay)) => {
            for (name, value) in overlay {
                if let Some(previous) = base.get_mut(&name) {
                    merge_profile_members(previous, value);
                } else {
                    base.insert(name, value);
                }
            }
        }
        (base, overlay) => *base = overlay,
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
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).expect("init agent"));
    assert_eq!(profile_question_bound(&generated), 80, "init keeps standard's bound");
    complete_profile_startup(&generated, &agent);
    let reference = reference_document(&generated);
    complete_profile_startup(&reference, &reference_agent());
    assert_eq!(profile_question_bound(&reference), 4, "the reference overrides standard's bound");
}

#[test]
fn shipped_payload_profiles_pass_complete_startup_checks() {
    let generated = super::stock::generated();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).expect("stock agent"));
    complete_profile_startup(&generated, &agent);
    complete_profile_startup(&reference_document(&generated), &reference_agent());
}

#[test]
fn raised_checkout_profiles_pass_complete_startup_checks() {
    let mut stock: serde_json::Value = serde_json::from_slice(&super::stock::generated()).unwrap();
    stock["application"]["limits"]["overrides"]["checkout.conflicts"] = 1024_u32.into();
    stock["application"]["limits"]["overrides"]["git_component.stdout_bytes"] = 262_144_u32.into();
    let stock_agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).expect("stock agent"));
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
    let mut document: serde_json::Value = serde_json::from_slice(generated).unwrap();
    merge_profile_members(
        &mut document,
        serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/limits.json")).unwrap(),
    );
    let room = reference_room();
    merge_profile_members(&mut document["engine"]["limits"], room["worker"]["limits"].clone());
    merge_profile_members(&mut document["application"], room["application"].clone());
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
    assert_eq!(engine["hub.event_bytes"].as_u64(), Some(payload - 2));
    for worker in document["workers"].as_array().unwrap() {
        assert!(worker.get("limits").is_none());
    }
    for (name, value) in [
        ("checkout.message_bytes", payload),
        ("forge.client.op_bytes", payload - 2),
        ("forge_component.body_bytes", 2 * (payload - 2) + 21),
        // A pull-file page keeps a patch and a previous name beside its path and row.
        (
            "forge.client.answer_bytes",
            2 * (2 * (payload - 2) + 21)
                + u64::from(crate::limits::stock().forge_component.string_bytes)
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
    let parsed = super::stock::engine(&serde_json::to_vec(&document).expect("reference document"));
    let defaults = crate::stock::with_models(parsed.llm.models.clone(), parsed.limits.core.tasks.result_bytes);
    for number in [5, 2] {
        let charter = super::reference_settings::charter(&defaults, number);
        assert_eq!(u64::from(charter.policy.model.output), output);
        for model in &charter.policy.alternatives {
            assert_eq!(u64::from(model.output), output);
        }
    }
    let settings: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/settings.json")).unwrap();
    for item in settings["items"].as_array().unwrap() {
        match item["setting"]["kind"].as_str().unwrap() {
            "template" if item["setting"]["number"] == 4_u32 => {
                assert_eq!(item["set"]["contract"]["report"]["words"].as_u64(), Some(payload - 2));
            }
            _ => {}
        }
    }
}

#[test]
fn landing_members_fits_the_prepare_prefixes_channel_envelope() {
    let mut limits = root_limits();
    limits.forge.landing_members = 5;
    assert_eq!(&*root(&limits), b"", "five prefixes fit");
    limits.forge.landing_members = 6;
    let words = root(&limits);
    assert!(!words.is_empty(), "a sixth prefix exceeds the wire envelope");
    assert!(String::from_utf8_lossy(&words).contains("forge.landing_members"));
}

#[test]
fn init_documents_start_without_carrier_overrides() {
    let generated = super::stock::generated();
    let original: serde_json::Value = serde_json::from_slice(&generated).unwrap();
    let mut input = original.clone();
    let jig_names = [
        "core.resume_bytes",
        "hub.turn_bytes",
        "hub.transcript_bytes",
        "hub.turn_queue_bytes",
        "io.output",
        "tasks.message_bytes",
        "tasks.result_bytes",
        "tasks.inbox_bytes",
        "hub.event_bytes",
        "hub.outcome_bytes",
        "core.call_answer_bytes",
    ];
    for at in ["/engine/limits/overrides"] {
        let overrides = input.pointer_mut(at).unwrap().as_object_mut().unwrap();
        for name in jig_names {
            overrides.remove(name);
        }
    }
    let own_names = [
        "root.journal.transcript_bytes",
        "root.loads.transcript_bytes",
        "root.loads.bytes",
        "root.loads.reply_bytes",
        "root.loads.read_bytes",
        "checkout.message_bytes",
        "checkout.workspaces",
        "git_component.operations",
        "forge.client.op_bytes",
        "forge.client.answer_bytes",
        "forge_component.body_bytes",
    ];
    for name in own_names {
        input["application"]["limits"]["overrides"].as_object_mut().unwrap().remove(name);
    }
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap());
    let bytes = serde_json::to_vec(&input).unwrap();
    let checked = checked_profile_with_files(&bytes, &agent, &super::stock::files());
    for name in jig_names {
        if let Some(previous) = original["engine"]["limits"]["overrides"][name].as_u64() {
            let named = jig_protocol_config::names::find(name.as_bytes()).unwrap();
            let expected = if name == "io.output" {
                previous.max(u64::from(temper_service_core::derived::OUTPUT_FLOOR))
            } else {
                previous
            };
            assert_eq!((named.value)(&checked.jig.limits), expected, "engine {name}");
        }
    }
    for name in own_names {
        if let Some(previous) = original["application"]["limits"]["overrides"][name].as_u64() {
            let named = crate::limits::find(name.as_bytes()).unwrap();
            assert_eq!((named.value)(&checked.limits), previous, "application {name}");
        }
    }
    let worker_bytes = serde_json::to_vec(&input["workers"][0]).unwrap();
    let document = jig_protocol_config::document(
        &worker_bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let worker = temper_service_core::derived::worker_document(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        temper_service_core::derived::Sources::agent(&agent.declaration()),
    )
    .unwrap();
    for name in jig_names {
        if let Some(previous) = original["workers"][0]["limits"]["overrides"][name].as_u64() {
            let named = jig_protocol_config::names::find(name.as_bytes()).unwrap();
            let expected = if name == "io.output" {
                previous.max(u64::from(temper_service_core::derived::OUTPUT_FLOOR))
            } else {
                previous
            };
            assert_eq!((named.value)(worker.limits.as_ref().unwrap()), expected, "worker {name}");
        }
    }
}

#[test]
fn reference_overrides_stand_over_the_derived_base() {
    let input = reference_document(&super::stock::generated());
    let agent = reference_agent();
    let checked = checked_profile_with_files(&input, &agent, &super::stock::files());
    let document: serde_json::Value = serde_json::from_slice(&input).unwrap();
    for (name, expected) in document["engine"]["limits"]["overrides"].as_object().unwrap() {
        let named = jig_protocol_config::names::find(name.as_bytes()).unwrap();
        assert_eq!((named.value)(&checked.jig.limits), expected.as_u64().unwrap(), "engine {name}");
    }
    for (name, expected) in document["application"]["limits"]["overrides"].as_object().unwrap() {
        let named = crate::limits::find(name.as_bytes()).unwrap();
        assert_eq!((named.value)(&checked.limits), expected.as_u64().unwrap(), "application {name}");
    }
}

#[test]
fn box_start_refuses_twice_the_stock_turn_without_panicking() {
    let document = super::stock::generated();
    let mut agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap());
    agent.limits.bodies.turn_body = temper_service_core::derived::STOCK.turn * 2;
    let found = crate::startup::checked_start_with(
        &document,
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        Some(&agent),
        &fixtures::seeds(),
        skein_lib::List::with_capacity(0),
        None,
        &crate::stock::Given::Stock,
        None,
    )
    .err()
    .unwrap();
    assert_eq!(lines(&found).as_ref(), b"engine.limits.overrides.io.output: cannot derive from the agent's largest turn (T) within the worker wire's ceilings\n");
}

#[test]
fn forge_body_follows_the_effective_operation_unless_overridden() {
    let jig = fixtures::guard_engine().jig;
    for (overrides, expected) in [
        (r#"{"forge.client.op_bytes":1000}"#, 2021),
        (r#"{"forge.client.op_bytes":1000,"forge_component.body_bytes":3000}"#, 3000),
    ] {
        let input = fixtures::with(&fixtures::document(), b"application.limits.overrides", overrides.as_bytes());
        let input = core::str::from_utf8(&input).unwrap().replace(",\"forge_component.body_bytes\":16384", "");
        let document = jig_protocol_config::document(
            input.as_bytes(),
            &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
        )
        .unwrap();
        let mut problems = Problems::empty();
        let own = crate::startup::application::parse(
            &document,
            document.member(0, b"application").unwrap(),
            &jig,
            &jig_protocol_config::Directories::working(),
            &mut problems,
        )
        .unwrap();
        assert!(problems.is_empty());
        assert_eq!(own.limits.forge_component.body_bytes, expected);
    }
}
