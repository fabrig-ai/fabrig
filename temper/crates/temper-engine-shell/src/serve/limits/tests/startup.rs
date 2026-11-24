use super::{LimitInputs, limit_failure, limits_hold_over, service};
use skein_lib::{List, Secret};
use std::path::Path;
use temper_service_core::derived;

#[test]
fn the_template_agent_refuses_jigs_small_hub_turn_bound_before_serving() {
    let data = Path::new("/fixture/bare");
    let document = temper_config_shell::init_document(false);
    let agent = temper_agent_fixtures::with_figures(
        &serde_json::from_slice(&document).expect("bare start"),
        &service::stock::agents(),
        data,
        &service::stock::providers(),
    );
    let agent = temper_worker_shell::agent::prepare_bytes_resolving(
        serde_json::to_vec(&agent).expect("template agent JSON").into_boxed_slice(),
        |_| Ok(()),
        |decoded| {
            decoded.resolve_with(
                |address| Ok(address.parse().unwrap_or_else(|_| ([127, 0, 0, 1], 9).into())),
                |_, _| {
                    let (trust, _) =
                        skein_tls::Config::from_der(&[skein_tls_kit::pki::ROOT.into()], &[]).expect("fixture trust");
                    Ok(trust)
                },
                |_| None,
            )
        },
    )
    .expect("template agent");
    let declaration = agent.agent.declaration();
    let turn = declaration.turn_body;
    let root = data.as_os_str().as_encoded_bytes();
    let directories = jig_protocol_config::Directories::given(root, root, root).expect("bare directories");
    let mut entries = List::with_capacity(1);
    entries.push((b"worker.secret".as_slice().into(), Secret::new(Box::new([4; 32])))).expect("worker secret");
    let files = service::settings::Files { entries };
    let seeds = service::settings::Seeds {
        deployment: [1; 16],
        installation: [2; 16],
        seed: 3,
        crypto: Secret::new(Box::new([4; 32])),
        link_secret: None,
        local_secret: None,
    };
    let mut inputs = LimitInputs::new(
        &document,
        &directories,
        &files,
        Some(&declaration),
        &seeds,
        &service::stock::Given::Stock,
        true,
        None,
    );
    // Store::resolve_settings takes the stock payload when a fresh store has no kept agents.
    inputs.sources.payload = derived::STOCK.payload;
    let hub = jig_protocol_config_fixtures::small().hub;
    let held = limits_hold_over(
        &data.join("limits-held"),
        &format!(
            "hub.turn_bytes {}\nhub.transcript_bytes {}\nhub.turn_queue_bytes {}\n",
            hub.turn_bytes, hub.transcript_bytes, hub.turn_queue_bytes
        ),
    )
    .expect("held limits parse");
    let why =
        service::startup::resolve_limits(&inputs, &held.jig, &held.temper).expect_err("short of the template's turn");
    let report = temper_service_shell::serve_error("temper-box", &limit_failure(&why, Some(&held)));
    // The other small overrides can also be refused. Pin the turn's first line, including its newline.
    let turn_line = format!(
        "temper-box serve: limits-held line 1: hub.turn_bytes (setting limits): 65536 bytes, short of the source's derivation, {turn} bytes\n"
    );
    assert_eq!(report.split_inclusive('\n').next(), Some(turn_line.as_str()));
}
