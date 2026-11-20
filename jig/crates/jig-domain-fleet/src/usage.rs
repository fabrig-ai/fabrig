//! Usage reads need a login, not an endpoint or a slot (domain/accounts.md, 3.7).
use crate::{Domain, HostKind, LoginState, Request, SourceName};
use jig_domain_link::UsageOutcome;
use skein_lib::{List, Queue, Token};

#[derive(Hash, Debug)]
pub(crate) struct Reads {
    held: List<Held>,
    next: u64,
}
#[derive(Hash, Debug)]
struct Held {
    read: u64,
    source: SourceName,
    order: u64,
    channel: Option<Token>,
}
impl Reads {
    pub(crate) fn new(capacity: u32) -> Self {
        Self { held: List::with_capacity(capacity), next: 0 }
    }
    pub(crate) fn push(&mut self, read: u64, source: SourceName) {
        let order = self.next;
        self.next = self.next.checked_add(1).expect("usage read order lasts this run");
        self.held.push(Held { read, source, order, channel: None }).expect("one usage read per account");
    }
    pub(crate) fn cancel(&mut self, read: u64) {
        for index in 0..self.held.len() {
            if self.held.get(index).expect("held usage index").read == read {
                self.held.swap_remove(index).expect("cancelled usage read");
                return;
            }
        }
    }
    pub(crate) fn lost(&mut self, channel: Token) {
        for held in self.held.iter_mut() {
            if held.channel == Some(channel) {
                held.channel = None;
            }
        }
    }
    pub(crate) fn answered(&mut self, channel: Token, read: u64, outcome: UsageOutcome, out: &mut Queue<Request>) {
        for index in 0..self.held.len() {
            let held = self.held.get(index).expect("held usage index");
            if held.read == read && held.channel == Some(channel) {
                self.held.swap_remove(index).expect("answered usage read");
                out.push(Request::UsageRead { read, outcome });
                return;
            }
        }
    }
    fn on(&self, channel: Token) -> bool {
        for held in &self.held {
            if held.channel == Some(channel) {
                return true;
            }
        }
        false
    }
}
fn choose(domain: &Domain, source: SourceName) -> Option<Token> {
    for (_, &id) in &domain.tokens {
        let channel = domain.channels.get(id).expect("connected channel");
        if channel.kind != HostKind::Worker
            || !channel.reads_usage
            || !channel.sources.ready()
            || channel.draining
            || channel.shutting_down
            || channel.mismatched
            || domain.usage_reads.on(channel.token)
        {
            continue;
        }
        for login in channel.sources.logins() {
            if login.name == source && login.state == LoginState::Usable {
                return Some(channel.token);
            }
        }
    }
    None
}
fn next(domain: &Domain) -> Option<(u32, Token)> {
    let mut next = None;
    for index in 0..domain.usage_reads.held.len() {
        let held = domain.usage_reads.held.get(index).expect("held usage index");
        if held.channel.is_some() {
            continue;
        }
        if let Some((_, _, first)) = next
            && first <= held.order
        {
            continue;
        }
        if let Some(channel) = choose(domain, held.source) {
            next = Some((index, channel, held.order));
        }
    }
    let (index, channel, _) = next?;
    Some((index, channel))
}
pub(crate) fn ready(domain: &Domain) -> bool {
    next(domain).is_some()
}
pub(crate) fn resume(domain: &mut Domain, out: &mut Queue<Request>) {
    let Some((index, channel)) = next(domain) else { return };
    let held = domain.usage_reads.held.get_mut(index).expect("queued usage read");
    held.channel = Some(channel);
    out.push(Request::ReadUsage { channel, read: held.read, source: held.source });
}
