//! Printed link secrets show only their length (jig/docs/design/protocol/README.md, 8 and 10).
use crate::{CEILINGS, LinkSignIn};
use alloc::fmt::format;
use skein_lib::{Reader, Writer};

#[test]
fn debug_of_a_link_sign_in_shows_the_secrets_length_never_its_bytes() {
    const CANARY: &[u8; 13] = b"canary-c42e8b";
    let mut writer = Writer::new(size_of::<u32>() + CANARY.len());
    writer.put(&u32::try_from(CANARY.len()).expect("canary length fits u32").to_be_bytes()).expect("length fits");
    writer.put(CANARY).expect("canary fits");
    let bytes = writer.finish();
    let record = LinkSignIn::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("bounded link secret");
    let debug = format(format_args!("{record:?}"));
    let pretty = format(format_args!("{record:#?}"));
    let canary_bytes = format(format_args!("{CANARY:?}"));
    assert!(!debug.contains("canary-c42e8b"));
    assert!(!debug.contains(&canary_bytes));
    assert!(!pretty.contains("canary-c42e8b"));
    assert!(!pretty.replace([' ', '\n'], "").contains(&canary_bytes.replace(' ', "")));
    assert_eq!(debug, "LinkSignIn { secret: Secret { len: 13 } }");

    let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut writer).expect("measured link secret");
    let encoded = writer.finish();
    assert_eq!(encoded, bytes);
    let decoded = LinkSignIn::decode(&CEILINGS, &mut Reader::new(&encoded)).expect("round trip link secret");
    assert_eq!(decoded.secret().expose(), CANARY);
}
