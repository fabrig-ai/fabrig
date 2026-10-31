//! One admitted exchange and its answer (protocol/client.md, 4.2).
use crate::{Ended, Reply};
use skein_lib::Token;
#[derive(Debug)]
pub(crate) struct Exchange {
    pub name: Token,
    pub stream: bool,
    pub response: Option<Reply>,
    pub ended: Option<Ended>,
    pub closing: bool,
}
impl Exchange {
    pub fn new(exchange: Token, stream: bool) -> Self {
        Self { name: exchange, stream, response: None, ended: None, closing: false }
    }
    pub fn respond(&mut self, reply: Reply) {
        self.response = Some(reply);
    }
    pub fn close(&mut self) {
        if self.stream {
            self.ended = Some(Ended::Stopping);
        } else {
            self.closing = true;
        }
    }
}
