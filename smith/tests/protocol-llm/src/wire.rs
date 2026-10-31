//! The application world's observed prepared shared Client and scripted byte peer.
//! Application descriptors/decoding stay here; transport and provider grammar
//! belong entirely to Skein. Existing World owns clocks, root and checkout work.
//! This module retains adapter context until the observed terminal and each physical
//! Client until observed lower Closed. Composition routes active callbacks separately
//! from retired outside observations; it never reads private domain state. Prepare,
//! start, tick, cancel, close and settle are distinct entrances under caller clocks
//! (programming-model.md, sections 5.2 and 9).

use crate::adapter::{self as adapter, Context, Limits, Receiving, ResolvedCall};
use skein_lib::Token;
use skein_llm::{self as shared, client};
use skein_llm_world::fake::Exchange;
use smith_domain::{Event, llm};

pub use smith_protocol_llm::schemas;

/// Caller-supplied endpoint and credential data for the wire fixture.
/// Caller owns this finite configuration; admission and terminals belong to Wire.
#[expect(missing_debug_implementations, reason = "fixture owns caller credential bytes")]
pub struct Configuration {
    /// Whole shared endpoint configuration, supplied as neutral caller data.
    pub endpoint: shared::Endpoint,
    /// Fields emitted by the shared peer, preserving absent native counts.
    pub usage_fields: skein_fake_llm_protocol::documents::UsageFields,
    /// Observed grant supplied to one Client and separately to its independent peer.
    pub credential: shared::Credential,
    /// Independent outside expectation for a native error result's text prefix.
    pub error_prefix: Box<[u8]>,
    /// Independent outside expectation for a native error result's parsed flag.
    pub error_flag: bool,
    /// Handwritten outside expectation for this fixture's continued arguments.
    /// Incoming delta bytes remain exact at the root. A native embedded-object
    /// continuation may serialize object whitespace while preserving every field.
    pub continuation_arguments: Box<[u8]>,
}

/// Native configurations are shared kit data, with no Smith dialect branching.
/// Returns two caller fixtures; it starts no effect and carries outside literal
/// expectations.
#[must_use]
pub fn configurations() -> Box<[Configuration]> {
    Box::new([
        Configuration {
            endpoint: shared::Endpoint::codex(),
            usage_fields: skein_fake_llm_protocol::documents::UsageFields::ALL,
            credential: shared::Credential {
                value: b"fixture-token".as_slice().into(),
                account_id: b"fixture-account".as_slice().into(),
                scheme: skein_llm::Scheme::Bearer,
            },
            error_prefix: b"Error: ".as_slice().into(),
            error_flag: false,
            continuation_arguments:
                br#"{ "opaque" : {"future":[1,true,null]}, "extra":"unchanged" }"#.as_slice().into(),
        },
        Configuration {
            endpoint: shared::Endpoint::anthropic(),
            usage_fields: skein_fake_llm_protocol::documents::UsageFields::ALL,
            credential: shared::Credential::anthropic(b"fixture-token".as_slice().into()),
            error_prefix: Box::new([]),
            error_flag: true,
            continuation_arguments: br#"{"opaque":{"future":[1,true,null]},"extra":"unchanged"}"#.as_slice().into(),
        },
    ])
}

/// Finite observed lower chronology, without retaining another completion body.
/// Shared Client emits these observations; at most four belong to one Wire.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Observed {
    /// Observed completed callback.
    Completed(
        /// Opaque root request owner supplied at preparation and echoed by the
        /// observed Client's one terminal.
        Token,
    ),

    /// Observed failed callback.
    Failed(
        /// Opaque root request owner supplied at preparation and echoed by the
        /// observed Client's one terminal.
        Token,
    ),

    /// Observed cancelled callback after settlement.
    Cancelled(
        /// Opaque root request owner supplied at preparation and echoed after
        /// observed lower settlement.
        Token,
    ),

    /// Terminal won; transport drainage permits reuse.
    Reusable,

    /// Lower physical Close requested.
    Close,

    /// Observed lower Closed received; physical owner may retire.
    Closed,
}

/// Owns the observed prepared machine and retained adapter context through settlement.
/// Caller supplies receiving caps and clocks; there is at most one root terminal
/// and four lifecycle observations.
pub struct Wire {
    /// The one observed Client, lower HTTP/SSE peer and shared script domain.
    pub peer: Exchange,
    /// At most one terminal plus Reusable/Close/Closed, without diagnostic bodies.
    pub observed: Vec<Observed>,
    /// Exact SDK usage copied before adapter translation; no duplicate body.
    pub accepted_usage: Option<shared::Usage>,
    context: Option<Context>,
}

impl core::fmt::Debug for Wire {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Wire")
            .field("waiting", &self.peer.machine.waiting())
            .field("observed", &self.observed)
            .field("accepted_usage", &self.accepted_usage)
            .field("live_context", &self.context.is_some())
            .finish()
    }
}

