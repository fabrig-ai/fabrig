use super::*;
use crate::credentials::tests::Files;
use std::path::Path;

fn configured(files: &Files, source_name: &str, program: bool) -> (super::super::Settings, std::path::PathBuf) {
    let executable = crate::fixture::program(files);
    let mut settings = configured_in(files.path(), &crate::fixture::trust(), source_name, program, Some(&executable));
    settings.paths.path = std::env::var_os("PATH");
    (settings, executable)
}

pub(super) fn configured_in(
    path: &Path,
    certificate: &Path,
    source_name: &str,
    program: bool,
    executable: Option<&Path>,
) -> super::super::Settings {
    let program = if program { "program='claude_code'" } else { "" };
    let program_setting =
        executable.map_or_else(String::new, |path| format!("[programs]\nclaude_code='{}'\n", path.display()));
    let declaration = source(
        Layer::User,
        &format!(
            "model='openai/gpt-6-luna'\ncredentials='{}'\n{program_setting}[accounts.openai]\nprovider='openai'\nsource='{source_name}'\n[endpoints.openai]\naddress='127.0.0.1:443'\ntrust.system=false\ntrust.der=['{}']\n{program}\n",
            path.display(),
            certificate.display()
        ),
    );
    crate::settings::from_sources(&crate::command::Shared::default(), "/workspace".into(), &[declaration])
        .expect("settings")
}

fn resolved(settings: &super::super::Settings) -> resolve::Resolved {
    crate::fixture::ready(resolve::resolve_startup(settings)).0
}

#[test]
fn a_none_accounts_models_name_no_account_in_the_charter() {
    let files = Files::new("smith-credentials");
    let resolved = resolved(&configured(&files, "none", false).0);
    assert!(resolved.local.models.iter().all(|model| model.account.is_none()), "no charter account");
    assert!(resolved.local.accounts.is_empty(), "no configured grant account");
    assert_eq!(resolved.accounts[0].number, 2_100_401_399, "none retains its number");
}

#[test]
fn a_key_accounts_models_name_its_number() {
    let files = Files::new("smith-credentials");
    let resolved = resolved(&configured(&files, "key", false).0);
    assert!(resolved.local.models.iter().all(|model| model.account == Some(2_100_401_399)), "stable charter account");
    assert_eq!(resolved.local.accounts.as_ref(), [2_100_401_399], "configured grant account");
}

#[test]
fn the_generated_document_names_the_program_directory() {
    let files = Files::new("smith-credentials");
    let (settings, executable) = configured(&files, "program", true);
    let (_, bytes) = crate::fixture::ready(resolve::resolve_startup(&settings));
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("agent document");
    assert_eq!(
        document["programs"]["claude_code"]["path"],
        executable.to_str().expect("UTF-8"),
        "configured executable"
    );
    assert!(
        document["programs"]["claude_code"]["directory"] == files.path().join("programs").to_str().expect("UTF-8"),
        "programs directory"
    );
    assert_eq!(document["endpoints"][0]["program"], "claude_code", "endpoint program");
    let text = String::from_utf8(bytes.into_vec()).expect("UTF-8 document");
    for secret in ["\"account\"", "key-for-work"] {
        assert!(!text.contains(secret), "no credential in configuration");
    }
    assert!(!files.path().join("programs").exists(), "resolution creates no program directory");
}

#[test]
fn a_document_without_a_program_endpoint_holds_no_programs() {
    let files = Files::new("smith-credentials");
    let (_, bytes) = crate::fixture::ready(resolve::resolve_startup(&configured(&files, "key", false).0));
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("agent document");
    assert!(document.get("programs").is_none(), "unused program settings omitted");
}

#[test]
fn locating_a_program_searches_the_given_path() {
    let path = Path::new("/fixture/bin");
    let executable = path.join("claude");
    assert_eq!(
        resolve::programs::locate_with(None, Some(path.as_os_str()), |path| {
            (path == executable).then_some(resolve::programs::FileFacts { is_file: true, mode: 0o700 })
        }),
        Ok(executable),
        "given PATH"
    );
}

#[test]
fn a_missing_or_non_executable_program_is_refused() {
    use resolve::programs::{FileFacts, locate_with};

    let directory = Path::new("/fixture/bin");
    let candidate = directory.join("claude");
    for (path, facts, why) in [
        (Some(directory.as_os_str()), None, "absent executable"),
        (Some(directory.as_os_str()), Some(FileFacts { is_file: true, mode: 0o600 }), "non-executable"),
        (None, None, "absent PATH"),
    ] {
        let mut checked = Vec::new();
        let result = locate_with(None, path, |file| {
            checked.push(file.to_path_buf());
            assert_eq!(file, candidate, "only claude on the supplied PATH can be checked");
            facts
        });
        assert_eq!(result, Err(()), "{why}");
        let expected = path.map_or(&[][..], |_| std::slice::from_ref(&candidate));
        assert_eq!(checked, expected, "lookup follows the supplied PATH only");
    }
}

#[test]
fn a_configured_program_path_must_be_absolute() {
    assert!(
        resolve::programs::locate(Some("claude"), Some(std::ffi::OsStr::new("/bin"))).is_err(),
        "absolute path required"
    );
}
