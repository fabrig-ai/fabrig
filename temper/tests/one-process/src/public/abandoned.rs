//! An abandoned answer refunds its reservation (jig/docs/design/protocol/client/public-reads.md).
use super::{checks::PROXY, proxy::Proxy, service_checks, visitor::Visitor};
use crate::fixtures;
use skein_http::Method;
use skein_lib::Time;

pub(super) struct Walk {
    stage: usize,
    calls: Vec<u64>,
    wake: Option<Time>,
}
impl Walk {
    pub fn new() -> Self {
        Self { stage: 0, calls: Vec::new(), wake: None }
    }
    pub fn wake(&self) -> Option<Time> {
        self.wake
    }
    pub fn act(&mut self, now: Time, proxy: &Proxy, visitor: &mut Visitor, project: u64) -> Result<bool, String> {
        if self.wake.is_some_and(|wake| now < wake) || self.calls.iter().any(|id| !visitor.answers.contains_key(id)) {
            return Ok(false);
        }
        self.wake = None;
        let path = format!("/jig/v1/public/2/2/2/{project:016x}");
        match self.stage {
            0 => {
                // Each landings read reserves 198,733 bytes, two within the 512 KiB burst.
                for address in [fixtures::CLIENTS, PROXY] {
                    let id = visitor.send(address, Method::Get, path.as_bytes(), &[(b"Host", b"one.example")]);
                    visitor.abandon(id);
                    self.calls.push(id);
                }
                self.stage = 1;
            }
            1 => {
                if !(self.calls.iter().all(|id| visitor.answers[id].is_none())) {
                    return Err(String::from("connections closed before reading their answers"));
                }
                if !proxy.idle() {
                    return Ok(false);
                }
                self.wake = Some(Time::from_nanos(now.as_nanos() + 1_000_000_000));
                self.stage = 2;
            }
            2 => {
                self.calls.push(visitor.send(
                    fixtures::CLIENTS,
                    Method::Get,
                    path.as_bytes(),
                    &[(b"Host", b"one.example")],
                ));
                self.stage = 3;
            }
            3 => {
                let answer = visitor.answers[self.calls.last().ok_or_else(|| String::from("fresh read"))?]
                    .as_ref()
                    .ok_or_else(|| String::from("HTTP answer"))?;
                match (&(answer.status), &(200)) {
                    (left, right) if left != right => {
                        let finding = String::from("all eight burst reads after abandonment are answered");
                        return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                    }
                    _ => {}
                }
                service_checks::policy(answer, false)?;
                if self.calls.len() == 10 {
                    return Ok(true);
                }
                self.stage = 2;
            }
            other => return Err(format!("abandonment stage {other}")),
        }
        Ok(false)
    }
}
