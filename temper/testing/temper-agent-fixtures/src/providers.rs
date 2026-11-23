//! Worlds build typed providers or keep their startup copy, never deployment members.
use jig_protocol_config as config;
use serde_json::{Value, json};
use std::path::Path;

/// A world's preparation-only provider input, removed by `keep` before serve reads the document.
pub fn input(document: &mut Value) -> &mut Value {
    document.as_object_mut().expect("world document").entry("world_providers").or_insert_with(stock)
}
#[must_use]
pub fn stock() -> Value {
    flat(&temper_engine_service_stock::providers())
}
#[must_use]
pub fn read(directory: &Path) -> Value {
    let path = if directory.join("world-providers.json").exists() {
        directory.join("world-providers.json")
    } else {
        directory.join("providers.setting")
    };
    flat(&temper_service_shell::providers::read(&path, temper_engine_service_stock::providers(), &mut Vec::new()))
}
/// Retain world inputs separately from the engine's current kept copy.
pub fn remember(directory: &Path, providers: &[config::ProviderSetting]) {
    let providers = without_scripted(providers);
    temper_service_shell::providers::write(&directory.join("world-providers.json"), &providers)
        .expect("world provider inputs");
    temper_service_shell::providers::write(&directory.join("providers.setting"), &providers)
        .expect("world provider copy");
}
/// Stage the same typed world inputs before each process start.
pub fn restore(directory: &Path) {
    if directory.join("world-providers.json").exists() {
        let providers = temper_service_shell::providers::read(
            &directory.join("world-providers.json"),
            Box::new([]),
            &mut Vec::new(),
        );
        temper_service_shell::providers::write(&directory.join("providers.setting"), &providers)
            .expect("world provider copy");
    }
}
#[must_use]
pub fn flat(providers: &[config::ProviderSetting]) -> Value {
    let bytes = temper_service_shell::providers::render(providers);
    let mut value: Value = serde_json::from_slice(&bytes).expect("provider JSON");
    let mut endpoints = Vec::new();
    for provider in value["providers"].as_array_mut().expect("providers") {
        for mut endpoint in provider
            .as_object_mut()
            .expect("provider")
            .remove("endpoints")
            .expect("endpoints")
            .as_array()
            .expect("endpoints")
            .clone()
        {
            endpoint["provider"] = provider["name"].clone();
            endpoints.push(endpoint);
        }
    }
    value["endpoints"] = endpoints.into();
    value
}
#[must_use]
pub fn values(llm: &Value) -> Box<[config::ProviderSetting]> {
    let mut providers = llm["providers"].as_array().expect("world providers").clone();
    for provider in &mut providers {
        let endpoints: Vec<_> = llm["endpoints"]
            .as_array()
            .expect("world endpoints")
            .iter()
            .filter(|endpoint| endpoint["provider"] == provider["name"])
            .map(|endpoint| {
                let mut endpoint = endpoint.clone();
                endpoint.as_object_mut().expect("endpoint").remove("provider");
                endpoint
            })
            .collect();
        let object = provider.as_object_mut().expect("provider");
        object.remove("models");
        object.entry("signed_in_plan").or_insert(json!(false));
        if !object.contains_key("endpoint") && !endpoints.is_empty() {
            object.insert("endpoint".into(), endpoints[0]["name"].clone());
        }
        object.insert("endpoints".into(), endpoints.into());
    }
    let bytes = serde_json::to_vec(&json!({"providers":providers})).expect("world provider list");
    parse(&bytes)
}
#[must_use]
pub fn parse(bytes: &[u8]) -> Box<[config::ProviderSetting]> {
    let document = config::document(bytes, &config::DocumentLimits { bytes: 16_777_216, tokens: 1_048_576, depth: 32 })
        .expect("world providers JSON");
    let mut problems = config::Problems::empty();
    let values = config::llm::providers_setting(&document, 0, &config::Path::root(), &mut problems)
        .expect("typed world providers");
    assert!(problems.is_empty(), "world providers: {problems:?}");
    values
}
#[must_use]
pub fn without_scripted(providers: &[config::ProviderSetting]) -> Box<[config::ProviderSetting]> {
    providers
        .iter()
        .filter(|value| value.provider.number != 3 || value.provider.name.as_ref() != b"demo")
        .cloned()
        .collect()
}
/// Take the preparation-only provider input, leaving only deployment members.
#[must_use]
pub fn take(document: &mut Value) -> Option<Box<[config::ProviderSetting]>> {
    document.as_object_mut().expect("world document").remove("world_providers").map(|value| values(&value))
}

pub fn given(
    document: &mut Value,
    directory: &Path,
    scripted: Option<std::net::SocketAddr>,
) -> temper_engine_service_stock::stock::Given {
    let providers = take(document).unwrap_or_else(|| values(&read(directory)));
    given_with(providers, scripted)
}

/// Attach the explicit providers and optional scripted destination to the stock settings.
#[must_use]
pub fn given_with(
    providers: Box<[config::ProviderSetting]>,
    scripted: Option<std::net::SocketAddr>,
) -> temper_engine_service_stock::stock::Given {
    let (providers, fixed): (_, Box<[_]>) = match scripted {
        Some(address) => {
            (without_scripted(&providers), Box::new([temper_engine_service_stock::stock::scripted(address)]))
        }
        None => (providers, Box::new([])),
    };
    temper_engine_service_stock::stock::Given::Stock.with_providers(providers, fixed)
}
pub fn write(directory: &Path, llm: &Value) {
    remember(directory, &values(llm));
}
/// Consume world-only inputs before writing a deployment document.
pub fn keep(document: &mut Value, directory: &Path) {
    if let Some(value) = document.as_object_mut().expect("world document").remove("world_providers") {
        write(directory, &value);
    }
}
