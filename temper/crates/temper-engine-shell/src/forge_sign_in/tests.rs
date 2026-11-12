use super::make_missing;
use crate::test_configuration as configuration;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};
use temper_service_shell::jig_directories;

#[test]
fn credentials_are_private_whole_and_left_in_place_on_a_second_run() {
    let directory = std::env::temp_dir().join(format!("temper-forge-sign-in-{}", std::process::id()));
    fs::create_dir(&directory).expect("fresh private scratch");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private scratch");
    for (name, key) in [("key.pem", true), ("client.secret", false)] {
        let path = directory.join(name);
        make_missing(&path, key).expect("make credential");
        let first = fs::metadata(&path).expect("credential metadata");
        assert_eq!(first.permissions().mode() & 0o777, 0o600);
        assert_eq!(first.uid(), skein_shell::effective_user());
        if key {
            assert!(first.len() > 2000);
        } else {
            assert_eq!(first.len(), 43);
        }
        make_missing(&path, key).expect("leave existing credential alone");
        let second = fs::metadata(&path).expect("credential metadata");
        assert_eq!(first.ino(), second.ino());
        assert_eq!(first.mtime_nsec(), second.mtime_nsec());
    }
    assert_eq!(fs::read_dir(&directory).expect("scratch entries").count(), 2, "no partial files");
    fs::remove_dir_all(directory).expect("scratch removed");
}

#[test]
fn a_public_client_refuses_before_creating_any_file() {
    use serde_json::json;
    let directory = std::env::temp_dir().join(format!("temper-public-sign-in-{}", std::process::id()));
    fs::create_dir(&directory).expect("fresh scratch");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private scratch");
    let (_, bytes) = configuration::documents(&directory, "/fixture/temper");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("fixture JSON");
    value["engine"]["clients"]["provider"] = json!({
        "key":"provider.key", "addresses":[1],
        "clients":[{"id":"forgejo", "secret":null,
            "redirect_uri":"http://127.0.0.1:3000/user/oauth2/temper/callback"}]
    });
    value["engine"]["providers"].as_array_mut().expect("providers").push(json!({"number":3,"kind":"application"}));
    value["engine"]["owners"] = json!([{"provider":3,"subject":"sam@example.com"}]);
    value["application"]["sign_in"] = json!({
        "google":{"provider":3,"client":"temper-test","secret":"google.secret","endpoints":{
            "issuer":"http://127.0.0.1:31999","authorization":"http://127.0.0.1:31999/auth",
            "token":"http://127.0.0.1:31999/token","keys":"http://127.0.0.1:31999/keys","transport":"plaintext"}},
        "members":[{"address":"sam@example.com","login":"sam"}],"forges":[{"forge":1,"client":"forgejo"}]
    });
    value["workers"] = json!([]);
    value["application"]["workers"] = json!([]);
    value["engine"]["hosts"]["listener"] = json!([{"socket":"127.0.0.1:7200"}]);
    let bytes = serde_json::to_vec(&value).expect("configuration");
    let directories = configuration::directories(&directory);
    let named =
        temper_engine_service::startup::named(&bytes, &jig_directories(&directories)).expect("named public client");
    let refusal = super::run(&directories, &bytes, &named, 1, &[]);
    let files = fs::read_dir(&directory).expect("scratch entries").count();
    let configuration = directory.join("temper.json");
    fs::write(&configuration, &bytes).expect("command configuration");
    let exit = crate::forge::run("temper", &directories, &["sign-in".into(), "1".into()], None, &[]);
    let command_files = fs::read_dir(&directory).expect("command scratch entries").count();
    fs::remove_dir_all(directory).expect("scratch removed");
    assert_eq!(refusal, Err("its client has no secret; a box adds its own source".into()));
    assert_eq!(files, 0, "refusal makes no files");
    assert_eq!(exit, 1, "the command uses its refusal exit");
    assert_eq!(command_files, 1, "only the supplied configuration remains");
}

#[test]
fn first_sign_in_uses_the_given_registration_switch() {
    use temper_forge_setup::sign_in::FirstSignIn;
    let environment = [(Box::from(b"TEMPER_INTERNAL_FORGE_NO_REGISTRATION".as_slice()), Box::from(b"1".as_slice()))];
    assert_eq!(super::first_sign_in(&environment), FirstSignIn::LinksOnly);
    assert_eq!(super::first_sign_in(&[]), FirstSignIn::MakesAccount);
    let empty = [(Box::from(b"TEMPER_INTERNAL_FORGE_NO_REGISTRATION".as_slice()), Box::from(b"".as_slice()))];
    assert_eq!(super::first_sign_in(&empty), FirstSignIn::MakesAccount);
}
