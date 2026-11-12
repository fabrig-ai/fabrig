#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary step tests generate documents and compare embedded instruction bytes"
)]
use super::fixtures;
use jig_protocol_config as config;

/// Replace one engine override, keeping every other generated entry.
pub(super) fn limited(document: &[u8], name: &[u8], value: u64) -> Box<[u8]> {
    let original = fixtures::value(document, b"engine.limits.overrides");
    let parsed =
        config::document(&original, &config::DocumentLimits { bytes: 16_384, tokens: 4096, depth: 4 }).unwrap();
    let mut overrides = format!("{{\"{}\":{value}", core::str::from_utf8(name).unwrap());
    for at in (1..parsed.len().checked_sub(1).unwrap()).step_by(2) {
        let key = parsed.text(parsed.token(at).unwrap()).unwrap();
        if key != name {
            let value = config::read::integer(&parsed, at.checked_add(1).unwrap(), u64::MAX).unwrap();
            overrides.push_str(",\"");
            overrides.push_str(core::str::from_utf8(key).unwrap());
            overrides.push_str("\":");
            overrides.push_str(&value.to_string());
        }
    }
    overrides.push('}');
    fixtures::with(document, b"engine.limits.overrides", overrides.as_bytes())
}

#[test]
fn the_stock_passes_jigs_one_check() {
    let init = generated();
    let without_demo = fixtures::with(&init, b"application.demo", b"null");
    for (row, document) in [("init", without_demo), ("single model", single_model()), ("demo", init)] {
        crate::named(&document, &config::Directories::working()).expect("stock passes startup parsing");
        let engine = engine(&document);
        let core = config::core(&engine, [1; 16], 7);
        let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
        assert_eq!(
            defaults.charters[0].policy.instructions.as_ref(),
            include_bytes!("../../../temper-engine-service/src/stock/chat.txt")
        );
        assert_eq!(
            defaults.charters[1].policy.instructions.as_ref(),
            temper_charter_set::implementer(&crate::stock::charter_set()).as_ref()
        );
        for charter in &defaults.charters {
            assert_eq!(charter.hosts, jig_domain_fleet::Kinds::Workers, "{row}");
        }
        let current = jig_domain_core::settings::Current::new(defaults, &engine.limits.core, &[]);
        assert_eq!(
            jig_domain_core::settings::check(&current, &core.settings, &core.connectors, &engine.limits.core),
            Ok(()),
            "{row}"
        );
    }
}

