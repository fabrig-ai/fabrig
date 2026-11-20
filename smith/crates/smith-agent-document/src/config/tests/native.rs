use super::shaped_document;
use crate::config::{ConfigurationRefusal, from_value, profile_from_value};
use smith_agent_profile::profile::{Check, Layer, LimitName, Refusal, Value};

#[cfg(unix)]
fn executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    // An invalid executable is enough: finding must never run it.
    std::fs::write(path, b"not an executable format\n").expect("rg stand-in");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
}

#[cfg(unix)]
#[test]
fn rg_finding_takes_the_first_executable_file_on_path() {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let scratch = skein_scratch::Scratch::new("smith-rg-path");
    for directory in ["A", "B", "C"] {
        std::fs::create_dir(scratch.path().join(directory)).expect("PATH directory");
        executable(&scratch.path().join(directory).join("rg"));
    }
    std::fs::set_permissions(scratch.path().join("A/rg"), std::fs::Permissions::from_mode(0o600))
        .expect("non-executable mode");
    let path = std::env::join_paths([
        std::path::PathBuf::from("rel"),
        scratch.path().join("missing"),
        scratch.path().join("A"),
        scratch.path().join("B"),
        scratch.path().join("C"),
    ])
    .expect("PATH");
    assert_eq!(
        crate::config::find_rg(Some(path.as_bytes())),
        Some(scratch.path().join("B/rg")),
        "first executable file"
    );
}

#[cfg(unix)]
#[test]
fn rg_finding_skips_relative_directories_and_a_directory_named_rg() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("smith-rg-relative");
    std::fs::create_dir(scratch.path().join("rel")).expect("relative PATH directory");
    executable(&scratch.path().join("rel/rg"));
    std::fs::create_dir_all(scratch.path().join("directory/rg")).expect("directory named rg");
    let up: std::path::PathBuf =
        std::env::current_dir().expect("working directory").components().skip(1).map(|_| "..").collect();
    let relative = up.join(scratch.path().strip_prefix("/").expect("absolute scratch")).join("rel");
    assert!(relative.join("rg").is_file(), "relative entry holds the executable fixture");
    let path = std::env::join_paths([relative, scratch.path().join("directory")]).expect("PATH");
    assert_eq!(
        crate::config::find_rg(Some(path.as_bytes())),
        None,
        "neither relative entries nor directories are programs"
    );
}

#[cfg(unix)]
#[test]
fn rg_finding_has_no_result_for_empty_or_absent_path() {
    assert_eq!(crate::config::find_rg(Some(b"")), None, "empty PATH");
    assert_eq!(crate::config::find_rg(None), None, "absent PATH");
}

#[cfg(unix)]
#[test]
fn resolving_finds_rg_after_decoding_without_running_it() {
    let scratch = skein_scratch::Scratch::new("smith-rg-startup");
    let mut document = shaped_document();
    document["environment"] = serde_json::json!([{"name":"PATH", "value":scratch.path()}]);
    let decoded = crate::config::decode(&serde_json::to_vec(&document).expect("document JSON"))
        .expect("decode without filesystem effects");
    let rg = scratch.path().join("rg");
    executable(&rg);
    let configuration = decoded.resolve(&mut Vec::new()).expect("native resolution");
    assert_eq!(configuration.service.rg.as_ref(), rg.as_os_str().as_encoded_bytes(), "startup's rg");
}

#[cfg(unix)]
#[test]
fn resolving_without_rg_on_path_uses_the_system_fallback() {
    let scratch = skein_scratch::Scratch::new("smith-rg-fallback");
    let absent = shaped_document();
    let mut missing = shaped_document();
    missing["environment"] = serde_json::json!([{"name":"PATH", "value":scratch.path()}]);
    for document in [absent, missing] {
        let configuration = from_value(document).expect("fallback configuration");
        assert_eq!(configuration.service.rg.as_ref(), b"/usr/bin/rg", "compatible fallback");
    }
}

#[cfg(unix)]
#[test]
fn reading_bytes_keeps_file_content_and_absent_file_refusal() {
    let scratch = skein_scratch::Scratch::new("smith-config-bytes");
    let path = scratch.path().join("agent.json");
    let bytes = b"unparsed configuration\0\xff";
    std::fs::write(&path, bytes).expect("configuration fixture");
    assert_eq!(crate::config::read_bytes(&path).expect("file bytes").as_ref(), bytes);
    let absent = scratch.path().join("absent.json");
    let refusal = format!("configuration metadata: {}: file absent", absent.display());
    assert_eq!(crate::config::read_bytes(&absent).expect_err("absent file"), refusal);
    assert_eq!(crate::config::read_decoded(&absent).err().as_deref(), Some(refusal.as_str()));
}

