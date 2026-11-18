//! Per-chat layout marker (domain/host.md, section 8; protocol/host.md, 5.3).
//! No state or schema parser is retained. `layouts`, `file` and `check`
//! compute fingerprints from embedded texts, write their marker and check it.
use crate::{File, local_store_v1 as wire};
use skein_lib::{Reader, Writer};
use smith_local_domain::{Older, StoreFamily};

#[expect(clippy::disallowed_macros, reason = "the marker embeds the local store's schema text")]
const TEXT: &str = include_str!("../schema/local-store-v1.schema");

/// The generated family's complete fixed-size Layouts encoding bound.
#[must_use]
pub fn max_bytes() -> u32 {
    u32::try_from(wire::Layouts::encoded_bound(&wire::CEILINGS).expect("fixed layouts"))
        .expect("layout encoding fits u32")
}

/// The current store and transcript fingerprints, computed from their embedded texts.
#[must_use]
pub fn layouts() -> wire::Layouts {
    let store = {
        let schema = skein_codec::schema::parse(TEXT.as_bytes()).expect("the embedded local store schema is valid");
        schema.fingerprints().family().digest()
    };
    let transcript = smith_transcript::wire::layout();
    wire::Layouts::new(&wire::CEILINGS, wire::LayoutsParts { store, transcript }).expect("fixed layouts fit")
}

/// The no-replace marker written only in an admitted empty directory.
#[must_use]
pub fn file() -> File {
    let value = layouts();
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured bytes fit usize"));
    value.encode(&mut writer).expect("measured layouts");
    File { name: b"format".as_slice().into(), bytes: writer.finish() }
}

/// Check store first, then transcript; malformed records are store-layout refusals.
pub fn check(bytes: &[u8]) -> Result<(), Older> {
    let value = wire::Layouts::decode(&wire::CEILINGS, &mut Reader::new(bytes))
        .or(Err(Older::Layout(StoreFamily::Store)))?
        .into_parts();
    let current = layouts().into_parts();
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
    #[expect(clippy::disallowed_macros, reason = "the test independently embeds the marker's schema inputs")]
    fn the_marker_fingerprints_its_embedded_texts_and_checks_its_file() {
        let store = include_str!("../schema/local-store-v1.schema");
        let transcript = include_str!("../../smith-transcript/schema/transcript-v7.schema");
        let parts = layouts().into_parts();
        for (text, found) in [(store, parts.store), (transcript, parts.transcript)] {
            let schema = skein_codec::schema::parse(text.as_bytes()).expect("marker schema");
            assert_eq!(found, schema.fingerprints().family().digest());
        }
        assert_eq!(check(&file().bytes), Ok(()));
    }
}
