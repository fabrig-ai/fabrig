//! The forge worlds' io dispatch (skein/docs/design/fake-forge.md, 10.6).
use skein_io::{Addr, Event, Request};
use skein_lib::{Queue, Token, stream::{Down, Up}};
use std::{collections::BTreeMap, ops::{Deref, DerefMut}};

/// What a world's peer takes from its owner and answers within the owner's demand.
pub trait Peer {
    fn take(&mut self, down: Down);
    fn answer(&mut self) -> Option<Up>;
    /// A world may hold completion at the connecting boundary.
    fn connected(&self) -> bool { true }
}

/// The owner -> peer table every forge world plays its io through.
#[derive(Debug)]
pub struct Sockets<P>(BTreeMap<Token, P>);
impl<P> Default for Sockets<P> {
    fn default() -> Self { Self(BTreeMap::new()) }
}
impl<P> Deref for Sockets<P> {
    type Target = BTreeMap<Token, P>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<P> DerefMut for Sockets<P> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl<P: Peer> Sockets<P> {
    /// Dispatches socket requests; the world supplies connection outcomes.
    pub fn request(&mut self, request: Request, mut make: impl FnMut(Addr) -> Option<P>, out: &mut Queue<Event>) {
        match request {
            Request::Connect { owner, addr } => {
                let peer = make(addr);
                out.push(Event::Connecting { owner, socket: owner });
                if let Some(peer) = peer {
                    let connected = peer.connected();
                    assert!(self.0.insert(owner, peer).is_none(), "fresh connection");
                    if connected { out.push(Event::Connected { owner }); }
                } else {
                    out.push(Event::Failed { owner, error: skein_io::Error::Refused });
                    out.push(Event::Closed { owner });
                }
            }
            Request::Stream { stream, down } => {
                if let Some(peer) = self.0.get_mut(&stream) { peer.take(down); }
            }
            Request::Close { entity } | Request::Abort { entity } => {
                self.close(entity, out);
            }
            other => panic!("unexpected socket io {other:?}"),
        }
    }
    /// Removes a peer and queues its settlement for the world to deliver.
    pub fn close(&mut self, owner: Token, out: &mut Queue<Event>) -> Option<P> {
        let peer = self.0.remove(&owner)?;
        out.push(Event::Closed { owner });
        Some(peer)
    }
    /// Gives each peer one turn, in owner order.
    pub fn pump(&mut self, out: &mut Queue<Event>) {
        for (owner, peer) in &mut self.0 {
            if let Some(up) = peer.answer() { out.push(Event::Stream { owner: *owner, up }); }
        }
    }
}
