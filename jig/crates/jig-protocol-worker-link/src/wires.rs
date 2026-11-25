//! Construction checks and the initiator's terms (jig/docs/design/protocol/channel/wires.md, 5).
use jig_protocol_worker_wire::{Kind, Root, Spoken};
use skein_channel::{Direction, Term, Version};
use skein_lib::List;

pub(crate) fn roots_present(spoken: &List<Spoken>, roots: &List<Root>) -> bool {
    for root in roots {
        let mut named = false;
        for wire in spoken {
            if wire.name == root.wire {
                named = true;
                break;
            }
        }
        if !named {
            return false;
        }
    }
    for wire in spoken {
        let mut schema = None;
        for root in roots {
            if root.wire == wire.name {
                if schema.is_none() {
                    let Ok(parsed) = skein_codec::schema::parse(wire.text.as_bytes()) else { return false };
                    schema = Some(parsed);
                }
                if schema.as_ref().expect("the root's text is parsed").find(root.declaration.as_bytes()).is_none() {
                    return false;
                }
            }
        }
    }
    true
}

pub(crate) fn optional(kind: u16) -> bool {
    for notice in [Kind::Paused, Kind::Part, Kind::Usage, Kind::UsageRead] {
        if notice.number() == kind {
            return true;
        }
    }
    false
}

pub(crate) fn terms_gap(table: &Version, terms: &List<Term>) -> Option<u16> {
    for rule in &table.kinds {
        if rule.direction == Direction::FromInitiator {
            let mut bound = None;
            for term in terms {
                if term.kind == rule.kind {
                    bound = Some(term.largest);
                    break;
                }
            }
            match bound {
                Some(bound) if bound >= rule.largest => {}
                None if optional(rule.kind) => {}
                Some(_) | None => return Some(rule.kind),
            }
        }
    }
    None
}
