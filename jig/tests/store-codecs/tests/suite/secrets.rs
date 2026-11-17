//! Secrets family ceilings stay those of their producers (protocol/store/secrets.md).
use jig_protocol_store::secrets_v1::CEILINGS;
#[test]
fn the_saved_record_bound_is_oauths_ceiling() {
    assert_eq!(
        u64::from(CEILINGS.source_record_record),
        skein_oauth::v3::SavedToken::encoded_bound(&skein_oauth::v3::CEILINGS).expect("OAuth bound")
    );
}
#[test]
fn carried_and_key_bounds_are_the_client_wires_ceiling() {
<<<<<<< HEAD
    let bound = jig_client_protocol_wire::v28::CEILINGS.keyed_value_some;
=======
    let bound = jig_client_protocol_wire::CEILINGS.keyed_value_some;
>>>>>>> 1ec4d558d2251b97ed59056727b4b65e5fbbf56b
    for ceiling in
        [CEILINGS.source_key_key, CEILINGS.carried_key_key, CEILINGS.carried_value_value, CEILINGS.carried_moved_moved]
    {
        assert_eq!(ceiling, bound);
    }
}
