//! Private app.ini from setup and the secrets recorded once (protocol/setup.md, 6.2).
use crate::{check::joined, place::Place, setup::Owned, state::Secrets};
use alloc::boxed::Box;
use core::net::{IpAddr, SocketAddr};
use skein_lib::{Decimal, Secret, Writer};
/// Canonical, uncompressed address text, bracketed only in a socket.
pub(crate) fn ip(address: IpAddr) -> Box<[u8]> {
    match address {
        IpAddr::V4(a) => {
            let octets = a.octets();
            let mut parts = skein_lib::List::with_capacity(7);
            for (i, octet) in octets.iter().enumerate() {
                if i > 0 {
                    parts.push(b".".as_slice().into()).expect("seven pieces");
                }
                parts.push(Decimal::of(u64::from(*octet)).as_bytes().into()).expect("seven pieces");
            }
            flatten(parts.as_slice())
        }
        IpAddr::V6(a) => {
            let mut w = Writer::new(39);
            for (i, segment) in a.segments().iter().enumerate() {
                if i > 0 {
                    w.put(b":").expect("IPv6 maximum");
                }
                for shift in [12, 8, 4, 0] {
                    let digit = u8::try_from((segment >> shift) & 15).expect("hex digit");
                    let c = if digit < 10 {
                        b'0'.checked_add(digit)
                    } else {
                        b'a'.checked_add(digit.checked_sub(10).expect("hex letter"))
                    }
                    .expect("ASCII digit");
                    w.put(&[c]).expect("IPv6 maximum");
                }
            }
            w.finish()
        }
    }
}
pub(crate) fn flatten(parts: &[Box<[u8]>]) -> Box<[u8]> {
    let mut length = 0_usize;
    for p in parts {
        length = length.checked_add(p.len()).expect("bounded output");
    }
    let mut w = Writer::new(length);
    for p in parts {
        w.put(p).expect("measured output");
    }
    w.finish()
}
pub(crate) fn socket(address: SocketAddr) -> Box<[u8]> {
    let ip = ip(address.ip());
    let port = Decimal::of(u64::from(address.port()));
    if address.is_ipv6() { joined(&[b"[", &ip, b"]:", port.as_bytes()]) } else { joined(&[&ip, b":", port.as_bytes()]) }
}
/// Forgejo settings. Only a configured listener adds an allowed webhook host.
#[must_use]
pub fn text(owned: &Owned, place: &Place, secrets: &Secrets, allowed_host: Option<IpAddr>, sign_in: bool) -> Secret {
    let work = place.work_path();
    let address = ip(owned.listen.ip());
    let socket = socket(owned.listen);
    let port = Decimal::of(u64::from(owned.listen.port()));
    let actions = if owned.runner.is_some() { b"true".as_slice() } else { b"false" };
    let webhook = match allowed_host {
        Some(a) => joined(&[b"[webhook]\nALLOWED_HOST_LIST = ", &ip(a), b"\n"]),
        None => Box::new([]),
    };
    let public = match &owned.public {
        Some(origin) => {
            let port = Decimal::of(u64::from(origin.port));
            let suffix = if origin.port == 443 { Box::new([]) } else { joined(&[b":", port.as_bytes()]) };
            joined(&[b"\nDOMAIN = ", &origin.host, b"\nROOT_URL = https://", &origin.host, &suffix, b"/"])
        }
        None => joined(&[b"\nROOT_URL = http://", &socket, b"/"]),
    };
    let registration = if sign_in {
        b"ALLOW_ONLY_EXTERNAL_REGISTRATION = true\n[oauth2_client]\nENABLE_AUTO_REGISTRATION = true\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n".as_slice()
    } else {
        b""
    };
    Secret::new(joined(&[
        b"RUN_USER = ",
        &place.user,
        b"\nWORK_PATH = ",
        &work,
        b"\n[server]\nPROTOCOL = http\nHTTP_ADDR = ",
        &address,
        b"\nHTTP_PORT = ",
        port.as_bytes(),
        &public,
        b"\nAPP_DATA_PATH = ",
        &work,
        b"/data\nDISABLE_SSH = true\nOFFLINE_MODE = true\n[database]\nDB_TYPE = sqlite3\nPATH = ",
        &work,
        b"/forgejo.db\n[repository]\nROOT = ",
        &work,
        b"/repositories\nDEFAULT_BRANCH = main\n[security]\nINSTALL_LOCK = true\nSECRET_KEY = ",
        secrets.secret_key.expose(),
        b"\nINTERNAL_TOKEN = ",
        secrets.internal_token.expose(),
        b"\n[oauth2]\nJWT_SECRET = ",
        secrets.jwt_secret.expose(),
        b"\n[service]\nDISABLE_REGISTRATION = true\n",
        registration,
        b"[mailer]\nENABLED = false\n",
        &webhook,
        b"[actions]\nENABLED = ",
        actions,
        b"\n[log]\nMODE = file\nLEVEL = Error\nROOT_PATH = ",
        &work,
        b"/log\n",
    ]))
}
