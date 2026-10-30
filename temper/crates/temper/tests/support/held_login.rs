//! The held login returns to its path on every ending of a live story that imports it.
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

fn quote(path: &Path) -> String {
    format!("'{}'", path.to_str().expect("UTF-8 path").replace('\'', "'\"'\"'"))
}

/// Installed before import; used both normally and by the launcher's EXIT trap.
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