#[test]
fn each_floor_refuses_one_below_the_stocks_need() {
    use jig_domain_core::settings::{self, Setting};
    let document = without_projects();
    let engine = engine(&document);
    let core = config::core(&engine, [1; 16], 7);
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let mut run = (0, Setting::Charter(1));
    for charter in &defaults.charters {
        let resolved = settings::charters::resolved(charter, &defaults.models, &engine.limits.core).unwrap();
        let bytes = jig_domain_core::charter_owned_bytes(&resolved).unwrap();
        if bytes > run.0 {
            run = (bytes, Setting::Charter(charter.number));
        }
    }
    // The main model and eight alternatives each carry eight aligned cache-lifetime bytes.
    assert_eq!(run, (26_744, Setting::Charter(5)));
    let current = settings::Current::new(defaults, &engine.limits.core, &[]);
    let rows: &[(&[u8], u64, Setting, &str)] = &[
        // Each policy authority includes the standing roles' charter executors.
        (b"core.policy_bytes", 6423, Setting::Policy(1), "the stock's policy 1 needs 6423 bytes"),
        (b"core.run_bytes", run.0, run.1, ""),
        (b"authority.grants", 15, Setting::Rules, "the stock's rules need 15 grants"),
        // The standing charters, closer procedure and close job are delegation targets.
        (b"authority.executors", 20, Setting::Rules, "the stock's rules need 20 executors"),
        (b"authority.segments", 1, Setting::Rules, "the stock's rules need 1 segment"),
        (b"authority.segment_bytes", 5, Setting::Rules, "the stock's rules need 5 bytes"),
        (b"authority.roles", 4, Setting::Policy(1), "the stock's policy 1 needs 4 roles"),
        (b"authority.projects", 1, Setting::Policy(1), "the stock offers 1 policy"),
        (b"tasks.charters", 12, Setting::Charter(1), "the stock offers 12 charters"),
        (b"core.templates", 6, Setting::Template(1), "the stock offers 6 templates"),
        (b"documents.text_bytes", 16, Setting::Template(4), "the stock's template 4 needs 16 bytes"),
        (b"tasks.grace_ms", 5000, Setting::Template(1), "the stock's template 1 needs 5000 ms of grace"),
        (b"core.settings.alternatives", 2, Setting::Charter(11), "the stock's charter 11 needs 2 alternatives"),
        (b"core.settings.permissions", 3, Setting::Policy(1), "the stock's policy 1 needs 3 permissions"),
    ];
    // Every positive scalar bound reached by jig's whole-set check is at its
    // floor together. This catches a stock value acquiring another unmet bound.
    let mut minimum = engine.limits;
    for &(name, needed, _, _) in rows {
        config::overrides::apply(&mut minimum, name, needed).unwrap();
    }
    assert_eq!(settings::check(&current, &core.settings, &core.connectors, &minimum.core), Ok(()));
    assert_eq!(
        crate::stock::floors(
            &crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes),
            &minimum
        ),
        Ok(())
    );
    for &(name, needed, setting, words) in rows {
        let mut at = fixtures::parse_engine(&fixtures::value(&document, b"engine"));
        config::overrides::apply(&mut at.limits, name, needed).unwrap();
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::stock(&at, &mut problems);
        assert!(problems.is_empty(), "{name:?}");
        assert_eq!(settings::check(&current, &core.settings, &core.connectors, &at.limits.core), Ok(()));
        if let Err(why) = crate::startup::named(&limited(&document, name, needed), &config::Directories::working()) {
            panic!("{name:?} at its floor: {:?}", crate::startup::lines(&why));
        }
        config::overrides::apply(&mut at.limits, name, needed - 1).unwrap();
        assert!(settings::check(&current, &core.settings, &core.connectors, &at.limits.core).is_err(), "{name:?}");
        let found = crate::stock::floors(
            &crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes),
            &at.limits,
        )
        .unwrap_err();
        assert_eq!(found, crate::stock::Floor { limit: name, configured: needed - 1, needed, setting });
        crate::startup::checks::stock(&at, &mut problems);
        let words = if name == b"core.run_bytes" {
            let Setting::Charter(number) = setting else { panic!("run policy's charter") };
            format!("the stock's charter {number} needs {needed} bytes")
        } else {
            words.to_owned()
        };
        let name = core::str::from_utf8(name).unwrap();
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            format!(
                "engine.limits.overrides.{name}: {name} is {}; {words}; raise it in engine.limits.overrides\n",
                needed - 1
            )
            .as_bytes()
        );
        let refused =
            crate::startup::named(&limited(&document, name.as_bytes(), needed - 1), &config::Directories::working())
                .unwrap_err();
        assert_eq!(crate::startup::lines(&refused), crate::startup::lines(&problems));
    }
}

#[test]
fn a_worlds_own_short_runs_do_not_need_the_stocks_run_bytes() {
    let document = without_projects();
    let engine = engine(&document);
    let mut own = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let mut needed = 0;
    for charter in &mut own.charters {
        charter.policy.instructions = Box::default();
        needed = needed.max(jig_domain_core::charter_owned_bytes(charter).unwrap());
    }
    let document = limited(&document, b"core.run_bytes", needed);
    let refused = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
    assert!(core::str::from_utf8(&crate::startup::lines(&refused)).unwrap().contains("the stock's charter"));
    crate::startup::named_with(
        &document,
        &config::Directories::working(),
        &crate::stock::Given::Own { jig: own, forge: Box::default(), ways: crate::stock::ways() },
    )
    .expect("a world sizes its own runs");
}

#[test]
fn a_stock_policy_past_its_limit_is_refused_while_naming_files() {
    let document = limited(&without_projects(), b"core.policy_bytes", 100);
    // named only parses the document and lists its files. No file read or
    // listener opening can occur before this refusal.
    let found = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
    // Two executor places in four authorities add 64 bytes to the policy.
    assert_eq!(
        crate::startup::lines(&found).as_ref(),
        b"engine.limits.overrides.core.policy_bytes: core.policy_bytes is 100; the stock's policy 1 needs 6423 bytes; raise it in engine.limits.overrides\n"
    );
}

#[test]
fn each_member_is_refused_at_startup_naming_temper_ctl_settings() {
    let init = generated();
    for member in ["charters", "templates", "policies", "rules"] {
        let original = fixtures::value(&init, b"engine");
        let mut engine = format!("{{\"{member}\":null,").into_bytes();
        engine.extend_from_slice(&original[1..]);
        let document = fixtures::with(&init, b"engine", &engine);
        let problems = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
        assert_eq!(problems.problems.len(), 1);
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            format!("engine.{member}: settings live in the store; read them with temper ctl DIR settings show\n")
                .as_bytes()
        );
    }
}

