use super::{read_files_with_asked_using, serve_error};
use jig_protocol_config::{File, Path as Member, Segment};
use skein_lib::Secret;
use skein_shell::Unread;
use std::{os::unix::ffi::OsStrExt, path::Path};

fn scripted_serve<T>(
    binary: &str,
    shape: temper_config_shell::Shape,
    text: &[u8],
    is_executable: impl Fn(&std::path::Path) -> bool,
    prepare: impl FnOnce(&[u8], &temper_config_shell::Programs) -> Result<T, String>,
) -> Result<T, String> {
    let facts = temper_config_shell::Facts { login: "tester".into(), hostname: "fixture".into(), provider_key: false };
    let temper_config_shell::Read { file, document, keys } =
        temper_config_shell::parse(text, shape, &facts).expect("fixture document");
    let read = super::configuration::Read { file, document, keys };
    let programs = read.programs_with(is_executable).expect("fixture programs");
    super::prepare_serve(&read, &programs, prepare).map_err(|why| super::serve_error(binary, &why))
}

#[test]
fn a_refused_start_is_its_binarys_serve_line() {
    assert_eq!(
        scripted_serve(
            "temper-worker",
            temper_config_shell::Shape::WorkerAlone,
            b"engine = \"127.0.0.1:7200\"\n",
            |_| false,
            |_, _| Err::<(), _>("refused".into()),
        ),
        Err("temper-worker serve: refused\n".into())
    );
}

#[test]
fn toml_serve_maps_member_lines_and_keeps_other_problems() {
    assert_eq!(
        scripted_serve(
            "temper-box",
            temper_config_shell::Shape::Standalone,
            b"# box\n\nlisten = \"127.0.0.1:7100\"\npublic = \"http://example.com\"\n",
            |_| false,
            |_, _| {
                Err::<(), _>("application.clients.public: start's words\napplication.webhooks.listener: listener words\nagents.setting: unchanged\n".into())
            },
        ),
        Err(format!(
            "temper-box serve: {}:4: public: start's words\ntemper-box serve: {}: webhooks (its default): listener words\ntemper-box serve: agents.setting: unchanged\n",
            temper_config_shell::FILE,
            temper_config_shell::FILE
        ))
    );
}

#[test]
fn toml_serve_passes_checked_programs_beside_the_document() {
    let program = std::path::Path::new("/fixture/program");
    let text = format!(
        "listen = \"127.0.0.1:7100\"\n[programs]\ngit = {program:?}\nclaude = {program:?}\nforgejo = {program:?}\nrunner = {program:?}\n"
    );
    assert_eq!(
        scripted_serve(
            "temper-box",
            temper_config_shell::Shape::Standalone,
            text.as_bytes(),
            |path| path == program,
            |bytes, programs| {
                assert_eq!(programs.git.as_deref(), Some(program));
                assert_eq!(programs.claude.as_deref(), Some(program));
                assert_eq!(programs.forgejo.as_deref(), Some(program));
                assert_eq!(programs.runner.as_deref(), Some(program));
                assert!(
                    !bytes.windows(b"\"programs\"".len()).any(|word| word == b"\"programs\""),
                    "file-only paths stay out of the document"
                );
                Err::<(), _>("checked programs reached preparation".into())
            },
        ),
        Err("temper-box serve: checked programs reached preparation\n".into())
    );
}

const USER: u32 = 1000;
const PROVIDER_KEY: &str = "/fixture/provider.key";
const CLIENT_SECRET: &str = "/fixture/provider-client.secret";
// The provider file bounds in jig-protocol-config's file enumeration.
const KEY_MAX: u32 = 8192;
const CLIENT_SECRET_MAX: u32 = 256;

fn provider_files() -> Box<[File]> {
    let provider = Member::root().member(b"engine").member(b"clients").member(b"provider");
    Box::new([
        named_file(provider.member(b"key"), PROVIDER_KEY, KEY_MAX, true),
        named_file(
            provider.member(b"clients").child(Segment::Index(0)).member(b"secret"),
            CLIENT_SECRET,
            CLIENT_SECRET_MAX,
            true,
        ),
    ])
}

fn named_file(at: Member, path: &str, max: u32, private: bool) -> File {
    File { at, path: path.as_bytes().into(), max, private }
}

fn provider_refusal(key: Result<Secret, Unread>, secret: Result<Secret, Unread>) -> String {
    let mut reads = [(PROVIDER_KEY, KEY_MAX, key), (CLIENT_SECRET, CLIENT_SECRET_MAX, secret)].into_iter();
    let refused = read_files_with_asked_using(provider_files(), Box::new([]), USER, |path, max, private| {
        let (expected, bound, read) = reads.next().expect("one read per provider file");
        assert_eq!((path, max, private), (Path::new(expected), bound, true));
        read
    })
    .expect_err("provider file read refused");
    assert!(reads.next().is_none(), "both provider files were read");
    refused
}

#[test]
fn an_absent_provider_key_names_its_member_and_path() {
    let secret = Secret::new(b"synthetic-provider-client-secret".as_slice().into());
    assert_eq!(
        provider_refusal(Err(Unread::Absent), Ok(secret)),
        "engine.clients.provider.key: /fixture/provider.key: absent\n"
    );
}

