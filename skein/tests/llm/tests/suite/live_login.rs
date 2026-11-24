//! Lent credentials, with the named path and opened entry supplied as facts.

use std::cell::Cell;
use std::io;
use std::path::Path;

use skein_llm_world::live_login::Login;

/// Synthetic: a Codex `auth.json`'s fields, its values fixtures.
const CODEX_AUTH: &[u8] = br#"{"OPENAI_API_KEY":null,"tokens":{"id_token":"fixture-id","access_token":"fixture-responses-access","refresh_token":"fixture-responses-refresh"},"last_refresh":"2026-11-01T00:00:00Z"}"#;
/// Synthetic: the login `codex login --with-api-key` writes, its key a fixture.
const CODEX_API_KEY: &[u8] = br#"{"OPENAI_API_KEY":"fixture-api-key","auth_mode":"apikey"}"#;
const MALFORMED: &[u8] = br#"{"OPENAI_API_KEY":"fixture-secret-value","#;

const DIR: &str = "lent";
const AUTH: &str = "lent/auth.json";
const NAMED_LOGIN: &str = "lent/codex-login.json";
const NAMED_KEY: &str = "lent/key";
/// The file size bound in skein/docs/design/llm/README.md, 5.
const BOUND: usize = 64 * 1024;
const PRIVATE: u32 = 0o600;
const PUBLIC: u32 = 0o644;
const SETUID_PRIVATE: u32 = 0o4600;

fn admitted(login: Login, path: &Path, contents: &[u8]) -> Result<Box<[u8]>, String> {
    login.admit(path, true, PRIVATE, |_| Ok(contents.to_vec()))
}

fn assert_refusal(result: Result<Box<[u8]>, String>, path: &str, expected: &str) {
    let refusal = result.expect_err("refuse the lent credential");
    assert_eq!(refusal, format!("{path}: {expected}"));
    for value in [
        "fixture-api-key",
        "apikey",
        "fixture-id",
        "fixture-responses-access",
        "fixture-responses-refresh",
        "2026-11-01T00:00:00Z",
        "fixture-token",
        "fixture-secret-value",
        "fixture-key",
    ] {
        assert!(!refusal.contains(value), "a refusal discloses a fixture value");
    }
}

fn padded_login(size: usize) -> Vec<u8> {
    let mut text = CODEX_API_KEY.to_vec();
    text.resize(size, b' ');
    text
}