#[test]
fn an_inline_seed_policy_is_refused_naming_its_path() {
    let document = fixtures::with(&generated(), b"engine.projects", fixtures::INLINE_PROJECTS);
    let problems = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(
        crate::startup::lines(&problems).as_ref(),
        b"engine.projects[0].policy: name a policy, policy: 1; change it with temper ctl settings apply\n"
    );
}

#[test]
fn stock_startup_requires_a_model_before_building_its_charters() {
    let mut document = generated();
    for path in [b"engine.llm.providers".as_slice(), b"engine.llm.endpoints"] {
        document = fixtures::with(&document, path, b"[]");
    }
    let found = crate::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(crate::startup::lines(&found).as_ref(), b"engine.llm.providers: past its bound\n");
}

#[test]
fn a_worlds_own_defaults_cannot_restore_document_members() {
    let init = generated();
    let stock = engine(&init);
    let mut own = crate::stock::with_models(stock.llm.models.clone(), stock.limits.core.tasks.result_bytes);
    own.charters[0].policy.turns = 9;
    let given = crate::stock::Given::Own { jig: own, forge: Box::default(), ways: crate::stock::ways() };
    for member in ["charters", "templates", "policies", "rules"] {
        let original = fixtures::value(&init, b"engine");
        let mut engine = format!("{{\"{member}\":null,").into_bytes();
        engine.extend_from_slice(&original[1..]);
        let document = fixtures::with(&init, b"engine", &engine);
        let parsed =
            config::document(&document, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 })
                .unwrap();
        let at = config::read::Object::new(&parsed, 0).unwrap().member(&parsed, b"engine").unwrap();
        let found = crate::startup::engine_with(&parsed, at, &config::Directories::working(), &given).unwrap_err();
        let mut problems = crate::startup::Problems::empty();
        problems.append(found, &config::Path::root().member(b"engine"));
        assert_eq!(problems.problems.len(), 1);
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            format!("engine.{member}: settings live in the store; read them with temper ctl DIR settings show\n")
                .as_bytes()
        );
    }
}

