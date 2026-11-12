use skein_llm as shared;
use smith_agent_profile::profile::Per;

mod native;
mod programs;
mod prompt_line;

use super::*;
use prompt_line::assert_native_prompt_line;

fn document() -> Document {
    Document {
        profile: ProfileDocument {
            name: "standard".into(),
            declared: DeclaredOverrides {
                memory: Some(u64::MAX),
                tool_payload: Some(256),
                read_window: Some(256),
                ..DeclaredOverrides::default()
            },
            policy: PolicyOverrides::default(),
        },
        endpoints: serde_json::from_value(shaped_document()["endpoints"].clone()).expect("tiny endpoint"),
        environment: vec![],
        trace: None,
        programs: std::collections::BTreeMap::new(),
    }
}

#[test]
fn withdrawn_endpoint_fields_are_refused_by_name() {
    for (field, value) in [("account", serde_json::json!(0)), ("identity", serde_json::json!("claude-code"))] {
        let mut document = shaped_document();
        document["endpoints"][0][field] = value;
        let refusal =
            decode(&serde_json::to_vec(&document).expect("configuration JSON")).err().expect("unknown endpoint field");
        assert!(refusal.contains(&format!("unknown field `{field}`")), "{refusal}");
    }
}

#[test]
fn an_empty_endpoint_list_names_the_missing_field() {
    let mut empty = document();
    empty.endpoints.clear();
    assert_eq!(build(empty).err().as_deref(), Some("field: declared.endpoints must not be empty"));
}

#[test]
fn four_endpoints_are_admitted_and_a_fifth_names_the_field() {
    for count in [4_u32, 5] {
        let mut document = shaped_document();
        let endpoint = document["endpoints"][0].clone();
        document["endpoints"] = (1..=count)
            .map(|number| {
                let mut endpoint = endpoint.clone();
                endpoint["number"] = number.into();
                endpoint["name"] = format!("endpoint-{number}").into();
                endpoint
            })
            .collect();
        let configured = parse_value(&document);
        if count == 4 {
            let configured = configured.expect("four endpoint declarations fit the standard profile");
            assert_eq!(configured.declarations.endpoints.len(), 4);
        } else {
            assert_eq!(configured.err().as_deref(), Some("too many endpoints"));
        }
    }
}

#[test]
fn shared_environment_validation_names_the_invalid_or_repeated_position() {
    for (name, value) in [("", "v"), ("A=B", "v"), ("A\0B", "v"), ("A", "v\0")] {
        let mut document = document();
        document.environment = vec![Variable { name: name.into(), value: value.into() }];
        assert_eq!(build(document).err().as_deref(), Some("environment: entry 0 has an invalid or repeated name"));
    }
    let mut document = document();
    document.environment = vec![
        Variable { name: "A".into(), value: "first".into() },
        Variable { name: "A".into(), value: "second".into() },
    ];
    assert_eq!(build(document).err().as_deref(), Some("environment: entry 1 has an invalid or repeated name"));
}

#[test]
fn empty_and_equals_values_reach_the_shared_environment_encoder() {
    let mut document = document();
    document.environment = vec![
        Variable { name: "EMPTY".into(), value: String::new() },
        Variable { name: "MULTI".into(), value: "a=b".into() },
    ];
    let configuration = build(document).expect("valid environment");
    assert_eq!(configuration.declarations.environment_bytes, 15);
    assert_eq!(configuration.service.limits.machine.env_bytes, 15);
}

#[test]
fn a_trust_file_that_is_not_a_regular_file_is_refused_naming_it() {
    let path = skein_scratch::paths::package();
    let error = trust(path.to_str()).expect_err("directory is not a certificate file");
    assert!(error.contains(path.to_str().expect("fixture path")), "refusal names the path");
    assert!(error.contains("NotAFile"), "refusal names the non-file");
}

#[test]
fn a_machine_bundle_past_its_certificate_bound_is_refused_naming_the_bound() {
    let scratch = skein_scratch::Scratch::new("smith-trust-bundle");
    let path = scratch.path().join("roots.pem");
    std::fs::write(&path, b"-----BEGIN CERTIFICATE-----\nAA==\n-----END CERTIFICATE-----\n".repeat(513))
        .expect("bounded PEM fixture");
    let error = trust_source(&skein_shell::Trust::Machine { ssl_cert_file: Some(path.clone()) })
        .expect_err("too many certificates");
    assert_eq!(error, "trust roots exceed 512 certificates");
}

