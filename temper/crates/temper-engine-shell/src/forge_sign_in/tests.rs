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
    use temper_config_shell::{
        File, ForgeSignIn, Member, OpenId, OpenIdClient, Shape, SignIn, Test, TestGoogle, Transport,
    };
    let directory = std::env::temp_dir().join(format!("temper-public-sign-in-{}", std::process::id()));
    fs::create_dir(&directory).expect("fresh scratch");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private scratch");
    let file = File {
        workers: Some("127.0.0.1:7200".parse().expect("workers address")),
        owner: Some("sam@example.com".into()),
        openid: Some(OpenId {
            key: Some("provider.key".into()),
            addresses: Some(vec![1]),
            clients: vec![OpenIdClient {
                id: "forgejo".into(),
                secret: None,
                redirect: "http://127.0.0.1:3000/user/oauth2/temper/callback".into(),
            }],
        }),
        sign_in: Some(SignIn {
            client: "temper-test".into(),
            secret: "google.secret".into(),
            members: vec![Member { address: "sam@example.com".into(), login: "sam".into() }],
            forges: vec![ForgeSignIn { forge: 1, client: "forgejo".into() }],
        }),
        test: Test {
            google: Some(TestGoogle {
                issuer: "http://127.0.0.1:31999".into(),
                authorization: "http://127.0.0.1:31999/auth".into(),
                token: "http://127.0.0.1:31999/token".into(),
                keys: "http://127.0.0.1:31999/keys".into(),
                transport: Transport::Plaintext,
            }),
            ..Test::default()
        },
        ..File::default()
    };
    let text = temper_config_shell::render(&file, Shape::EngineAlone);
    let bytes = temper_config_shell::parse(
        &text,
        Shape::EngineAlone,
        &temper_config_shell::Facts { login: "sam".into(), hostname: "fixture".into(), provider_key: false },
    )
    .expect("engine file")
    .document;
    let directories = configuration::directories(&directory);
    let named =
        temper_engine_service::startup::named(&bytes, &jig_directories(&directories)).expect("named public client");
    let refusal = super::run(&directories, &bytes, &named, 1, &[]);
    let files = fs::read_dir(&directory).expect("scratch entries").count();
    let configuration = directory.join(temper_config_shell::FILE);
    fs::write(&configuration, &text).expect("command configuration");
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
