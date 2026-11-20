//! The opt-in walk issuer, with the same plan refill as a box's standalone peer.
use skein_fake_peers::oauth::Peer;
use skein_io::kernel::{Complete, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;

pub struct Issuer {
    peer: Peer,
    seed: u64,
}
impl Issuer {
    #[must_use]
    pub fn new(document: &temper_fake::oauth::Document, seed: u64) -> Self {
        Self { peer: document.peer(seed).expect("walk issuer"), seed }
    }
    pub fn shutdown(&mut self) {
        self.peer.shutdown();
    }
}
impl Host for Issuer {
    fn iterate(&mut self, now: Time, wall: Wall) {
        temper_fake::oauth::refill(&mut self.peer, self.seed).expect("bounded issuer plans");
        self.peer.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.peer.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.peer.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.peer.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.peer.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.peer.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.peer.is_empty()
    }
    fn operations(&self) -> u32 {
        self.peer.operations()
    }
}

/// Provider 1's `OpenAI` dialect is hosted at the story model's endpoint zero.
pub(super) fn endpoints() -> smith_protocol_channel::Endpoints {
    let mut entries = skein_lib::List::with_capacity(1);
    entries
        .push(smith_protocol_channel::Endpoint { name: b"openai".as_slice().into(), number: 0, dialect: 3 })
        .expect("one model endpoint");
    smith_protocol_channel::Endpoints::new(entries)
}

pub(super) fn runtime(document: &crate::documents::Configuration) -> Option<temper_service::AccountsRuntime> {
    let issuer = document["world_settings"]["issuer"].as_str()?;
    let endpoint = || skein_oauth_accounts::Endpoint {
        address: issuer.parse().expect("walk issuer socket"),
        transport: skein_oauth_accounts::Transport::Plaintext,
    };
    let mut origins = skein_lib::List::with_capacity(1);
    origins
        .push(skein_oauth_accounts::Origin {
            uri: format!("http://{issuer}").into_bytes().into_boxed_slice(),
            endpoint: endpoint(),
        })
        .expect("one issuer origin");
    let mut providers = skein_lib::List::with_capacity(1);
    providers.push((1, endpoint(), origins)).expect("the model's plan provider");
    Some(temper_service::AccountsRuntime { providers, trust: None })
}
