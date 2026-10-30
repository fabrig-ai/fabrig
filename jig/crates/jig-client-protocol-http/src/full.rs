//! Named-bound conflict bodies (jig/docs/design/protocol/client.md, 17.2).
use jig_client_domain::{StreamBound, StreamFull};
use jig_client_protocol_wire::v19 as w;
use skein_lib::Reader;

pub(crate) const BYTES: usize = 5;

pub(crate) fn decode(body: &[u8]) -> Option<StreamFull> {
    if body.len() != BYTES {
        return None;
    }
    let record = w::StreamFull::decode(&w::CEILINGS, &mut Reader::new(body)).ok()?;
    let bound = match record.bound() {
        w::StreamBound::Component => StreamBound::Component,
        w::StreamBound::Transport => StreamBound::Transport,
        w::StreamBound::Exchanges => StreamBound::Exchanges,
    };
    Some(StreamFull { bound, size: record.size() })
}
