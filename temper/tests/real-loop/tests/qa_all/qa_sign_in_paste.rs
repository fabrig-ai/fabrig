//! Sign-in restart and expiry races awaiting lower-tier coverage.

#[test]
#[ignore = "needs: real-loop fake issuer capacity for two outstanding authorization codes"]
fn restarting_after_approval_then_pasting_the_new_callback_succeeds() {}

#[test]
#[ignore = "needs: real-loop sign-in deadline control and a held code exchange"]
fn expiry_and_native_paste_race_take_at_most_one_redirect() {}
