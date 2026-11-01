//! The held-change restart control's owned forge and fixture-only handover.
use crate::fixtures;
pub(crate) const ADMIN_TOKEN: &[u8] = b"restart-admin-token";
const API_TOKEN: &[u8] = b"0000000000000000000000000000000000000002";
// The base table's two tokens plus this control's admin, API and git tokens.
const TOKENS: u32 = 5;
const GIT_TOKEN: &[u8] = b"0000000000000000000000000000000000000003";
pub(crate) fn table() -> skein_fake_forge_protocol::Config {
    let mut table = temper_fake_git_hosted::fixtures::table(b"acme/app", None);
    let mut tokens = skein_lib::List::with_capacity(TOKENS);
    for token in table.tokens.into_boxed() {
        tokens.push(token).expect("fixture tokens");
    }
    for (token, user) in [(ADMIN_TOKEN, 2), (API_TOKEN, 1), (GIT_TOKEN, 1)] {
        tokens.push((skein_lib::Secret::new(token.into()), user)).expect("restart tokens");
    }
    table.tokens = tokens;
    table
}
pub(crate) fn setup_peer(peer: &mut skein_fake_peers::forge::Peer) {
    let setup = peer.setup_mut();
    let worker = setup.account(b"worker", false, b"temper:bot");
    assert_eq!(worker, fixtures::WORKER_USER);
    // Match stories::forge's existing domain grant at the setup protocol face.
    setup
        .grant(fixtures::REPOSITORY, b"worker", skein_forge::forgejo::request::Grant::Write)
        .expect("fixture collaborator");
    let admin = setup.account(b"stranger", true, b"Restart site admin");
    setup.token(
        admin,
        b"admin",
        &[b"write:admin", b"read:user", b"write:organization", b"write:repository"],
        skein_lib::Secret::new(ADMIN_TOKEN.into()),
    );
}
