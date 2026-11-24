//! Clients' exchanges and armed deadlines (jig/docs/design/domain/testing-fakes.md, 1.1).
use jig_protocol_engine_clients::Component;
use skein_lib::{Time, Token};
use skein_world::inspect::{Entry, Inspection, Release, Table, Wait, Wake};
use std::collections::{BTreeMap, BTreeSet};

/// The clients' waits, in the client protocol's words.
pub const ENGINE_CLIENTS_KINDS: &[&str] =
    &["awaiting the engine's answer", "a link's expiry", "an approval's expiry", "the heartbeat's span"];

/// Read pending exchanges and every deadline source of `Component::next_deadline`.
pub fn engine_clients(clients: &Component, into: &mut Inspection) {
    let mut sign_ins: BTreeMap<u64, BTreeSet<Token>> = BTreeMap::new();
    for index in 0..clients.inspection_exchange_count() {
        if let Some((exchange, sign_in)) = clients.inspection_exchange(index) {
            into.wait(Wait {
                waiter: format!("exchange {}", exchange.raw()),
                kind: "awaiting the engine's answer".into(),
                wakes: vec![Wake::Party("engine".into())],
                bound: None,
            });
            if let Some(sign_in) = sign_in {
                sign_ins.entry(sign_in).or_default().insert(clients.inspection_exchange_owner(exchange));
            }
        }
    }
    for exchanges in sign_ins.into_values() {
        into.table(Table {
            name: "a sign-in's pending exchanges",
            capacity: u64::from(clients.inspection_exchange_capacity()),
            entries: exchanges
                .into_iter()
                .map(|exchange| Entry {
                    key: format!("exchange {}", exchange.raw()),
                    rule: "each request's answer",
                    release: Release::Pending,
                })
                .collect(),
            refused: Vec::new(),
        });
    }
    for (local, name) in [(false, "printed link"), (true, "local link")] {
        if let Some(deadline) = clients.inspection_link_deadline(local) {
            timer(into, name.into(), "a link's expiry", deadline);
        }
    }
    for index in 0..clients.inspection_asked_link_count() {
        if let Some((sign_in, Some(deadline))) = clients.inspection_asked_link(index) {
            timer(into, format!("asked link of sign-in {sign_in}"), "a link's expiry", deadline);
        }
    }
    for index in 0..clients.inspection_approval_count() {
        if let Some(deadline) = clients.inspection_approval_deadline(index) {
            timer(into, format!("approval {index}"), "an approval's expiry", deadline);
        }
    }
    for index in 0..clients.inspection_stream_count() {
        if let Some((stream, Some(deadline))) = clients.inspection_stream(index) {
            timer(into, format!("stream {}", stream.raw()), "the heartbeat's span", deadline);
        }
    }
}

fn timer(into: &mut Inspection, waiter: String, kind: &str, deadline: Time) {
    into.wait(Wait { waiter, kind: kind.into(), wakes: vec![Wake::Timer(deadline)], bound: Some(deadline) });
}
