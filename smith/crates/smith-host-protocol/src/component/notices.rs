//! Decode bounded account notices for the host (protocol/channel.md, section 4).
use crate::{Error, Received};
use alloc::boxed::Box;
use skein_lib::Reader;

pub(super) fn decode(limits: &smith_channel::Limits, kind: u16, body: &[u8]) -> Result<Received, Error> {
    match kind {
        smith_channel::kind::UNAUTHORIZED => {
            let notice = smith_channel::Unauthorized::decode(limits, &mut Reader::new(body))?;
            Ok(Received::Message {
                message: smith_host_domain::Up::Unauthorized {
                    account: notice.account(),
                    generation: notice.generation(),
                },
            })
        }
        smith_channel::kind::LIMITED => {
            let notice = smith_channel::Limited::decode(limits, &mut Reader::new(body))?;
            Ok(Received::Message {
                message: smith_host_domain::Up::Limited { account: notice.account(), reset: *notice.reset() },
            })
        }
        smith_channel::kind::ACCOUNT_REFUSED => {
            let notice = smith_channel::AccountRefused::decode(limits, &mut Reader::new(body))?;
            Ok(Received::Message {
                message: smith_host_domain::Up::Refused {
                    account: notice.account(),
                    code: Box::from(notice.code()),
                    reason: Box::from(notice.reason()),
                },
            })
        }
        _ => Err(Error::InvalidAnswer),
    }
}
