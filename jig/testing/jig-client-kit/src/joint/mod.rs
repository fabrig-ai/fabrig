//! The core and client composed through their shipping protocol components
//! in memory (client/domain/testing.md, 5; protocol/client/README.md, 2).
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
    business_callbacks(callbacks)
}

pub mod role_fixture;

/// Application callbacks; the fixture consumes the stream's implicit me and negotiation.
#[cfg(test)]
fn business_callbacks(
    callbacks: Vec<(crate::harness::ClientId, jig_client_test_domain::Event)>,
) -> Vec<(crate::harness::ClientId, jig_client_test_domain::Event)> {
    callbacks
        .into_iter()
        .filter(|(_, event)| {
            !matches!(
                event,
                jig_client_test_domain::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Served { versions: Some(_), .. }
                            | jig_client::Up::FamilyServed { .. }
                            | jig_client::Up::Watched {
                                event: jig_client::contract::WatchEvent::Alive
                                    | jig_client::contract::WatchEvent::Opened,
                                ..
                            }
                    }
                }
            ) && !matches!(event, jig_client_test_domain::Event::Client { event: jig_client::Event::Up { up:
        jig_client::Up::Watched { watch, .. } | jig_client::Up::WatchEnded { watch, .. }
    } } if *watch == skein_lib::Token::new(0))
        })
        .collect()
}
