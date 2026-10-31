//! Borrowed framed consumers of the shared lower stages.

use super::{Frame, channel, domain, io};

pub(super) fn domain(frame: Option<&Frame>) -> Option<&domain::Domain> {
    match frame {
        Some(frame) => Some(&frame.domain),
        None => None,
    }
}
pub(super) fn domain_mut(frame: Option<&mut Frame>) -> Option<&mut domain::Domain> {
    match frame {
        Some(frame) => Some(&mut frame.domain),
        None => None,
    }
}
pub(super) fn channel(frame: Option<&Frame>) -> Option<&channel::Component> {
    match frame {
        Some(frame) => Some(&frame.channel),
        None => None,
    }
}
pub(super) fn channel_mut(frame: Option<&mut Frame>) -> Option<&mut channel::Component> {
    match frame {
        Some(frame) => Some(&mut frame.channel),
        None => None,
    }
}
pub(super) fn io(frame: Option<&Frame>) -> Option<&io::Io> {
    match frame {
        Some(frame) => Some(&frame.io),
        None => None,
    }
}
pub(super) fn io_mut(frame: Option<&mut Frame>) -> Option<&mut io::Io> {
    match frame {
        Some(frame) => Some(&mut frame.io),
        None => None,
    }
}
