//! Document-credential fixtures retained for standalone preparation and refusal tests.
//! temper/docs/design/protocol/setup.md, 13.4.
use std::path::Path;

#[must_use]
pub fn document(directory: &Path) -> serde_json::Value {
    let mut value = super::configuration::check_document(directory);
    value["application"]["forges"] = serde_json::json!([{
        "number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:9"},"transport":"plaintext",
        "git":"http://127.0.0.1:9","webhook_secret":null,"api_token":"secrets/forge-1.api-token",
        "user":{"file":"secrets/forge-1.user"},"identity":1}]);
    value["application"]["identities"] = serde_json::json!([{"number":1,"token":"secrets/forge-1.git-token",
        "login":"temper","author":{"name":"Temper","email":"temper@example.test"}}]);
    value["application"]["setup"] = serde_json::json!([]);
    value
}

/// File credentials belong only to tests that inspect or reject these document members.
pub fn files(directory: &Path) {
    for (name, bytes) in [
        ("user", b"42\n".as_slice()),
        ("api-token", b"forge-1-api-token-value"),
        ("git-token", b"forge-1-git-token-value"),
    ] {
        super::write_private(&directory.join(format!("secrets/forge-1.{name}")), bytes);
    }
}

/// A deployment whose assertions require the document's file credentials.
pub fn configuration(directory: &Path) {
    super::configuration(directory, &super::Forge::Unreached);
    files(directory);
    super::write_private(&directory.join("temper.json"), &serde_json::to_vec(&document(directory)).expect("JSON"));
}

/// temper-15: the explicit old-form fake and managed command fixtures, without creating files.
#[must_use]
pub fn old_form_check_document(directory: &Path) -> serde_json::Value {
    let mut value = document(directory);
    value["application"]["setup"] = serde_json::json!([{
        "forge":1, "directory":"forge-1", "bot":{"login":"temper","email":"temper@example.test"}
    }]);
    super::configuration::fake_entry(&mut value["application"]["setup"][0], "127.0.0.1:9");
    value["application"]["forges"][0]["api_token"] = "forge-1/api_token".into();
    value["application"]["forges"][0]["user"] = serde_json::json!({"file":"forge-1/user"});
    value["application"]["identities"][0]["token"] = "forge-1/git_token".into();
    value
}

/// The fake starts from records; its identity joins the forge explicitly.
#[must_use]
pub fn check_document(directory: &Path) -> serde_json::Value {
    let mut value = old_form_check_document(directory);
    let forge = value["application"]["forges"][0].as_object_mut().expect("forge");
    for member in ["api_token", "user", "webhook_secret", "identity"] {
        forge.remove(member);
    }
    let identity = value["application"]["identities"][0].as_object_mut().expect("identity");
    identity.remove("token");
    identity.insert("forge".into(), 1.into());
    value
}

/// Startup gate references require a document forge before a service can accept runtime requests.
pub fn landing_forge(directory: &Path, value: &mut serde_json::Value) {
    let forge = super::forges::fixture(directory).expect("throwaway forge manifest");
    let api = forge["api"].as_str().expect("forge API").parse().expect("forge socket");
    let mut application = super::owned::application(&super::owned::Owned::beside(api));
    super::owned::old_form(&mut application);
    for member in ["forges", "identities", "setup"] {
        value["application"][member] = application[member].take();
    }
}
