use super::{
    CONFIG_BYTES, StartupReads, configure, read_configuration,
    testing::{agent_document, agent_endpoint_document, scratch},
};
use std::fs;

#[test]
fn serve_without_a_configuration_path_is_a_usage_error() {
    assert_eq!(configure(["serve".into()].into_iter()), Err(super::USAGE.into()));
}

#[test]
fn a_configuration_file_past_its_bound_is_refused() {
    let directory = scratch("configuration-bound");
    let path = directory.join("config.json");
    let file = fs::File::create(&path).expect("configuration file");
    file.set_len(u64::from(CONFIG_BYTES) + 1).expect("one byte past bound");
    assert!(
        read_configuration(&path, &StartupReads::Native).expect_err("oversized configuration").contains("TooLarge")
    );
    fs::remove_dir_all(directory).expect("scratch cleanup");
}

fn fixture_bundle(directory: &std::path::Path) {
    fs::write(directory.join("index.html"), b"<!doctype html><title>t</title>").expect("page");
    let digest = directory.join("0123456789abcdef");
    fs::create_dir_all(digest.join("files")).expect("digest files");
    fs::write(digest.join("boot.js"), b"boot();").expect("script");
    fs::write(digest.join("files/robots.txt"), b"User-agent: *\nDisallow:\n").expect("root file");
}
#[test]
fn a_bundle_directory_is_read_with_its_one_digest_directory() {
    let directory = scratch("bundle-read");
    fixture_bundle(&directory);
    let bundle = super::bundle::read(&directory).expect("bundle read");
    assert_eq!(bundle.page.as_ref(), b"<!doctype html><title>t</title>");
    assert_eq!(bundle.etag.as_ref(), b"\"edaa28e315690734\"");
    assert_eq!(bundle.digest.as_ref(), b"0123456789abcdef");
    assert_eq!(bundle.files.len(), 2);
    let script = bundle.files.get(b"boot.js".as_slice()).expect("script");
    assert_eq!((script.kind, script.bytes.as_ref()), (b"text/javascript".as_slice(), b"boot();".as_slice()));
    let robots = bundle.files.get(b"files/robots.txt".as_slice()).expect("root file");
    assert_eq!(robots.bytes.as_ref(), b"User-agent: *\nDisallow:\n");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn a_file_off_the_closed_list_refuses_the_bundle_naming_it() {
    let directory = scratch("bundle-type");
    fixture_bundle(&directory);
    let file = directory.join("0123456789abcdef/private.exe");
    fs::write(&file, b"not served").expect("unknown file");
    let error = super::bundle::read(&directory).expect_err("closed list");
    assert!(error.contains(file.to_str().expect("path")), "{error}");
    assert!(error.contains("closed list"), "{error}");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn a_second_digest_directory_refuses_the_bundle() {
    let directory = scratch("bundle-digests");
    fixture_bundle(&directory);
    fs::create_dir(directory.join("fedcba9876543210")).expect("second digest");
    let error = super::bundle::read(&directory).expect_err("one digest");
    assert!(error.contains("digest directory"), "{error}");
    assert!(error.contains(directory.to_str().expect("path")), "{error}");
    fs::remove_dir_all(directory).expect("scratch cleanup");
}

const DEV_MODULE_BYTES: usize = 9 << 20;
const MODULE_FILL: u8 = 0;
#[test]
fn a_module_above_four_mebibytes_is_read() {
    let directory = scratch("dev-module");
    fixture_bundle(&directory);
    fs::write(directory.join("0123456789abcdef/test_bg.wasm"), vec![MODULE_FILL; DEV_MODULE_BYTES])
        .expect("dev-sized module");
    let result = super::bundle::read(&directory)
        .map(|bundle| bundle.files.get(b"test_bg.wasm".as_slice()).expect("module").bytes.len());
    fs::remove_dir_all(directory).expect("scratch cleanup");
    assert_eq!(result, Ok(DEV_MODULE_BYTES));
}

const OVERSIZED_FILE_BYTES: u64 = 4_194_305;
#[test]
fn another_oversized_file_is_refused_naming_it() {
    let directory = scratch("oversized-script");
    fixture_bundle(&directory);
    let path = directory.join("0123456789abcdef/boot.js");
    fs::File::create(&path).expect("script").set_len(OVERSIZED_FILE_BYTES).expect("oversized script");
    let error = super::bundle::read(&directory).expect_err("oversized script refused");
    fs::remove_dir_all(directory).expect("scratch cleanup");
    assert!(error.contains(path.to_str().expect("path")), "{error}");
    assert!(error.contains("TooLarge"), "{error}");
}

const FULL_BUNDLE_MODULE_BYTES: u64 = 33_554_432;
#[test]
fn a_bundle_above_thirty_two_mebibytes_is_refused() {
    let directory = scratch("oversized-bundle");
    fixture_bundle(&directory);
    let path = directory.join("0123456789abcdef/test_bg.wasm");
    fs::File::create(&path).expect("module").set_len(FULL_BUNDLE_MODULE_BYTES).expect("full bundle module");
    let error = super::bundle::read(&directory).expect_err("oversized bundle refused");
    fs::remove_dir_all(directory).expect("scratch cleanup");
    assert!(error.contains("bundle exceeds its limits"), "{error}");
}

#[test]
fn a_worker_reads_program_resources_from_the_agent_configuration() {
    use std::os::unix::ffi::OsStrExt;
    let directory = scratch("program-agent");
    let parsed = super::agent(&agent_document(&directory, true), &mut StartupReads::Native).expect("configured agent");
    let sources = parsed.sources.expect("configured Claude Code");
    assert_eq!(sources.executable.as_ref(), directory.join("private-claude").as_os_str().as_bytes());
    assert_eq!(sources.directory.as_ref(), directory.join("private-programs").as_os_str().as_bytes());
    assert_eq!(sources.environment.as_ref(), &[Box::<[u8]>::from(b"JIG_SOURCE_CHECK=configured".as_slice())]);
    assert!(
        jig_test_service::program_sources::from_paths(
            sources.directory,
            sources.executable,
            sources.environment,
            1000,
            None
        )
        .is_some()
    );
    fs::remove_dir_all(directory).expect("scratch cleanup");
}
#[test]
fn an_agent_without_a_program_supplies_no_program_resources() {
    let directory = scratch("no-program-agent");
    let parsed = super::agent(&agent_document(&directory, false), &mut StartupReads::Native).expect("configured agent");
    assert!(parsed.sources.is_none());
    fs::remove_dir_all(directory).expect("scratch cleanup");
}

#[test]
fn advertised_program_endpoints_follow_actual_endpoint_selections() {
    for (program_endpoint, expected) in [(false, &[][..]), (true, &[0][..])] {
        let directory = scratch("endpoint-evidence");
        let parsed =
            super::agent(&agent_endpoint_document(&directory, true, program_endpoint), &mut StartupReads::Native)
                .expect("parsed endpoint");
        assert_eq!(parsed.agent.bounds.program_endpoints.as_ref(), expected);
        fs::remove_dir_all(directory).expect("scratch cleanup");
    }
}

#[test]
fn debug_of_an_agent_start_shows_the_environments_count_never_its_entries() {
    const CANARY: &[u8] = b"canary-7f3a9c";
    let sources = super::SourceSettings {
        directory: Box::from(b"/private/programs".as_slice()),
        executable: Box::from(b"/private/claude".as_slice()),
        environment: Box::new([Box::from(b"API_KEY=canary-7f3a9c".as_slice())]),
    };
    let expected = format!(
        "SourceSettings {{ directory: {:?}, executable: {:?}, environment_entries: 1 }}",
        sources.directory, sources.executable
    );
    let sources_debug = format!("{sources:?}");
    let directory = tempfile::tempdir().expect("agent configuration scratch");
    let mut start =
        super::agent(&agent_document(directory.path(), false), &mut StartupReads::Native).expect("configured agent");
    start.sources = Some(sources);
    let start_debug = format!("{start:?}");
    let canary_debug = format!("{CANARY:?}");
    for debug in [&sources_debug, &start_debug] {
        assert!(skein_lib::bytes::find(debug.as_bytes(), CANARY).is_none());
        assert!(!debug.contains(canary_debug.trim_matches(['[', ']'])));
        assert!(debug.contains("environment_entries: 1"));
    }
    assert_eq!(sources_debug, expected);
}
