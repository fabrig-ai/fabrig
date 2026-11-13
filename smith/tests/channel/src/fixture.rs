//! Scripted root activation from the channel's decoded host input.
//! Contract: smith/docs/design/protocol/channel.md, sections 3 and 5.

use skein_lib::Token;
use smith_domain::{Grant, Start};

/// Carry decoded fields into an activation without attaching filesystem roots.
#[must_use]
pub fn start(decoded: smith_protocol_translate::DecodedStart, host_run: Token) -> Start {
    Start {
        messages: decoded.messages,
        host_run,
        identity: decoded.identity,
        activation: decoded.activation,
        window: decoded.window,
        transcript: decoded.transcript,
        answered: decoded.answered,
        grants: decoded
            .grants
            .iter()
            .map(|grant| Grant { renewable: true, name: grant.name, valid: grant.valid })
            .collect(),
        ..smith_domain_fixtures::start(Token::new(1), decoded.charter)
    }
}
