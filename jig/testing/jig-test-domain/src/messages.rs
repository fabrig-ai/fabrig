//! Route the fleet's opaque payloads through its hub link (hosts.md, 6.4).
use crate::{Domain, Work};
pub(super) use jig_domain_hub_link::Payload;
use skein_lib::{Queue, Token};
pub(super) fn receive(domain: &mut Domain, work: &mut Queue<Work>, channel: Token, payload: Payload) {
    crate::link::route(domain, skein_lib::Time::ZERO, jig_domain_hub_link::Event::Receive { channel, payload }, work);
}
