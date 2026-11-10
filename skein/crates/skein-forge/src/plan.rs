//! What a plan's exchange can end in, in both dialects (forge/README.md, 4.4).
use crate::api;

/// Whether a broken exchange could have reached the server (forge/README.md, 3.4,
/// "Unavailable is provable only before the first byte").
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Failure {
    /// The server never saw the request: no byte written (a refused connect, a failed
    /// handshake, the deadline while waiting), or skein-http's `Closed`: `Unavailable`.
    Unseen,
    /// It may have been read: a reset, the stream's end, `Truncated`, `Stream`, the
    /// deadline in flight, an answer HTTP refuses: `Timeout`.
    Seen,
}

#[must_use]
pub fn transport(failure: Failure) -> api::Error {
    match failure {
        Failure::Unseen => api::Error::Unavailable,
        Failure::Seen => api::Error::Timeout,
    }
}
