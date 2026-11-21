//! The owned probe identity used by relay value and framing tests.
use crate::relay::Guard;

pub fn guard() -> Guard {
    Guard { owner: "owner".into(), repository: "probe".into(), installation: 17, prefix: "temper/live-1/".into() }
}

/// A receive-pack body: one pkt-line command per ref, then the flush.
pub fn push(refs: &[&str]) -> Vec<u8> {
    let mut body = Vec::new();
    for (index, name) in refs.iter().enumerate() {
        let caps = if index == 0 { "\0report-status" } else { "" };
        let line = format!("{} {} {name}{caps}\n", "0".repeat(40), "1".repeat(40));
        body.extend_from_slice(format!("{:04x}{line}", line.len() + 4).as_bytes());
    }
    body.extend_from_slice(b"0000PACK");
    body
}
