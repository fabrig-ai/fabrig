use skein_io as io;
pub fn address() -> io::Addr {
    io::Addr::Ip((::core::net::Ipv4Addr::LOCALHOST, 7100).into())
}
