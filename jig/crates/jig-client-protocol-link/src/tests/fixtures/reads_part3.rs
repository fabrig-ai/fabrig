use super::*;
pub fn read_wire_person_contract() -> w::PersonContract {
    w::PersonContract::Choice(read_wire_person_contract_choice())
}

pub fn read_wire_person_contract_choice() -> w::PersonContractChoice {
    let lim = w::CEILINGS;
    w::PersonContractChoice::new(&lim, w::PersonContractChoiceParts { options: Box::from([]) }).unwrap()
}

pub fn read_wire_owner_person_task() -> w::OwnerPersonTask {
    jig_client_test_records::read_wire_owner_person_task()
}

pub fn read_wire_result() -> w::ResultDocument {
    jig_client_test_records::read_wire_result()
}