#[test]
fn generated_values_build_the_native_agent_without_a_file() {
    let configuration = from_value(shaped_document()).expect("native configuration");
    assert_eq!(configuration.profile.declared.tool_payload, 256);
    assert_eq!(configuration.declarations.endpoints[0].limits.window, 65_536);
}

#[test]
fn a_native_value_keeps_both_sides_of_a_profile_refusal() {
    let mut document = shaped_document();
    document["profile"]["declared"]["read_window"] = serde_json::json!(257);
    let refusal = from_value(document).err().expect("read exceeds payload");
    match refusal {
        ConfigurationRefusal::Profile(Refusal::Broken { check, left, right }) => {
            assert_eq!(check, Check::ReadInPayload);
            assert_eq!(left.name, LimitName::deployment(Layer::Declared, "read_window"));
            assert_eq!(left.value, Value::Bytes(257));
            assert_eq!(right.name, LimitName::deployment(Layer::Declared, "tool_payload"));
            assert_eq!(right.value, Value::Bytes(256));
        }
        other @ (ConfigurationRefusal::Input(_) | ConfigurationRefusal::Profile(_)) => {
            panic!("expected typed relationship, got {other:?}");
        }
    }
}

#[test]
fn a_native_value_keeps_a_missing_list_as_a_typed_field() {
    let mut document = shaped_document();
    document["endpoints"] = serde_json::json!([]);
    let refusal = from_value(document).err().expect("missing endpoints");
    assert!(
        matches!(refusal, ConfigurationRefusal::Profile(Refusal::Field { limit })
        if limit == LimitName::deployment(Layer::Declared, "endpoints")),
        "typed missing field"
    );
}

#[test]
fn profile_values_take_check_and_wrap_up_durations() {
    let profile = profile_from_value(serde_json::json!({
        "name":"standard", "policy":{"check_timeout":1234,"wrap_up":5678}
    }))
    .expect("profile values");
    assert_eq!(profile.policy.check_timeout.as_nanos(), 1_234_000_000);
    assert_eq!(profile.policy.wrap_up.as_nanos(), 5_678_000_000);
}

#[test]
fn trust_arrays_accept_der_and_pem_with_system_roots_disabled() {
    use skein_lib::{
        Writer,
        base64::{self, Alphabet},
    };
    let der = include_bytes!("../../../../../tests/protocol-llm/fixtures/root.der");
    let encoded = base64::encoded_len(Alphabet::Standard, u32::try_from(der.len()).expect("root bytes"))
        .expect("encoded root length");
    let mut writer = Writer::new(usize::try_from(encoded).expect("encoded root fits"));
    base64::encode(Alphabet::Standard, der, &mut writer).expect("root encoding");
    let pem = format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        String::from_utf8(writer.finish().into_vec()).expect("base64 text")
    );
    let scratch = skein_scratch::Scratch::new("smith-native-trust");
    let path = scratch.path().join("roots.pem");
    std::fs::write(&path, pem).expect("PEM fixture");
    let mut document = shaped_document();
    document["endpoints"][0]["transport"] = serde_json::json!("tls");
    document["endpoints"][0]["server_name"] = serde_json::json!("example.test");
    document["endpoints"][0]["trust"] = serde_json::json!({"system":false,
        "der":[skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der")],
        "pem":[path]});
    let built = from_value(document);
    assert!(built.is_ok(), "mixed explicit trust roots accepted");
}

#[test]
fn trust_arrays_with_system_off_and_no_roots_are_refused() {
    let mut document = shaped_document();
    document["endpoints"][0]["transport"] = serde_json::json!("tls");
    document["endpoints"][0]["server_name"] = serde_json::json!("example.test");
    document["endpoints"][0]["trust"] = serde_json::json!({"system":false,"der":[],"pem":[]});
    let refusal = from_value(document).err().expect("no trusted roots").detail();
    assert_eq!(refusal, "TLS trust : Roots");
}

