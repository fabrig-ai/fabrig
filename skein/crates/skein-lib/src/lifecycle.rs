//! Component lifecycle phases (skein/docs/foundation/programming-model.md, 5.2).

/// Where a component is in its lifecycle (programming-model.md, 5.2):
/// taking requests, closing gracefully, aborting, or closed after its
/// one `Closed`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lifecycle {
    /// Taking new requests.
    Live,
    /// Refusing new requests and draining work gracefully.
    Closing,
    /// Refusing new requests and cancelling held work.
    Aborting,
    /// Ended after emitting its one `Closed`.
    Closed,
}