#[test]
fn a_worlds_own_defaults_replace_the_stock_without_document_members() {
    let bytes = fixtures::with(&generated(), b"engine.projects", br#"[{"number":1,"policy":1}]"#);
    let document =
        config::document(&bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    let at = config::read::Object::new(&document, 0).unwrap().member(&document, b"engine").unwrap();
    let stock = crate::startup::engine(&document, at, &config::Directories::working()).unwrap();
    let mut own = crate::stock::with_models(stock.llm.models.clone(), stock.limits.core.tasks.result_bytes);
    own.charters[0].policy.waiting = skein_lib::Duration::from_millis(1);
    own.templates[0].flags.listed = true;
    let policy = &mut own.policies.get_mut(&1).unwrap().policy;
    policy.period_spend = 41;
    // The project copies this policy; every role allowance fits its period.
    for role in &mut policy.roles {
        role.period_spend = role.period_spend.min(policy.period_spend);
    }
    own.rules.maximum_run_spend = 2;
    let given = crate::stock::Given::Own { jig: own, forge: Box::default(), ways: crate::stock::ways() };
    let engine = crate::startup::engine_with(&document, at, &config::Directories::working(), &given).unwrap();
    assert_eq!(stock.charters[0].policy.waiting, skein_lib::Duration::from_secs(10));
    assert_eq!(engine.charters[0].policy.waiting, skein_lib::Duration::from_millis(1));
    assert!(engine.templates[0].flags.listed);
    assert_eq!(engine.policies.iter().find(|policy| policy.number == 1).unwrap().policy.period_spend, 41);
    assert_eq!(engine.projects[0].policy.period_spend, 41);
    assert_eq!(engine.rules.maximum_run_spend, 2);
}

#[test]
fn startup_gives_the_stock_ways_or_the_worlds_own() {
    let document = fixtures::engine_alone_document();
    let files = fixtures::files();
    let directories = config::Directories::working();
    let engine = engine(&document);
    let own = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let empty = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let mut custom = crate::stock::ways().into_vec();
    custom.remove(0);
    custom[0].line = b"The world's review way.".as_slice().into();
    for (given, expected) in [
        (crate::stock::Given::Stock, crate::stock::ways()),
        (
            crate::stock::Given::Own { jig: own, forge: Box::default(), ways: custom.clone().into_boxed_slice() },
            custom.into_boxed_slice(),
        ),
        (crate::stock::Given::Own { jig: empty, forge: Box::default(), ways: Box::default() }, Box::default()),
    ] {
        let start = crate::startup::checked_start_with(
            &document,
            &directories,
            &files,
            None,
            &fixtures::seeds(),
            fixtures::forge_runtime(),
            None,
            &given,
        )
        .unwrap();
        assert_eq!(start.engine.ways_stock, expected);
    }
}

#[test]
fn a_documents_models_are_refused_at_their_paths() {
    let init: serde_json::Value = serde_json::from_slice(&generated()).unwrap();
    for (parent, member, path) in [
        ("/engine/llm", "models", "engine.llm.models"),
        ("/engine/llm/providers/0", "models", "engine.llm.providers[0].models"),
    ] {
        let mut document = init.clone();
        document
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(member.into(), serde_json::from_slice::<serde_json::Value>(b"[]").unwrap());
        let problems =
            crate::named(&serde_json::to_vec(&document).unwrap(), &config::Directories::working()).unwrap_err();
        assert_eq!(problems.problems.len(), 1);
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            format!("{path}: settings live in the store; read them with temper ctl DIR settings show\n").as_bytes()
        );
    }
    let mut document = init;
    document["engine"]["llm"]["providers"][0].as_object_mut().unwrap().remove("endpoint");
    let problems = crate::named(&serde_json::to_vec(&document).unwrap(), &config::Directories::working()).unwrap_err();
    assert_eq!(
        crate::startup::lines(&problems).as_ref(),
        b"engine.llm.providers[0].endpoint: missing\nengine.llm.providers[0].keep: unknown reference\n"
    );
}

#[test]
fn a_model_added_by_rest_is_checked_against_the_agents_bounds() {
    let engine = engine(&generated());
    let mut agent = fixtures::parsed_agent(fixtures::stock_configuration(10_662, 8192).unwrap());
    let mut problems = crate::startup::Problems::empty();
    let core = config::core(&engine, [7; 16], 7);
    let warnings = crate::startup::checks::admission(&core, &engine, &agent.declaration(), &mut problems);
    assert!(problems.is_empty());
    assert!(warnings.is_empty());
    let demo = agent.bounds.endpoints.iter_mut().find(|endpoint| endpoint.endpoint == 3).unwrap();
    demo.output = 8191;
    let warnings = crate::startup::checks::admission(&core, &engine, &agent.declaration(), &mut problems);
    assert!(problems.is_empty());
    assert_eq!(warnings.problems.len(), 1);
    assert_eq!(
        crate::startup::lines(&warnings).as_ref(),
        b"engine.charters[0].alternatives[6]: needs 8192 output tokens; the agent offers 8191 tokens\n"
    );
}

pub(super) fn generated() -> Box<[u8]> {
    use core::num::NonZeroU16;
    let answer = temper_init::files(
        &temper_init::Facts {
            sign_in: false,
            binary: b"/bin/temper",
            git: Some(b"/bin/git"),
            claude: None,
            login: Some(b"ada"),
            ports: temper_init::Ports {
                clients: NonZeroU16::new(8367).unwrap(),
                owned: NonZeroU16::new(8368).unwrap(),
                forge: NonZeroU16::new(8369).unwrap(),
                llm: NonZeroU16::new(8370).unwrap(),
                webhooks: NonZeroU16::new(8371).unwrap(),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
        &[1; 32],
    )
    .unwrap();
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("document bytes") };
            return bytes;
        }
    }
    panic!("init writes temper.json")
}

pub(super) fn engine(document: &[u8]) -> config::Engine {
    fixtures::parse_engine(&fixtures::value(document, b"engine"))
}

fn single_model() -> Box<[u8]> {
    let bytes = generated();
    let document = fixtures::Json::new(&bytes);
    let providers = [b"[".as_slice(), document.value(b"engine.llm.providers[0]"), b"]"].concat();
    let endpoints = [b"[".as_slice(), document.value(b"engine.llm.endpoints[0]"), b"]"].concat();
    document.with(&[
        (b"engine.llm.providers", &providers),
        (b"engine.llm.endpoints", &endpoints),
        (b"application.demo", b"null"),
    ])
}

fn without_projects() -> Box<[u8]> {
    let document = generated();
    fixtures::Json::new(&document).with(&[(b"engine.projects", b"[]"), (b"application.demo", b"null")])
}

