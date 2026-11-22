//! The engine's place (temper/docs/design/client/tui/directory.md, 1).
use core::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

#[test]
fn a_box_names_only_its_non_default_port() {
    for (socket, boxed, expected) in [
        (SocketAddr::from((Ipv4Addr::LOCALHOST, 8367)), true, b"box".as_slice()),
        (SocketAddr::from((Ipv4Addr::LOCALHOST, 40691)), true, b"box:40691"),
        (SocketAddr::from((Ipv4Addr::LOCALHOST, 40691)), false, b"127.0.0.1:40691"),
        (SocketAddr::from((Ipv6Addr::LOCALHOST, 8367)), true, b"box"),
        (SocketAddr::from((Ipv6Addr::LOCALHOST, 40691)), true, b"box:40691"),
        (SocketAddr::from((Ipv6Addr::LOCALHOST, 8367)), false, b"[::1]:8367"),
    ] {
        assert_eq!(crate::place(&crate::Origin::Http(socket), boxed).as_ref(), expected, "{socket:?}");
    }
}
