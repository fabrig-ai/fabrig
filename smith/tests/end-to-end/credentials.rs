//! Credential commands on scratch state, the fake issuer and fake program.
use super::support;
use skein_lib::{Duration, Wall};
use skein_shell::{Clock, Host};
use skein_world::end_to_end::{Mode, Streams};
use smith_local_process_world::{
    process::{IssuerProcess, Proc},
    terminal::{Action, Terminal},
    world,
};
use smith_real_world::Scratch;
use std::os::unix::fs::DirBuilderExt;

const FULL_SCOPE: &[u8] = b"openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";

fn settings(scratch: &Scratch, extra: &str) {
    let path = scratch.path().join("config/smith");
    std::fs::create_dir_all(&path).expect("credential fixture");
    let host = smith_real_world::loopback();
    std::fs::write(path.join("config.toml"), format!(
        "[accounts.test]\nprovider='openai'\nsource='sign_in'\nissuer='http://{host}:34444'\nauthorization_url='http://{host}:34444/authorize'\ntoken_url='http://{host}:34444/token'\nkeys_url='http://{host}:34444/keys'\nrevocation_url='http://{host}:34444/revoke'\nredirect_port={}\n{extra}", smith_real_world::redirect_port())).expect("credential fixture");
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
        id_token: Some(b"prior-id-for-work".as_slice().into()),
        earliest_refresh_at,
        key: 0,
        generation: 1,
        access_token: b"access-for-work".as_slice().into(),
        refresh_token: Some(b"refresh-for-work".as_slice().into()),
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

// smith/docs/design/testing.md, 2.3 and 8; skein/docs/design/oauth.md, 6.3.
fn refresh_fixture(scratch: &Scratch, earliest_refresh_at: Option<Wall>) -> (Box<[u8]>, Box<[u8]>) {
    let now = Clock::new().now().wall;
    let expires_at = Wall::from_nanos(
        now.as_nanos().checked_add(Duration::from_secs(3600).as_nanos()).expect("fixture expiry fits"),
    );
    let held = world::login_at(now, false);
    let issued = world::login_at(expires_at, false);
    let bytes = seed_bytes(b"person-for-work", expires_at, earliest_refresh_at);
    let mut record = skein_oauth::decode_record(&bytes, &limits()).expect("current fixture record");
    record.issued_client_id = Some(b"client".as_slice().into());
    record.access_token.clone_from(&held);
    record.refresh_token = Some(b"refresh-old".as_slice().into());
    let path = path(scratch, "test");
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path.parent().expect("tokens parent"))
        .expect("private fixture tokens");
    support::provider_login::keep_record(
        &path,
        &skein_oauth::encode_record(&record, &limits()).expect("current fixture encoding"),
    )
    .expect("current fixture kept");
    let host = smith_real_world::loopback();
    let configuration = format!(
        "model='codex/gpt-6-luna'\neffort='low'\ninstructions='@local-shell Assist'\n\
         [accounts.test]\nsource='sign_in'\nclient_id='client'\nscope='{scope}'\n\
         authorization_url='http://{host}:34444/authorize'\ntoken_url='http://{host}:34444/token'\n\
         redirect_uri='http://127.0.0.1:{}/callback'\ntoken_format='form'\n\
         [endpoints.codex]\naccount='test'\naddress='{host}:34443'\nserver_name='skein.test'\n\
         trust.system=false\ntrust.der=['{}']\n",
        smith_real_world::redirect_port(),
        scratch.path().join("root.der").display(),
        scope = std::str::from_utf8(FULL_SCOPE).expect("fixture scope text"),
    );
    let _file = support::user_layer(scratch, configuration.as_bytes());
    (held, issued)
}