#[test]
fn a_dialect_name_and_number_must_match() {
    let rows = [
        ("openai", 3, None),
        ("anthropic", 2, None),
        ("codex", 1, Some("endpoint provider must be openai or anthropic")),
        ("openai", 1, Some("endpoint provider must be openai or anthropic")),
        ("anthropic", 3, Some("endpoint dialect does not match provider")),
    ];
    for (index, (provider, dialect, expected)) in rows.into_iter().enumerate() {
        let mut endpoint = document().endpoints.remove(0);
        endpoint.provider = provider.into();
        endpoint.dialect = dialect;
        let refusal = build_limits(&endpoint).err().map(ConfigurationRefusal::detail);
        assert_eq!(refusal.as_deref(), expected, "row {index}");
    }
}

#[test]
fn an_openai_endpoint_takes_the_public_route() {
    let mut document = document();
    let name = document.endpoints[0].number;
    document.endpoints[0].provider = "openai".into();
    document.endpoints[0].dialect = 3;
    let configured = build(document).expect("public endpoint derives");
    let options = configured.service.llm_endpoints.resolve(domain::llm::Endpoint(name)).expect("endpoint");
    assert_eq!(options.provider, shared::Provider::OpenAi);
    assert_eq!(options.endpoint.authority.as_ref(), b"api.openai.com");
    assert_eq!(options.endpoint.target.as_ref(), b"/v1/responses");
}

#[test]
fn affinity_headers_are_refused_by_name() {
    for name in ["session-id", "SESSION-ID", "thread-id", "ThReAd-Id"] {
        let mut document = shaped_document();
        document["endpoints"][0]["headers"] = serde_json::json!([{ "name": name, "value": "host override" }]);
        assert_eq!(parse_value(&document).err().as_deref(), Some("invalid endpoint HTTP header"));
    }
}

#[test]
fn static_cache_keys_are_unknown_endpoint_fields() {
    let mut endpoint = shaped_document()["endpoints"][0].clone();
    endpoint["cache_key"] = "static".into();
    let error = serde_json::from_value::<Endpoint>(endpoint).err().expect("removed field is rejected");
    assert!(error.to_string().contains("unknown field `cache_key`"));
}

#[test]
fn plaintext_endpoints_refuse_addresses_outside_loopback() {
    for address in ["192.0.2.1:8080", "[2001:db8::1]:8080"] {
        let mut document = shaped_document();
        document["endpoints"][0]["address"] = address.into();
        assert_eq!(parse_value(&document).err().as_deref(), Some("plaintext endpoint address must be loopback"));
    }
}

#[test]
fn ambiguous_or_unknown_transports_name_the_refused_setting() {
    for (extra, expected) in [
        (
            serde_json::json!({"transport":"plaintext", "server_name":"example.test"}),
            "plaintext endpoint cannot name TLS server_name or trust_der",
        ),
        (
            serde_json::json!({"transport":"plaintext", "trust_der":"missing.der"}),
            "plaintext endpoint cannot name TLS server_name or trust_der",
        ),
        (serde_json::json!({"transport":"unknown"}), "endpoint transport must be tls or plaintext"),
        (serde_json::json!({"transport":"tls"}), "TLS endpoint requires server_name"),
    ] {
        let mut document = shaped_document();
        document["endpoints"][0]
            .as_object_mut()
            .expect("endpoint")
            .extend(extra.as_object().expect("settings").clone());
        assert_eq!(parse_value(&document).err().as_deref(), Some(expected));
    }
}

#[test]
fn overflowing_declared_durations_name_the_first_field_and_its_scope() {
    let mut policy = document();
    policy.profile.policy.max_time = Some(u64::MAX);
    assert_eq!(build(policy).err().as_deref(), Some("derived: policy.max_time does not fit its type"));
    let endpoint = LimitName { layer: Layer::Declared, field: "idle", per: Per::Endpoint(7) };
    assert_eq!(
        duration(u64::MAX, endpoint).err().map(ConfigurationRefusal::detail).as_deref(),
        Some("derived: declared.idle[endpoint=7] does not fit its type")
    );
}

#[test]
fn an_unknown_profile_refuses_before_a_channel_is_opened() {
    let mut bad = document();
    bad.profile.name = "unknown".into();
    assert_eq!(build(bad).err().as_deref(), Some("unknown profile name"));
}