impl Wire {
    /// The root's observed Complete request supplies owner/prompt/receiving metadata.
    /// Preparation is effect free; start is a separate Client entrance.
    /// Success owns one prepared Client/context; failure starts nothing.
    ///
    /// # Errors
    /// Returns the adapter's admission error when the complete prompt, application
    /// descriptors or receiving contract cannot fit the supplied limits.
    pub fn prepare(
        owner: Token,
        prompt: llm::Prompt,
        receiving: Receiving,
        configuration: &Configuration,
        limits: &Limits,
        scripts: Box<[skein_fake_llm_domain::api::Script]>,
    ) -> Result<Self, adapter::Error> {
        let endpoint_name = prompt.endpoint;
        let application = schemas(&prompt, &limits.stated);
        let endpoint = configuration.endpoint.clone();
        let peer_endpoint = endpoint.clone();
        let credential = shared::Credential {
            value: configuration.credential.value.clone(),
            account_id: configuration.credential.account_id.clone(),
            scheme: skein_llm::Scheme::Bearer,
        };
        let peer_credential = shared::Credential {
            value: credential.value.clone(),
            account_id: credential.account_id.clone(),
            scheme: skein_llm::Scheme::Bearer,
        };
        let adapter::Prepared { client, context } = adapter::prepare(
            adapter::Input { owner, prompt, endpoint_name, endpoint, credential, application, receiving },
            limits,
        )?;
        let peer = Exchange::prepared_with_options(
            client,
            peer_endpoint,
            Some(peer_credential),
            limits.client,
            scripts,
            skein_fake_llm_protocol::documents::Options {
                usage_fields: configuration.usage_fields,
                ..skein_fake_llm_protocol::documents::Options::DEFAULT
            },
        );
        Ok(Self { peer, context: Some(context), observed: Vec::new(), accepted_usage: None })
    }

    /// Caller starts the admitted observed Client once under its installed clocks.
    /// Returns only observed translated events; no callback is fabricated.
    pub fn start(&mut self) -> Vec<Event> {
        self.peer.start();
        self.take()
    }

    /// One shared-world progress step. The root world may interleave timers,
    /// observed cancellation requests and other delegated effects between steps.
    /// Returns immediate progress and at most one observed root terminal, preserving
    /// its complete receiving contract.
    pub fn tick(&mut self) -> (bool, Vec<Event>) {
        let progress = self.peer.tick(true);
        (progress, self.take())
    }

    /// Root requests cancellation; the callback remains owed until observed lower
    /// settlement. Returns only observed events.
    pub fn cancel(&mut self) -> Vec<Event> {
        self.peer.request(client::Request::Cancel);
        self.take()
    }

    /// Physical owner requests explicit Close after Reusable; no second root
    /// terminal is invented.
    pub fn close(&mut self) -> Vec<Event> {
        self.peer.request(client::Request::Close);
        self.take()
    }

    /// Called only after the world's observed lower Close has settled.
    /// Feeds shared lower Closed and returns any observed owed cancellation terminal.
    pub fn settle(&mut self) -> Vec<Event> {
        self.peer.settle();
        self.take()
    }

    fn take(&mut self) -> Vec<Event> {
        let mut returned = Vec::new();
        for event in core::mem::take(&mut self.peer.seen) {
            match event {
                client::Event::Completed { owner, completion } => {
                    self.observed.push(Observed::Completed(owner));
                    self.accepted_usage = Some(completion.usage);
                    let context = self.context.take().expect("one observed Client terminal owns the context");
                    let decoded = resolutions(&context, &completion);
                    returned
                        .push(adapter::completion(context, owner, completion, decoded).expect(
                            "observed admitted Client completion and explicit application codec fit receiving",
                        ));
                }
                client::Event::Failed { owner, failure, evidence, report, detail } => {
                    self.observed.push(Observed::Failed(owner));
                    let context = self.context.take().expect("one observed Client terminal owns the context");
                    returned.push(adapter::failed(context, owner, failure, evidence, detail, report).expect(
                        "observed admitted failure retains owner, class, evidence and full bounded diagnostic",
                    ));
                }
                client::Event::Cancelled { owner } => {
                    self.observed.push(Observed::Cancelled(owner));
                    let context = self.context.take().expect("one observed Client terminal owns the context");
                    returned.push(adapter::cancelled(context, owner).expect("observed cancellation callback owner"));
                }
                client::Event::Reusable => self.observed.push(Observed::Reusable),
                client::Event::Close => self.observed.push(Observed::Close),
                client::Event::Closed => self.observed.push(Observed::Closed),
                client::Event::Delta { .. } | client::Event::Block { .. } => {}
            }
        }
        assert!(self.observed.len() <= 4, "one observed terminal and finite lower settlement chronology");
        assert!(returned.len() <= 1, "one root terminal per observed wire call");
        returned
    }
}

fn resolutions(context: &Context, completion: &shared::Completion) -> Box<[ResolvedCall]> {
    completion
        .content
        .iter()
        .enumerate()
        .filter_map(|(position, block)| match block {
            shared::Block::ToolCall { name, arguments, .. } => {
                if context.served().iter().any(|declaration| match declaration {
                    llm::Served::Host(tool) => tool.name == *name,
                    llm::Served::Finish | llm::Served::Deliver | llm::Served::SubAgent | llm::Served::Wait => false,
                }) {
                    return None;
                }
                Some(ResolvedCall {
                    position: u32::try_from(position).expect("bounded observed assistant position"),
                    name: name.clone(),
                    input: arguments.clone(),
                    call: match name.as_ref() {
                        b"finish" | b"deliver" => crate::translate::decode(
                            name,
                            arguments,
                            context.grants(),
                            context.served(),
                            context.asker(),
                        ),
                        _ => adapter::decode(
                            name,
                            arguments,
                            context.grants(),
                            context.served(),
                            context.asker(),
                            &context.limits(),
                        ),
                    },
                })
            }
            shared::Block::Text { .. }
            | shared::Block::Refusal { .. }
            | shared::Block::Reasoning { .. }
            | shared::Block::ToolResult { .. }
            | shared::Block::Oversize { .. }
            | shared::Block::Cut { .. }
            | shared::Block::Dropped { .. } => None,
        })
        .collect()
}