#[test]
fn an_absent_provider_client_secret_names_its_member_and_path() {
    // The public 3072-bit fixture used by jig's provider admission tests.
    let key = include_bytes!("../../../../jig/crates/jig-protocol-config/src/tests/keys/rsa-3072-pkcs8.pem");
    assert_eq!(
        provider_refusal(Ok(Secret::new(key.as_slice().into())), Err(Unread::Absent)),
        "engine.clients.provider.clients[0].secret: /fixture/provider-client.secret: absent\n"
    );
}

#[test]
fn an_unsafe_worker_secret_is_named_without_its_bytes() {
    // The worker's 256-byte secret bound allows a final newline on disk.
    const MAX: u32 = 257;
    let at = Member::root().member(b"workers").child(Segment::Index(0)).member(b"secret");
    let path = "/fixture/worker.secret";
    let names = Box::new([named_file(at, path, MAX, true)]);
    let why = read_files_with_asked_using(names, Box::new([]), USER, |resolved, max, private| {
        assert_eq!((resolved, max, private), (Path::new(path), MAX, true));
        Err(Unread::Unsafe(skein_io::file::Unsafe::Mode(0o044)))
    })
    .expect_err("start refused");
    let refused = serve_error("temper", &why);
    assert_eq!(
        refused,
        format!("temper serve: workers[0].secret: {path}: readable by others or reached through a link\n")
    );
    assert!(!refused.contains("forge-1-git-token-value"), "no secret in a diagnostic");
}

// A small bound for the asked file fixtures.
const ASKED_MAX: u32 = 64;

fn asked_files() -> Box<[File]> {
    Box::new([
        named_file(Member::root().member(b"optional"), "/fixture/optional", ASKED_MAX, false),
        named_file(Member::root().member(b"ca"), "/fixture/ca", ASKED_MAX, false),
    ])
}

#[test]
fn absent_asked_files_are_skipped_and_successful_reads_kept() {
    let files = read_files_with_asked_using(Box::new([]), asked_files(), USER, |path, max, private| {
        match (path.as_os_str().as_bytes(), max, private) {
            (b"/fixture/optional", ASKED_MAX, false) => Err(Unread::Absent),
            (b"/fixture/ca", ASKED_MAX, false) => Ok(Secret::new(b"trust fixture".as_slice().into())),
            _ => panic!("unexpected read: {}", path.display()),
        }
    })
    .expect("an absent asked file is not a refusal");
    assert_eq!(files.contents(b"/fixture/optional"), None);
    assert_eq!(files.contents(b"/fixture/ca"), Some(b"trust fixture".as_slice()));
}

#[test]
fn required_refusals_precede_asked_refusals_in_the_diagnostic() {
    let names = Box::new([named_file(Member::root().member(b"key"), "/fixture/key", KEY_MAX, true)]);
    let refused = read_files_with_asked_using(names, asked_files(), USER, |path, max, private| {
        match (path.as_os_str().as_bytes(), max, private) {
            (b"/fixture/key", KEY_MAX, true) | (b"/fixture/optional", ASKED_MAX, false) => Err(Unread::Absent),
            (b"/fixture/ca", ASKED_MAX, false) => Err(Unread::TooLarge { max }),
            _ => panic!("unexpected read: {}", path.display()),
        }
    })
    .expect_err("required absence and an oversized asked file are refused");
    assert_eq!(refused, "key: /fixture/key: absent\nca: /fixture/ca: past 64 bytes\n");
}

#[test]
fn wait_check_uses_the_given_switch_outside_debug_builds() {
    let environment = [(Box::from(b"TEMPER_INTERNAL_WAIT_CHECK".as_slice()), Box::from(b"1".as_slice()))];
    assert!(super::host::wait_check(&environment, false));
    assert!(!super::host::wait_check(&[], false));
    let empty = [(Box::from(b"TEMPER_INTERNAL_WAIT_CHECK".as_slice()), Box::from(b"".as_slice()))];
    assert!(!super::host::wait_check(&empty, false));
    assert!(super::host::wait_check(&[], true));
}

#[test]
fn restart_cut_keeps_the_given_marker_only_with_the_wait_switch() {
    let cut = (Box::from(b"TEMPER_INTERNAL_CUT_AT_LISTEN".as_slice()), Box::from(b"/fixture/cut".as_slice()));
    let wait = (Box::from(b"TEMPER_INTERNAL_WAIT_CHECK".as_slice()), Box::from(b"1".as_slice()));
    assert_eq!(
        super::restart_fault::marker(&[wait.clone(), cut.clone()]),
        Some(std::path::PathBuf::from("/fixture/cut"))
    );
    assert_eq!(super::restart_fault::marker(&[cut]), None);
    assert_eq!(super::restart_fault::marker(&[wait]), None);
    assert_eq!(super::restart_fault::marker(&[]), None);
}

#[test]
fn local_link_uses_the_given_runtime_directory() {
    let arguments = ["--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
    let environment = [(Box::from(b"XDG_RUNTIME_DIR".as_slice()), Box::from(b"/fixture/runtime".as_slice()))];
    let link = super::local_link(&directories, &environment).expect("local link");
    if cfg!(target_os = "linux") {
        assert_eq!(link.parent(), Some(std::path::Path::new("/fixture/runtime/temper")));
        assert_eq!(super::local_link(&directories, &[]), None);
    } else {
        assert_eq!(link, std::path::Path::new("/d/printed-link"));
    }
}
