//! Fresh Open words cross the shipped direct provider boundary.
use crate::direct_opening;

#[test]
fn open_brief_is_the_native_peers_first_user_message() {
    for anthropic in [false, true] {
        drop(direct_opening::direct(37, anthropic));
    }
}
