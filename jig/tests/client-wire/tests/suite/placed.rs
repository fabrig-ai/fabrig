//! The existing hold tag carries the placed hold's deployment number.
<<<<<<< HEAD
use jig_client_protocol_wire::v28 as wire;
=======
use jig_client_protocol_wire as wire;
>>>>>>> 1ec4d558d2251b97ed59056727b4b65e5fbbf56b
use skein_lib::{Reader, Writer};
#[test]
fn hold_category_placed_round_trips_with_its_number_at_the_existing_tag() {
    for hold in [0, 1, u64::MAX] {
        let value = wire::HoldCategory::Placed(
            wire::HoldCategoryPlaced::new(&wire::CEILINGS, wire::HoldCategoryPlacedParts { hold }).expect("number"),
        );
        let mut writer = Writer::new(value.measure() as usize);
        value.encode(&mut writer).expect("measured hold");
        let bytes = writer.finish();
        assert_eq!(bytes[0], 4);
        assert_eq!(wire::HoldCategory::decode(&wire::CEILINGS, &mut Reader::new(&bytes)), Ok(value));
    }
}
