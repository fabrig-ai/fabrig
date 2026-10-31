//! Service token namespaces (jig/docs/design/protocol/service.md, 7 and 10;
//! skein/docs/design/io.md, 3.1).
use skein_io as io;
use skein_lib::Token;

pub(super) const SIGNALS: Token = Token::new(1);
pub(super) const WORKER_LISTENERS: u8 = 1;
pub(super) const WORKER_CONNECTIONS: u8 = 2;
pub(super) const ACCOUNTS: u8 = 3;
pub(super) const CLIENTS: u8 = 4;
pub(super) const WORKERS: u8 = 5;
pub(super) const STORE: [u8; 3] = [8, 9, 10];

pub(super) fn token(namespace: u8, value: u64) -> Token {
    Token::tagged(namespace, value).expect("service token fits its namespace")
}

/// Each worker has a nonzero subrange below its namespace's top nibble.
pub(super) fn worker(index: u32) -> Option<u64> {
    let number = index.checked_add(1)?;
    if number >= 4096 {
        return None;
    }
    Some(Token::tagged(WORKERS, u64::from(number) << 48_u32)?.raw())
}

pub(super) fn store_index(token: Token) -> Option<usize> {
    for (index, namespace) in STORE.iter().enumerate() {
        if token.tag() == *namespace {
            return Some(index);
        }
    }
    None
}

pub(super) fn tag(request: io::Request, namespace: u8) -> io::Request {
    match request.owner() {
        Some(owner) => request.with_owner(token(namespace, owner.raw())),
        None => request,
    }
}

pub(super) fn untag(event: io::Event) -> io::Event {
    match event.owner() {
        Some(owner) => event.with_owner(Token::new(owner.untagged())),
        None => event,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_namespaces_are_disjoint() {
        let namespaces =
            [WORKER_LISTENERS, WORKER_CONNECTIONS, ACCOUNTS, CLIENTS, WORKERS, STORE[0], STORE[1], STORE[2]];
        for (index, namespace) in namespaces.iter().enumerate() {
            for other in namespaces.iter().skip(index + 1) {
                assert_ne!(namespace, other);
                for value in [0, 1, 0x0fff_ffff_ffff_ffff] {
                    assert_ne!(token(*namespace, value).tag(), token(*other, value).tag());
                }
            }
            assert_ne!(SIGNALS.tag(), *namespace);
        }
        let first = Token::new(worker(0).expect("first worker"));
        assert!(!crate::accounts::owns(first));
        assert!(!crate::clients::owns(first));
        assert!(!crate::store::owns(first));
        assert_eq!(first.tag(), WORKERS);
        let last = Token::new(worker(4094).expect("last worker"));
        assert_eq!(last.tag(), WORKERS);
        assert_ne!(first, last);
        assert_eq!(worker(4095), None);
        assert_eq!(worker(u32::MAX), None);
    }
}
