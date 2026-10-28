//! The protocol layer's edge limits (programming-model.md, 7).

use skein_lib::Duration;

/// The answer to an `Open` the domain refused (examples.md, 3.1).
pub const BUSY: &[u8] = b"busy\n";

/// The answer to a line past `Limits::line` (examples.md, 3.1).
pub const TOO_LONG: &[u8] = b"too long\n";

/// The protocol layer's limits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Connections at once: an accepted socket past it is rejected.
    pub conns: u32,
    /// The longest line, its `\n` included: the most a scan asks for, and the
    /// room each answer asks for. At least the longest refusal.
    pub line: u32,
    /// How long a connection may make no progress (a line read, room for an
    /// answer granted) before it is closed.
    pub idle: Duration,
    /// How far each idle deadline is spread past `idle`, drawn from the seed,
    /// so that connections opened together do not all expire together.
    pub spread: Duration,
    /// How long the listener waits to listen again after io refused it for
    /// want of resources (`Error::Busy`, io.md, 2). Never zero, or it would
    /// spin while the shortage lasts.
    pub retry: Duration,
}

impl Limits {
    /// The largest read the layer demands of io: a scan of `line` bytes.
    /// Startup checks it against io's intake (io.md, 2).
    #[must_use]
    pub const fn largest_read(&self) -> u32 {
        self.line
    }

    /// The largest room the layer demands of io: `line` bytes, for any
    /// answer. Startup checks it against io's output cap (io.md, 2).
    #[must_use]
    pub const fn largest_room(&self) -> u32 {
        self.line
    }

    /// Whether the layer can run under these limits: a connection, a line
    /// that holds the longest refusal, and an idle deadline and a retry that
    /// wait.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        let holds_refusals = match usize::try_from(self.line) {
            Ok(line) => line >= TOO_LONG.len() && line >= BUSY.len(),
            Err(_) => true,
        };
        self.conns > 0 && holds_refusals && self.idle.as_nanos() > 0 && self.retry.as_nanos() > 0
    }
}
