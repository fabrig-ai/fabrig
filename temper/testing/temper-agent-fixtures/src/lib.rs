//! Agent fixtures come from the stock figures' writer, not a retained document.
//!
//! temper/docs/design/testing/README.md, 5.
use serde_json::Value;

pub mod providers;
#[must_use]
pub fn stock(boxed: bool) -> Value {
    let source = if boxed {
        include_bytes!("../../../crates/temper-init/template/temper.json").as_slice()
    } else {
        include_bytes!("../../../crates/temper-init/template/bare.json").as_slice()
    };
    let mut document: Value = serde_json::from_slice(source).expect("init template");
    document["engine"]["clients"]["provider"] = Value::Null;
    let mut agent = given_with(
        &document,
        &temper_engine_service_stock::stock::agents(),
        std::path::Path::new("/fixture/data"),
        boxed,
    );
    agent["programs"] = serde_json::json!({"claude_code":{"path":"/fixture/claude","directory":"/fixture/programs"}});
    agent
}
#[must_use]
pub fn configuration(document: &Value) -> Value {
    given(document, &temper_engine_service_stock::stock::agents(), std::path::Path::new("/fixture/data"))
}
fn given(document: &Value, agents: &temper_client_protocol_wire::v1::AgentFigures, data: &std::path::Path) -> Value {
    given_with(document, agents, data, true)
}
fn given_with(
    document: &Value,
    agents: &temper_client_protocol_wire::v1::AgentFigures,
    data: &std::path::Path,
    boxed: bool,
) -> Value {
    let providers = document
        .get("world_providers")
        .map_or_else(|| crate::providers::values(&crate::providers::read(data)), crate::providers::values);
    given_values(document, agents, data, boxed, &providers)
}
/// Compose prospective figures from a world's typed provider configuration.
#[must_use]
pub fn with_providers(document: &Value, providers: &[jig_protocol_config::ProviderSetting]) -> Value {
    given_values(
        document,
        &temper_engine_service_stock::stock::agents(),
        std::path::Path::new("/fixture/data"),
        true,
        providers,
    )
}
fn given_values(
    document: &Value,
    agents: &temper_client_protocol_wire::v1::AgentFigures,
    data: &std::path::Path,
    boxed: bool,
    providers: &[jig_protocol_config::ProviderSetting],
) -> Value {
    let bytes = serde_json::to_vec(&document["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
    )
    .expect("engine document");
    let mut engine = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::PassedProviders,
        &temper_engine_service_startup::startup::fixed(&jig_protocol_config::Directories::working())
            .expect("fixture constants"),
    )
    .expect("engine configuration");
    let fixed = if boxed {
        Box::new([temper_engine_service_stock::stock::scripted(([127, 0, 0, 1], 8370).into())]) as Box<[_]>
    } else {
        Box::new([])
    };
    let defaults = temper_engine_service_stock::stock::defaults(&engine.llm, 32);
    let running = if boxed { crate::providers::without_scripted(providers) } else { providers.into() };
    jig_protocol_config::pass_providers(&mut engine, defaults, &running, fixed).expect("fixture providers");
    let own = temper_engine_service_vocabulary::limits::stock();
    let figures = temper_engine_service_vocabulary::figures::Composer::new(&engine, &own).compose(agents);
    let bytes = temper_worker_shell::agent::figures::configuration(&figures, None, data, &[]).expect("agent writer");
    serde_json::from_slice(&bytes).expect("written agent JSON")
}

