use jig_protocol_config as config;
use serde_json::json;
#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::default_numeric_fallback,
    reason = "JSON fixtures pin the stock provider fields"
)]
fn providers_pin_the_stock_sources_registrations_and_endpoints() {
    let providers = crate::providers();
    assert_eq!(providers.len(), 2);
    let limits = skein_json::writer::Limits { depth: 16, length: 16_384 };
    let mut measured = skein_json::writer::Encoder::measure(&limits);
    config::llm::render_providers(&providers, &mut measured);
    let mut writer = skein_json::writer::Encoder::write(measured.measured().unwrap(), &limits);
    config::llm::render_providers(&providers, &mut writer);
    let actual: serde_json::Value = serde_json::from_slice(&writer.finish()).unwrap();
    assert_eq!(
        actual,
        json!({"providers": [{
            "number": 1, "name": "openai", "endpoint": "openai", "signed_in_plan": true,
            "keep": ["gpt-*"],
            "drop": ["*audio*", "*image*", "*realtime*", "*search*", "*transcribe*", "*tts*", "*-20??-??-??"],
            "manage_usage": "https://chatgpt.com/settings/usage",
            "sources": {"keys": true, "sign_in": {
                "issuer": "https://auth.openai.com", "client": "dynamic_agent_client", "name_hint": "temper",
                "authorization": "https://auth.openai.com/api/accounts/authorize",
                "token": "https://auth.openai.com/api/accounts/oauth/token",
                "revocation": "https://auth.openai.com/api/accounts/oauth/revoke",
                "keys": "https://auth.openai.com/.well-known/jwks.json",
                "transport": "tls", "server_name": "auth.openai.com",
                "scopes": ["openid", "profile", "email", "offline_access", "resource.invoke", "chatgpt.tokens.use.direct"],
                "must_grant": ["chatgpt.tokens.use.direct"], "resource": "https://api.openai.com/v1",
                "redirect_path": "auth/callback", "redirect_port": 1455
            }},
            "endpoints": [{"name": "openai", "number": 1, "dialect": 3,
                "address": {"host": "api.openai.com", "port": 443}, "transport": "tls", "server_name": "api.openai.com",
                "headers": [], "connect_ms": 10000, "handshake_ms": 10000}]
        }, {
            "number": 2, "name": "anthropic", "endpoint": "anthropic", "direct": "anthropic-api",
            "signed_in_plan": false, "manage_usage": "https://claude.ai/settings/usage",
            "keep": ["claude-*"], "drop": ["*-20??????"], "sources": {"keys": true, "programs": ["claude_code"]},
            "endpoints": [
                {"name": "anthropic", "number": 2, "dialect": 2, "program": "claude_code",
                 "address": {"host": "api.anthropic.com", "port": 443}, "transport": "tls", "server_name": "api.anthropic.com",
                 "headers": [], "connect_ms": 10000, "handshake_ms": 10000},
                {"name": "anthropic-api", "number": 4, "dialect": 2,
                 "address": {"host": "api.anthropic.com", "port": 443}, "transport": "tls", "server_name": "api.anthropic.com",
                 "headers": [], "connect_ms": 10000, "handshake_ms": 10000}
            ]
        }]})
    );
}
#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary test collects ordered catalogue numbers"
)]
fn the_catalogue_keeps_seven_models_without_running_stock_providers() {
    let mut llm = config::Llm { models: Box::new([]), providers: Box::new([]), endpoints: Box::new([]) };
    assert_eq!(crate::stock::models(&llm).iter().map(|row| row.number).collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6, 7]);
    let scripted = crate::stock::scripted(([127, 0, 0, 1], 12345).into());
    llm.providers = Box::new([scripted.provider]);
    llm.endpoints = scripted.endpoints;
    let defaults = crate::stock::Given::Stock.defaults(&llm, 32).unwrap();
    assert_eq!(defaults.providers, crate::providers());
    assert_eq!(defaults.models.last().unwrap().number, 8);
    for charter in &defaults.charters {
        assert_eq!(charter.policy.model.name.as_ref(), b"demo");
    }
}
