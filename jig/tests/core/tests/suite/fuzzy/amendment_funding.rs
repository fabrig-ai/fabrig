//! Seeded agreement of the client's rule and direct amendment admission.
#[test]
fn amending_offers_and_direct_requests_agree_across_funding_and_authority() {
    for seed in 0..8 {
        crate::amendment_funding::sweep(seed);
    }
}
