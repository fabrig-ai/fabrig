//! Startup refusals promised by protocol/sign-in.md 9.2 and 9.7.
use crate::support;

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};

fn provider(directory: &Path, bits: Option<u32>, secret: bool) {
    support::configuration(directory, &support::Forge::Unreached);
    let key = directory.join("provider.key");
    if let Some(bits) = bits {
        let status = Command::new("openssl")
            .args(["genpkey", "-algorithm", "RSA", "-pkeyopt", &format!("rsa_keygen_bits:{bits}"), "-out"])
            .arg(&key)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("generate scratch key without exposing it");
        assert!(status.success());
        fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private scratch key");
    }
    let secret_path = directory.join("provider-client.secret");
    if secret {
        fs::write(&secret_path, b"synthetic-provider-client-secret").expect("scratch secret");
        fs::set_permissions(&secret_path, fs::Permissions::from_mode(0o600)).expect("private scratch secret");
    }
    let path = directory.join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["clients"]["provider"] = serde_json::json!({
        "key":key,
        "clients":[{"id":"forgejo","secret":secret_path,"redirect_uri":"http://127.0.0.1:9/callback"}],
        "addresses":[]
    });
    fs::write(path, serde_json::to_vec(&value).expect("JSON")).expect("configuration");
}

fn refused(directory: &Path, field: &str) {
    let bytes = fs::read(directory.join("temper.json")).expect("configuration");
    let why = temper::operations_in(&support::configuration::directories(directory), &bytes)
        .expect_err("provider admission refuses");
    assert!(why.contains("provider") && why.contains(field), "{why}");
}

#[test]
fn missing_provider_key_refuses_before_serving() {
    let scratch = support::Scratch::new("provider-missing-key");
    provider(scratch.path(), None, true);
    refused(scratch.path(), "key");
    scratch.remove();
}

#[test]
fn rsa_2048_is_refused_even_when_the_client_is_valid() {
    // 9.7 requires RSA 3072, not merely an RSA key accepted by OpenSSL.
    let scratch = support::Scratch::new("provider-short-key");
    provider(scratch.path(), Some(2048), true);
    refused(scratch.path(), "key");
    scratch.remove();
}

#[test]
fn missing_client_secret_refuses_with_a_valid_signing_key() {
    let scratch = support::Scratch::new("provider-missing-client-secret");
    provider(scratch.path(), Some(3072), false);
    refused(scratch.path(), "secret");
    scratch.remove();
}
