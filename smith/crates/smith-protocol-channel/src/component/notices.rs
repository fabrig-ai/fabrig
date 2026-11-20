//! Bounded account notices (smith/docs/design/protocol/channel.md, section 4).
use super::Component;
use crate::{Error, Heard};
use alloc::boxed::Box;
use skein_channel::{Lower, Request};
use skein_lib::{Duration, Queue, Token};
use smith_channel::Record;

impl Component {
    /// Tell the host a provider rejected one named credential generation.
    pub fn send_unauthorized(
        &mut self,
        name: smith_domain::GrantName,
        token: Token,
        to_service: &mut Queue<Heard>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let record = smith_channel::Unauthorized { account: name.account, generation: name.generation };
        match record.measure(&self.bodies) {
            Ok(_) => (),
            Err(problem) => return Err(Error::ChannelBody(problem)),
        }
        let frame = match smith_channel::frame(smith_channel::Body::Unauthorized(record)) {
            Ok(value) => value,
            Err(problem) => return Err(Error::Frame(problem)),
        };
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Tell the host one program allowance observation, before any resulting limit.
    pub fn send_usage(
        &mut self,
        account: u32,
        window: smith_domain::UsageWindow,
        token: Token,
        to_service: &mut Queue<Heard>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let state = match window.state {
            smith_domain::UsageState::Open => smith_channel::UsageState::Open,
            smith_domain::UsageState::Warned => smith_channel::UsageState::Warned,
            smith_domain::UsageState::Limited => smith_channel::UsageState::Limited,
        };
        let window = smith_channel::UsageWindow {
            label: window.label,
            length: window.length,
            used: window.used,
            resets: window.resets,
            state,
        };
        let record = smith_channel::Usage { account, window };
        match record.measure(&self.bodies) {
            Ok(_) => (),
            Err(problem) => return Err(Error::ChannelBody(problem)),
        }
        let frame = match smith_channel::frame(smith_channel::Body::Usage(record)) {
            Ok(value) => value,
            Err(problem) => return Err(Error::Frame(problem)),
        };
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Tell the host an account reached its usage limit, with an optional reset.
    pub fn send_limited(
        &mut self,
        account: u32,
        reset: Option<Duration>,
        token: Token,
        to_service: &mut Queue<Heard>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let record = smith_channel::Limited { account, reset };
        match record.measure(&self.bodies) {
            Ok(_) => (),
            Err(problem) => return Err(Error::ChannelBody(problem)),
        }
        let frame = match smith_channel::frame(smith_channel::Body::Limited(record)) {
            Ok(value) => value,
            Err(problem) => return Err(Error::Frame(problem)),
        };
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Tell the host why the provider refused an account's request.
    pub fn send_refused(
        &mut self,
        account: u32,
        code: &[u8],
        reason: &[u8],
        token: Token,
        to_service: &mut Queue<Heard>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let record = refused_notice(&self.bodies, account, code, reason)?;
        match record.measure(&self.bodies) {
            Ok(_) => (),
            Err(problem) => return Err(Error::ChannelBody(problem)),
        }
        let frame = match smith_channel::frame(smith_channel::Body::AccountRefused(record)) {
            Ok(value) => value,
            Err(problem) => return Err(Error::Frame(problem)),
        };
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }
}

fn refused_notice(
    limits: &smith_channel::Limits,
    account: u32,
    code: &[u8],
    reason: &[u8],
) -> Result<smith_channel::AccountRefused, Error> {
    let code =
        if code.len() <= usize::try_from(limits.account_refused_code.min(128)).expect("u32 fits") { code } else { b"" };
    let mut end = reason.len().min(usize::try_from(limits.account_refused_reason.min(512)).expect("u32 fits"));
    for _ in 0_u8..3_u8 {
        match reason.get(end) {
            Some(byte) if byte & 0xc0 == 0x80 => end = end.saturating_sub(1),
            Some(_) | None => break,
        }
    }
    Ok(smith_channel::AccountRefused {
        account,
        code: Box::from(code),
        reason: Box::from(reason.get(..end).ok_or(Error::Grants)?),
    })
}

#[cfg(test)]
mod tests {
    use super::refused_notice;

    #[test]
    fn a_refused_notice_keeps_its_bounds() {
        // protocol/limits.md section 2.4 fixes these text bounds.
        const CODE_BOUND: usize = 128;
        const REASON_BOUND: usize = 512;
        let limits = smith_channel::CEILINGS;
        let long_code = [b'x'; CODE_BOUND + 1];
        let prefix = [b'a'; REASON_BOUND - 1];
        let mut cut_reason = [b'a'; REASON_BOUND + 1];
        cut_reason[REASON_BOUND - 1] = 0xc3;
        cut_reason[REASON_BOUND] = 0xa9;
        for (code, reason, expected_code, expected_reason) in [
            (&long_code[..], b"reason".as_slice(), b"".as_slice(), b"reason".as_slice()),
            (b"code".as_slice(), &cut_reason[..], b"code".as_slice(), &prefix[..]),
        ] {
            let notice = refused_notice(&limits, 7, code, reason).expect("bounded notice");
            assert_eq!((notice.code.as_ref(), notice.reason.as_ref()), (expected_code, expected_reason));
        }
    }
}
