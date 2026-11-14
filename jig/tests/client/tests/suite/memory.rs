use jig_client_world::{Settings, stories};

/// A real client allocation crosses a deliberately small scenario ceiling.
/// The same callback guard used by the full fixture must catch it.
#[test]
fn a_client_that_exceeds_its_declared_heap_budget_is_caught() {
    let mut world = stories::configured(Settings { memory: true, heap_ceiling: Some(1024), ..Settings::default() });
    let finding = world.try_client(7, stories::inbox()).unwrap_err();
    assert!(finding.contains("bounded: client and view exceeded the heap envelope"), "{finding}");
}

/// A supplied client constructor also checks the scenario ceiling.
#[test]
fn a_supplied_client_constructor_keeps_the_declared_heap_budget_guard() {
    use stories::client_construction as fixture;
    let mut world = fixture::world(Settings { memory: true, heap_ceiling: Some(1024), ..Settings::default() });
    let finding = world.try_client_with(7, stories::inbox(), fixture::limits(), fixture::client).unwrap_err();
    assert!(finding.contains("bounded: client and view exceeded the heap envelope"), "{finding}");
}