#[test]
fn a_login_is_read_from_its_file_or_the_directory_holding_it() {
    let at_bound = padded_login(BOUND);
    let rows: [(_, _, _, &[u8], &[u8]); 9] = [
        (NAMED_LOGIN, false, NAMED_LOGIN, CODEX_API_KEY, b"fixture-api-key"),
        (DIR, true, AUTH, CODEX_API_KEY, b"fixture-api-key"),
        (
            NAMED_LOGIN,
            false,
            NAMED_LOGIN,
            br#"{"OPENAI_API_KEY":"fixture-api-key","tokens":{"access_token":"fixture-token"}}"#,
            b"fixture-api-key",
        ),
        (DIR, true, AUTH, br#"{"OPENAI_API_KEY":"fixture-api-key","tokens":null}"#, b"fixture-api-key"),
        (NAMED_LOGIN, false, NAMED_LOGIN, CODEX_AUTH, b"fixture-responses-access"),
        (DIR, true, AUTH, CODEX_AUTH, b"fixture-responses-access"),
        (NAMED_LOGIN, false, NAMED_LOGIN, br#"{"tokens":{"access_token":"fixture-token"}}"#, b"fixture-token"),
        (DIR, true, AUTH, br#"{"OPENAI_API_KEY":42,"tokens":{"access_token":"fixture-token"}}"#, b"fixture-token"),
        (NAMED_LOGIN, false, NAMED_LOGIN, &at_bound, b"fixture-api-key"),
    ];
    for (named, directory, expected_path, contents, expected) in rows {
        let path = Login::Codex.file(Path::new(named), directory);
        assert_eq!(path, Path::new(expected_path));
        assert_eq!(admitted(Login::Codex, &path, contents).expect("a lent login").as_ref(), expected);
    }
}

#[test]
fn a_login_lacking_a_field_is_refused_naming_only_the_field() {
    let no_token = "no non-empty string at .tokens.access_token";
    let rows: [(_, _, _, &[u8], _); 4] = [
        (DIR, true, AUTH, br#"{"tokens":{}}"#, no_token),
        (DIR, true, AUTH, br#"{"tokens":{"access_token":null}}"#, no_token),
        (DIR, true, AUTH, br#"{"tokens":{"access_token":""}}"#, no_token),
        (
            NAMED_LOGIN,
            false,
            NAMED_LOGIN,
            br#"{"OPENAI_API_KEY":"","tokens":{"access_token":"fixture-token"}}"#,
            "no non-empty string at .OPENAI_API_KEY",
        ),
    ];
    for (named, directory, expected_path, contents, expected) in rows {
        let path = Login::Codex.file(Path::new(named), directory);
        assert_refusal(admitted(Login::Codex, &path, contents), expected_path, expected);
    }
}

#[test]
fn non_private_or_non_regular_entries_are_refused_before_reading() {
    let login_mode = "login file must have mode 0600";
    for (login, named, directory, expected_path, regular, mode, contents, expected) in [
        (Login::Codex, NAMED_LOGIN, false, NAMED_LOGIN, true, PUBLIC, CODEX_API_KEY, login_mode),
        (Login::Codex, DIR, true, AUTH, true, PUBLIC, MALFORMED, login_mode),
        (Login::Codex, DIR, true, AUTH, false, PRIVATE, b"".as_slice(), login_mode),
        (Login::Codex, DIR, true, AUTH, true, SETUID_PRIVATE, CODEX_API_KEY, login_mode),
        (
            Login::Anthropic,
            NAMED_KEY,
            false,
            NAMED_KEY,
            true,
            PUBLIC,
            b"fixture-key".as_slice(),
            "key file must have mode 0600",
        ),
    ] {
        let path = login.file(Path::new(named), directory);
        let read = Cell::new(false);
        let result = login.admit(&path, regular, mode, |_| {
            read.set(true);
            Ok(contents.to_vec())
        });
        assert!(!read.get(), "an ineligible entry must not be read");
        assert_refusal(result, expected_path, expected);
    }
}

#[test]
fn a_login_one_byte_over_its_bound_is_refused() {
    let path = Login::Codex.file(Path::new(NAMED_LOGIN), false);
    let result = Login::Codex.admit(&path, true, PRIVATE, |cap| {
        assert_eq!(cap, u64::try_from(BOUND + 1).expect("bound fits u64"));
        let marker = b"fixture-secret-value";
        let mut text = padded_login(BOUND + 1 - marker.len());
        text.extend_from_slice(marker);
        Ok(text)
    });
    assert_refusal(result, NAMED_LOGIN, "login file is too large");
}

#[test]
fn a_login_refuses_malformed_json_without_disclosing_values() {
    let path = Login::Codex.file(Path::new(DIR), true);
    assert_refusal(admitted(Login::Codex, &path, MALFORMED), AUTH, "not a login's JSON: Tokenizer(Truncated)");
}

#[test]
fn an_anthropic_key_trims_its_trailing_newline() {
    let path = Login::Anthropic.file(Path::new(NAMED_KEY), false);
    assert_eq!(admitted(Login::Anthropic, &path, b"fixture-key\n").expect("a lent key").as_ref(), b"fixture-key");
}

#[test]
fn an_anthropic_key_refuses_whitespace_only() {
    let path = Login::Anthropic.file(Path::new(NAMED_KEY), false);
    assert_refusal(admitted(Login::Anthropic, &path, b" \n"), NAMED_KEY, "key file is empty");
}

#[test]
fn a_failed_read_names_its_path_and_error() {
    for (login, named, directory, expected_path) in
        [(Login::Codex, DIR, true, AUTH), (Login::Anthropic, NAMED_KEY, false, NAMED_KEY)]
    {
        let path = login.file(Path::new(named), directory);
        let result = login.admit(&path, true, PRIVATE, |_| Err(io::Error::other("fixture read failed")));
        assert_refusal(result, expected_path, "fixture read failed");
    }
}
