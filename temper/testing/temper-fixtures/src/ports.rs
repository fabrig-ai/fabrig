//! Loopback ports a test lets go of and listens on again (temper/docs/design/domain/testing.md, 2.6).
use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    ops::Range,
    sync::Mutex,
};

const LOWEST: u16 = 10_000;
const BLOCK: u16 = 64;
const RANGE_FILE: &str = "/proc/sys/net/ipv4/ip_local_port_range";
static BLOCKS: Mutex<Option<Blocks>> = Mutex::new(None);

/// `N` distinct listeners, none at an address in `taken`, on ports below the
/// kernel's ephemeral range, in blocks this process holds until it ends.
pub fn reserve<const N: usize>(taken: &[SocketAddr]) -> [TcpListener; N] {
    let mut blocks = BLOCKS.lock().expect("port chooser lock");
    let blocks = blocks.get_or_insert_with(|| Blocks::new(ephemeral_first(), std::process::id()));
    std::array::from_fn(|_| blocks.listener(taken))
}

/// As [`reserve`], with the listeners let go: nothing listens at these
/// addresses now, and no other test process will draw them.
pub fn addresses<const N: usize>(taken: &[SocketAddr]) -> [SocketAddr; N] {
    reserve::<N>(taken).map(|listener| listener.local_addr().expect("chosen listener address"))
}

/// The first port of the kernel's ephemeral range. Reads
/// `/proc/sys/net/ipv4/ip_local_port_range`; where that file doesn't exist
/// (macOS, temper/docs/design/domain/testing.md, 2.8), 49152,
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

struct Blocks {
    claims: Vec<TcpListener>,
    next: Range<u16>,
    first: u16,
    end: u16,
}
impl Blocks {
    fn new(ephemeral_first: u16, start: u32) -> Self {
        let blocks = ephemeral_first.saturating_sub(LOWEST) / BLOCK;
        assert!(blocks >= 2, "{RANGE_FILE}: first port {ephemeral_first} leaves fewer than two blocks above {LOWEST}");
        let first = LOWEST + u16::try_from(start % u32::from(blocks)).expect("block index fits") * BLOCK;
        Self { claims: Vec::new(), next: first..first, first, end: LOWEST + blocks * BLOCK }
    }

    fn claim(&mut self) {
        let began = self.first;
        loop {
            let first = self.first;
            self.first = if first + BLOCK == self.end { LOWEST } else { first + BLOCK };
            if let Ok(claim) = TcpListener::bind((Ipv4Addr::LOCALHOST, first)) {
                self.claims.push(claim);
                self.next = first + 1..first + BLOCK;
                return;
            }
            assert_ne!(self.first, began, "no free port block in {LOWEST}..{}", self.end);
        }
    }

    fn listener(&mut self, taken: &[SocketAddr]) -> TcpListener {
        loop {
            for port in self.next.by_ref() {
                let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
                if !taken.contains(&address)
                    && let Ok(listener) = TcpListener::bind(address)
                {
                    return listener;
                }
            }
            self.claim();
        }
    }
}

#[cfg(test)]
mod tests;
