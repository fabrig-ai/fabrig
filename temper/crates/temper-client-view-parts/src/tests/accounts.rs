use super::fixtures;

#[test]
fn bindings_at_capacity_refuse_another_control() {
    let counts = fixtures::limits().client.domain.accounts;
    let mut bindings =
        crate::accounts::Bindings::new(counts, fixtures::limits().row_uses, fixtures::limits().client.domain.rows);
    for _ in 0..crate::accounts::binding_capacity(&counts, fixtures::limits().client.domain.rows) {
        assert!(bindings.insert(crate::accounts::Meaning::Key).is_some());
    }
    assert!(bindings.insert(crate::accounts::Meaning::Key).is_none());
}
