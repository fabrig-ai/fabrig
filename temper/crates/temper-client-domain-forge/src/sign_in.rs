//! Service links (client/ctl/README.md, 7) and shared shell words for a full deployment (client/line.md, 7).
use alloc::boxed::Box;
use skein_lib::base64::Alphabet;
use skein_lib::{Decimal, Secret, Wall, Writer, base64, bytes};
pub use temper_client_protocol_wire::LINK_PAGE;

const HTTP: &[u8] = b"http://";
const SERVICE: &[u8] = b"#service=";
/// The one-time service credential has this binary width.
pub const CREDENTIAL_BYTES: u32 = 40;
const ENCODED_BYTES: usize = 54;

/// A service link split into its engine address and its private credential.
#[derive(Debug)]
pub struct ServiceLink {
    pub address: Box<[u8]>,
    pub credential: Secret,
}

/// Writes a service link from an HTTP address (no trailing slash or newline) and a 40-byte credential.
#[must_use]
pub fn service_link(address: &[u8], credential: &Secret) -> Secret {
    assert!(u32::try_from(credential.len()) == Ok(CREDENTIAL_BYTES), "a service credential holds 40 bytes");
    let parts = [address, LINK_PAGE, SERVICE];
    let mut length = ENCODED_BYTES;
    for part in parts {
        length = length.checked_add(part.len()).expect("service link length fits");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured service link");
    }
    base64::encode(Alphabet::Url, credential.expose(), &mut writer).expect("40 bytes encode to 54 digits");
    Secret::new(writer.finish())
}

/// Reads only the service link's form; the caller checks that the address is on loopback.
#[must_use]
pub fn read_service_link(link: &[u8]) -> Option<ServiceLink> {
    let link = link.strip_suffix(b"\n").unwrap_or(link);
    let rest = link.strip_prefix(HTTP)?;
    let slash = bytes::find(rest, b"/")?;
    let (authority, path) = rest.split_at_checked(slash)?;
    host_and_port(authority)?;
    let text = path.strip_prefix(LINK_PAGE)?.strip_prefix(SERVICE)?;
    if text.len() != ENCODED_BYTES {
        return None;
    }
    let credential = base64::decode(Alphabet::Url, text, CREDENTIAL_BYTES).ok()?;
    if u32::try_from(credential.len()) != Ok(CREDENTIAL_BYTES) {
        return None;
    }
    let address = bytes::copy_of(link.get(..HTTP.len().checked_add(slash)?)?);
    Some(ServiceLink { address, credential: Secret::new(credential) })
}

fn host_and_port(authority: &[u8]) -> Option<()> {
    let bracketed = authority.strip_prefix(b"[");
    let host_port = bracketed.unwrap_or(authority);
    let separator = if bracketed.is_some() { b"]:".as_slice() } else { b":" };
    let (host, port) = host_port.split_at_checked(bytes::find(host_port, separator)?)?;
    if host.is_empty() {
        return None;
    }
    for byte in host {
        if bracketed.is_some() {
            if !byte.is_ascii_hexdigit() && *byte != b':' && *byte != b'.' {
                return None;
            }
        } else if !byte.is_ascii_alphanumeric() && *byte != b'.' && *byte != b'-' {
            return None;
        }
    }
    let _: u16 = u16::try_from(bytes::decimal(port.strip_prefix(separator)?)?).ok()?;
    Some(())
}

const FULL: [&[u8]; 4] = [
    b": all ",
    b" sign-ins are in use; the earliest ends at ",
    b"; this link stays usable until ",
    b"; after that, temper serve prints a fresh one when it is tried\n",
];

/// Universal time to the second. Wall's u64 nanoseconds bound the year to 2554.
#[must_use]
pub fn utc_second(at: Wall) -> [u8; 20] {
    at.utc_second()
}

