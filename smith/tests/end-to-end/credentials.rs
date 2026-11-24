//! Credential commands on scratch state, the fake issuer and fake program.
use super::support;
use skein_lib::Wall;
use smith_real_world::Scratch;
use std::os::unix::fs::DirBuilderExt;

const FULL_SCOPE: &[u8] = b"openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";

fn settings(scratch: &Scratch) {
    let path = scratch.path().join("config/smith");
    std::fs::create_dir_all(&path).expect("credential fixture");
    let origin = smith_real_world::address(34_444);
    std::fs::write(path.join("config.toml"), format!(
        "[accounts.test]\nprovider='openai'\nsource='sign_in'\nissuer='http://{origin}'\nauthorization_url='http://{origin}/authorize'\ntoken_url='http://{origin}/token'\nkeys_url='http://{origin}/keys'\nrevocation_url='http://{origin}/revoke'\nredirect_port={}\n", smith_real_world::redirect_port())).expect("credential fixture");
}

fn limits() -> skein_oauth::Limits {
    let mut limits = smith_local_shell::credentials::token_limits();
    limits.issued_client_bytes = 1024;
    limits.subject_bytes = 1024;
    limits
}

fn path(scratch: &Scratch, name: &str) -> std::path::PathBuf {
    scratch.path().join("state/smith/tokens").join(name)
}

fn record(scratch: &Scratch, name: &str) -> skein_oauth::SavedToken {
    let bytes =
        skein_shell::read_private_file(&path(scratch, name), limits().record_bytes, skein_shell::effective_user())
            .expect("credential fixture");
    skein_oauth::decode_record(bytes.expose(), &limits()).expect("credential fixture")
}

fn seed_bytes(subject: &[u8], expires_at: Wall, earliest_refresh_at: Option<Wall>) -> Vec<u8> {
    let record = skein_oauth::SavedToken {
        issued_client_id: Some(b"oaiapp_for_work".as_slice().into()),
        granted_scopes: Some(FULL_SCOPE.into()),
        subject: Some(subject.into()),
        id_token: Some(skein_lib::Secret::new(b"prior-id-for-work".as_slice().into())),
        earliest_refresh_at,
        key: 0,
        generation: 1,
        access_token: skein_lib::Secret::new(b"access-for-work".as_slice().into()),
        refresh_token: Some(skein_lib::Secret::new(b"refresh-for-work".as_slice().into())),
        metadata: None,
        expires_at,
    };
    skein_oauth::encode_record(&record, &limits()).expect("credential fixture").into_vec()
}

fn seed(scratch: &Scratch, name: &str, subject: &[u8]) {
    let path = path(scratch, name);
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path.parent().expect("credential fixture"))
        .expect("credential fixture");
    support::provider_login::keep_record(&path, &seed_bytes(subject, Wall::from_nanos(u64::MAX), None))
        .expect("credential fixture");
}

fn no_values(ran: &support::Ran) {
    for bytes in [&ran.stdout, &ran.stderr] {
        let words = String::from_utf8_lossy(bytes);
        assert!(!words.contains("access-for-work"), "credential value absent from output");
        assert!(!words.contains("refresh-for-work"), "credential value absent from output");
        assert!(!words.contains("key-for-work"), "credential value absent from output");
    }
}

#[test]
fn a_first_sign_in_registers() {
    let scratch = Scratch::new();
    settings(&scratch);
    let (ran, urls, _) = support::provider_login::run(&scratch, "test", false, b"person-for-work", true, false, None);
    assert_eq!(ran.code, 0);
    assert!(
        record(&scratch, "test").issued_client_id.as_deref() == Some(b"oaiapp_for_work".as_slice()),
        "issued registration kept"
    );
    assert!(scratch.path().join("state/smith/host-id").exists());
    assert!(
        record(&scratch, "test").subject.as_deref() == Some(b"person-for-work".as_slice()),
        "validated subject kept"
    );
    assert!(String::from_utf8_lossy(&ran.stderr).contains("ssh -L "));
    assert!(String::from_utf8_lossy(&ran.stderr).contains(" 192.0.2.1"));
    assert!(path(&scratch, "test.lock").is_dir());
    assert_eq!(String::from_utf8_lossy(&ran.stderr).matches("You're using your ChatGPT plan.").count(), 1);
    assert!(skein_lib::bytes::find(&urls[0], b"agent_name_hint=smith").is_some());
    no_values(&ran);
}

#[test]
fn logout_revokes_the_record() {
    let scratch = Scratch::new();
    settings(&scratch);
    seed(&scratch, "test", b"person-for-work");
    let (ran, _, revoked) =
        support::provider_login::run(&scratch, "test", true, b"person-for-work", true, true, Some(true));
    assert_eq!(ran.code, 0);
    assert_eq!(revoked, 1);
    assert!(!path(&scratch, "test").exists());
    assert!(!scratch.path().join("state/smith/host-id").exists());
    no_values(&ran);
}

#[test]
fn key_commands_keep_values_private() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, b"[accounts.work]\nsource='key'\n");
    let login = support::run_product(&scratch, &["login", "work"], b"key-for-work\n", None);
    assert_eq!(login.code, 0);
    assert!(scratch.path().join("state/smith/keys/work").is_file());
    no_values(&login);
    let terminal = support::provider_login::terminal_key(&scratch, "work");
    assert_eq!(terminal.code, 2);
    no_values(&terminal);
}
