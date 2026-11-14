use skein_oauth_accounts_world::world::Contracts;

#[test]
fn a_candidate_lent_before_kept_fails_the_referee() {
    let mut referee = Contracts::with_pending_grant();
    referee.keep(1).expect("valid preceding event");
    let finding = referee.granted(1).expect_err("the contract is broken");
    assert!(finding.contains("nothing lent before its keep terminal"), "{finding}");
}
#[test]
fn a_duplicate_keep_fails_the_referee() {
    let mut referee = Contracts::with_pending_grant();
    referee.keep(1).expect("valid preceding event");
    let finding = referee.keep(1).expect_err("the contract is broken");
    assert!(finding.contains("each candidate keeps in flight has a key of its own"), "{finding}");
}
#[test]
fn a_duplicate_grant_terminal_fails_the_referee() {
    let mut referee = Contracts::with_pending_grant();
    referee.granted(0).expect("valid preceding event");
    let finding = referee.granted(0).expect_err("the contract is broken");
    assert!(finding.contains("held notices name a new generation"), "{finding}");
}
#[test]
fn a_duplicate_closed_fails_the_referee() {
    let mut referee = Contracts::with_pending_grant();
    referee.granted(0).expect("valid preceding event");
    referee.closing().expect("valid preceding event");
    referee.closed().expect("valid preceding event");
    let finding = referee.closed().expect_err("the contract is broken");
    assert!(finding.contains("one Closed"), "{finding}");
}
#[test]
fn an_event_after_closed_fails_the_referee() {
    let mut referee = Contracts::with_pending_grant();
    referee.granted(0).expect("valid preceding event");
    referee.closing().expect("valid preceding event");
    referee.closed().expect("valid preceding event");
    let finding = referee.keep(1).expect_err("the contract is broken");
    assert!(finding.contains("Closed is the last owner event"), "{finding}");
}
#[test]
fn sign_in_success_before_keep_fails_the_referee() {
    let mut referee = skein_oauth_accounts_world::sign_in::Contracts::new();
    let finding = referee.signed_in(1).expect_err("the contract is broken");
    assert!(finding.contains("signed in before kept"), "{finding}");
}
#[test]
fn a_duplicate_sign_in_terminal_fails_the_referee() {
    let mut referee = skein_oauth_accounts_world::sign_in::Contracts::new();
    referee.failed().expect("valid preceding event");
    let finding = referee.failed().expect_err("the contract is broken");
    assert!(finding.contains("a sign-in and keeper terminals ends once"), "{finding}");
}
#[test]
fn closed_with_a_pending_sign_in_fails_the_referee() {
    let mut referee = skein_oauth_accounts_world::sign_in::Contracts::new();
    let finding = referee.closed().expect_err("the contract is broken");
    assert!(finding.contains("every sign-in and keeper terminals has ended"), "{finding}");
}
#[test]
fn an_unlock_without_a_held_answer_fails_the_referee() {
    let finding = Contracts::with_pending_grant().unlocked(0).expect_err("the contract is broken");
    assert!(finding.contains("owner locks"), "{finding}");
}
#[test]
fn closed_with_a_held_lock_fails_the_referee() {
    let mut referee = Contracts::with_pending_grant();
    referee.granted(0).expect("valid preceding event");
    referee.locked(0).expect("valid preceding event");
    referee.closing().expect("valid preceding event");
    let finding = referee.closed().expect_err("the contract is broken");
    assert!(finding.contains("owner locks"), "{finding}");
}
#[test]
fn an_unlock_after_closed_fails_the_referee() {
    let mut referee = Contracts::with_pending_grant();
    referee.granted(0).expect("valid preceding event");
    referee.closing().expect("valid preceding event");
    referee.closed().expect("valid preceding event");
    let finding = referee.unlocked(0).expect_err("the contract is broken");
    assert!(finding.contains("Closed is the last owner event"), "{finding}");
}
