use super::Opened;
use crate::openai::DecodeError;
use skein_lib::List;

pub(crate) use crate::wire::reasoning::{append, charge, replay_item, write_replay};

pub(super) fn following_index(opened: &List<(u32, Opened)>) -> Result<u32, DecodeError> {
    match opened.last() {
        Some((index, _)) => index.checked_add(1).ok_or(DecodeError::Malformed),
        None => Ok(0),
    }
}
pub(super) fn slot(opened: &List<(u32, Opened)>, index: u32) -> Result<u32, DecodeError> {
    for position in 0..opened.len() {
        if let Some((provider, _)) = opened.get(position)
            && *provider == index
        {
            return Ok(position);
        }
    }
    Err(DecodeError::Malformed)
}