fn exec_on_raised_lead(scratch: &Scratch, accepted: Box<[u8]>, refreshed: Box<[u8]>) -> (support::Ran, u64) {
    use smith_real_world::settled::assert_no_children;
    use support::Process;

    assert_no_children();
    let mut binary = support::start(
        ["exec", "--json", "-c", "deployment.refresh_lead=34560000", "Ask first"].into_iter().map(Into::into).collect(),
        scratch.path(),
        Mode::Pipes,
    );
    let streams = binary.take_streams().expect("exec streams");
    let descriptors = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { unreachable!("exec pipes") };
    let clock = Clock::new();
    let now = clock.now().now;
    let mut world =
        skein_world::real::World::new(support::Settling::new(now, now.saturating_add(Duration::from_secs(10))));
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(descriptors, || {
        Process::Script(Proc::Terminal(Box::new(Terminal::piped(input, output, vec![Action::Eof]))))
    });
    world.spawn(|| {
        Process::Script(Proc::Peer(Box::new(world::headless_provider(
            false,
            skein_llm::Credential {
                value: accepted,
                account_id: b"acc".as_slice().into(),
                scheme: skein_llm::Scheme::Bearer,
            },
        ))))
    });
    world.spawn(|| {
        Process::Script(Proc::Issuer(Box::new(IssuerProcess::with_token(
            skein_fake_peers::Transport::Plaintext,
            34_444,
            refreshed,
        ))))
    });
    let outcome = world.run(&clock, Duration::from_secs(12));
    assert!(outcome.procs.iter().all(Host::is_empty), "exec, person, provider and issuer drain");
    let Process::Binary(binary) = outcome.procs.first().expect("exec observation") else { unreachable!("exec binary") };
    let Process::Script(Proc::Terminal(person)) = outcome.procs.get(1).expect("person observation") else {
        unreachable!("exec person")
    };
    let Process::Script(Proc::Issuer(issuer)) = outcome.procs.get(3).expect("issuer observation") else {
        unreachable!("fake issuer")
    };
    let Some(skein_io::kernel::Exit::Code(code)) = binary.exit_status() else {
        let errors = String::from_utf8_lossy(binary.stderr());
        let panic_at = errors.lines().find(|line| line.starts_with("thread 'main' panicked at "));
        let statuses: Vec<_> = issuer
            .peer()
            .observations()
            .iter()
            .filter_map(|observation| {
                if let skein_fake_peers::oauth::Observation::Answered { status, .. } = observation {
                    Some(*status)
                } else {
                    None
                }
            })
            .collect();
        panic!(
            "exec exit={:?}, stderr bytes={}, panic location={panic_at:?}, issuer statuses={statuses:?}, posts={}, generation={}",
            binary.exit_status(),
            binary.stderr().len(),
            smith_local_process_world::oauth::posts(issuer.peer()),
            record(scratch, "test").generation
        );
    };
    let posts = smith_local_process_world::oauth::posts(issuer.peer());
    let answered = issuer
        .peer()
        .observations()
        .iter()
        .filter(|observation| matches!(observation, skein_fake_peers::oauth::Observation::Answered { status: 200, .. }))
        .count();
    assert_eq!(u64::try_from(answered).expect("bounded issuer observations"), posts, "refresh POSTs succeed");
    assert_no_children();
    (support::Ran { code, stdout: person.shown().into(), stderr: binary.stderr().into() }, posts)
}

fn accepted_exec(ran: &support::Ran) {
    // The binary harness captures at most 64 KiB per stream.
    const STREAM_BYTES: u32 = 65_536;
    assert_eq!(ran.code, 0, "fake-backed exec succeeds");
    let limits = {
        let mut limits = smith_events::fixture::SMALL;
        limits.string = STREAM_BYTES;
        limits.content = STREAM_BYTES;
        limits.items = STREAM_BYTES;
        limits
    };
    let records: Vec<_> = ran
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            smith_events::read_with_ceiling(line, &limits, STREAM_BYTES)
                .expect("typed exec event")
                .expect("known exec event")
        })
        .collect();
    let answers: Vec<_> = records
        .iter()
        .filter_map(|record| {
            if let smith_events::Event::RunCompleted(answer) = &record.event { Some(&answer.status) } else { None }
        })
        .collect();
    assert_eq!(answers, [&smith_events::Status::Accepted], "one accepted answer");
    no_values(ran);
}

#[test]
fn an_exec_on_a_raised_lead_refreshes_its_record_at_once() {
    let scratch = Scratch::new();
    let (_held, issued) = refresh_fixture(&scratch, None);
    assert_eq!(record(&scratch, "test").generation, 1);
    let (ran, refreshes) = exec_on_raised_lead(&scratch, issued.clone(), issued);
    accepted_exec(&ran);
    assert_eq!(refreshes, 1, "one refresh answered at the issuer");
    assert_eq!(record(&scratch, "test").generation, 2, "refreshed record kept");
}

