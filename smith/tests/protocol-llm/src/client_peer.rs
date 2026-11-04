//! Isolated codec fixtures over a prepared skein Client and scripted byte peer.
//! Root stories use the production Component backend. This fixture observes native
//! codec handoffs and the Client's lower closure without domain scheduling.
//! Contract: smith/docs/design/protocol/llm.md, sections 3, 4 and 10.

use crate::codec::{self as codec, Context, Limits, Receiving};
use skein_lib::Token;
use skein_llm::client;
use skein_llm_world::fake::Exchange;
use smith_domain::{Event, llm};

pub use smith_protocol_llm::fixture::schemas;

pub use crate::fixture::{Configuration, configurations};

/// Finite observed lower chronology, without retaining another completion body.
/// The skein Client emits these observations; at most four belong to one `ClientPeer`.
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

/// Owns the observed prepared machine and retained codec context through settlement.
/// Caller supplies receiving caps and clocks; there is at most one root terminal
/// and four lifecycle observations.
pub struct ClientPeer {
    /// The one observed Client, lower HTTP/SSE peer and shared script domain.
    pub peer: Exchange,
    /// At most one terminal plus Reusable/Close/Closed, without diagnostic bodies.
    pub observed: Vec<Observed>,
    context: Option<Context>,
}

impl core::fmt::Debug for ClientPeer {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ClientPeer")
            .field("waiting", &self.peer.machine.waiting())
            .field("observed", &self.observed)
            .field("live_context", &self.context.is_some())
            .finish()
    }
}

impl ClientPeer {
    /// The root's observed Complete request supplies owner/prompt/receiving metadata.
    /// Preparation is effect free; start is a separate Client entrance.
    /// Success owns one prepared Client/context; failure starts nothing.
    ///
    /// # Errors
    /// Returns the codec's admission error when the complete prompt, tool
    /// descriptors or receiving contract cannot fit the supplied limits.
    pub fn prepare(
        owner: Token,
        prompt: llm::Prompt,
        receiving: Receiving,
        configuration: &Configuration,
        limits: &Limits,
        scripts: Box<[skein_fake_llm_domain::api::Script]>,
    ) -> Result<Self, codec::Error> {
        let endpoint_name = prompt.endpoint;
        let endpoint = configuration.endpoint.clone();
        let peer_endpoint = endpoint.clone();
        let credential = match configuration.credential.scheme {
            skein_llm::Scheme::Bearer => skein_llm::Credential::bearer(configuration.credential.value.clone()),
            skein_llm::Scheme::Key => skein_llm::Credential::key(configuration.credential.value.clone()),
        };
        let peer_credential = match credential.scheme {
            skein_llm::Scheme::Bearer => skein_llm::Credential::bearer(credential.value.clone()),
            skein_llm::Scheme::Key => skein_llm::Credential::key(credential.value.clone()),
        };
        let codec::Prepared { client, context } =
            codec::prepare(codec::Input { owner, prompt, endpoint_name, endpoint, credential, receiving }, limits)?;
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
        Ok(Self { peer, context: Some(context), observed: Vec::new() })
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
                    let context = self.context.take().expect("one observed Client terminal owns the context");
                    returned.push(
                        codec::completion(context, completion)
                            .expect("observed admitted Client completion and production decoder fit receiving"),
                    );
                }
                client::Event::Failed { owner, failure, evidence, report, detail } => {
                    self.observed.push(Observed::Failed(owner));
                    let context = self.context.take().expect("one observed Client terminal owns the context");
                    returned.push(codec::failed(context, failure, evidence, detail, report).expect(
                        "observed admitted failure retains owner, class, evidence and full bounded diagnostic",
                    ));
                }
                client::Event::Cancelled { owner } => {
                    self.observed.push(Observed::Cancelled(owner));
                    let context = self.context.take().expect("one observed Client terminal owns the context");
                    returned.push(codec::cancelled(context));
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
