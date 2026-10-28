//! Shared landing scripts name the forge that the world actually bound.
use serde_json::Value;
use skein_fake_llm_domain::api::{Line, Turn};
use temper_real_loop_world::landing;

fn call_arguments(turn: &Turn) -> Value {
    let Line::Call { arguments, .. } = &turn.lines[0] else { panic!("tool call") };
    serde_json::from_slice(arguments).expect("call JSON")
}

#[test]
fn the_landing_grants_name_the_forges_bound_authority() {
    let forge = std::net::SocketAddr::from(([127, 0, 0, 1], 41_234));
    let scripts = landing::scripts(forge, 2, landing::README);
    let delegate = call_arguments(&scripts[0].turns[0]);
    let grants = delegate["batch"][0]["authority"]["grants"].as_array().expect("grants");
    let named = serde_json::json!(["forge", forge.to_string()]);
    assert!(!grants.is_empty() && grants.iter().all(|grant| grant["segments"] == named), "{grants:?}");
}
