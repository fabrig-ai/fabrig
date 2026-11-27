//! Application frame admission against direction, peer terms and queue room
//! (net/channel.md, section 7). The owner measures a frame; this module judges
//! the complete frame before ownership moves into the channel's queue.
use skein_lib::List;

use super::{Direction, Frame, Role, Room, Schema, Term, Unsent};

/// Checks one measured frame in the specified entrance order.
pub(super) fn admit(schema: &Schema, role: Role, terms: &List<Term>, room: Room, frame: &Frame) -> Result<(), Unsent> {
    let kind = schema.kind(frame.kind()).ok_or(Unsent::WrongDirection)?;
    let own_direction = match role {
        Role::Initiator => kind.direction == Direction::FromInitiator,
        Role::Responder => kind.direction == Direction::FromResponder,
    };
    if !own_direction {
        return Err(Unsent::WrongDirection);
    }
    let mut peer_bound = None;
    for term in terms {
        if term.kind == frame.kind() {
            peer_bound = Some(term.largest);
        }
    }
    let largest = peer_bound.ok_or(Unsent::PeerDoesNotTake)?;
    if frame.body_len() > largest || frame.body_len() > kind.largest {
        return Err(Unsent::TooLarge);
    }
    if frame.wire_len() > room.bytes || room.frames == 0 {
        return Err(Unsent::Full);
    }
    Ok(())
}
