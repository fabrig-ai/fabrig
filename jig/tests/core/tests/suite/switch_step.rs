use crate::run_helpers as f;
use crate::switch_support::{RESTORES, mark, wires};
use jig_core_world::world::{Setup, World};
use jig_protocol_store::component_v3::{self, Record as _};
use skein_lib::Reader;

#[test]
fn a_first_start_takes_the_switch_step() {
    let mut world = World::new(Setup {
        configuration: Some(jig_test_profile::config(f::SEED)),
        limits: jig_test_profile::limits(),
        boot: jig_core_world::world::Boot::Empty,
        rows: Some(Box::new([])),
        ..Setup::default()
    });
    let restores = &world.restart_steps[..RESTORES.len()];
    assert_eq!(restores, RESTORES);
    assert_eq!(world.store.switch_requests, 0);
    assert_eq!(mark(&world, 0).len(), 2);
    assert!(world.domain.ready());
    let bytes = wires(&mut world);
    let row = component_v3::Wires::decode(&component_v3::CEILINGS, &mut Reader::new(&bytes)).expect("empty wires row");
    assert!(row.wires.is_empty(), "an empty shell wire list is still committed");
}
