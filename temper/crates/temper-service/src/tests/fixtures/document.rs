#![expect(clippy::disallowed_methods, reason = "document member tests edit JSON fixture bytes")]
//! Document-member inputs (temper/docs/design/protocol/configuration.md, 3).
pub const APPLICATION: &[u8] = br#"{
"webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:7100"},"public":null,"bundle":null},
"forges":[{"number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:9"},"transport":"plaintext","git":"http://127.0.0.1:9","webhook_secret":null,"api_token":"secrets/forge-1.api-token","user":{"file":"secrets/forge-1.user"},"identity":1}],
"identities":[{"number":1,"token":"secrets/forge-1.git-token","login":"temper","author":{"name":"Temper","email":"temper@example.test"}}],
"landing":{"deployment":[],"projects":[{"project":1,"rules":[{"connector":1,"kind":4,"pattern":{"segments":["forge","127.0.0.1:9","acme","app"],"last":"open","value":"main"},"enforce":true,"ci":false,"up_to_date":true,"gates":[],"approvals":[]}]}]},
"setup":[],"demo":null,"sign_in":null,"charter":1,"limits":{"profile":"small","overrides":{}},"workers":[{"root":"worker-root","git":"/usr/bin/git"}]
}"#;

pub fn document() -> Box<[u8]> {
    let input = super::document();
    let limits = super::value(&input, b"application.limits");
    let input = super::with(&input, b"application", APPLICATION);
    super::with(&input, b"application.limits", &limits)
}
pub fn demo_application() -> Box<[u8]> {
    core::str::from_utf8(APPLICATION)
        .unwrap()
        .replace("\"setup\":[]", DEMO_SETUP)
        .replace("\"demo\":null", DEMO)
        .into_bytes()
        .into_boxed_slice()
}
pub const DEMO: &str = r#""demo":{"llm":{"listen":{"socket":"$llm"},"provider":3,"key":"demo/llm.key",
"asked":[{"cue":"Add CSV export to widget","turns":["CSV"]},
{"cue":"Add a dark theme to widget","turns":["Theme"]},
{"cue":"Fix widget's empty search state","turns":["Search"]}],
"cued":[{"cue":"Export widget's table as CSV","turns":["Code"]}],"otherwise":["Demo"]},
"forge":1,"project":1}"#;
pub const DEMO_SETUP: &str = r#""setup":[{"forge":1,"directory":"demo/forge",
"bot":{"login":"temper","email":"temper@example.test"},"mode":{"fake":{
"listen":{"socket":"$forge"},"git":"demo/bin/git","owner":"demo",
"repositories":[{"name":"widget","landing":"main"},{"name":"other","landing":"main"}]}}}]"#;
