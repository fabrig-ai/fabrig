//! Exact request bytes and ordered answers (jig/docs/design/protocol/client/default-http.md, 5).
use super::{
    http::Answer,
    wire::{Event, Wire},
};
use skein_http::Method;
use skein_io::kernel::{Complete, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use std::{collections::BTreeMap, net::SocketAddr};

pub(crate) struct Visitor {
    pub answers: BTreeMap<u64, Option<Answer>>,
    wire: Wire,
}
impl Visitor {
    pub fn new() -> Self {
        Self { answers: BTreeMap::new(), wire: Wire::new(None) }
    }
    pub fn abandon(&mut self, id: u64) {
        self.wire.abandon(id);
    }
    pub fn send(&mut self, address: SocketAddr, method: Method, path: &[u8], headers: &[(&[u8], &[u8])]) -> u64 {
        self.send_body(address, method, path, headers, &[])
    }
    pub fn send_body(
        &mut self,
        address: SocketAddr,
        method: Method,
        path: &[u8],
        headers: &[(&[u8], &[u8])],
        body: &[u8],
    ) -> u64 {
        let mut bytes = method.as_bytes().to_vec();
        bytes.extend_from_slice(b" ");
        bytes.extend_from_slice(path);
        bytes.extend_from_slice(b" HTTP/1.1\r\n");
        for (name, value) in headers {
            bytes.extend_from_slice(name);
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(value);
            bytes.extend_from_slice(b"\r\n");
        }
        bytes.extend_from_slice(format!("Connection: close\r\nContent-Length: {}\r\n\r\n", body.len()).as_bytes());
        bytes.extend_from_slice(body);
        self.wire.start(address, method, bytes.into())
    }
}
impl Host for Visitor {
    fn iterate(&mut self, _: Time, _: Wall) {
        self.wire.iterate();
        while let Some(event) = self.wire.events.pop_front() {
            match event {
                Event::Answer { id, answer } => {
                    self.answers.insert(id, answer);
                }
                Event::Request { .. } => panic!("visitor never listens"),
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.wire.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.wire.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        !self.wire.completions.is_empty() || !self.wire.submissions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.wire.empty()
    }
    fn operations(&self) -> u32 {
        256
    }
}
