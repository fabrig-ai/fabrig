//! The candidates to retire (temper/docs/design/client/journey-writer.md, 4, step 5).

use std::collections::BTreeSet;

use temper_client_shell_web::journey::read_fields;
use temper_client_walk_lines::{
    layout::{TUI, WEB},
    named,
};

use crate::{
    host::{Host, Program, Stream},
    look_compare::{Journey, Side, git_ok, request, text, version_two},
    looks,
};

use super::{Checkout, Stop, child, diagnostic};

type Candidates = [Vec<(String, String)>; 2];

pub(super) fn candidates(
    checkout: &Checkout,
    host: &mut Host,
    tip: &str,
    identical: &[(&str, String)],
) -> Result<Candidates, Stop> {
    let mut additions = BTreeSet::new();
    for (name, commit) in identical {
        if !checkout.have_request(host, name, commit)? {
            continue;
        }
        for journey in checkout.added_walks(host, commit).map_err(Stop::Host)? {
            additions.insert((journey, *name));
        }
    }
    let mut candidates = Candidates::default();
    if additions.is_empty() {
        return Ok(candidates);
    }
    let walks = checkout.walks_at(host, tip).map_err(Stop::Host)?;
    for (journey, request) in additions {
        if walks.journeys.contains(&journey) && !walks.parents.contains(&journey.name) {
            let [web, tui] = &mut candidates;
            let root = if looks::is_tui(request) { tui } else { web };
            let path = format!("{}/journeys/{}", journey.side.name(), journey.name);
            root.push((path, request.into()));
        }
    }
    Ok(candidates)
}

impl Checkout {
    fn have_request(&self, host: &mut Host, name: &str, commit: &str) -> Result<bool, Stop> {
        if self.has_commit(host, commit)? {
            return Ok(true);
        }
        let branch = request::branch(name);
        let why = if self.request_branch_exists(host, name, &branch)? {
            request::fetch(host, Some(&self.root), &branch).map_err(Stop::Host)?;
            // The branch may have moved; the standing entry names the commit to read.
            if self.has_commit(host, commit)? {
                return Ok(true);
            }
            format!("fetching docs/look/{name} did not supply {commit}")
        } else {
            format!("origin lacks docs/look/{name}")
        };
        diagnostic(host, format!("cannot read what {name} added: {why}").as_bytes());
        Ok(false)
    }

    fn has_commit(&self, host: &mut Host, commit: &str) -> Result<bool, Stop> {
        let revision = format!("{commit}^{{commit}}");
        let mut git = self.here(Program::new(b"git", &[b"rev-parse", b"--verify", b"--quiet", revision.as_bytes()]));
        git.stderr = Stream::Keep;
        let ran = child(host, git)?;
        match ran.code {
            Some(0) if ran.stdout.strip_suffix(b"\n") == Some(commit.as_bytes()) => Ok(true),
            Some(1) if ran.stderr.is_empty() => Ok(false),
            Some(_) | None => Err(Stop::Host(format!("cannot look up commit {commit}"))),
        }
    }

    fn added_walks(&self, host: &mut Host, commit: &str) -> Result<Vec<Journey>, String> {
        let root = Some(self.root.as_path());
        let parents = git_ok(host, root, &["show", "-s", "--format=%P", commit])?;
        let parent = text(&parents, "request parent")?.trim_end_matches('\n');
        if !looks::hash(parent) {
            return Err(format!("{commit} must have exactly one parent"));
        }
        let paths = git_ok(
            host,
            root,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "--no-renames",
                "--diff-filter=A",
                "-r",
                "-z",
                parent,
                commit,
                "--",
                WEB,
                TUI,
            ],
        )?;
        let mut added = Vec::new();
        for path in text(&paths, "added path")?.split_terminator('\0') {
            if let Some(journey) = walk_path(path) {
                added.push(journey);
            }
        }
        Ok(added)
    }

    fn walks_at(&self, host: &mut Host, tip: &str) -> Result<Walks, String> {
        let root = Some(self.root.as_path());
        // Only walks held by TIP can be candidates or start after one.
        let tree = git_ok(host, root, &["ls-tree", "-r", "--full-tree", "-z", tip, "--", WEB, TUI])?;
        let mut walks = Walks::default();
        for entry in text(&tree, "journey tree entry")?.split_terminator('\0') {
            let (metadata, path) = entry.split_once('\t').ok_or("malformed journey tree entry")?;
            let Some(journey) = walk_path(path) else { continue };
            let Some((mode, object)) = metadata.rsplit_once(' ') else {
                return Err(format!("malformed tree entry for {tip}:{path}"));
            };
            if mode == "160000 commit" {
                continue;
            }
            if !mode.ends_with(" blob") || !looks::hash(object) {
                return Err(format!("unexpected tree entry for {tip}:{path}"));
            }
            let bytes = git_ok(host, root, &["cat-file", "blob", object])?;
            if version_two(&bytes) {
                if let Some(parent) = after(&bytes) {
                    walks.parents.insert(parent);
                }
                walks.journeys.insert(journey);
            }
        }
        Ok(walks)
    }
}

#[derive(Default)]
struct Walks {
    journeys: BTreeSet<Journey>,
    parents: BTreeSet<String>,
}

fn walk_path(path: &str) -> Option<Journey> {
    for side in [Side::Web, Side::Tui] {
        let Some(relative) = path.strip_prefix(side.root()) else { continue };
        let Some(relative) = relative.strip_prefix('/') else { continue };
        if let Some((name, "walk")) = relative.split_once('/') {
            return Some(Journey { side, name: name.into() });
        }
    }
    None
}

fn after(bytes: &[u8]) -> Option<String> {
    let line = bytes.split(|&byte| byte == b'\n').nth(1)?;
    if !line.starts_with(b"after ") {
        return None;
    }
    let mut fields = read_fields(line).ok()?;
    fields.word().ok()?;
    let parent = fields.word().ok()?;
    let parent = std::str::from_utf8(&parent).ok()?;
    if named(parent) && fields.field().is_none() { Some(parent.into()) } else { None }
}