#[test]
fn an_exec_before_the_earliest_refresh_time_keeps_its_record() {
    let scratch = Scratch::new();
    let now = Clock::new().now().wall;
    let earliest = Wall::from_nanos(
        now.as_nanos().checked_add(Duration::from_secs(3600).as_nanos()).expect("fixture earliest refresh fits"),
    );
    let (held, issued) = refresh_fixture(&scratch, Some(earliest));
    assert_eq!(record(&scratch, "test").generation, 1);
    let (ran, refreshes) = exec_on_raised_lead(&scratch, held, issued);
    accepted_exec(&ran);
    assert_eq!(refreshes, 0, "no refresh before the earliest refresh time");
    assert_eq!(record(&scratch, "test").generation, 1, "held record stays current");
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
    settings(&scratch, "");
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
fn signing_in_again_uses_the_issued_id() {
    let scratch = Scratch::new();
    settings(&scratch, "");
    seed(&scratch, "test", b"person-for-work");
    let (ran, urls, _) = support::provider_login::run(&scratch, "test", false, b"person-for-work", true, true, None);
    assert_eq!(ran.code, 0);
    let url = String::from_utf8_lossy(&urls[0]);
    assert!(url.contains("client_id=oaiapp_for_work"));
    assert!(!url.contains("agent_name_hint"));
    assert!(url.contains("ext_agent_host_id="));
    assert!(url.contains("id_token_hint"));
    assert!(!String::from_utf8_lossy(&ran.stderr).contains("You're using your ChatGPT plan."));
    no_values(&ran);
}

#[test]
fn another_subject_preserves_the_record() {
    let scratch = Scratch::new();
    settings(&scratch, "");
    seed(&scratch, "test", b"person-for-work");
    let before = std::fs::read(path(&scratch, "test")).expect("credential fixture");
    let (ran, urls, _) = support::provider_login::run(&scratch, "test", false, b"person-for-home", true, true, None);
    assert_eq!(ran.code, 2);
    assert_eq!(urls.len(), 1);
    assert!(String::from_utf8_lossy(&ran.stderr).contains("another subject was refused"));
    assert!(before == std::fs::read(path(&scratch, "test")).expect("credential fixture"), "record stays byte-for-byte");
    no_values(&ran);
}

#[test]
fn a_declined_plan_scope_keeps_the_record() {
    let scratch = Scratch::new();
    settings(&scratch, "");
    let (ran, _, _) = support::provider_login::run(&scratch, "test", false, b"person-for-work", false, false, None);
    assert_eq!(ran.code, 3);
    assert!(path(&scratch, "test").exists());
    assert!(
        String::from_utf8_lossy(&ran.stderr)
            .contains("Signed in without plan usage: run `smith login test` again and allow it, or use a key")
    );
    no_values(&ran);
}

#[test]
fn a_duplicate_subject_names_its_account() {
    let scratch = Scratch::new();
    settings(&scratch, "[accounts.first]\nprovider='openai'\nsource='sign_in'\n");
    seed(&scratch, "first", b"person-for-work");
    let (ran, _, revoked) =
        support::provider_login::run(&scratch, "test", false, b"person-for-work", true, false, None);
    assert_eq!(ran.code, 2);
    assert!(String::from_utf8_lossy(&ran.stderr).contains("account first"));
    assert!(!path(&scratch, "test").exists());
    assert!(path(&scratch, "first").exists());
    assert_eq!(revoked, 1);
    no_values(&ran);
}

#[test]
fn logout_revokes_the_record() {
    let scratch = Scratch::new();
    settings(&scratch, "");
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
fn an_unconfirmed_revocation_says_where_to_disconnect() {
    let scratch = Scratch::new();
    settings(&scratch, "");
    seed(&scratch, "test", b"person-for-work");
    let (ran, _, revoked) =
        support::provider_login::run(&scratch, "test", true, b"person-for-work", true, true, Some(false));
    assert_eq!(ran.code, 0);
    assert_eq!(revoked, 1);
    assert!(!path(&scratch, "test").exists());
    assert!(String::from_utf8_lossy(&ran.stderr).contains("disconnect smith at chatgpt.com/settings"));
    no_values(&ran);
}

#[test]
fn key_commands_keep_values_private() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, b"[accounts.work]\nsource='key'\n[accounts.empty]\nsource='none'\n[accounts.variable]\nsource='env'\nvariable='WORK_KEY'\n");
    let login = support::run_product(&scratch, &["login", "work"], b"key-for-work\n", None);
    assert_eq!(login.code, 0);
    assert!(scratch.path().join("state/smith/keys/work").is_file());
    no_values(&login);
    let terminal = support::provider_login::terminal_key(&scratch, "work");
    assert_eq!(terminal.code, 2);
    no_values(&terminal);
    for (account, code) in [("work", 0), ("empty", 0), ("variable", 0)] {
        let logout = support::run_product(&scratch, &["logout", account], b"", None);
        assert_eq!(logout.code, code);
        no_values(&logout);
    }
    assert!(!scratch.path().join("state/smith/keys/work").exists());
}

#[test]
fn a_racing_first_login_preserves_the_winner() {
    let scratch = Scratch::new();
    settings(&scratch, "");
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path(&scratch, "test").parent().expect("tokens parent"))
        .expect("tokens fixture");
    let winner = seed_bytes(b"person-for-home", Wall::from_nanos(u64::MAX), None);
    let (ran, urls, _) = support::provider_login::run_arriving(
        &scratch,
        "test",
        false,
        b"person-for-work",
        true,
        false,
        (None, Some((path(&scratch, "test"), winner.clone()))),
    );
    assert_eq!(ran.code, 2);
    assert_eq!(urls.len(), 1);
    assert!(String::from_utf8_lossy(&ran.stderr).contains("account test: another subject was refused"));
    assert!(std::fs::read(path(&scratch, "test")).expect("winner record") == winner, "winner stays byte-for-byte");
    no_values(&ran);
}

#[test]
fn a_seed_refuses_an_existing_public_file() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new();
    let path = scratch.path().join("record");
    std::fs::write(&path, b"previous").expect("public fixture");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("public fixture mode");
    support::provider_login::keep_record(&path, b"record-for-work").expect_err("never write into a public file");
    assert!(std::fs::read(&path).expect("untouched fixture") == b"previous", "existing file stays untouched");
}
