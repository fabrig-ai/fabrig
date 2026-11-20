use super::{fixtures, stock};
use crate::{settings, startup, stock::Given};
use jig_protocol_config as config;
use serde_json::json;
#[expect(clippy::disallowed_types, clippy::disallowed_macros, reason = "ordinary JSON fixture assembly")]
fn document() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(&stock::generated()).unwrap();
    value["workers"] = json!([]);
    value["engine"]["hosts"]["listener"] = json!([{"socket":"127.0.0.1:0"}]);
    serde_json::to_vec(&value).unwrap()
}
fn local() -> config::ProviderSetting {
    let mut provider = crate::stock::scripted(([127, 0, 0, 1], 8080).into());
    provider.provider.number = 4;
    provider.provider.name = b"local".as_slice().into();
    provider.provider.endpoint = Some(b"local".as_slice().into());
    provider.endpoints[0].name = b"local".as_slice().into();
    provider.endpoints[0].number = 5;
    provider.endpoints[0].provider = b"local".as_slice().into();
    provider
}
fn start(given: &Given) -> Result<startup::CheckedStart, startup::Problems> {
    startup::checked_start_with(
        &document(),
        &config::Directories::working(),
        &settings::Files { entries: skein_lib::List::uncapped() },
        None,
        &settings::Seeds {
            seed: 1,
            deployment: [1; 16],
            installation: [2; 16],
            crypto: skein_lib::Secret::new(Box::new([3; 32])),
            link_secret: None,
            local_secret: None,
        },
        skein_lib::List::uncapped(),
        None,
        given,
    )
}
#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary filesystem fixture captures logs and edits typed providers"
)]
fn starts_run_the_kept_providers_and_their_endpoints_reach_worker_figures() {
    let scratch = skein_scratch::Scratch::new("providers-start");
    let path = scratch.path().join("providers.setting");
    let mut log = Vec::new();
    let run = |log: &mut Vec<u8>| {
        let providers = temper_service_shell::providers::read(&path, crate::stock::providers(), log);
        start(&Given::Stock.with_providers(providers, Box::new([]))).unwrap()
    };
    let absent = run(&mut log);
    assert_eq!(config::llm::provider_values(&absent.engine.jig.llm), crate::stock::providers());
    assert!(absent.engine.demo.is_none());
    assert!(absent.engine.jig.fixed_llm_providers.is_empty());
    let mut values = crate::stock::providers().into_vec();
    values.push(local());
    temper_service_shell::providers::write(&path, &values).unwrap();
    let kept = run(&mut log);
    assert_eq!(config::llm::provider_values(&kept.engine.jig.llm).as_ref(), values);
    let figures = crate::figures::Composer::new(&kept.engine.jig, &kept.engine.limits).compose(&crate::stock::agents());
    assert!(figures.agents.endpoints.iter().any(|endpoint| endpoint.name.as_ref() == b"local" && endpoint.number == 5));
    values.last_mut().unwrap().endpoints[0].address = config::EndpointAddress::Socket(([127, 0, 0, 1], 8081).into());
    temper_service_shell::providers::write(&path, &values).unwrap();
    assert_eq!(config::llm::provider_values(&run(&mut log).engine.jig.llm).as_ref(), values);
    std::fs::write(&path, b"invalid JSON").unwrap();
    assert_eq!(config::llm::provider_values(&run(&mut log).engine.jig.llm), crate::stock::providers());
    assert_eq!(
        String::from_utf8(log).unwrap(),
        "providers.setting: cannot decode the provider list; the stock's providers run until the store's are kept again\n"
    );
}
#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary diagnostic and repeated endpoint fixtures"
)]
fn start_checks_name_the_running_providers_copy() {
    let mut provider = local();
    provider.endpoints[0].transport = config::Transport::Tls {
        server_name: b"local".as_slice().into(),
        trust: Some(b"missing.der".as_slice().into()),
    };
    let given = Given::Stock.with_providers(Box::new([provider]), Box::new([]));
    let why = start(&given).unwrap_err();
    assert!(String::from_utf8_lossy(&startup::lines(&why)).contains("providers.setting.endpoints[0].trust: not read"));
    let mut providers = crate::stock::providers();
    providers[0].provider.sign_in.as_mut().unwrap().redirect_port = Some(8367);
    let why = start(&Given::Stock.with_providers(providers, Box::new([]))).unwrap_err();
    assert!(
        String::from_utf8_lossy(&startup::lines(&why))
            .starts_with("providers.setting.providers[0].sources.sign_in.redirect_port:")
    );
    let mut engine = fixtures::engine().jig;
    engine.llm.providers = Box::new([local().provider]);
    engine.llm.providers[0].programs = Box::new([b"not_offered".as_slice().into()]);
    let mut problems = startup::Problems::empty();
    startup::checks::agent(&engine, &fixtures::agent().declaration(), &mut problems);
    assert!(
        String::from_utf8_lossy(&startup::lines(&problems))
            .starts_with("providers.setting.providers[0].sources.programs[0]:")
    );
    engine.llm.endpoints = vec![local().endpoints[0].clone(); 65].into_boxed_slice();
    let mut problems = startup::Problems::empty();
    startup::checks::figures(&engine, &mut problems);
    assert_eq!(
        startup::lines(&problems).as_ref(),
        b"providers.setting: 65 endpoints, at most 64, the workers' figures carry no more\n"
    );
}