#[test]
fn zero_group_stop_refuses_before_a_channel_is_opened() {
    let mut bad = document();
    bad.profile.policy.group_stop = Some(0);
    assert_eq!(build(bad).err().as_deref(), Some("deadlines: policy.group_stop=0 ns; policy.deadline_minimum=0 ns"));
}

#[test]
fn a_document_without_endpoints_names_the_missing_field() {
    let mut document = shaped_document();
    document.as_object_mut().expect("configuration").remove("endpoints");
    let refusal = decode(&serde_json::to_vec(&document).expect("configuration JSON")).err().expect("missing endpoints");
    assert!(refusal.contains("missing field `endpoints`"), "{refusal}");
}

#[test]
fn a_credential_field_is_not_part_of_configuration() {
    let mut document = shaped_document();
    document["credential"] = "bad".into();
    let refusal = decode(&serde_json::to_vec(&document).expect("configuration JSON")).err().expect("unknown field");
    assert!(refusal.contains("unknown field `credential`"), "{refusal}");
}

fn shaped_document() -> serde_json::Value {
    serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":256,"read_window":256}},
        "endpoints":[{"name":"local","number":0,"dialect":3,"provider":"openai",
            "address":"127.0.0.1:8080","transport":"plaintext","connect":10000,"handshake":10000,
            "limits": {"window":65536,"output":512,"reasoning_item":64,"head":60000,"idle":30000}}],
        "environment":[]
    })
}

fn parse_value(value: &serde_json::Value) -> Result<Configuration, String> {
    parse(&serde_json::to_vec(value).expect("configuration JSON"))
}

#[test]
fn omitted_memory_uses_the_standard_process_declaration() {
    let mut document = shaped_document();
    document["profile"]["declared"].as_object_mut().expect("declarations").remove("memory");
    let configuration = parse_value(&document).expect("default memory is accepted");
    assert_eq!(configuration.memory, 2_281_701_376);
}

#[test]
fn quantities_override_one_value_and_derive_through_the_service() {
    let mut document = shaped_document();
    document["profile"]["declared"]["conversations"] = 2.into();
    document["profile"]["declared"]["shell_tail"] = 64.into();
    document["profile"]["policy"] =
        serde_json::json!({"inbox":3,"group_stop":7,"max_turns":9,"unacknowledged":2,"job_poll":2000,"job_wait":7000});
    let configured = parse_value(&document).expect("quantity overrides");
    let limits = configured.service.limits;
    assert_eq!(limits.domain.run.conversations, 2);
    assert_eq!(limits.domain.session.sessions, 2);
    assert_eq!(limits.domain.session.tools.kits, 2);
    assert_eq!(limits.domain.run.messages, 3);
    assert_eq!(limits.domain.run.budget.turns, 9);
    assert_eq!(limits.channel.turns, 2);
    assert_eq!(limits.machine.group_stop, Duration::from_millis(7));
    assert_eq!(configured.profile.policy.job_poll, Duration::from_secs(2));
    assert_eq!(configured.profile.policy.job_wait, Duration::from_secs(7));
    assert_eq!(limits.domain.session.tools.job_poll, Duration::from_secs(2));
    assert_eq!(limits.domain.session.tools.job_wait, Duration::from_secs(7));
    assert_eq!(limits.llm.translation.stated.job_poll, Duration::from_secs(2));
    assert_eq!(limits.llm.translation.stated.job_wait, Duration::from_secs(7));
    assert_eq!(limits.domain.session.tools.shell_tail, 64);
    assert_eq!(limits.domain.run.check_tail, 64);
    assert_eq!(limits.domain.session.tools.tool_deadline, configured.profile.policy.tool_deadline);
    assert_eq!(limits.domain.session.tools.shell_output, configured.profile.declared.shell_output);
    assert_eq!(limits.domain.run.host_timeout_max, configured.profile.policy.tool_deadline);
    assert_eq!(limits.llm.translation.stated.tool_deadline, configured.profile.policy.tool_deadline);
    assert_eq!(configured.profile.declared.shell_head, service::profile::standard().declared.shell_head);
    assert_eq!(configured.declarations.endpoints[0].limits.window, 65536);
}

#[test]
fn a_profile_string_is_refused_as_the_old_shape() {
    let mut document = shaped_document();
    document["profile"] = "standard".into();
    let refusal = parse_value(&document).err().expect("old profile shape");
    assert!(refusal.contains("expected struct ProfileDocument"), "{refusal}");
}

