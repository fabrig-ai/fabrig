#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
mod checkpoints;
mod derived;
mod family;
mod figures;
pub(crate) mod fixtures;
mod limit_settings;
mod scaled;
mod store_route;
mod tools;

#[test]
fn undeclared_connector_names_the_missing_configuration_path() {
    use crate::startup::checks::membership;
    use jig_domain_contract::StartProblem;
    use jig_protocol_config::{Path, What};

    const CONNECTOR: u16 = 2;
    let expected = (Path::root().member(b"engine").member(b"connectors"), What::Missing);
    assert_eq!(membership(StartProblem::Undeclared { connector: CONNECTOR }), expected);
}
