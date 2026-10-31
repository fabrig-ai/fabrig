use super::*;
use crate::settings::{Origin, Sourced};
pub(crate) use skein_scratch::Scratch as Files;
use std::collections::BTreeMap;

fn wall() -> Wall {
    Wall::from_nanos(100_000_000_000)
}
fn resolved(files: &Files) -> settings::resolve::Resolved {
    let mut values = BTreeMap::new();
    for (name, source) in [("codex", "none"), ("unused", "env")] {
        for (field, value) in [("source", source), ("variable", "SMITH_UNSET_FIXTURE")] {
            values.insert(
                format!("accounts.{name}.{field}"),
                Sourced { value: toml::Value::String(value.into()), origin: Origin::BuiltIn },
            );
        }
    }
    values.insert("environment.inherit".into(), Sourced { value: toml::Value::Array(vec![]), origin: Origin::BuiltIn });
    let fields = values
        .iter()
        .filter_map(|(key, value)| key.strip_prefix("accounts.codex.").map(|field| (field.into(), value.clone())))
        .collect();
    let mut resolved = crate::fixture::resolved(files.path().to_path_buf());
    resolved.settings.values = values;
    resolved.accounts = vec![Account::from_fields("codex".into(), 0, fields)];
    resolved
}

#[test]
fn the_credential_catalog_uses_the_resolvers_needed_accounts_and_omits_values() {
    let files = Files::new("smith-credentials");
    let resolved = resolved(&files);
    let accounts = report(&resolved, wall(), &ProgramStates::new());
    assert!(accounts["codex"].required && accounts["codex"].credential.usable(), "the selected account is usable");
    assert!(
        !accounts["unused"].required && !accounts["unused"].credential.usable(),
        "the unset unused account is only a warning"
    );
    let json = serde_json::to_string(&accounts).expect("redacted accounts");
    assert!(!json.contains("signature") && !json.contains("UNISSUED"), "no login value is a report field");
}

#[test]
fn an_env_value_is_access_only_and_an_invalid_header_value_is_refused() {
    let record = env_record(b"UNISSUED_ACCESS", false, wall()).expect("synthetic API value");
    assert!(record.record.access_token.as_ref() == b"UNISSUED_ACCESS", "the variable is the access value");
    assert!(
        record.record.refresh_token.is_none() && record.account_id.is_empty(),
        "an API variable holds no refresh or Codex identity"
    );
    assert!(env_record(b"UNISSUED\nHEADER", false, wall()).is_none(), "a variable cannot inject a header");
    assert!(env_record(b"UNISSUED_ACCESS", true, wall()).is_none(), "Codex's variable must supply its account claim");
}

pub(crate) fn resolved_source(files: &Files, source: &str) -> settings::resolve::Resolved {
    let mut resolved = resolved(files);
    let value = Sourced { value: toml::Value::String(source.into()), origin: Origin::BuiltIn };
    let mut fields = resolved.accounts[0].fields.clone();
    fields.insert("source".into(), value.clone());
    resolved.accounts[0] = Account::from_fields("codex".into(), 0, fields);
    resolved.settings.values.insert("accounts.codex.source".into(), value);
    resolved.settings.values.insert(
        "credentials".into(),
        Sourced { value: toml::Value::String(files.path().display().to_string()), origin: Origin::BuiltIn },
    );
    resolved
}

pub(crate) fn private_key(files: &Files) {
    use std::os::unix::fs::PermissionsExt;
    let key = files.path().join("keys/codex");
    std::fs::create_dir_all(key.parent().expect("keys parent")).expect("keys directory");
    std::fs::write(&key, b"key-for-work").expect("scratch key");
    std::fs::set_permissions(key, std::fs::Permissions::from_mode(0o600)).expect("private key");
}

#[test]
fn preparing_refuses_a_key_account_without_a_key() {
    let files = Files::new("smith-credentials");
    for interactive in [false, true] {
        let refusal = prepare(&resolved_source(&files, "key"), wall(), interactive, &ProgramStates::new())
            .err()
            .expect("missing key");
        assert_eq!(refusal.exit_code(), 3, "credentials refusal");
        assert!(refusal.line().contains("codex") && refusal.line().contains("smith login"), "account and next action");
    }
}

fn no_source(resolved: &settings::resolve::Resolved) {
    let prepared = prepare(resolved, wall(), false, &ProgramStates::new()).expect("usable source");
    assert!(prepared.sources.is_empty(), "the shell owns this source");
    assert_eq!(prepared.locations.len(), 1, "one selected location");
    assert_eq!(prepared.locations.iter().next().expect("location").name, "codex", "selected account");
}

#[test]
fn preparing_a_key_account_hands_no_source() {
    let files = Files::new("smith-credentials");
    private_key(&files);
    no_source(&resolved_source(&files, "key"));
}

#[test]
fn preparing_a_program_account_hands_no_source() {
    let files = Files::new("smith-credentials");
    let programs = BTreeMap::from([("codex".into(), Credential::Usable { until: None })]);
    let prepared = prepare(&resolved_source(&files, "program"), wall(), false, &programs).expect("usable program");
    assert!(prepared.sources.is_empty(), "the shell lends the program name");
}

#[test]
fn preparing_a_none_account_hands_no_source() {
    let files = Files::new("smith-credentials");
    no_source(&resolved_source(&files, "none"));
}

#[test]
fn file_sources_require_a_credentials_directory_before_inspection() {
    let files = Files::new("smith-credentials");
    for name in ["key", "sign_in"] {
        let resolved = resolved_source(&files, name);
        assert!(matches!(source(&resolved.accounts[0], false, None), Err(Credential::NotSignedIn)));
    }
}
