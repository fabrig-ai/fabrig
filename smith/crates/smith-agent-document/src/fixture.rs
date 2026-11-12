//! Step-test fixtures (smith/docs/design/testing.md, section 7).
//! The standard product quantities shared by fixed and drawn startup stories.

/// The standard product startup document for fixed and drawn configuration stories.
#[must_use]
pub fn product_document() -> serde_json::Value {
    serde_json::json!({
      "profile":{"name":"standard"},
      "endpoints":[{"name":"openai","number":1,"dialect":3,"provider":"openai",
        "address":"127.0.0.1:443","transport":"tls","server_name":"api.openai.com",
        "connect":10000,"handshake":10000,
        "limits": {"window":272_000,"output":32000,"reasoning_item":65536,"head":60000,"idle":300_000}}],
      "environment":[{"name":"X","value":"x".repeat(833)}],
      "trace":{"path":"/tmp/unused-diagnostic-trace","capture":"everything"}
    })
}
