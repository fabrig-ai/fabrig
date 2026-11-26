//! Newly landed look requests, after the print (temper/docs/design/client/journey-writer.md, 4, step 5).

use std::{fmt::Write as _, os::unix::ffi::OsStrExt as _};

use crate::{
    host::{Ask, Host, Program, Stream},
    look_compare::request::{self, ReadError},
    looks::{self, Landing, Render, Verdict},
};

use super::{Checkout, Stop, changed, child, diagnostic, fail};

mod retire;

impl Checkout {
    pub(super) fn check_requests(&self, host: &mut Host, tip: &str, printed: &[u8]) -> Result<Vec<u8>, Stop> {
        match self.check_landed(host, tip, printed) {
            Ok(lines) => {
                let lines = lines.into_bytes();
                host.out(&lines);
                Ok(lines)
            }
            Err(stop) => {
                if self.restore_roots(host, tip).is_err() {
                    diagnostic(host, b"could not restore both journeys roots after the failed check");
                }
                Err(stop)
            }
        }
    }

    fn check_landed(&self, host: &mut Host, tip: &str, printed: &[u8]) -> Result<String, Stop> {
        let log = self
            .git(host, &[b"log", b"--topo-order", b"-z", looks::LANDINGS.as_bytes(), tip.as_bytes()], Stream::Keep)
            .map_err(|_| fail(&[b"cannot read Look-landed history"]))?;
        let landings = looks::landings(&log).map_err(Stop::Host)?;
        let mut lines = String::new();
        let mut roots = [Root::new(false), Root::new(true)];
        let mut identical = Vec::new();
        for landing in landings.iter().rev() {
            let [web, tui] = &mut roots;
            let root = if looks::is_tui(&landing.name) { tui } else { web };
            let page = root.page(host, self, tip)?;
            if let Some(entry) = page.standing(&landing.name, &landing.commit) {
                if entry.result == "identical" {
                    identical.push((landing.name.as_str(), entry.commit.clone()));
                }
                continue;
            }
            let checked = self.check_request(host, landing, tip, printed)?;
            match checked {
                Check::Pending(journey) => {
                    writeln!(lines, "pending {}: {journey} stopped", landing.name).expect("write to String");
                }
                Check::Ready { commit, verdict } => {
                    writeln!(lines, "checked {}: {}", landing.name, verdict.result()).expect("write to String");
                    root.entries.push(verdict.section(landing, &commit, tip));
                    if verdict.result() == "identical" {
                        identical.push((landing.name.as_str(), commit));
                    }
                }
            }
        }
        let candidates = retire::candidates(self, host, tip, &identical)?;
        for (root, candidates) in roots.into_iter().zip(candidates) {
            root.write(host, self, &candidates)?;
        }
        Ok(lines)
    }

    fn check_request(&self, host: &mut Host, landing: &Landing, tip: &str, printed: &[u8]) -> Result<Check, Stop> {
        let name = &landing.name;
        let branch = request::branch(name);
        if !self.request_branch_exists(host, name, &branch)? {
            return Ok(Check::not_checked("0".repeat(tip.len()), format!("origin lacks docs/look/{name}")));
        }
        request::fetch(host, Some(&self.root), "origin", &branch).map_err(Stop::Host)?;
        let request = match request::fetched(host, Some(&self.root), name, None) {
            Ok(request) => request,
            Err(ReadError::Invalid { commit, why }) => return Ok(Check::not_checked(commit, why)),
            Err(ReadError::Failed(why)) => return Err(Stop::Host(why)),
        };
        if printed.split(|&byte| byte == b'\n').any(|line| line.starts_with(b"stopped ")) {
            let changes = request.changes(host, &self.root).map_err(Stop::Host)?;
            if let Some(journey) = changes.stopped(printed) {
                return Ok(Check::Pending(journey.into()));
            }
        }
        let program = self.here(Program::new(
            b"cargo",
            &[
                b"run",
                b"--locked",
                b"--quiet",
                b"-p",
                b"temper-client-journeys",
                b"--",
                b"look-render",
                name.as_bytes(),
                request.commit.as_bytes(),
            ],
        ));
        let ran = child(host, program)?;
        if ran.code != Some(0) {
            return Err(fail(&[b"look-render could not check ", name.as_bytes()]));
        }
        let render = Render::read(&ran.stdout).map_err(Stop::Host)?;
        Ok(Check::Ready { commit: request.commit, verdict: Verdict::Checked(render) })
    }

    fn request_branch_exists(&self, host: &mut Host, name: &str, branch: &str) -> Result<bool, Stop> {
        let remote =
            self.here(Program::new(b"git", &[b"ls-remote", b"--exit-code", b"--refs", b"origin", branch.as_bytes()]));
        match child(host, remote)?.code {
            Some(0) => Ok(true),
            Some(2) => Ok(false),
            _ => Err(fail(&[b"cannot look up docs/look/", name.as_bytes(), b" on origin"])),
        }
    }

    fn restore_roots(&self, host: &mut Host, tip: &str) -> Result<(), Stop> {
        let web = looks::root(false).as_bytes();
        let tui = looks::root(true).as_bytes();
        let restore = self.restore_tracked(host, tip.as_bytes(), web, tui);
        let clean = self.git(host, &[b"clean", b"-ffdx", b"--", web, tui], Stream::Err);
        restore.and(clean.map(|_| ()))
    }

    fn restore_tracked(&self, host: &mut Host, tip: &[u8], web: &[u8], tui: &[u8]) -> Result<(), Stop> {
        // Pass only tracked paths that differ so a root absent from TIP does not fail restore.
        let paths =
            self.git(host, &[b"diff", b"--name-only", b"--no-renames", b"-z", tip, b"--", web, tui], Stream::Keep)?;
        if !paths.is_empty() {
            let file = self.lock.join("restore-paths");
            changed(host, Ask::Write(file.clone(), paths))?;
            let paths = [b"--pathspec-from-file=".as_slice(), file.as_os_str().as_bytes()].concat();
            self.git(
                host,
                &[
                    b"--literal-pathspecs",
                    b"restore",
                    b"--source",
                    tip,
                    b"--staged",
                    b"--worktree",
                    &paths,
                    b"--pathspec-file-nul",
                ],
                Stream::Err,
            )?;
        }
        Ok(())
    }
}

enum Check {
    Ready { commit: String, verdict: Verdict },
    Pending(String),
}

impl Check {
    fn not_checked(commit: String, why: String) -> Self {
        Self::Ready { commit, verdict: Verdict::NotChecked(why) }
    }
}

struct Root {
    path: String,
    page: Option<looks::Page>,
    entries: Vec<String>,
}

impl Root {
    fn new(tui: bool) -> Self {
        Self { path: looks::path(tui), page: None, entries: Vec::new() }
    }

    fn page(&mut self, host: &mut Host, checkout: &Checkout, tip: &str) -> Result<&looks::Page, Stop> {
        if let Some(ref page) = self.page {
            return Ok(page);
        }
        let page = looks::Page::at(host, Some(&checkout.root), tip, &self.path, "journey-pages").map_err(Stop::Host)?;
        Ok(self.page.insert(page))
    }

    fn write(mut self, host: &mut Host, checkout: &Checkout, candidates: &[(String, String)]) -> Result<(), Stop> {
        if let Some(page) = self.page {
            self.entries.reverse();
            let updated =
                page.updated(&self.entries, candidates).map_err(|why| Stop::Host(format!("{}: {why}", self.path)))?;
            if let Some(bytes) = updated {
                changed(host, Ask::Write(checkout.root.join(self.path), bytes))?;
            }
        }
        Ok(())
    }
}
