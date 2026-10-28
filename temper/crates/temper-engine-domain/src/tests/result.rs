use crate::link::settled_resource;
use skein_forge::api::Commit;
use temper_engine_domain_forge::{Name, What};

fn settled_head(repository: u32) -> (Name, Commit) {
    (Name { forge: 1, repository, what: What::Branch(Box::new([b"temper/change".as_slice().into()])) }, [7; 32])
}

#[test]
fn no_settled_head_leaves_the_change_resource_missing() {
    assert_eq!(settled_resource(&[]), None);
}

#[test]
fn one_settled_head_names_the_change_resource() {
    assert_eq!(settled_resource(&[settled_head(42)]), Some(42));
}

#[test]
fn more_than_one_settled_head_is_refused_as_ambiguous() {
    assert_eq!(settled_resource(&[settled_head(42), settled_head(73)]), None);
}
