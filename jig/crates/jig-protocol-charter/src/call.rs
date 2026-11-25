//! Smith's call name, read at the engine's protocol boundary
//! (protocol/channel/README.md, section 6).
use crate::generated::channel;
use alloc::boxed::Box;
use skein_lib::Reader;
use smith_channel::Record;

/// The name's key, checked against the attempt at the protocol boundary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CallKey {
    pub activation: u64,
    pub completion: u32,
    pub position: u32,
}

#[must_use]
pub fn call_key(name: &[u8], attempt: u64) -> Option<CallKey> {
    let mut reader = Reader::new(name);
    let value = smith_channel::CallName::decode(&smith_channel::CEILINGS, &mut reader).ok()?;
    if !reader.is_empty() || value.activation != attempt {
        return None;
    }
    Some(channel::call_name_to_domain(value))
}

/// Smith's call name for a settled call, from its key (protocol/store/records.md, 1):
/// sent again in a replay or through the in-process hub.
#[must_use]
pub fn call_name(key: &CallKey) -> Box<[u8]> {
    let value = channel::call_name_to_record(key);
    value.encode(&smith_channel::CEILINGS).expect("call name has no variable bounds")
}
