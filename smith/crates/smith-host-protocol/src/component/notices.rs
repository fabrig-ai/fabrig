//! Decode bounded account notices for the host (protocol/channel.md, section 4).
use crate::{Error, Received};
use alloc::boxed::Box;
use skein_lib::Reader;
use smith_channel::Record;

pub(super) fn decode(limits: &smith_channel::Limits, kind: u16, body: &[u8]) -> Result<Received, Error> {
    match kind {
        smith_channel::kind::UNAUTHORIZED => {
            let notice = smith_channel::Unauthorized::decode(limits, &mut Reader::new(body))?;
            Ok(Received::Message {
                message: smith_host_domain::Up::Unauthorized { account: notice.account, generation: notice.generation },
            })
        }
        smith_channel::kind::USAGE => {
            let notice = smith_channel::Usage::decode(limits, &mut Reader::new(body))?;
            let window = notice.window;
            let state = match window.state {
                smith_channel::UsageState::Open => smith_host_domain::UsageState::Open,
                smith_channel::UsageState::Warned => smith_host_domain::UsageState::Warned,
                smith_channel::UsageState::Limited => smith_host_domain::UsageState::Limited,
            };
            let resets = match window.resets {
                Some(seconds) => match seconds.checked_mul(1_000_000_000) {
                    Some(nanos) => Some(skein_lib::Wall::from_nanos(nanos)),
                    None => None,
                },
                None => None,
            };
            Ok(Received::Message {
                message: smith_host_domain::Up::Usage {
                    account: notice.account,
                    window: smith_host_domain::UsageWindow {
                        label: window.label,
                        length: window.length,
                        used: window.used,
                        resets,
                        state,
                    },
                },
            })
        }
        smith_channel::kind::LIMITED => {
            let notice = smith_channel::Limited::decode(limits, &mut Reader::new(body))?;
            Ok(Received::Message {
                message: smith_host_domain::Up::Limited { account: notice.account, reset: notice.reset },
            })
        }
        smith_channel::kind::ACCOUNT_REFUSED => {
            let notice = smith_channel::AccountRefused::decode(limits, &mut Reader::new(body))?;
            Ok(Received::Message {
                message: smith_host_domain::Up::Refused {
                    account: notice.account,
                    code: Box::from(notice.code.as_ref()),
                    reason: Box::from(notice.reason.as_ref()),
                },
            })
        }
        _ => Err(Error::InvalidAnswer),
    }
}
