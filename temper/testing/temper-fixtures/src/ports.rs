//! Loopback ports a test lets go of and listens on again (temper/docs/design/testing/process-tier.md).
use std::{
    net::{SocketAddr, TcpListener},
    sync::Mutex,
};

pub(crate) mod blocks;
use blocks::{Blocks, RANGE_FILE};
static BLOCKS: Mutex<Option<Blocks>> = Mutex::new(None);

/// `N` distinct listeners, none at an address in `taken`, on ports below the
/// kernel's ephemeral range, in blocks this process holds until it ends.
pub fn reserve<const N: usize>(taken: &[SocketAddr]) -> [TcpListener; N] {
    let mut blocks = BLOCKS.lock().expect("port chooser lock");
    let blocks = blocks.get_or_insert_with(|| {
        Blocks::new(ephemeral_first(), std::process::id()).unwrap_or_else(|finding| panic!("{finding}"))
    });
    std::array::from_fn(|_| blocks.listener(taken))
}

/// As [`reserve`], with the listeners let go: nothing listens at these
/// addresses now, and no other test process will draw them.
pub fn addresses<const N: usize>(taken: &[SocketAddr]) -> [SocketAddr; N] {
    reserve::<N>(taken).map(|listener| listener.local_addr().expect("chosen listener address"))
}

/// The first port of the kernel's ephemeral range. Reads
/// `/proc/sys/net/ipv4/ip_local_port_range`; where that file doesn't exist
/// (macOS, temper/docs/design/testing/tiers.md, 1.8), 49152,
/// `net.inet.ip.portrange.first`'s default.
pub fn ephemeral_first() -> u16 {
    match std::fs::read_to_string(RANGE_FILE) {
        Ok(text) => first_of(&text).unwrap_or_else(|| panic!("{RANGE_FILE}: invalid range {text:?}")),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => 49152,
        Err(why) => panic!("{RANGE_FILE}: {why}"),
    }
}

fn first_of(text: &str) -> Option<u16> {
    let mut words = text.split_whitespace();
    let first = words.next()?.parse().ok()?;
    let last: u16 = words.next()?.parse().ok()?;
    (first <= last && words.next().is_none()).then_some(first)
}

#[cfg(test)]
mod tests;
