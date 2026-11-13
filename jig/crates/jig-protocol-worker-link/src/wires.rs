//! The engine's statement, or the worker's single versions (protocol/channel/bridges.md, 3).
use jig_protocol_worker_wire::{Agreed, Spoken, v12};
use skein_lib::List;

pub(crate) fn singletons(wires: &List<Spoken>) -> Option<List<Agreed>> {
    let mut agreed = List::with_capacity(wires.len());
    for wire in wires {
        if wire.layouts.len() != 1 {
            return None;
        }
        agreed.push(Agreed { name: wire.name, version: wire.lowest }).expect("one agreement per wire");
    }
    Some(agreed)
}

pub(crate) fn statement(wires: &List<Spoken>, statement: &v12::Agreed) -> Option<List<Agreed>> {
    let mut agreed: List<Agreed> = List::with_capacity(statement.wires().len());
    for stated in statement.wires() {
        let mut found = None;
        for wire in wires {
            if wire.name.as_bytes() == stated.name() && wire.layout(stated.version()).is_some() {
                found = Some(Agreed { name: wire.name, version: stated.version() });
            }
        }
        let found = found?;
        for earlier in &agreed {
            if earlier.name == found.name {
                return None;
            }
        }
        agreed.push(found).expect("one agreement per stated wire");
    }
    Some(agreed)
}
