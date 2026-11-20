//! Per-chat layout marker (domain/host.md, section 8; protocol/host.md, 5.3).
//! `layouts` computes fingerprints from embedded texts for the store to retain.
//! `file` and `check` reuse that value; no schema parser is retained.
use crate::local_store_v1::Record;
use crate::{File, local_store_v1 as wire};
use skein_lib::Reader;
use smith_local_domain::{Older, StoreFamily};

const TEXT: &str = crate::SCHEMA;

/// The generated family's complete fixed-size Layouts encoding bound.
#[must_use]
pub fn max_bytes() -> u32 {
    u32::try_from(wire::Layouts::encoded_bound(&wire::CEILINGS).expect("fixed layouts"))
        .expect("layout encoding fits u32")
}

/// Compute the store and transcript fingerprints once for their owner's lifetime.
#[must_use]
pub fn layouts() -> wire::Layouts {
    let store = {
        let schema = skein_codec::schema::parse(TEXT.as_bytes()).expect("the embedded local store schema is valid");
        schema.fingerprints().family().digest()
    };
    let transcript = smith_transcript::wire::layout();
    wire::Layouts { store, transcript }
}

/// The no-replace marker written only in an admitted empty directory.
#[must_use]
pub fn file(value: &wire::Layouts) -> File {
    let bytes = value.encode(&wire::CEILINGS).expect("layouts have no variable bounds");
    File { name: b"format".as_slice().into(), bytes }
}

/// Check store first, then transcript; malformed records are store-layout refusals.
pub fn check(bytes: &[u8], current: &wire::Layouts) -> Result<(), Older> {
    let value =
        wire::Layouts::decode(&wire::CEILINGS, &mut Reader::new(bytes)).or(Err(Older::Layout(StoreFamily::Store)))?;
    if value.store != current.store {
        return Err(Older::Layout(StoreFamily::Store));
    }
    if value.transcript != current.transcript {
        return Err(Older::Layout(StoreFamily::Transcript));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(clippy::disallowed_macros, reason = "the fixture records the deployed build's marker bytes")]
    fn the_deployed_marker_is_accepted_and_written_unchanged() {
        let fixture = include_bytes!("../tests/fixtures/format").as_slice();
        let current = layouts();
        assert_eq!(check(fixture, &current), Ok(()));
        assert_eq!(file(&current).bytes.as_ref(), fixture);
    }

    #[test]
    fn marker_operations_reuse_the_supplied_layouts() {
        let current = wire::Layouts { store: [1; 32], transcript: [2; 32] };
        let marker = file(&current);
        let written = wire::Layouts::decode(&wire::CEILINGS, &mut Reader::new(&marker.bytes)).expect("written layouts");
        assert_eq!(written.store, [1; 32]);
        assert_eq!(written.transcript, [2; 32]);
        assert_eq!(check(&marker.bytes, &current), Ok(()));
    }
}
