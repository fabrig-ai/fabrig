//! Native stable-prefix observations (smith/docs/design/protocol/llm.md, section 10).
use skein_lib::Token;
use skein_world::domain::{Expectations, Judge};

/// Exact JSON values from one observed native request, retained by the protocol oracle.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NativeRequest {
    pub affinity: Option<String>,
    pub system: String,
    pub tools: Option<String>,
    pub history: Vec<String>,
}

impl NativeRequest {
    /// Read the captured HTTP body's raw values without re-encoding their
    /// bytes, inflated when the endpoint sends gzip.
    ///
    /// # Errors
    /// Returns the first finding about native cache markers or affinity.
    #[expect(
        clippy::too_many_lines,
        reason = "keep native marker checks in first-finding order beside raw byte removal"
    )]
    pub fn from_http(bytes: &[u8]) -> Result<Self, String> {
        let bytes = &skein_llm_kit::identity(bytes)[..];
        let head = skein_lib::bytes::find(bytes, b"\r\n\r\n").expect("observed HTTP request head");
        let fields: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
            serde_json::from_slice(&bytes[head + 4..]).expect("observed native JSON request");
        let system = fields.get("instructions").or_else(|| fields.get("system")).expect("native system");
        let history = fields.get("input").or_else(|| fields.get("messages")).expect("native history");
        let items: Vec<Box<serde_json::value::RawValue>> =
            serde_json::from_str(history.get()).expect("native history array");
        let mut history = items.into_iter().map(|item| String::from(item.get())).collect::<Vec<_>>();
        if fields.contains_key("messages") {
            const MARKER: &str = ",\"cache_control\":{\"type\":\"ephemeral\"}";
            let system_blocks: Vec<serde_json::Value> =
                serde_json::from_str(system.get()).expect("Anthropic system block array");
            let system_markers = system_blocks.iter().filter(|block| block.get("cache_control").is_some()).count();
            if system_markers != usize::from(!system_blocks.is_empty()) {
                return Err("one marker for nonempty system text".into());
            }
            for (index, block) in system_blocks.iter().enumerate() {
                if let Some(marker) = block.get("cache_control") {
                    if index + 1 != system_blocks.len() {
                        return Err("fixed marker is on the last system block".into());
                    }
                    if block["type"] != "text" {
                        return Err("system marker belongs to text".into());
                    }
                    if marker != &serde_json::json!({ "type": "ephemeral" }) {
                        return Err("provider default cache lifetime".into());
                    }
                }
            }
            let values = history
                .iter()
                .map(|item| serde_json::from_str::<serde_json::Value>(item).expect("native message"))
                .collect::<Vec<_>>();
            let last = values.last().expect("Anthropic request has a last message");
            let content = last["content"].as_array().expect("native content blocks");
            let eligible = content
                .iter()
                .rposition(|block| matches!(block["type"].as_str(), Some("text" | "tool_use" | "tool_result")))
                .expect("last message has an eligible block");
            let markers = values
                .iter()
                .flat_map(|message| message["content"].as_array().expect("native content blocks"))
                .filter(|block| block.get("cache_control").is_some())
                .count();
            let tool_markers = fields.get("tools").map_or(0, |tools| {
                let tools: Vec<serde_json::Value> = serde_json::from_str(tools.get()).expect("native tools");
                tools.iter().filter(|tool| tool.get("cache_control").is_some()).count()
            });
            if system_markers + markers + tool_markers > 4 {
                return Err("Anthropic breakpoint allowance".into());
            }
            for block in values.iter().flat_map(|message| message["content"].as_array().expect("native blocks")) {
                if matches!(block["type"].as_str(), Some("thinking" | "redacted_thinking"))
                    && block.get("cache_control").is_some()
                {
                    return Err("thinking never carries a marker".into());
                }
            }
            if markers > 1 {
                return Err("at most one moving Anthropic tail marker".into());
            }
            for (message, item) in history.iter_mut().enumerate() {
                let fields: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
                    serde_json::from_str(item).expect("native message fields");
                let content = fields.get("content").expect("native content");
                let blocks: Vec<Box<serde_json::value::RawValue>> =
                    serde_json::from_str(content.get()).expect("raw native blocks");
                let mut cursor = item.find(content.get()).expect("exact native content bytes");
                for (part, block) in blocks.iter().enumerate() {
                    let offset = cursor + item[cursor..].find(block.get()).expect("next exact native block");
                    cursor = offset + block.get().len();
                    if values[message]["content"][part].get("cache_control").is_some() {
                        if message + 1 != values.len() || part != eligible {
                            return Err(
                                "moving Anthropic marker is on the last eligible block of the last message".into()
                            );
                        }
                        if !block.get().ends_with(&format!("{MARKER}}}")) {
                            return Err("exact dialect tail marker bytes".into());
                        }
                        let position = offset + block.get().len() - 1 - MARKER.len();
                        item.replace_range(position..position + MARKER.len(), "");
                        cursor -= MARKER.len();
                    }
                }
            }
            if markers != 1 {
                return Err("exactly one moving Anthropic tail marker".into());
            }
        }
        let affinity = if let Some(cache) = fields.get("prompt_cache_key") {
            let head = core::str::from_utf8(&bytes[..head]).expect("native HTTP head");
            if head.lines().any(|line| {
                line.split_once(':').is_some_and(|(name, _)| {
                    name.eq_ignore_ascii_case("session-id") || name.eq_ignore_ascii_case("thread-id")
                })
            }) {
                return Err("public affinity has no headers".into());
            }
            Some(cache.get().to_string())
        } else {
            None
        };
        Ok(Self {
            affinity,
            system: system.get().into(),
            tools: fields.get("tools").map(|tools| tools.get().into()),
            history,
        })
    }
}

/// A request independently read from the byte peer.
#[derive(Debug)]
pub enum Seen {
    /// A native body associated with a logical completion owner.
    Request { owner: Token, request: NativeRequest },
    /// The first finding while reading a native request from the byte peer.
    RequestFinding(String),
}

/// Native prefix state, independent of the domain referee.
#[derive(Debug, Default)]
pub struct Meeting {
    requests: std::collections::BTreeMap<Token, NativeRequest>,
}

impl Expectations for Meeting {
    type Seen = Seen;
    type Name = &'static str;
    type Stimulus = ();
    fn observe(&mut self, seen: Seen, judge: &mut Judge<Self::Name, Self::Stimulus>) {
        match seen {
            Seen::RequestFinding(finding) => judge.fail(finding),
            Seen::Request { owner, request } => {
                if let Some(previous) = self.requests.get(&owner) {
                    judge.check(previous.affinity == request.affinity, "native affinity stays fixed within a window");
                    judge.check(previous.system == request.system, "native system bytes stay fixed within a window");
                    judge.check(previous.tools == request.tools, "native tool bytes stay fixed within a window");
                    judge.check(
                        request.history.starts_with(&previous.history),
                        "native history bytes extend within a window",
                    );
                }
                self.requests.insert(owner, request);
            }
        }
    }
}
