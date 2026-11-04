use crate::{fixture, service::Service};
#[test]
fn the_fixture_builds_a_valid_service() {
    for rows in [fixture::repositories(), fixture::widgets(1)] {
        let mut service = Service::new(fixture::config(rows), &fixture::limits(), 1).expect("valid fixture");
        fixture::install_three(&mut service);
        let store = service.installations_mut();
        let mut ids = skein_lib::List::with_capacity(3);
        for row in store.iter() { ids.push(row.id).expect("three ids"); }
        assert_eq!(ids.as_slice(), &[1, 2, 3]);
        assert!(store.get(3).expect("third installation").suspended);
    }
}
