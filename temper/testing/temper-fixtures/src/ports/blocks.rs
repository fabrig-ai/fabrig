use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    ops::Range,
};

pub const LOWEST: u16 = 10_000;
pub const BLOCK: u16 = 64;
pub const RANGE_FILE: &str = "/proc/sys/net/ipv4/ip_local_port_range";

pub struct Blocks {
    claims: Vec<TcpListener>,
    pub next: Range<u16>,
    first: u16,
    end: u16,
}
impl Blocks {
    pub fn new(ephemeral_first: u16, start: u32) -> Result<Self, String> {
        let blocks = ephemeral_first.saturating_sub(LOWEST) / BLOCK;
        if blocks < 2 {
            return Err(format!(
                "{RANGE_FILE}: first port {ephemeral_first} leaves fewer than two blocks above {LOWEST}"
            ));
        }
        let first = LOWEST + u16::try_from(start % u32::from(blocks)).expect("block index fits") * BLOCK;
        Ok(Self { claims: Vec::new(), next: first..first, first, end: LOWEST + blocks * BLOCK })
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

    pub fn listener(&mut self, taken: &[SocketAddr]) -> TcpListener {
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
