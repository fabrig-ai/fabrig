//! The small store's client records and sign-in address (temper/docs/design/client/README.md, 7).
/// Seven bytes: `Shell::finish` counts only 16-byte request keys for its closing warning.
pub const SIGN_IN_KEY: &[u8] = b"sign-in";
#[derive(Debug, PartialEq, Eq)]
pub struct SignIn {
    pub source: u32,
    pub address: Box<[u8]>,
    pub refused: bool,
}
#[must_use]
pub fn encode_sign_in(sign_in: &SignIn) -> Box<[u8]> {
    let mut value = Vec::new();
    value.push(if sign_in.refused { b'1' } else { b'0' });
    value.extend_from_slice(sign_in.source.to_string().as_bytes());
    value.push(b' ');
    value.extend_from_slice(&sign_in.address);
    value.into_boxed_slice()
}
#[must_use]
pub fn decode_sign_in(value: &[u8]) -> Option<SignIn> {
    let (&flag, rest) = value.split_first()?;
    let refused = match flag {
        b'0' => false,
        b'1' => true,
        _ => return None,
    };
    let space = rest.iter().position(|byte| *byte == b' ')?;
    let (digits, address) = rest.split_at(space);
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let source = std::str::from_utf8(digits).ok()?.parse().ok()?;
    Some(SignIn { source, address: address.strip_prefix(b" ")?.into(), refused })
}

#[must_use]
pub fn encode(record: jig_client::Record) -> (Box<[u8]>, Box<[u8]>) {
    (record.key, record.bytes)
}

#[must_use]
pub fn decode(record: skein_ui_web::store::Record) -> Option<jig_client::Record> {
    if record.key.len() == 16 || record.key.as_ref() == jig_client::STREAM_KEY {
        return Some(jig_client::Record { key: record.key, bytes: record.value });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sign_in_round_trips_decimal_sources_and_address_bytes() {
        assert_eq!(SIGN_IN_KEY.len(), 7);
        for source in [0, u32::MAX] {
            for refused in [false, true] {
                let record = SignIn {
                    source,
                    address: b"https://issuer.example/visit?state=a b\xff".as_slice().into(),
                    refused,
                };
                let value = encode_sign_in(&record);
                assert_eq!(value[0], if refused { b'1' } else { b'0' });
                assert_eq!(decode_sign_in(&value), Some(record));
            }
        }
    }
    #[test]
    fn malformed_sign_in_values_are_dropped() {
        for value in [
            b"".as_slice(),
            b"043",
            b"04294967296 address",
            b"243 address",
            b"0 address",
            b"0-1 address",
            b"0x address",
        ] {
            assert_eq!(decode_sign_in(value), None, "{value:?}");
        }
    }
}
