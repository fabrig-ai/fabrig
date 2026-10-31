//! The core and client composed through their shipping protocol components
//! in memory (client/domain/testing.md, 5; protocol/client.md, 2).
mod component;
pub mod engine;
mod fixture;
pub mod front;
pub mod random;
pub use front::Joint;
pub mod referee;
pub mod session;
pub mod transport;
pub mod truth;

#[cfg(test)]
mod account_moves;
#[cfg(test)]
mod stories;

#[cfg(test)]
fn take_signed_in(
    client: crate::harness::ClientId,
    mut callbacks: Vec<(crate::harness::ClientId, jig_client_test_domain::Event)>,
) -> Vec<(crate::harness::ClientId, jig_client_test_domain::Event)> {
    let signed_in = |value: &(crate::harness::ClientId, jig_client_test_domain::Event)| {
        matches!(value, (recipient, jig_client_test_domain::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::SignedIn }
        }) if *recipient == client)
    };
    let at = callbacks.iter().position(signed_in).expect("the real sign-in tells the public client");
    callbacks.remove(at);
    assert!(!callbacks.iter().any(signed_in), "one authentication notification");
    callbacks
}
