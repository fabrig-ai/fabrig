//! Components own disjoint token ranges; nibble 3 is subdivided by worker.
use skein_lib::Token;
/// Termination signals' source, outside component ranges.
pub const SIGNALS: u64 = 1;
/// Workers' listeners, as jig's reference allocates them.
pub const WORKER_LISTENERS: u64 = 1_u64 << 60;
/// Workers' accepted connections, as jig's reference allocates them.
pub const WORKER_CONNECTIONS: u64 = 2_u64 << 60;
/// Accounts' driver; workers use nonzero subranges in the same nibble.
pub const ACCOUNTS: u64 = 3_u64 << 60;
/// Clients' component.
pub const CLIENTS: u64 = 4_u64 << 60;
/// Forge component.
pub const FORGE: u64 = 5_u64 << 60;
/// Workers' git components, each in a nonzero subrange.
pub const GIT: u64 = 6_u64 << 60;
/// Clients listener and its HTTP connections.
pub const SERVER: u64 = 7_u64 << 60;
/// Store file drivers' kernel operations, as jig's reference allocates them.
pub const STORE: [u64; 3] = [8_u64 << 60, 9_u64 << 60, 10_u64 << 60];
/// Setup host IO and durable-write terminals.
pub const SETUP: u64 = 11_u64 << 60;
/// Worker index's tokens; refuses an index that reaches the next nibble.
#[must_use]
pub fn worker(index: u32) -> Option<u64> {
    subrange(ACCOUNTS, index)
}
/// Worker index's git tokens; refuses an index that reaches the next nibble.
#[must_use]
pub fn git(index: u32) -> Option<u64> {
    subrange(GIT, index)
}
fn subrange(base: u64, index: u32) -> Option<u64> {
    let number = index.checked_add(1)?;
    if number >= 4096 {
        return None;
    }
    base.checked_add(u64::from(number) << 48)
}
/// Token's top four bits, shifted down.
#[must_use]
pub fn prefix(token: Token) -> u8 {
    u8::try_from(token.raw() >> 60).expect("four bits fit u8")
}
