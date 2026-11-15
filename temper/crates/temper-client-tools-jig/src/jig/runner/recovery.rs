//! Restored work keeps operation facts, with identities left in the client.
use super::{Out, Runner};
use crate::jig::render::{Text, outcome::Operation};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Ask, Client, Key, Latest, contract as c};
use skein_json::writer::Encoder;
use skein_lib::Queue;
pub(super) fn kept(runner: &mut Runner, key: Key, request: &Ask, client: &Client) {
    let mut operation = Operation::of(request);
    if let Some(pending) = client.pending(key) {
        operation.party = Some(pending.party);
    }
    runner.kept.insert(key, operation);
}
fn party(runner: &Runner, client: &Client) -> Option<u64> {
    match client.document(runner.me())?.latest? {
        Latest::Jig(c::Delivery::Me(me)) => Some(me.party.number),
        Latest::Jig(_) | Latest::Family(_) => None,
    }
}
pub(super) fn count(runner: &Runner, client: &Client) -> u32 {
    let Some(party) = party(runner, client) else { return 0 };
    let mut count = 0u32;
    for key in runner.kept.keys() {
        if let Some(pending) = client.pending(*key)
            && pending.party == party
        {
            count = count.checked_add(1).expect("bounded requests");
        }
    }
    count
}
pub(super) fn parked(runner: &Runner, client: &Client, out: &mut Queue<Out>) {
    let Some(party) = party(runner, client) else { return };
    let mut parked: BTreeMap<u64, u32> = BTreeMap::new();
    for key in runner.kept.keys() {
        if let Some(pending) = client.pending(*key)
            && pending.party != party
        {
            let count = parked.entry(pending.party).or_insert(0);
            *count = count.checked_add(1).expect("bounded requests");
        }
    }
    for (party, count) in parked {
        out.push(Out::Kept { line: line(runner.kept_json, Some(party), count) });
    }
}
pub(super) fn dropped(runner: &Runner, count: u32, out: &mut Queue<Out>) {
    out.push(Out::Kept { line: line(runner.kept_json, None, count) });
}
fn line(json: bool, party: Option<u64>, count: u32) -> Box<[u8]> {
    if json {
        let bounds = skein_json::writer::Limits { depth: 1, length: 256 };
        let mut measure = Encoder::measure(&bounds);
        encode(party, count, &mut measure);
        let mut write = Encoder::write(measure.measured().expect("count line fits"), &bounds);
        encode(party, count, &mut write);
        crate::jig::render::outcome::newline(&write.finish())
    } else {
        let mut measure = Text::measure();
        text(party, count, &mut measure);
        let mut write = Text::writing_exact(measure);
        text(party, count, &mut write);
        write.finish()
    }
}
fn encode(party: Option<u64>, count: u32, out: &mut Encoder) {
    out.object_start();
    match party {
        Some(party) => {
            out.key(b"party");
            out.unsigned(party);
            out.key(b"unsent");
        }
        None => out.key(b"dropped"),
    }
    out.unsigned(u64::from(count));
    out.object_end();
}
fn text(party: Option<u64>, count: u32, out: &mut Text) {
    match party {
        Some(party) => {
            out.put(b"kept: ");
            out.number(u64::from(count));
            out.put(b" for party ");
            out.number(party);
            out.put(b", unsent while it is not signed in here\n");
        }
        None => {
            out.put(b"dropped: ");
            out.number(u64::from(count));
            out.put(b" kept records these tools cannot read\n");
        }
    }
}
