//! Fixed-rule client values (skein/docs/design/data/codec/README.md, 6).
use skein_codegen::{Golden, goldens, parse};
use std::sync::OnceLock;

pub(super) fn client() -> &'static [Golden] {
    static VALUES: OnceLock<Vec<Golden>> = OnceLock::new();
    VALUES.get_or_init(|| {
        let schema = parse(&text()).expect("client schema");
        let mut values = goldens(&schema);
        values.sort_by(|left, right| left.name.cmp(&right.name));
        values
    })
}

pub(super) fn value(name: &str) -> &'static [u8] {
    &client().iter().find(|value| value.name == name).expect("named client value").bytes
}

fn text() -> String {
    include_str!("../crates/jig-client-protocol-wire/schema/client-v36.schema").to_owned()
        + include_str!("../crates/jig-client-protocol-wire/schema/client-v36-part1.schema")
}
