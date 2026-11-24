use crate::{
    CHECKPOINT, DOCUMENT, OBSERVATIONS, SEED, arguments,
    disk::{Tree, directories},
    document, observations as records, private_hex,
    wire::Running,
};
use serde_json::{Value, json};
use skein_lib::{Duration, Time};
use std::path::Path;
use temper_box_shell::fake::{Arguments, Kind, Source};
use temper_fake::credential::{ADMIN_TOKEN, credential};

const STATE: &str = "/fixture/forge.state";
const TOKEN: &str = "/fixture/forge/admin_token";
const CI_MS: u64 = 1500;

fn forge_document(landing: &str) -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"admin_token":TOKEN,"owner":"fake",
        "repositories":[{"name":"widget","landing":landing,"ci":null}]})
}
fn forge_state(tree: &mut Tree, value: &Value, observations: &str) -> Arguments {
    let mut args = arguments(Kind::Forge, document(tree, value));
    args.state = Some(STATE.into());
    args.observations = Some(observations.into());
    args
}
fn grant(run: &mut Running, owner: &str, auth: &str) {
    run.request(
        "PUT",
        &format!("/api/v1/repos/{owner}/widget/collaborators/temper-admin"),
        auth,
        br#"{"permission":"admin"}"#,
    )
    .status(204);
}
fn token_auth(token: &[u8]) -> String {
    format!("token {}", std::str::from_utf8(token).expect("hex token"))
}
fn base64(bytes: &[u8]) -> String {
    let alphabet = skein_lib::base64::Alphabet::Standard;
    let size = skein_lib::base64::encoded_len(alphabet, u32::try_from(bytes.len()).expect("bounded fixture"))
        .expect("base64 length");
    let mut encoded = skein_lib::Writer::new(usize::try_from(size).expect("bounded encoding"));
    skein_lib::base64::encode(alphabet, bytes, &mut encoded).expect("sized encoding");
    String::from_utf8(encoded.finish().into_vec()).expect("base64 ASCII")
}

#[test]
fn forge_state_keeps_an_api_write_only_after_an_orderly_stop() {
    let mut tree = Tree::new();
    let state = Path::new(STATE);
    let value = forge_document("main");
    let args = forge_state(&mut tree, &value, OBSERVATIONS);
    let mut run = Running::start(args, &mut tree);
    assert!(!tree.contains(state), "startup writes no state");
    let auth = token_auth(&credential(SEED, ADMIN_TOKEN));
    grant(&mut run, "fake", &auth);
    run.request(
        "POST",
        "/api/v1/repos/fake/widget/issues",
        &auth,
        br#"{"title":"Kept across stop","body":"Kept body"}"#,
    )
    .status(201);
    assert!(!tree.contains(state), "API writes do not checkpoint");
    run.stop(&mut tree);
    assert!(tree.contains(state), "orderly stop stores the image");

    let args = forge_state(&mut tree, &value, "/fixture/restarted-observations");
    let mut run = Running::start(args, &mut tree);
    let reply = run.request("GET", "/api/v1/repos/fake/widget/issues/1", &auth, b"");
    reply.status(200);
    assert_eq!(reply.json()["title"], "Kept across stop");
    run.stop(&mut tree);
}

