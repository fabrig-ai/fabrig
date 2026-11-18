//! Agent fixtures come from the stock figures' writer, not a retained document.
//!
//! temper/docs/design/testing/README.md, 5.
use serde_json::Value;
#[must_use]
pub fn stock(boxed: bool) -> Value {
    let source = if boxed {
        include_bytes!("../../../crates/temper-init/template/temper.json").as_slice()
    } else {
        include_bytes!("../../../crates/temper-init/template/bare.json").as_slice()
    };
    let mut document: Value = serde_json::from_slice(source).expect("init template");
    document["engine"]["clients"]["provider"] = Value::Null;
    for endpoint in document["engine"]["llm"]["endpoints"].as_array_mut().expect("endpoints") {
        if endpoint["address"]["socket"] == "$llm" {
            endpoint["address"]["socket"] = "127.0.0.1:8370".into();
        }
    }
    let mut agent = configuration(&document);
    agent["programs"] = serde_json::json!({"claude_code":{"path":"/fixture/claude","directory":"/fixture/programs"}});
    agent
}
#[must_use]
pub fn configuration(document: &Value) -> Value {
    given(document, &temper_engine_service_stock::stock::agents(), std::path::Path::new("/fixture/data"))
}
fn given(document: &Value, agents: &temper_client_protocol_wire::v1::AgentFigures, data: &std::path::Path) -> Value {
    let bytes = serde_json::to_vec(&document["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
    )
    .expect("engine document");
    let engine = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
        &temper_engine_service_startup::startup::fixed(&jig_protocol_config::Directories::working())
            .expect("fixture constants"),
    )
    .expect("engine configuration");
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
pub fn apply(document: &mut Value, directory: &std::path::Path, agent: &Value) {
    use skein_lib::List;
    use temper_client_protocol_wire::v1 as w;
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
        if let Some(previous) = document["engine"]["llm"]["endpoints"]
            .as_array()
            .and_then(|entries| entries.iter().find(|entry| entry["name"] == endpoint["name"]))
        {
            engine["provider"] = previous["provider"].clone();
        }
        engine["address"] = if address.parse::<std::net::SocketAddr>().is_ok() {
            serde_json::json!({"socket":address})
        } else {
            let (host, port) = address.rsplit_once(':').expect("host:port");
            serde_json::json!({"host":host, "port":port.parse::<u16>().expect("port")})
        };
        endpoints.push(engine);
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
    document["engine"]["llm"]["endpoints"] = endpoints.into();
}

/// The same decoder and launcher substitution the product uses for prospective configuration.
#[must_use]
pub fn prepared(config: &std::path::Path) -> temper_worker_shell::agent::Prepared {
    let bytes = serde_json::to_vec(&read(config)).expect("prospective agent JSON");
    temper_worker_shell::agent::prepare_bytes(bytes.into_boxed_slice()).expect("prospective agent metadata")
}
