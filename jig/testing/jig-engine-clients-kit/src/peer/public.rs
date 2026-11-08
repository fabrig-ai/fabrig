//! Direct public component messages (jig/docs/design/protocol/client.md, 19).
use super::{Event, Peer, ReplyTo, Token, core};
use jig_protocol_engine_clients::{FromTransport, PublicRead, PublicReply};
impl Peer {
    /// Sends a public read directly, with no transport credential.
    pub fn public_read(&mut self, read: PublicRead) -> Token {
        let exchange = Token::new(self.next_socket);
        self.next_socket += 1;
        self.event(Event::Transport(FromTransport::PublicRead { exchange, read }));
        exchange
    }
    pub fn public_answer(&mut self, reply: ReplyTo, answer: core::PublicAnswer) {
        self.event(Event::Public { reply, answer });
    }
    pub fn public_reply(&mut self, exchange: Token) -> Option<PublicReply> {
        self.public_answers.remove(&exchange)
    }
    pub fn public_gone(&mut self, exchange: Token) {
        self.event(Event::Transport(FromTransport::ExchangeGone { exchange }));
    }
}