/// The complete diagnostic, including its prefix and final newline.
#[must_use]
pub fn full_line(prefix: &str, sign_ins: u32, earliest: Wall, link_until: Wall) -> Box<[u8]> {
    let count = Decimal::of(u64::from(sign_ins));
    let earliest = utc_second(earliest);
    let link_until = utc_second(link_until);
    let [lead, end, link, fresh] = FULL;
    let parts = [prefix.as_bytes(), lead, count.as_bytes(), end, &earliest, link, &link_until, fresh];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("diagnostic length fits");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured full diagnostic");
    }
    writer.finish()
}

/// Largest full diagnostic, including a u32 count and the two UTC moments.
#[must_use]
pub fn full_line_bound(prefix: &str) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(prefix.len()).checked_add(10)?.checked_add(40)?;
    for literal in FULL {
        bytes = bytes.checked_add(skein_lib::len_u64(literal.len()))?;
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDRESS: &[u8] = b"http://127.0.0.1:4100";
    const LINK_PREFIX: &[u8] = b"http://127.0.0.1:4100/sign-in#service=";
    // 40 bytes of 0xff exercise the URL alphabet and the unpadded tail.
    const CREDENTIAL: [u8; 40] = [255; 40];
    const ENCODED: &[u8] = b"_____________________________________________________w";

    #[test]
    fn a_service_link_round_trips_with_an_optional_newline() {
        let credential = Secret::new(Box::new(CREDENTIAL));
        let link = service_link(ADDRESS, &credential);
        assert_eq!(link.expose(), bytes::concat(&[LINK_PREFIX, ENCODED]).as_ref());
        for ending in [b"".as_slice(), b"\n"] {
            let text = bytes::concat(&[LINK_PREFIX, ENCODED, ending]);
            let read = read_service_link(&text).expect("service link");
            assert_eq!(read.address.as_ref(), ADDRESS);
            assert_eq!(read.credential, credential);
        }
    }

    #[test]
    fn the_service_link_reader_rejects_other_forms() {
        const PERSON: &[u8] = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let short = ENCODED.get(1..).expect("encoding has a first byte");
        let cases: &[(&[u8], &[u8], &[u8])] = &[
            (b"http://127.0.0.1:4100/sign-in?secret=", PERSON, b""),
            (b"http://127.0.0.1:4100/jig/v1/sign-in/link?secret=", PERSON, b""),
            (LINK_PREFIX, short, b""),
            (LINK_PREFIX, ENCODED, b"A"),
            (LINK_PREFIX, PERSON, b""),
            (LINK_PREFIX, ENCODED, b"=="),
            (LINK_PREFIX, short, b"+"),
            (LINK_PREFIX, short, b"/"),
            (LINK_PREFIX, short, b"="),
            (LINK_PREFIX, short, b"x"),
            (LINK_PREFIX, ENCODED, b"\n\n"),
            (b"http://127.0.0.1:4100/other#service=", ENCODED, b""),
            (b"http://127.0.0.1:4100/sign-in?x=y#service=", ENCODED, b""),
            (b"http://127.0.0.1:4100?x=y/sign-in#service=", ENCODED, b""),
            (b"https://127.0.0.1:4100/sign-in#service=", ENCODED, b""),
        ];
        for (index, &(prefix, text, ending)) in cases.iter().enumerate() {
            let link = bytes::concat(&[prefix, text, ending]);
            assert!(read_service_link(&link).is_none(), "accepted invalid form {index}");
        }
    }

    #[test]
    fn epoch_is_universal_midnight() {
        assert_eq!(&utc_second(Wall::EPOCH), b"1970-01-01T00:00:00Z");
    }
    #[test]
    fn a_leap_day_keeps_its_date_and_drops_fractional_seconds() {
        assert_eq!(&utc_second(Wall::from_nanos(951_827_696_999_999_999)), b"2000-02-29T12:34:56Z");
    }
    #[test]
    fn the_last_second_of_a_year_precedes_next_years_midnight() {
        assert_eq!(&utc_second(Wall::from_nanos(1_767_225_599_000_000_000)), b"2025-12-31T23:59:59Z");
        assert_eq!(&utc_second(Wall::from_nanos(1_767_225_600_000_000_000)), b"2026-01-01T00:00:00Z");
    }
}
