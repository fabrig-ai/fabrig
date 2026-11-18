//! Endpoint capacity at temper's start (protocol/configuration/figures.md, 2).
use super::fixtures;
use skein_lib::{Decimal, List};

fn document(count: u32) -> Box<[u8]> {
    let mut rows = List::uncapped();
    rows.push(b'[').unwrap();
    for number in 1..=count {
        if number != 1 {
            rows.push(b',').unwrap();
        }
        let number = Decimal::of(u64::from(number));
        for part in [b"{\"name\":\"fake".as_slice(), if number.as_bytes() == b"1" { b"" } else { b"-" }, if number.as_bytes() == b"1" { b"" } else { number.as_bytes() }, b"\",\"number\":", number.as_bytes(), br#", "dialect":3, "provider":"fake-provider", "address":{"socket":"127.0.0.1:7101"}, "transport":"plaintext", "headers":[], "connect_ms":100, "handshake_ms":200}"#] {
            for byte in part { rows.push(*byte).unwrap(); }
        }
    }
    rows.push(b']').unwrap();
    fixtures::with(&fixtures::engine_part(&fixtures::document()), b"engine.llm.endpoints", rows.as_slice())
}

#[test]
fn startup_accepts_64_endpoints_and_names_65() {
    let given = crate::stock::Given::Own {
        jig: crate::stock::checked_with(Box::new([fixtures::small_model()]), 32).unwrap(),
        forge: Box::new([]),
        ways: Box::new([]),
    };
    let directories = jig_protocol_config::Directories::working();
    let accepted = crate::startup::named_with(&document(64), &directories, &given).unwrap();
    assert_eq!(accepted.llm.endpoints.len(), 64);
    let found = crate::startup::named_with(&document(65), &directories, &given).unwrap_err();
    assert_eq!(crate::startup::lines(&found).as_ref(), b"engine.llm.providers[0]: past its bound\n");
}
