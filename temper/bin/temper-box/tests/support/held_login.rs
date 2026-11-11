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
    write_hook(binary, store, client, account, false);
}

/// Install an export-only recovery hook for a fresh client, keeping later imports excluded.
/// Installation shares the give-back lock and preserves terminal cleanup.
pub fn recovery_hook(binary: &Path, store: &Path, client: &Path, account: &Path) {
    write_hook(binary, store, client, account, true);
}

fn write_hook(binary: &Path, store: &Path, client: &Path, account: &Path, recovering: bool) {
    let hook = store.join("give-back");
    let body = format!(
        r#"#!/bin/sh
set -eu
exec 9>{lock}
flock -x 9
: > {returning}
if [ "${{1-}}" != --box-ended ]; then : > {terminal}; fi
if [ ! -e {account} ]; then
  rung_account=$({binary} ctl {client} account list --json </dev/null | jq -er '.accounts.party | map(select(.provider == 1 and .plan == true)) | if length == 1 then .[0].number else error("expected one held ChatGPT plan") end')
  {binary} ctl {client} account export "$rung_account" {account} </dev/null
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
        terminal = quote(&store.join("terminal-return")),
        returning = quote(&store.join("returning"))
    );
    install_hook(store, &hook, &body, recovering);
}

fn install_hook(store: &Path, hook: &Path, body: &str, recovering: bool) {
    let pending = store.join("give-back-next");
    super::write_private(&pending, body.as_bytes());
    fs::set_permissions(&pending, fs::Permissions::from_mode(0o700)).expect("executable return hook");
    let admission = if recovering {
        ": > \"$1/terminal-return\"\n: > \"$1/returning\""
    } else {
        "test ! -e \"$1/terminal-return\" || exit 1"
    };
    let reopen = if recovering { "" } else { "rm -f -- \"$1/returning\"" };
    let install = format!("set -eu\n{admission}\nmv -f -- \"$2\" \"$3\"\n{reopen}\n");
    let installed = std::process::Command::new("flock")
        .arg("-x")
        .arg(store.join("login.lock"))
        .args(["sh", "-c", &install, "sh"])
        .arg(store)
        .arg(pending)
        .arg(hook)
        .stdin(std::process::Stdio::null())
        .status()
        .expect("install return hook under the import lock");
    assert!(installed.success(), "terminal return excludes another import");
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

/// Each box returns the account before stopping, on success and failure alike.
#[must_use]
pub const fn due_back(first: bool, failed: bool) -> bool {
    let _ = (first, failed);
    true
}

/// The return-before-stop stage shared by the live driver and its fake box.
pub struct Ending {
    returning: bool,
    started: bool,
    until: skein_lib::Time,
}

/// One effect of ending a box; a failed return still stops and fails the story.
pub enum End {
    /// Run and verify the held record's give-back.
    Return,
    /// Let the outstanding ctl or give-back process finish.
    Wait,
    /// Stop the box, retaining any return failure as the story's failure.
    Stop(Option<&'static str>),
}

impl Ending {
    /// Begin teardown with the same return policy for successful and failed boxes.
    #[must_use]
    pub fn new(held: bool, first: bool, failed: bool, now: skein_lib::Time) -> Self {
        Self {
            returning: held && due_back(first, failed),
            started: false,
            until: now.saturating_add(skein_lib::Duration::from_secs(60)),
        }
    }

    /// The return's deadline, while its process is outstanding.
    #[must_use]
    pub const fn deadline(&self) -> skein_lib::Time {
        self.until
    }

    /// Admit export only after ctl drains, and stop only after its result or deadline.
    #[must_use]
    pub fn next(&mut self, now: skein_lib::Time, idle: bool, exit: Option<Option<skein_io::kernel::Exit>>) -> End {
        if !self.returning {
            End::Stop(None)
        } else if let Some(exit) = exit {
            End::Stop((exit != Some(skein_io::kernel::Exit::Code(0))).then_some("give-back failed"))
        } else if now >= self.until {
            End::Stop(Some("give-back exceeded its 60 second deadline"))
        } else if !self.started && idle {
            self.started = true;
            End::Return
        } else {
            End::Wait
        }
    }
}

/// A held record lent to a story's store, from before its import to the test's end
/// (temper/docs/design/testing/live-tier.md). A record at its path owes nothing, so the
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
