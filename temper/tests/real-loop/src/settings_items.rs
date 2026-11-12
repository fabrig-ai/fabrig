//! Forge defaults in a process owner's settings file (temper/docs/design/protocol/configuration/settings.md, 4).
use serde_json::{Value, json};
/// Render the world's forge defaults for the owner's settings transaction.
#[must_use]
pub fn forge_items(stock: &[temper_engine_domain_forge::settings::Stock]) -> Vec<Value> {
    use temper_engine_domain_forge::{GateLast, JudgeFreshness, settings};
    let freshness = |value| match value {
        JudgeFreshness::Exact => "exact",
        JudgeFreshness::Clean => "clean",
    };
    let mut items = Vec::new();
    for stock in stock {
        let mut setting =
            json!({"kind": if stock.kind == 3 { "forge-charter" } else { "landing-rule" }, "number":stock.number});
        if stock.kind == 2 {
            setting["project"] = stock.scope.into();
        }
        let value = match &stock.value.value {
            settings::Value::Charter(charter) => json!(charter),
            settings::Value::Goal(_)
            | settings::Value::Questions(_)
            | settings::Value::Ways(_)
            | settings::Value::CharterWords(_)
            | settings::Value::Classes(_)
            | settings::Value::ProjectAreas(_)
            | settings::Value::ProjectTriager(_)
            | settings::Value::ProjectWords(_) => {
                unreachable!("these kinds have no stock defaults")
            }
            settings::Value::Rule(rule) => {
                let last = match &rule.pattern.last {
                    GateLast::Exact(word) => json!({"exact":hex(word)}),
                    GateLast::Open(word) => json!({"open":hex(word)}),
                };
                json!({"pattern":{"segments":rule.pattern.segments.iter().map(|segment| hex(segment)).collect::<Vec<_>>(),"last":last},
                    "enforce":rule.enforce,"ci":rule.ci,"up_to_date":rule.up_to_date,
                    "gates":rule.gates.iter().map(|gate| json!({"number":gate.number,"blocking":gate.blocking,"freshness":freshness(gate.freshness)})).collect::<Vec<_>>(),
                    "approvals":rule.approvals.iter().map(|approval| json!({"role":approval.role,"people":approval.people,"freshness":freshness(approval.freshness)})).collect::<Vec<_>>()})
            }
        };
        items.push(json!({"setting":setting,"set":value}));
    }
    items
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("hex bytes");
    }
    text
}
