//! The existing hold tag carries the placed hold's deployment number.
use jig_client_protocol_wire as wire;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Writer};
#[test]
fn hold_category_placed_round_trips_with_its_number_at_the_existing_tag() {
    for hold in [0, 1, u64::MAX] {
        let value = wire::HoldCategory::Placed(wire::HoldCategoryPlaced { hold });
        let mut writer =
            Writer::new(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture") as usize);
        value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured hold");
        let bytes = writer.finish();
        assert_eq!(bytes[0], 4);
        assert_eq!(wire::HoldCategory::decode(&wire::CEILINGS, &mut Reader::new(&bytes)), Ok(value));
    }
}