#[test]
fn trust_arrays_keep_one_combined_certificate_bound() {
    let scratch = skein_scratch::Scratch::new("smith-combined-trust");
    let path = scratch.path().join("roots.pem");
    std::fs::write(&path, b"-----BEGIN CERTIFICATE-----\nAA==\n-----END CERTIFICATE-----\n".repeat(256))
        .expect("bounded PEM fixture");
    let mut document = shaped_document();
    document["endpoints"][0]["transport"] = serde_json::json!("tls");
    document["endpoints"][0]["server_name"] = serde_json::json!("example.test");
    document["endpoints"][0]["trust"] = serde_json::json!({"system":false,
        "der":[skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der")],
        "pem":[path,path]});
    let built = from_value(document);
    assert_eq!(built.err().expect("combined root bound").detail(), "trust roots exceed their combined bound");
}

#[test]
fn decoding_keeps_unresolved_names_and_unread_trust_sources() {
    let scratch = skein_scratch::Scratch::new("smith-decode");
    let mut document = shaped_document();
    document["endpoints"][0]["address"] = serde_json::json!("unresolved.invalid:443");
    document["endpoints"][0]["transport"] = serde_json::json!("tls");
    document["endpoints"][0]["server_name"] = serde_json::json!("example.test");
    document["endpoints"][0]["trust_der"] = serde_json::json!(scratch.path().join("absent.der"));
    let bytes = serde_json::to_vec(&document).expect("configuration JSON");
    super::decode(&bytes).expect("decode neither resolves the name nor opens the absent trust source");
}

#[test]
fn resolving_says_nothing_of_well_formed_roots() {
    let mut document = shaped_document();
    document["endpoints"][0]["transport"] = serde_json::json!("tls");
    document["endpoints"][0]["server_name"] = serde_json::json!("example.test");
    document["endpoints"][0]["trust_der"] =
        serde_json::json!(skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der"));
    let decoded = super::decode(&serde_json::to_vec(&document).expect("configuration JSON")).expect("pure decode");
    let mut errors = Vec::new();
    decoded.resolve(&mut errors).expect("native configuration");
    assert!(errors.is_empty(), "well-formed roots produce no diagnostic: {errors:?}");
}

fn resolve_with_pem(pem: &[u8]) -> Vec<u8> {
    let scratch = skein_scratch::Scratch::new("smith-report-trust");
    let path = scratch.path().join("roots.pem");
    std::fs::write(&path, pem).expect("PEM fixture");
    let mut document = shaped_document();
    document["endpoints"][0]["transport"] = serde_json::json!("tls");
    document["endpoints"][0]["server_name"] = serde_json::json!("example.test");
    document["endpoints"][0]["trust"] = serde_json::json!({"system":false,
        "der":[skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der")],
        "pem":[path]});
    let decoded = super::decode(&serde_json::to_vec(&document).expect("configuration JSON")).expect("pure decode");
    let mut errors = Vec::new();
    decoded.resolve(&mut errors).expect("native configuration");
    errors
}

#[test]
fn resolving_says_nothing_of_skipped_roots_and_other_blocks() {
    let errors = resolve_with_pem(
        b"-----BEGIN CERTIFICATE-----\nAA==\n-----END CERTIFICATE-----\n-----BEGIN X509 CRL-----\nignored\n-----END X509 CRL-----\n",
    );
    assert!(errors.is_empty(), "skipped roots and other blocks produce no diagnostic: {errors:?}");
}

#[test]
fn a_malformed_block_is_reported_once_to_the_supplied_writer() {
    let errors = resolve_with_pem(b"-----BEGIN CERTIFICATE-----\n!\n-----END CERTIFICATE-----\n");
    assert_eq!(errors, b"smith: trust roots: 1 taken, 0 skipped, 1 malformed blocks, 0 other blocks\n");
}

#[test]
fn decoding_rejects_an_invalid_later_endpoint_before_loading_trust() {
    let scratch = skein_scratch::Scratch::new("smith-decode-refusal");
    let mut document = shaped_document();
    let mut invalid = document["endpoints"][0].clone();
    invalid["name"] = serde_json::json!("invalid");
    invalid["number"] = serde_json::json!(1);
    invalid["headers"] = serde_json::json!([{ "name": "session-id", "value": "override" }]);
    document["endpoints"].as_array_mut().expect("endpoints").push(invalid);
    document["endpoints"][0]["transport"] = serde_json::json!("tls");
    document["endpoints"][0]["server_name"] = serde_json::json!("example.test");
    document["endpoints"][0]["trust_der"] = serde_json::json!(scratch.path().join("absent.der"));
    let refusal = super::decode(&serde_json::to_vec(&document).expect("configuration JSON")).err();
    assert_eq!(refusal.as_deref(), Some("invalid endpoint HTTP header"));
}
