//! Values encoded at the deployment floor for written-job tests.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary Rust schema fixtures"
)]
use crate::{jobs, records};
use jig_protocol_store::JobLayout;
use skein_codec::{
    layout::{Plan, Workspace},
    schema,
};
use temper_engine_domain_forge_vocabulary::Stored;

pub const FLOOR: &str = include_str!("store-floor-store-v25.schema");
pub const BINARY: &str = include_str!("../../../schema/store-v25.schema");

pub fn wide() -> String {
    jobs::test_ids::wide_text(BINARY).unwrap()
}
pub fn layout(text: &str, variant: &str) -> JobLayout {
    JobLayout { root: "Row".into(), variant: Some(variant.into()), text: text.as_bytes().into(), held: Box::new([]) }
}
pub fn at_floor(stored: &Stored, variant: &str) -> Box<[u8]> {
    let wide = schema::parse(wide().as_bytes()).unwrap();
    let floor = schema::parse(FLOOR.as_bytes()).unwrap();
    let plan = Plan::derive_dropping(
        &wide,
        wide.root(b"Row", Some(variant.as_bytes())).unwrap(),
        &[],
        &floor,
        floor.root(b"Row", Some(variant.as_bytes())).unwrap(),
        &[],
    )
    .expect("fixture plans back to floor");
    let mut work = Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1_048_576);
    plan.convert(&records::encode(stored), &mut out, &mut work).unwrap();
    out.as_slice().into()
}

/// Replace distinctive test-id content and its count, leaving the surrounding value alone.
pub fn replace_id(bytes: &[u8], marker: &[u8], id: &[u8]) -> Box<[u8]> {
    let before = [&u32::try_from(marker.len()).unwrap().to_be_bytes(), marker].concat();
    let after = [&u32::try_from(id.len()).unwrap().to_be_bytes(), id].concat();
    let mut out = Vec::new();
    let mut start = 0;
    let mut found = false;
    while let Some(offset) = bytes[start..].windows(before.len()).position(|part| part == before) {
        let at = start.checked_add(offset).unwrap();
        out.extend_from_slice(&bytes[start..at]);
        out.extend_from_slice(&after);
        start = at.checked_add(before.len()).unwrap();
        found = true;
    }
    assert!(found, "id marker is encoded");
    out.extend_from_slice(&bytes[start..]);
    out.into()
}