/// Read the agent's prospective configuration without retaining a configuration-directory agent file.
#[must_use]
pub fn read(directory: &std::path::Path) -> Value {
    let document: Value =
        serde_json::from_slice(&std::fs::read(directory.join("temper.json")).expect("temper fixture"))
            .expect("temper JSON");
    at(&document, &data(directory))
}
#[must_use]
pub fn at(document: &Value, directory: &std::path::Path) -> Value {
    use temper_client_protocol_wire::v1 as w;
    let agents = std::fs::read(directory.join("agents.setting"))
        .ok()
        .and_then(|bytes| {
            let mut reader = skein_lib::Reader::new(&bytes);
            w::AgentFigures::decode(&w::CEILINGS, &mut reader).ok()
        })
        .unwrap_or_else(temper_engine_service_stock::stock::agents);
    given(document, &agents, directory)
}
fn data(config: &std::path::Path) -> std::path::PathBuf {
    let sibling = config.parent().unwrap_or(config).join("data");
    if config.ends_with("config") && sibling.is_dir() { sibling } else { config.into() }
}
/// Apply a fixture's agent changes to the engine document and the kind-21 startup copy.
pub fn install(directory: &std::path::Path, agent: &Value) {
    let path = directory.join("temper.json");
    let mut document = serde_json::from_slice(&std::fs::read(&path).expect("temper fixture")).expect("temper JSON");
    apply(&mut document, &data(directory), agent);
    std::fs::write(path, serde_json::to_vec_pretty(&document).expect("fixture JSON")).expect("engine fixture");
}
/// Update a fixture already being edited, keeping the store copy beside its data.
#[expect(
    clippy::too_many_lines,
    reason = "the fixture writes prospective figures and their typed kept providers together"
)]
pub fn apply(document: &mut Value, directory: &std::path::Path, agent: &Value) {
    use skein_lib::List;
    use temper_client_protocol_wire::v1 as w;
    crate::providers::keep(document, directory);
    let providers = crate::providers::values(&crate::providers::read(directory));
    let mut providers = crate::providers::without_scripted(&providers).into_vec();
    let mut endpoints = Vec::new();
    let mut limits = List::with_capacity(w::CEILINGS.agent_figures_endpoints);
    for endpoint in agent["endpoints"].as_array().expect("agent endpoints") {
        let mut engine = endpoint.clone();
        let object = engine.as_object_mut().expect("endpoint object");
        object.remove("limits");
        object.entry("headers").or_insert_with(|| serde_json::json!([]));
        for (old, new) in [("connect", "connect_ms"), ("handshake", "handshake_ms"), ("trust_der", "trust")] {
            if let Some(value) = object.remove(old) {
                object.insert(new.into(), value);
            }
        }
        let address = endpoint["address"].as_str().expect("agent address");
        // Smith names its protocol provider; the engine retains the logical provider's name.
        let mut previous = None;
        for provider in &providers {
            for candidate in &provider.endpoints {
                if endpoint["name"].as_str().is_some_and(|name| name.as_bytes() == candidate.name.as_ref()) {
                    previous = Some(String::from_utf8(candidate.provider.to_vec()).expect("provider name"));
                }
            }
        }
        if let Some(previous) = previous {
            engine["provider"] = previous.into();
        }
        engine["address"] = if address.parse::<std::net::SocketAddr>().is_ok() {
            serde_json::json!({"socket":address})
        } else {
            let (host, port) = address.rsplit_once(':').expect("host:port");
            serde_json::json!({"host":host, "port":port.parse::<u16>().expect("port")})
        };
        if endpoint["name"] != "demo" || endpoint["number"] != 3 {
            endpoints.push(engine);
        }
        let given = &endpoint["limits"];
        let limit = w::AgentLimits::new(
            &w::CEILINGS,
            w::AgentLimitsParts {
                window: u32::try_from(given["window"].as_u64().expect("window")).expect("u32"),
                output: u32::try_from(given["output"].as_u64().expect("output")).expect("u32"),
                reasoning_item: u32::try_from(given["reasoning_item"].as_u64().expect("reasoning")).expect("u32"),
                head_ms: given["head"].as_u64().expect("head"),
                idle_ms: given["idle"].as_u64().expect("idle"),
                oversized: if given["oversized_reasoning"] == "drop" { w::Oversized::Drop } else { w::Oversized::Fail },
            },
        )
        .expect("endpoint limits");
        limits
            .push(
                w::AgentEndpointLimits::new(
                    &w::CEILINGS,
                    w::AgentEndpointLimitsParts {
                        endpoint: endpoint["name"].as_str().expect("endpoint name").as_bytes().into(),
                        limits: limit,
                    },
                )
                .expect("endpoint row"),
            )
            .expect("uncapped fixture limits");
    }
    let mut profile = List::with_capacity(w::CEILINGS.agent_figures_profile);
    for group in ["declared", "policy"] {
        if let Some(values) = agent["profile"][group].as_object() {
            for (name, value) in values {
                profile
                    .push(
                        w::AgentFigure::new(
                            &w::CEILINGS,
                            w::AgentFigureParts {
                                name: format!("{group}.{name}").into_bytes().into(),
                                value: value.as_u64().expect("scalar"),
                            },
                        )
                        .expect("profile figure"),
                    )
                    .expect("uncapped profile");
            }
        }
    }
    let value = w::AgentFigures::new(
        &w::CEILINGS,
        w::AgentFiguresParts {
            profile,
            endpoints: limits,
            fallback: temper_engine_service_stock::stock::agents().fallback().clone(),
        },
    )
    .expect("agents setting");
    let mut bytes = skein_lib::Writer::new(skein_lib::usize_of(value.measure()));
    value.encode(&mut bytes).expect("setting bytes");
    std::fs::write(directory.join("agents.setting"), bytes.finish()).expect("fixture setting copy");
    providers.retain(|provider| {
        endpoints.iter().any(|endpoint| {
            endpoint["provider"].as_str().is_some_and(|name| name.as_bytes() == provider.provider.name.as_ref())
        })
    });
    for provider in &mut providers {
        let mut value = provider_json(provider);
        value["endpoints"] = endpoints
            .iter()
            .filter(|endpoint| {
                endpoint["provider"].as_str().is_some_and(|name| name.as_bytes() == provider.provider.name.as_ref())
            })
            .map(|endpoint| {
                let mut value = endpoint.clone();
                value.as_object_mut().expect("endpoint").remove("provider");
                value
            })
            .collect::<Vec<_>>()
            .into();
        let names: Vec<_> =
            value["endpoints"].as_array().expect("endpoints").iter().map(|endpoint| endpoint["name"].clone()).collect();
        if !names.contains(&value["endpoint"]) {
            value["endpoint"] = names[0].clone();
        }
        if !names.contains(&value["direct"]) {
            value.as_object_mut().expect("provider").remove("direct");
        }
        let bytes = serde_json::to_vec(&value).expect("provider JSON");
        let document = jig_protocol_config::document(
            &bytes,
            &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
        )
        .expect("provider document");
        *provider = jig_protocol_config::llm::provider_setting(
            &document,
            0,
            &jig_protocol_config::Path::root(),
            &mut jig_protocol_config::Problems::empty(),
        )
        .expect("provider value");
    }
    crate::providers::remember(directory, &providers);
    document["engine"].as_object_mut().expect("engine").remove("llm");
}

fn provider_json(provider: &jig_protocol_config::ProviderSetting) -> Value {
    let bytes = temper_service_shell::providers::render(core::slice::from_ref(provider));
    serde_json::from_slice::<Value>(&bytes).expect("provider JSON")["providers"][0].clone()
}

/// The same decoder and launcher substitution the product uses for prospective configuration.
#[must_use]
pub fn prepared(config: &std::path::Path) -> temper_worker_shell::agent::Prepared {
    let bytes = serde_json::to_vec(&read(config)).expect("prospective agent JSON");
    temper_worker_shell::agent::prepare_bytes(bytes.into_boxed_slice()).expect("prospective agent metadata")
}
