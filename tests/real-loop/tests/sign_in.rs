//! The person's browser returns to the accounts driver's real loopback listener.
use skein_world::domain::heap::Counting;
use temper_real_loop_world::{
    Exit, argv,
    calls::{D1, Step, call, sign_in},
    run_sign_in, terminal,
};
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn a_terminal_sign_in_returns_to_the_real_listener() {
    let ran = run_sign_in(|| {
        vec![
            sign_in(D1),
            call(D1, argv!["me", "--json"]),
            terminal::open(),
            terminal::type_on(b"Accounts", b""),
            terminal::see(b"Connect a model"),
            terminal::type_on(b"Connect a model", b""),
            terminal::see(b"Continue with ChatGPT"),
            terminal::type_on(b"Continue with ChatGPT", b""),
            terminal::see(b"Open the sign-in page <http://"),
            Step::Authorize,
            Step::Return,
            terminal::see(b"Signed in to ChatGPT"),
            call(D1, argv!["accounts", "--json"]),
            terminal::close(),
        ]
    });
    assert!(ran.calls().iter().all(|call| call.exit == Some(Exit::Code(0))), "ctl calls succeeded");
    assert_eq!(ran.terminal().exit, Some(Exit::Code(0)), "terminal drained");
    let [answer] = ran.returned() else { panic!("one native listener answer") };
    assert!(answer.error.is_none(), "listener answered over a real connection");
    assert_eq!(answer.status, Some(200), "listener success page");
    assert!(answer.body.as_ref() == b"Sign-in received. You may close this page.\n", "fixed success page");
    assert_eq!(ran.code_exchanges(), 1, "exactly one code exchange");
    let me: serde_json::Value = serde_json::from_slice(&ran.calls()[1].stdout).expect("me JSON");
    let person = me["party"]["number"].as_u64().expect("person number");
    let accounts: serde_json::Value = serde_json::from_slice(&ran.calls()[2].stdout).expect("accounts JSON");
    let rows = accounts["accounts"]["party"].as_array().expect("person's accounts");
    assert_eq!(rows.len(), 1, "one account for the person");
    let account = &rows[0];
    assert_eq!(account["provider"], 1);
    assert_eq!(account["plan"], true);
    assert_eq!(account["holder"]["party"], person);
    let sources = account["sources"].as_array().expect("account's sources");
    assert_eq!(sources.len(), 1, "one signed-in source");
    assert_eq!(sources[0]["kind"], "sign_in");
    assert_eq!(sources[0]["usable"], true);
    assert_eq!(sources[0]["state"]["kind"], "granted");
}
