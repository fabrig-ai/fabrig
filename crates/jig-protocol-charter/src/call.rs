//! Smith's call name, read at the engine's protocol boundary
//! (protocol/channel.md, section 7).
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

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
    if !reader.is_empty() || value.activation() != attempt {
        return None;
    }
    Some(CallKey { activation: value.activation(), completion: value.completion(), position: value.position() })
}

/// Smith's call name for a settled call, from its key (store.md, 5.1):
/// sent again in a replay or through the in-process hub.
#[must_use]
pub fn call_name(key: &CallKey) -> Box<[u8]> {
    let value = smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation: key.activation, completion: key.completion, position: key.position },
    )
    .expect("call name integers fit their codec");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("call name length fits"));
    value.encode(&mut writer).expect("measured call name room");
    writer.finish()
}
