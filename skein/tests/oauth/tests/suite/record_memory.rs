//! A decoded schema record at every ceiling (codec.md, 6).

use skein_heap::Meter;
use skein_lib::{Reader, Secret, Writer};
use skein_oauth::v3::{CEILINGS, SavedToken, SavedTokenParts};

fn field(length: u32) -> Box<[u8]> {
    vec![b'a'; usize::try_from(length).expect("ceiling fits usize")].into_boxed_slice()
}

fn full_record() -> SavedToken {
    SavedToken::new(
        &CEILINGS,
        SavedTokenParts {
            key: u32::MAX,
            generation: u64::MAX,
            expires_at: u64::MAX,
            access_token: Secret::new(field(CEILINGS.saved_token_access_token)),
            refresh_token: Some(Secret::new(field(CEILINGS.saved_token_refresh_token_some))),
            metadata: Some(field(CEILINGS.saved_token_metadata_some)),
            issued_client_id: Some(field(CEILINGS.saved_token_issued_client_id_some)),
            granted_scopes: Some(field(CEILINGS.saved_token_granted_scopes_some)),
            subject: Some(field(CEILINGS.saved_token_subject_some)),
            id_token: Some(Secret::new(field(CEILINGS.saved_token_id_token_some))),
            earliest_refresh_at: Some(u64::MAX),
        },
    )
    .expect("record at ceilings")
}

#[test]
fn decoding_at_every_ceiling_stays_within_the_heap_bound() {
    let record = full_record();
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("wire length fits"));
    record.encode(&mut writer).expect("measured room");
    let encoded = writer.finish();
    drop(record);
    let bound = SavedToken::decoded_bound(&CEILINGS).expect("schema heap bound");
    let meter = Meter::new();
    meter.start();
    let decoded = SavedToken::decode(&CEILINGS, &mut Reader::new(&encoded)).expect("full record");
    let measured = meter.end();
    assert!(measured.peak() <= bound, "decoded fields stay within the heap bound");
    assert!(measured.held() <= bound, "retained fields stay within the heap bound");
    assert_eq!(
        decoded.access_token().expose().len(),
        usize::try_from(CEILINGS.saved_token_access_token).expect("ceiling")
    );
    assert_eq!(
        decoded.refresh_token().as_ref().expect("present").expose().len(),
        usize::try_from(CEILINGS.saved_token_refresh_token_some).expect("ceiling")
    );
    assert_eq!(
        decoded.metadata().as_ref().expect("present").len(),
        usize::try_from(CEILINGS.saved_token_metadata_some).expect("ceiling")
    );
    assert_eq!(
        decoded.issued_client_id().as_ref().expect("present").len(),
        usize::try_from(CEILINGS.saved_token_issued_client_id_some).expect("ceiling")
    );
    assert_eq!(
        decoded.granted_scopes().as_ref().expect("present").len(),
        usize::try_from(CEILINGS.saved_token_granted_scopes_some).expect("ceiling")
    );
    assert_eq!(
        decoded.subject().as_ref().expect("present").len(),
        usize::try_from(CEILINGS.saved_token_subject_some).expect("ceiling")
    );
    assert_eq!(
        decoded.id_token().as_ref().expect("present").expose().len(),
        usize::try_from(CEILINGS.saved_token_id_token_some).expect("ceiling")
    );
    drop(decoded);
    assert_eq!(meter.held(), 0, "decoded fields released");
}
