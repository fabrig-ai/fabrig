//! Seeded agreement of the client's rule and direct amendment admission.

use skein_world::{sweep::cases, this_test};
use std::convert::Infallible;

#[test]
fn amending_offers_and_direct_requests_agree_across_funding_and_authority() {
    cases(this_test!("amending_offers_and_direct_requests_agree_across_funding_and_authority"), 0..8, |seed| {
        crate::amendment_funding::case(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
