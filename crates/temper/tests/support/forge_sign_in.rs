//! Configuration and commands shared by forge sign-in proofs.
use super::{configuration, write_private};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

/// Add the Google member and the matching confidential Forgejo client.
pub fn configure(directory: &Path, forge: &str, temper: &str, google: &str) {
    let path = directory.join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["providers"].as_array_mut().expect("providers").push(json!({"number":3,"kind":"application"}));
    value["engine"]["owners"] = json!([{"provider":3,"subject":"sam@example.com"}]);
    value["engine"]["clients"]["provider"] = json!({
        "key":"provider.key", "addresses":[3],
        "clients":[{"id":"forgejo", "secret":"forgejo.secret",
            "redirect_uri":format!("http://{forge}/user/oauth2/temper/callback")}]
    });
    value["application"]["clients"]["listener"] = json!({"socket":temper});
    value["application"]["sign_in"] = json!({
        "google":{"provider":3,"client":"temper-test","secret":"google.secret","endpoints":{
            "issuer":format!("http://{google}"),"authorization":format!("http://{google}/auth"),
            "token":format!("http://{google}/token"),"keys":format!("http://{google}/keys"),"transport":"plaintext"}},
        "members":[{"address":"sam@example.com","login":"sam"}],
        "forges":[{"forge":1,"client":"forgejo"}]
    });
    write_private(&directory.join("google.secret"), b"throwaway-google-secret");
    write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
}

#[must_use]
pub fn command(directory: &Path) -> Command {
    let mut command = Command::new(skein_scratch::paths::binary("temper"));
    command.args(["forge", "sign-in"]).args(configuration::flags(directory)).arg("1").current_dir(directory);
    command
}

pub fn success(output: &Output) {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
