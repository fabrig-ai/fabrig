//! A chat's change is checked, reviewed and landed by its connected GitHub App.
use temper_real_loop_world::{
    Ran, argv,
    calls::{D1, call, number},
    github::{
        self, check_chat_heard_the_landing, check_connection, check_landing, connect_adopt_and_ask_for_a_change,
        scripts,
    },
    landing, llm, run_github, terminal,
};

const ONE_PUSH: usize = 1;
// The connected story's first chat delegates its first change.
const CHANGE_TASK: u64 = 2;
// Plain change reads in the connected landing recipe and its continuation.
const FIRST_LANDED_READ: usize = 11;
const REPEATED_LANDED_READ: usize = 13;

#[test]
fn a_chats_change_lands_on_connected_github() {
    let mut fixture = github::fixture();
    fixture.connected = false;
    fixture.configure = |value, agent, at| (landing::fixture().configure)(value, agent, at);
    let ran = run_github(
        fixture,
        |at| llm::cued(scripts(at)),
        || {
            let mut steps = connect_adopt_and_ask_for_a_change();
            steps.extend([
                call(D1, argv!["task", "show", number(CHANGE_TASK), "--json"]),
                terminal::open(),
                terminal::see(b"Work"),
                terminal::type_on(b"Work", b""),
                terminal::see("Chats · 1".as_bytes()),
                terminal::type_on("Chats · 1".as_bytes(), b""),
                terminal::see(b"T1"),
                terminal::type_on(b"T1", b""),
                terminal::see(b"The change landed."),
                terminal::close(),
            ]);
            steps
        },
    );
    check_connection(&ran);
    let commit = check_landing(&ran);
    check_chat_heard_the_landing(&ran, &commit);
    let before: serde_json::Value = serde_json::from_slice(&ran.calls()[FIRST_LANDED_READ].stdout).expect("ended task");
    let after: serde_json::Value =
        serde_json::from_slice(&ran.calls()[REPEATED_LANDED_READ].stdout).expect("repeated task read");
    assert_eq!(before["result"], after["result"], "the landing result survives repeated reads");
    assert!(
        ran.terminal()
            .pages
            .iter()
            .any(|page| { skein_lib::bytes::find(&page.bytes, b"The change landed.").is_some() })
    );
    check_authenticated_git(&ran);
    let pem = ran.github().app.as_ref().expect("converted App").pem.expose();
    let _files = github::check_secrets(&ran, pem, Some(&ran.path().join("github-app.pem")));
}

fn check_authenticated_git(ran: &Ran) {
    let seen = ran.github();
    assert_eq!(seen.installations, [(github::INSTALLATION, 10, b"acme".to_vec())]);
    assert!(seen.minted >= 1);
    assert_eq!(seen.git_calls.iter().filter(|call| call.writes).count(), ONE_PUSH);
    assert!(seen.git_calls.iter().any(|call| !call.writes && call.status == 200));
    for call in &seen.git_calls {
        let minted = call.minted.expect("installation token on every git call");
        assert_eq!(minted.installation, github::INSTALLATION);
        assert!(minted.mint > 0 && minted.mint <= seen.minted);
        assert!(call.status == 200 || (!call.writes && call.status == 404), "{call:?}");
    }
}
