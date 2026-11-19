use super::*;
use skein_io::kernel::{Done, Error, Op};
use skein_lib::Token;
// Two fixture binds exercise kernel-assigned hooks and fixed service ports.
const HOOKS: SocketAddr = SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 41000));
fn bind(asked: u16, result: Result<Done, Error>) -> Complete {
    Complete { op: Token::new(1), kind: Op::Bind { fd: Fd::new(1), addr: ([127, 0, 0, 1], asked).into() }, result }
}
fn listen() -> Complete {
    Complete { op: Token::new(2), kind: Op::Listen { fd: Fd::new(1), backlog: 8 }, result: Ok(Done::Nothing) }
}
#[test]
fn a_port_zero_listener_names_the_hooks_address_once_it_listens() {
    let mut bound = None;
    let complete = bind(0, Ok(Done::Bound(HOOKS)));
    assert_eq!(hooks_address(&mut bound, &complete), None);
    assert_eq!(hooks_address(&mut bound, &listen()), Some(HOOKS));
}
#[test]
fn a_fixed_port_bind_is_not_a_hooks_address() {
    let mut bound = None;
    let fixed = ([127, 0, 0, 1], 7102).into();
    let complete = bind(7102, Ok(Done::Bound(fixed)));
    assert_eq!(hooks_address(&mut bound, &complete), None);
    assert_eq!(hooks_address(&mut bound, &listen()), None);
}
#[test]
fn a_failed_bind_is_not_a_hooks_address() {
    let mut bound = None;
    let complete = bind(0, Err(Error::AddressInUse));
    assert_eq!(hooks_address(&mut bound, &complete), None);
}