#[test]
fn forge_state_keeps_made_workflows_and_manual_merge_settings() {
    let mut tree = Tree::new();
    let mut value = forge_document("main");
    value["workflows"] = json!([{"file":"gate.yml","jobs":["gate"],"cue":{"declared":".fake-forge/gate"}}]);
    value["pace"] = json!({"ci_ms":CI_MS});
    value["manual_merges"] = json!({"allow":true,"autodetect":true});
    let args = forge_state(&mut tree, &value, OBSERVATIONS);
    let mut run = Running::start(args, &mut tree);
    let token = credential(SEED, ADMIN_TOKEN);
    let auth = token_auth(&token);
    for (target, payload) in [
        ("/api/v1/admin/users/temper-admin/orgs", json!({"username":"session","visibility":"private"})),
        ("/api/v1/orgs/session/repos", json!({"name":"made","private":true,"auto_init":true,"default_branch":"main"})),
    ] {
        run.request("POST", target, &auth, &serde_json::to_vec(&payload).expect("setup")).status(201);
    }
    let credentials = format!("temper-admin:{}", std::str::from_utf8(&token).expect("hex token"));
    let git_auth = format!("Basic {}", base64(credentials.as_bytes()));
    made_gate(&mut run, &auth, "main", "success");
    let first = fetch(&mut run, &git_auth, br#"{"repository":"session/made"}"#);
    let create = json!({"repository":"session/made","branch":"work","commit":first});
    run.request("POST", "/.fake/git/create", &git_auth, &serde_json::to_vec(&create).expect("branch")).status(200);
    run.request("POST", "/api/v1/repos/session/made/contents/.fake-forge/gate", &auth,
        br#"{"branch":"work","content":"ZmFpbA==","message":"Fail the gate","author":{"name":"member","email":"member@example.invalid"},"committer":{"name":"member","email":"member@example.invalid"}}"#).status(201);
    let reply = run.request(
        "POST",
        "/api/v1/repos/session/made/pulls",
        &auth,
        br#"{"title":"A member","body":"","head":"work","base":"main"}"#,
    );
    reply.status(201);
    let number = reply.json()["number"].as_u64().expect("pull number");
    run.stop(&mut tree);
    assert!(tree.contains(STATE.as_ref()), "orderly stop keeps an image");

    // Changed defaults are for later creations, not the restored repository.
    value["workflows"][0]["cue"] = Value::Null;
    value["manual_merges"] = json!({});
    let args = forge_state(&mut tree, &value, "/fixture/restarted-observations");
    let mut run = Running::start(args, &mut tree);
    made_gate(&mut run, &auth, "work", "failure");
    let pull = format!("/api/v1/repos/session/made/pulls/{number}");
    assert_eq!(run.request("GET", &pull, &auth, b"").json()["merged"], false);
    let head = fetch(&mut run, &git_auth, br#"{"repository":"session/made","branch":"work"}"#);
    let push = json!({"repository":"session/made","branch":"main","commit":head,"expected":first});
    run.request("POST", "/.fake/git/push", &git_auth, &serde_json::to_vec(&push).expect("push")).status(200);
    assert_eq!(run.request("GET", &pull, &auth, b"").json()["merged"], true);
    run.stop(&mut tree);
}

fn fetch(run: &mut Running, auth: &str, body: &[u8]) -> u64 {
    let response = run.request("POST", "/.fake/git/fetch", auth, body);
    response.status(200);
    response.json()["commit"].as_u64().expect("commit")
}
fn made_gate(run: &mut Running, auth: &str, branch: &str, expected: &str) {
    let response = run.request("GET", &format!("/api/v1/repos/session/made/branches/{branch}"), auth, b"");
    response.status(200);
    let branch_head = response.json();
    let head = branch_head["commit"]["id"].as_str().expect("head");
    let dispatched = run.now();
    run.request(
        "POST",
        "/api/v1/repos/session/made/actions/workflows/gate.yml/dispatches",
        auth,
        &serde_json::to_vec(&json!({"ref":branch,"inputs":{}})).expect("dispatch"),
    )
    .status(204);
    let runs = format!("/api/v1/repos/session/made/actions/runs?workflow_id=gate.yml&head_sha={head}");
    let latest = |run: &mut Running| {
        let response = run.request("GET", &runs, auth, b"");
        response.status(200);
        let runs = response.json();
        let latest = &runs["workflow_runs"][0];
        assert_eq!(latest["commit_sha"], head, "run is for the dispatched tree");
        latest.clone()
    };
    let pending = latest(run);
    assert!(matches!(pending["status"].as_str(), Some("running" | "waiting")), "{pending}");
    let id = pending["id"].as_u64().expect("run id");
    let due = dispatched.saturating_add(Duration::from_millis(CI_MS));
    run.advance_to(Time::from_nanos(due.as_nanos() - 1));
    let almost_due = latest(run);
    assert!(matches!(almost_due["status"].as_str(), Some("running" | "waiting")), "{almost_due}");
    run.advance_to(due);
    let completed = latest(run);
    assert_eq!(completed["id"], id);
    assert_eq!(completed["status"], expected);
    let response = run.request("GET", &format!("/api/v1/repos/session/made/actions/runs/{id}/jobs"), auth, b"");
    response.status(200);
    let jobs = response.json();
    assert_eq!(jobs[0]["name"], "gate");
    assert_eq!(jobs[0]["status"], expected);
}

#[test]
fn forge_run() {
    for from_setup in [false, true] {
        let mut tree = Tree::new();
        let document = document(&mut tree, &forge_document("trunk"));
        let address = "127.0.0.1:8200".parse().expect("setup listen address");
        let (source, token_path, token, owner, landing) = if from_setup {
            tree.setup_at(address);
            let token = *b"0123456789abcdef0123456789abcdef01234567";
            let path = "/fixture/playground-1/admin_token";
            tree.seed(path.as_ref(), &token);
            (Source::Setup { directories: directories(), forge: 1 }, path, token, "playground", "main")
        } else {
            (document, TOKEN, credential(SEED, ADMIN_TOKEN), "fake", "trunk")
        };
        let mut run = Running::start(arguments(Kind::Forge, source), &mut tree);
        assert!(run.control.is_none());
        if from_setup {
            assert_eq!(run.address, address, "ready on fake.listen");
        }
        let token_path = Path::new(token_path);
        let observations = Path::new(OBSERVATIONS);
        private_hex(&tree, token_path, &token);
        assert_eq!(tree.mode(token_path.parent().expect("token directory")), 0o700);
        assert_eq!(tree.mode(observations), 0o600);
        assert_eq!(tree.mode(observations.parent().expect("observations directory")), 0o700);
        let auth = token_auth(&token);
        let user = run.request("GET", "/api/v1/user", &auth, b"");
        user.status(200);
        assert_eq!(user.json()["login"], "temper-admin");
        let repository = run.request("GET", "/api/v1/repositories/1", &auth, b"");
        repository.status(200);
        let repository = repository.json();
        assert_eq!(repository["default_branch"], landing);
        assert_eq!(repository["html_url"], format!("http://{}/{owner}/widget", run.address));
        let objects = run.request("POST", "/.fake/git/objects", "", br#"{"commits":[1]}"#);
        objects.status(200);
        let objects = objects.json();
        let object = &objects["objects"][0];
        assert_eq!(object["commit"], 1);
        assert!(object["parent"].is_null());
        let files = object["tree"].as_array().expect("commit tree");
        assert_eq!(files.len(), temper_fake::forge::FILES.len());
        let path = base64(b".forgejo/workflows/check.yml");
        assert!(files.iter().any(|file| file["path"] == path));

        grant(&mut run, owner, &auth);
        let writer = tree.capture(observations);
        run.before_reply(move || {
            let bytes = writer.bytes();
            let records = records(&bytes);
            assert!(records.iter().any(|line| line["observation"] == "moved" && line["branch"] == "work"));
            assert!(bytes.ends_with(CHECKPOINT), "checkpoint precedes the reply");
        });
        run.request(
            "POST",
            &format!("/api/v1/repos/{owner}/widget/branches"),
            &auth,
            br#"{"new_branch_name":"work","old_ref_name":"910a2dec89025cc1000000000000000100000000"}"#,
        )
        .status(201);
        run.stop(&mut tree);
        assert_eq!(tree.bytes(token_path), token, "token kept at stop");
        assert!(tree.contains(observations));
        assert!(tree.contains(DOCUMENT.as_ref()));
    }
}
