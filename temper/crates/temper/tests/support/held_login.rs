//! The held login returns to its path on every ending of a live story that imports it.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

fn quote(path: &Path) -> String {
    format!("'{}'", path.to_str().expect("UTF-8 path").replace('\'', "'\"'\"'"))
}

/// Installed before import; used normally, by Lent and by the launcher's EXIT trap.
/// An interrupted import is covered even if its reply never reached the driver.
pub fn return_hook(binary: &Path, store: &Path, client: &Path, account: &Path) {
    let hook = store.join("give-back");
    let body = format!(
        r#"#!/bin/sh
set -eu
exec 9>{lock}
flock -x 9
: > {returning}
if [ ! -e {account} ]; then
  rung_account=$({binary} ctl {client} account list --json | jq -er '.accounts.party | map(select(.provider == 1 and .plan == true)) | if length == 1 then .[0].number else error("expected one held ChatGPT plan") end')
  {binary} ctl {client} account export "$rung_account" {account}
fi
[ "$(LC_ALL=C stat -c %F -- {account})" = 'regular file' ]
[ "$(stat -c %a -- {account})" = 600 ]
[ "$(stat -c %s -- {account})" -gt 0 ]
stat -c 'LOGIN BACK: %F mode=%a bytes=%s owner=%u' -- {account}
"#,
        account = quote(account),
        binary = quote(binary),
        client = quote(client),
        lock = quote(&store.join("login.lock")),
        returning = quote(&store.join("returning"))
    );
    super::write_private(&hook, body.as_bytes());
    fs::set_permissions(hook, fs::Permissions::from_mode(0o700)).expect("executable return hook");
}

/// The import and return share exclusion, including a return after an interrupted reply.
#[must_use]
pub fn import_guard(binary: &Path, store: &Path, client: &Path, arguments: &[&str]) -> super::ctl::Call {
    let wrapper = store.join("import-held");
    let body = format!(
        r#"#!/bin/sh
set -eu
exec 9>{lock}
flock -x 9
test ! -e {returning} || exit 1
exec {binary} ctl {client} "$@" </dev/null
"#,
        lock = quote(&store.join("login.lock")),
        returning = quote(&store.join("returning")),
        binary = quote(binary),
        client = quote(client),
    );
    super::write_private(&wrapper, body.as_bytes());
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).expect("executable import guard");
    super::ctl::Call::command(&wrapper, arguments)
}

/// A successful first run retains the account for the restart proof.
#[must_use]
pub const fn due_back(first: bool, failed: bool) -> bool {
    !first || failed
}

/// A held record lent to a story's store, from before its import to the test's end
/// (temper/docs/design/domain/testing.md, 2.7). A record at its path owes nothing, so the
/// story's own return disarms this, and an import that never ran leaves it nothing to do.
pub struct Lent {
    binary: PathBuf,
    paths: [PathBuf; 3],
    account: PathBuf,
}

impl Lent {
    #[must_use]
    pub fn new(binary: &Path, paths: &[PathBuf; 3], account: &Path) -> Self {
        assert!(paths[0].parent().is_some(), "deployment store");
        Self { binary: binary.into(), paths: paths.clone(), account: account.into() }
    }

    fn settle(&self) -> Result<(), String> {
        if fs::symlink_metadata(&self.account).is_ok() {
            return at_rest(&self.account);
        }
        let store = self.paths[0].parent().expect("deployment store");
        match super::forge_land::rung::return_through_hook(&self.binary, store) {
            Ok(()) => Ok(()),
            Err(hook) => {
                if fs::symlink_metadata(&self.account).is_ok() {
                    return at_rest(&self.account);
                }
                super::forge_land::rung::return_on_a_new_box(&self.binary, store, &self.paths, &self.account)
                    .map_err(|boxed| format!("{hook}; {boxed}"))
            }
        }
    }
}

/// What a record that did not come back leaves in the notices.
#[must_use]
pub fn not_back(message: &str, store: &Path) -> String {
    format!("LOGIN NOT BACK: {message}; store {} retained for recovery", store.display())
}

/// A record's path by stat alone: a regular file, 0600, not empty, as give-back checks it.
fn at_rest(account: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(account).map_err(|error| format!("returned record metadata: {error}"))?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o7777 != 0o600 || metadata.len() == 0 {
        return Err("returned record must be a regular file, mode 0600, not empty".into());
    }
    Ok(())
}

impl Drop for Lent {
    #[expect(clippy::manual_assert, reason = "The guard explicitly preserves an unwinding story's panic.")]
    fn drop(&mut self) {
        let settled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.settle()))
            .unwrap_or_else(|_| Err("the return panicked".to_owned()));
        if let Err(message) = settled {
            let message = not_back(&message, self.paths[0].parent().expect("deployment store"));
            super::live::notice(&message);
            if !std::thread::panicking() {
                panic!("{message}");
            }
        }
    }
}
