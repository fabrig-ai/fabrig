//! One serialized startup record for the hosted agent; parent options describe
//! channel input independently (smith/docs/design/testing.md, sections 2 and 5).
use super::TraceCase;
use skein_io::kernel::Spawn;
use smith_protocol_events::sinks::Capture;

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub(crate) enum Mode {
    #[default]
    Ordinary,
    Environment,
    Roots,
    Coding,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "skein_llm::Provider")]
enum ProviderWire {
    OpenAi,
    OpenAiCodex,
    Anthropic,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Capture")]
enum CaptureWire {
    None,
    Calls,
    Everything,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) struct Setup {
    pub mode: Mode,
    pub tracing: Option<(TraceCase, u32)>,
    #[serde(with = "CaptureWire")]
    pub capture: Capture,
    #[serde(with = "ProviderWire")]
    pub provider: skein_llm::Provider,
    pub output: u32,
    pub order_requests: bool,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            mode: Mode::Ordinary,
            tracing: None,
            capture: Capture::Everything,
            provider: skein_llm::Provider::OpenAiCodex,
            output: 0,
            order_requests: false,
        }
    }
}

impl Setup {
    pub fn arguments(self) -> Box<[Box<[u8]>]> {
        Box::new([serde_json::to_vec(&self).expect("scenario setup").into()])
    }

    pub fn read(spawn: &Spawn) -> Self {
        assert_eq!(spawn.args.len(), 1, "one startup record");
        serde_json::from_slice(&spawn.args[0]).expect("scenario setup")
    }
}

pub(crate) struct ParentOptions<'a> {
    pub hang_up: bool,
    pub window: u32,
    pub message: Option<&'a smith_channel::Message>,
    pub live: Option<smith_channel::LiveAsk>,
    pub setup: Setup,
}

impl Default for ParentOptions<'_> {
    fn default() -> Self {
        Self { hang_up: false, window: 8, message: None, live: None, setup: Setup::default() }
    }
}
