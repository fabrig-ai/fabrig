//! JSON startup records, with no resolution or filesystem effects.
//! Durations are milliseconds (smith/docs/design/protocol/agent.md, section 4).
use super::programs;
use serde::Deserialize;
use smith_domain::run;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Document {
    #[serde(default)]
    pub(super) programs: std::collections::BTreeMap<String, programs::ProgramDocument>,
    pub(super) profile: ProfileDocument,
    pub(super) endpoints: Vec<Endpoint>,
    pub(super) environment: Vec<Variable>,
    #[serde(default)]
    pub(super) trace: Option<TraceDocument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProfileDocument {
    pub(super) name: String,
    #[serde(default)]
    pub(super) declared: DeclaredOverrides,
    #[serde(default)]
    pub(super) policy: PolicyOverrides,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeclaredOverrides {
    pub(super) conversations: Option<u32>,
    pub(super) tool_payload: Option<u32>,
    pub(super) file_bytes: Option<u32>,
    pub(super) calls_per_response: Option<u32>,
    pub(super) jobs: Option<u32>,
    pub(super) read_window: Option<u32>,
    pub(super) shell_output: Option<u32>,
    pub(super) shell_head: Option<u32>,
    pub(super) shell_tail: Option<u32>,
    pub(super) search_hits: Option<u32>,
    pub(super) search_bytes: Option<u32>,
    pub(super) list_entries: Option<u32>,
    pub(super) guide: Option<u32>,
    pub(super) llm_pool: Option<u64>,
    pub(super) memory: Option<u64>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PolicyOverrides {
    pub(super) max_turns: Option<u32>,
    pub(super) max_spend: Option<u64>,
    pub(super) max_time: Option<u64>,
    pub(super) max_waiting: Option<u64>,
    pub(super) sections: Option<u32>,
    pub(super) host_tools: Option<u32>,
    pub(super) verdicts: Option<u32>,
    pub(super) items: Option<u32>,
    pub(super) fields: Option<u32>,
    pub(super) inbox: Option<u32>,
    pub(super) unacknowledged: Option<u32>,
    pub(super) compaction_threshold: Option<FractionDocument>,
    pub(super) parallel_tools: Option<u32>,
    pub(super) connection_keep: Option<u64>,
    pub(super) tool_deadline: Option<u64>,
    pub(super) shell_timeout: Option<u64>,
    pub(super) job_poll: Option<u64>,
    pub(super) job_wait: Option<u64>,
    pub(super) check_timeout: Option<u64>,
    pub(super) wrap_up: Option<u64>,
    pub(super) group_stop: Option<u64>,
    pub(super) close: Option<u64>,
    pub(super) write_deadline: Option<u64>,
    pub(super) cancel_grace: Option<u64>,
    pub(super) exit_grace: Option<u64>,
    pub(super) refresh_lead: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FractionDocument {
    pub(super) numerator: u32,
    pub(super) denominator: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ModelDocument {
    pub(super) prices: PriceDocument,
    pub(super) name: String,
    pub(super) window: u32,
    pub(super) output: u32,
    pub(super) reasoning_item: u32,
    #[serde(default)]
    pub(super) oversized_reasoning: Option<String>,
    pub(super) head: u64,
    pub(super) idle: u64,
    #[serde(default)]
    pub(super) efforts: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PriceDocument {
    pub(super) input: u64,
    pub(super) cached: u64,
    #[serde(default)]
    pub(super) cache_write: Option<u64>,
    pub(super) output: u64,
    pub(super) unit: u32,
}

impl PriceDocument {
    pub(super) fn rates(&self) -> run::Prices {
        run::Prices {
            input: self.input,
            cached: self.cached,
            cache_write: self.cache_write,
            output: self.output,
            unit: self.unit,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TraceDocument {
    pub(super) path: String,
    #[serde(default = "default_capture")]
    pub(super) capture: String,
    #[serde(default = "default_delivery")]
    pub(super) delivery: String,
}

fn default_capture() -> String {
    "none".into()
}

fn default_delivery() -> String {
    "complete".into()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Endpoint {
    #[serde(default)]
    pub(super) program: Option<String>,
    pub(super) connect: u64,
    pub(super) handshake: u64,
    pub(super) models: Vec<ModelDocument>,
    pub(super) name: String,
    pub(super) number: u32,
    pub(super) dialect: u32,
    /// Withdrawn endpoint account, ignored; accounts-sources removes it.
    #[serde(default, rename = "account")]
    pub(super) _account: u32,
    /// Provider route: openai, anthropic, or the withdrawn codex route.
    pub(super) provider: String,
    pub(super) address: String,
    #[serde(default = "tls_transport")]
    pub(super) transport: String,
    #[serde(default)]
    pub(super) server_name: Option<String>,
    #[serde(default)]
    pub(super) authority: Option<String>,
    #[serde(default)]
    pub(super) target: Option<String>,
    #[serde(default)]
    pub(super) trust_der: Option<String>,
    #[serde(default)]
    #[cfg(unix)]
    pub(super) trust: Option<TrustDocument>,
    #[serde(default)]
    #[cfg(not(unix))]
    pub(super) trust: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) headers: Vec<HeaderField>,
    #[serde(default)]
    pub(super) identity: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HeaderField {
    pub(super) name: String,
    pub(super) value: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg(unix)]
pub(super) struct TrustDocument {
    #[serde(default = "system_trust")]
    pub(super) system: bool,
    #[serde(default)]
    pub(super) der: Vec<String>,
    #[serde(default)]
    pub(super) pem: Vec<String>,
}

#[cfg(unix)]
const fn system_trust() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Variable {
    pub(super) name: String,
    pub(super) value: String,
}

fn tls_transport() -> String {
    "tls".into()
}