#[test]
fn raw_or_unknown_deployment_fields_are_named() {
    for field in ["request_bytes", "unexpected"] {
        let mut document = shaped_document();
        document["profile"]["declared"][field] = 100.into();
        let refusal = parse_value(&document).err().expect("unknown declaration");
        assert!(refusal.contains(&format!("unknown field `{field}`")), "{refusal}");
    }
}

#[test]
fn an_endpoint_with_models_is_refused_naming_the_field() {
    let mut document = shaped_document();
    document["endpoints"][0]["models"] = serde_json::json!([]);
    let refusal = parse_value(&document).err().expect("unknown endpoint field");
    assert!(refusal.contains("unknown field `models`"), "{refusal}");
}

#[test]
fn an_endpoint_without_limits_is_refused() {
    let mut document = shaped_document();
    document["endpoints"][0].as_object_mut().expect("endpoint").remove("limits");
    let refusal = parse_value(&document).err().expect("missing endpoint limits");
    assert!(refusal.contains("missing field `limits`"), "{refusal}");
}

#[test]
fn zero_endpoint_quantities_name_the_refused_setting() {
    for (field, expected) in [
        ("window", "endpoint limits contain a zero quantity"),
        ("output", "endpoint limits contain a zero quantity"),
        ("reasoning_item", "endpoint limits contain a zero quantity"),
        ("head", "deadlines: declared.head[endpoint=0]=0 ns; policy.deadline_minimum=0 ns"),
        ("idle", "deadlines: declared.idle[endpoint=0]=0 ns; policy.deadline_minimum=0 ns"),
    ] {
        let mut document = shaped_document();
        document["endpoints"][0]["limits"][field] = 0.into();
        assert_eq!(parse_value(&document).err().as_deref(), Some(expected), "{field}");
    }
}

#[test]
fn zero_deployment_quantities_name_the_refused_setting() {
    for field in ["conversations", "tool_payload", "file_bytes", "calls_per_response", "memory"] {
        let mut document = shaped_document();
        document["profile"]["declared"][field] = 0.into();
        assert_eq!(parse_value(&document).err(), Some(format!("profile.declared.{field} must be positive")));
    }
}
#[test]
fn the_whole_file_declaration_is_independent_and_payload_checked() {
    let mut document = shaped_document();
    document["profile"]["declared"]["file_bytes"] = 1_048_576.into();
    let configured = parse_value(&document).expect("explicit whole file ceiling");
    assert_eq!(configured.profile.declared.file_bytes, 1_048_576);
    document["profile"]["declared"]["file_bytes"] = 1.into();
    let reason = parse_value(&document).err().expect("payload does not fit");
    assert!(reason.starts_with("payload-in-file:"), "{reason}");
}
#[test]
fn oversized_reasoning_defaults_to_fail_and_drop_requires_explicit_selection() {
    let mut document = shaped_document();
    let configured = parse_value(&document).expect("default policy");
    assert_eq!(
        configured.declarations.endpoints[0].limits.oversized_reasoning,
        service::profile::OversizedReasoning::Fail
    );
    document["endpoints"][0]["limits"]["oversized_reasoning"] = "drop".into();
    let configured = parse_value(&document).expect("explicit drop");
    assert_eq!(
        configured.declarations.endpoints[0].limits.oversized_reasoning,
        service::profile::OversizedReasoning::Drop
    );
    document["endpoints"][0]["limits"]["oversized_reasoning"] = "fallback".into();
    assert_eq!(parse_value(&document).err().as_deref(), Some("endpoint oversized_reasoning must be fail or drop"));
}

use crate::fixture::product_document;

fn all_product_models(mut document: serde_json::Value) -> serde_json::Value {
    let mut anthropic = document["endpoints"][0].clone();
    anthropic["name"] = "anthropic".into();
    anthropic["number"] = 2.into();
    anthropic["dialect"] = 2.into();
    anthropic["provider"] = "anthropic".into();
    anthropic["server_name"] = "api.anthropic.com".into();
    anthropic["limits"]["window"] = 200_000.into();
    anthropic["limits"]["idle"] = 120_000.into();
    document["endpoints"].as_array_mut().unwrap().push(anthropic);
    let trust = skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der");
    for endpoint in document["endpoints"].as_array_mut().unwrap() {
        endpoint["trust_der"] = trust.to_str().expect("fixture trust path").into();
    }
    document
}

