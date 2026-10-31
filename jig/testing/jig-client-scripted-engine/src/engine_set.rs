//! Normal document deliveries exclude the separate provisional text stream (protocol/client.md, 7).
use jig_client_domain::contract::Delivery;
pub(crate) fn delivery(value: &Delivery) -> bool {
    match value {
        Delivery::Run(_) => false,
        Delivery::Task(_)
        | Delivery::OwnerTree(_)
        | Delivery::Chips(_)
        | Delivery::OwnerInbox(_)
        | Delivery::Tasks(_)
        | Delivery::OwnerProject(_)
        | Delivery::OwnerProposal(_)
        | Delivery::OwnerDeployment(_)
        | Delivery::Me(_)
        | Delivery::RunState(_)
        | Delivery::Settings(_)
        | Delivery::OwnerAccounts(_) => true,
    }
}