#[path = "stock_brief.rs"]
mod brief;

#[test]
fn a_fresh_box_admits_the_stock_at_inits_catalogue() {
    let document = generated();
    crate::named(&document, &jig_protocol_config::Directories::working()).expect("fresh box admits stock");
    let engine = engine(&document);
    let own = crate::limits::profile(crate::limits::Profile::Standard);
    assert_eq!(own.root.journal.run_bytes, 65_536);
    assert_eq!(crate::limits::profile(crate::limits::Profile::Small).root.journal.run_bytes, 65_536);
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    assert_eq!(crate::stock::floors(&defaults, &engine.limits), Ok(()));
    for (number, expected) in [(2, 11_279), (5, 26_744)] {
        let charter = defaults.charters.iter().find(|charter| charter.number == number).unwrap();
        let resolved =
            jig_domain_core::settings::charters::resolved(charter, &defaults.models, &engine.limits.core).unwrap();
        let bytes = jig_domain_core::charter_owned_bytes(&resolved).unwrap();
        assert_eq!(bytes, expected, "charter {number}");
        assert!(bytes <= u64::from(own.root.journal.run_bytes));
    }
}

// temper/docs/design/protocol/configuration/settings.md, 1 and 5.
#[test]
fn a_standard_document_seeds_project_one_from_stock_policy_one() {
    let mut document: serde_json::Value = serde_json::from_slice(&generated()).unwrap();
    document["engine"]["limits"] = serde_json::json!({"profile": "standard"});
    document["engine"]["projects"] = serde_json::json!([{"number": 1_i32, "policy": 1_i32}]);
    let bytes = serde_json::to_vec(&document["engine"]).unwrap();
    let parsed = config::document(&bytes, &config::DocumentLimits { bytes: 65_536, tokens: 8192, depth: 32 }).unwrap();
    let mut engine = config::engine(&parsed, 0, &config::Directories::working(), config::Defaults::Passed).unwrap();
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let policy = defaults.policies.get(&1).unwrap();
    let snapshot = jig_domain_core::policy_snapshot(&policy.policy, &policy.permissions, &policy.templates).unwrap();
    let bytes = jig_domain_people::policy_bytes(&snapshot).unwrap();
    assert_eq!(bytes, 6078);
    assert_eq!(policy.policy.ceiling.delegation.kinds.len(), 20);
    assert!(bytes <= engine.limits.core.policy_bytes, "snapshot {bytes}, limit {}", engine.limits.core.policy_bytes);
    assert_eq!(
        jig_domain_authority::policy_refusal(&defaults.rules, &engine.limits.core.authority, &policy.policy),
        None,
        "snapshot {bytes}, executors {}, limit {}",
        policy.policy.ceiling.delegation.kinds.len(),
        engine.limits.core.authority.executors
    );
    config::pass(&mut engine, defaults).expect("a standard deployment seeds stock policy 1");
    assert_eq!(engine.projects.len(), 1);
    assert_eq!(engine.projects[0].number, 1);
    assert_eq!(engine.projects[0].policy, engine.policies[0].policy);
    let core = config::core(&engine, [1; 16], 7);
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    assert_eq!(crate::stock::floors(&defaults, &engine.limits), Ok(()));
    let current = jig_domain_core::settings::Current::new(defaults, &engine.limits.core, &[]);
    assert_eq!(
        jig_domain_core::settings::check(&current, &core.settings, &core.connectors, &engine.limits.core),
        Ok(())
    );
}

#[test]
fn a_seeded_stock_document_names_the_executor_limit() {
    let mut document: serde_json::Value = serde_json::from_slice(&generated()).unwrap();
    document["engine"]["projects"] = serde_json::json!([{"number": 1_i32, "policy": 1_i32}]);
    document["engine"]["limits"]["overrides"]["authority.executors"] = 18_i32.into();
    let document = serde_json::to_vec(&document).unwrap();
    let problems = crate::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(crate::startup::lines(&problems).as_ref(), b"engine.projects[0].policy.ceiling.delegation.kinds: policy has 20 executor kinds; authority.executors limit is 18.\n");
}

#[test]
fn inits_document_can_seed_stock_policy_one() {
    let mut document: serde_json::Value = serde_json::from_slice(&generated()).unwrap();
    document["engine"]["projects"] = serde_json::json!([{"number": 1_i32, "policy": 1_i32}]);
    let document = serde_json::to_vec(&document).unwrap();
    crate::named(&document, &config::Directories::working()).expect("init's limits admit a seeded stock project");
}