#[test]
fn native_validated_dense_prompts_fit_the_seen_joint_receiving_ceiling() {
    let configured = parse_value(&product_document()).expect("product startup");
    let client = configured.service.limits.llm.translation.client;
    let events = configured.service.limits.events.expect("product destination");
    let ceiling = events.sinks.line_ceiling(Capture::Everything).expect("joint ceiling");
    let credential = shared::Credential::bearer(b"secret-prompt-bound-token".as_slice().into());
    for dense in [true, false] {
        let count = if dense { client.history_items } else { 8 };
        let text = if dense { b"x".as_slice().into() } else { vec![0; 256_000].into_boxed_slice() };
        let native = shared::Prompt {
            model: b"gpt-6-luna".as_slice().into(),
            instructions: b"native system".as_slice().into(),
            tools: Box::new([]),
            choice: shared::ToolChoice::None,
            messages: (0..count)
                .map(|_| shared::Message {
                    role: shared::Role::User,
                    content: Box::new([shared::Block::Text { text: text.clone(), replay: None }]),
                })
                .collect(),
            reasoning_effort: None,
            affinity: None,
            max_output_tokens: None,
        };
        let measured = shared::client::measure(&native, Some(&credential), &shared::Endpoint::openai(), &client)
            .expect("the native writer admits the complete source payload");
        assert!(measured.body <= client.request, "native body stays within its owning cap");
        let mut messages = List::with_capacity(count);
        for _ in 0..count {
            let mut blocks = List::with_capacity(1);
            blocks.push(smith_events::Block::Text { text: text.clone() }).expect("one block");
            messages.push(smith_events::Message { role: smith_events::Role::User, blocks }).expect("admitted history");
        }
        let record = smith_events::Record {
            t_ms: u64::MAX,
            event: smith_events::Event::ResponseStarted(smith_events::ResponseStarted {
                run: u64::MAX,
                conversation: u64::MAX,
                response: u64::MAX,
                turn: u64::MAX,
                attempt: u64::MAX,
                window: u64::MAX,
                model: smith_events::Model {
                    account: None,
                    backend: smith_events::Backend::Direct,
                    endpoint: b"codex".as_slice().into(),
                    model: native.model.clone(),
                    window_tokens: 272_000,
                    output_tokens: 32_000,
                    effort: Some(b"low".as_slice().into()),
                },
                messages: u64::from(count),
                output_tokens: 32_000,
                tool_choice: smith_events::ToolChoice::None,
                prompt: Some(smith_events::Prompt {
                    system: Some(native.instructions.clone()),
                    tools: Some(List::with_capacity(0)),
                    messages,
                }),
            }),
        };
        let line = smith_events::write_with_ceiling(
            &record,
            &smith_events::Capture::Everything,
            &events.sinks.records,
            ceiling,
        )
        .expect("source-admitted prompt fits both semantic and joint caps")
        .expect("response start is emitted");
        assert!(
            line.len() > if dense { 1_000_000 } else { 12_000_000 },
            "exercise dense framing and escaped payload maxima"
        );
        assert_native_prompt_line(&line, &native, &text, &credential);
    }
}

#[test]
fn product_values_pass_every_startup_check() {
    check_product_startup(false);
}

#[test]
fn a_program_product_passes_every_startup_check() {
    check_product_startup(true);
}

fn check_product_startup(with_program: bool) {
    let mut document = all_product_models(product_document());
    if with_program {
        document["programs"] = serde_json::json!({"claude_code": {"path":"/programs/claude", "directory":"/programs"}});
        document["endpoints"][1]["program"] = "claude_code".into();
    }
    let configuration = parse_value(&document).expect("product declarations pass every startup check");
    let limits = &configuration.service.limits;
    assert!(limits.file_bytes >= limits.domain.session.tools.file_bytes);
    assert!(configuration.service.domain.valid(&limits.domain), "product declarations fit the root");
}

#[test]
fn invalid_limits_are_refused() {
    for domain in [false, true] {
        let mut configuration = build(document()).expect("valid test configuration");
        assert!(check_limits(&configuration.service).is_ok());
        if domain {
            configuration.service.limits.domain.run.conversations =
                configuration.service.limits.domain.session.sessions + 1;
        } else {
            configuration.service.limits.io.receive += 1;
        }
        let refusal = check_limits(&configuration.service).expect_err("invalid limits");
        assert!(matches!(refusal, ConfigurationRefusal::Profile(Refusal::Derived { limit })
            if limit == LimitName::deployment(Layer::Service, "limits")));
    }
}
