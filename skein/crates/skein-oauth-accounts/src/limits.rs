//! Account limits and edge capacity validation (oauth.md, section 6.7).

use skein_lib::Duration;
use skein_oauth::ClientLimits;

pub(crate) const MAX_ORIGINS: u32 = 3;

/// The component's limits supplied by its owner before the loop.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Configured and added accounts, including removals still settling.
    pub accounts: u32,
    /// Deadline stated on each private keeper request, including cleanup.
    pub file_stall: Duration,
    /// How long a shared write waits for the owner lock.
    pub lock_wait: Duration,
    pub exchanges: u32,
    /// Public redirect listeners admitted at once; zero permits only confidential sign-ins.
    pub listeners: u32,
    pub server: skein_http::server::Limits,
    /// How long before expiry a held record refreshes or an access-only one announces Expiring.
    pub refresh_lead: Duration,
    pub client: ClientLimits,
    pub http: skein_http::client::Limits,
    pub tls: skein_tls::client::Limits,
    /// The owner's socket limits for transport admission and demands.
    pub io: skein_io::Limits,
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
/// Account timing, socket tags, stacked demands and child limits are usable.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.file_stall > Duration::ZERO
        && limits.lock_wait > Duration::ZERO
        && limits.accounts > 0
        && limits.exchanges > 0
        && limits.exchanges <= u32::MAX.checked_div(3).expect("nonzero divisor")
        && match socket_count(limits) {
            Some(sockets) => limits.io.sockets >= sockets,
            None => false,
        }
        && (limits.listeners == 0
            || (skein_http::server::valid(&limits.server)
                && skein_http::server::largest_read(&limits.server) <= limits.io.intake
                && skein_http::server::largest_room(&limits.server) <= limits.io.output))
        && limits.client.state_bytes >= 32
        && limits.client.document.token_bytes > 0
        && limits.client.document.record_bytes > 0
        && limits.http.head >= 2
        && limits.http.headers >= 2
        && limits.http.read > 0
        && limits.http.send > 0
        && limits.http.request > 0
        && limits.tls.read >= skein_http::client::largest_read(&limits.http)
        && limits.tls.send >= skein_http::client::largest_room(&limits.http)
        && limits.io.intake >= skein_tls::client::LARGEST_READ
        && limits.io.output >= skein_tls::client::largest_room(&limits.tls)
        && skein_oauth::client_valid(&limits.client)
        && skein_http::client::valid(&limits.http)
        && skein_tls::client::valid(&limits.tls)
        && io_capacity_bound(limits).is_some()
}

fn socket_count(limits: &Limits) -> Option<u32> {
    limits.listeners.checked_mul(2)?.checked_add(limits.exchanges)
}

// io retires operation slots until reclaim and reserves a second set.
// This bound also implies its two deadline slots per socket fit u32.
fn io_capacity_bound(limits: &Limits) -> Option<u32> {
    skein_io::operations(&limits.io)?.checked_mul(2)
}
