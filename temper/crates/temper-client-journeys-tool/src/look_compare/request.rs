use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::{Change, Host, Journey, Side, TUI, WEB, git_ok, text, valid_name, version_two};
use crate::looks::hash;

pub(crate) struct Request {
    pub(crate) parent: String,
    pub(crate) commit: String,
}

pub(crate) enum ReadError {
    Invalid { commit: String, why: String },
    Failed(String),
}

pub(crate) fn read(arguments: &[Box<[u8]>], host: &mut Host) -> Result<Request, String> {
    let [name, prefix] = arguments else { return Err("expected NAME and C".into()) };
    let name =
        std::str::from_utf8(name).ok().filter(|name| valid_name(name)).ok_or("NAME must be [a-z0-9][a-z0-9-]*")?;
    let prefix = std::str::from_utf8(prefix)
        .ok()
        .filter(|prefix| (7..=64).contains(&prefix.len()) && hex(prefix))
        .ok_or("C must be 7 to 64 lowercase hexadecimal digits")?;
    fetch(host, None, &branch(name))?;
    fetched(host, None, name, Some(prefix)).map_err(|error| match error {
        ReadError::Invalid { why, .. } | ReadError::Failed(why) => why,
    })
}

pub(crate) fn branch(name: &str) -> String {
    format!("refs/heads/docs/look/{name}")
}

pub(crate) fn fetch(host: &mut Host, root: Option<&Path>, branch: &str) -> Result<(), String> {
    git_ok(host, root, &["fetch", "--no-tags", "--no-recurse-submodules", "--refmap=", "origin", branch])?;
    Ok(())
}

pub(crate) fn fetched(
    host: &mut Host,
    root: Option<&Path>,
    name: &str,
    prefix: Option<&str>,
) -> Result<Request, ReadError> {
    let result = git_ok(host, root, &["rev-parse", "--verify", "FETCH_HEAD^{commit}"]).map_err(ReadError::Failed)?;
    let commit = text(&result, "fetched tip").map_err(ReadError::Failed)?.trim_end_matches('\n');
    if let Some(prefix) = prefix
        && (!hash(commit) || !commit.starts_with(prefix))
    {
        return Err(ReadError::Failed(format!("docs/look/{name} fetched {commit}, not C ({prefix})")));
    }
    if !hash(commit) {
        return Err(ReadError::Failed(format!("docs/look/{name} fetched an invalid commit: {commit}")));
    }
    let invalid = |why| ReadError::Invalid { commit: commit.into(), why };
    let result = git_ok(host, root, &["show", "-s", "--format=%P", commit]).map_err(ReadError::Failed)?;
    let parent = text(&result, "request parent").map_err(ReadError::Failed)?.trim_end_matches('\n');
    if !hash(parent) {
        return Err(invalid(format!("{commit} must have exactly one parent")));
    }
    let result = git_ok(host, root, &["show", "-s", "--format=%(trailers:key=Look-request,unfold)", commit])
        .map_err(ReadError::Failed)?;
    let trailers = text(&result, "request trailer").map_err(ReadError::Failed)?.trim_end_matches('\n');
    let trailer = trailers.split_once(':');
    if trailers.contains('\n')
        || !trailer.is_some_and(|(key, value)| {
            key.eq_ignore_ascii_case("Look-request") && value.trim_start_matches([' ', '\t']) == name
        })
    {
        return Err(invalid(format!("{commit} must carry exactly one Look-request: {name} trailer")));
    }
    Ok(Request { parent: parent.into(), commit: commit.into() })
}

fn hex(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(crate) struct Changes {
    pub(crate) journeys: BTreeMap<Journey, Change>,
    pub(crate) overlays: BTreeSet<Side>,
}

impl Changes {
    pub(crate) fn stopped(&self, stdout: &[u8]) -> Option<&str> {
        self.journeys.iter().find_map(|(journey, change)| {
            if !matches!(change, Change::Pages(_)) {
                return None;
            }
            let prefix = format!("stopped {}: ", journey.name);
            stdout
                .split(|&byte| byte == b'\n')
                .any(|line| line.starts_with(prefix.as_bytes()))
                .then_some(journey.name.as_str())
        })
    }
}

impl Request {
    pub(crate) fn changes(&self, host: &mut Host, root: &Path) -> Result<Changes, String> {
        let paths = git_ok(
            host,
            Some(root),
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "--no-renames",
                "-r",
                "-z",
                &self.parent,
                &self.commit,
                "--",
                WEB,
                TUI,
            ],
        )?;
        let mut directories = BTreeMap::<Journey, BTreeSet<String>>::new();
        let mut overlays = BTreeSet::new();
        for path in text(&paths, "changed path")?.split_terminator('\0') {
            let Some((side, relative)) = [Side::Web, Side::Tui].into_iter().find_map(|side| {
                path.strip_prefix(side.root()).and_then(|rest| rest.strip_prefix('/')).map(|rest| (side, rest))
            }) else {
                continue;
            };
            if relative == "overlay.css" {
                overlays.insert(side);
            } else if let Some((name, file)) = relative.split_once('/') {
                let numbered = directories.entry(Journey { side, name: name.into() }).or_default();
                if side.numbered_page(file) {
                    numbered.insert(file.into());
                }
            }
        }
        let mut journeys = BTreeMap::new();
        for (journey, files) in directories {
            if !blob(host, &self.commit, &format!("{}/walk", journey.path()))?.is_some_and(|bytes| version_two(&bytes))
            {
                journeys.insert(journey, Change::HandMade);
            } else if !files.is_empty() {
                journeys.insert(journey, Change::Pages(files));
            }
        }
        Ok(Changes { journeys, overlays })
    }
}

pub(crate) fn blob(host: &mut Host, commit: &str, path: &str) -> Result<Option<Box<[u8]>>, String> {
    blob_at(host, None, commit, path)
}

pub(crate) fn blob_at(
    host: &mut Host,
    root: Option<&Path>,
    commit: &str,
    path: &str,
) -> Result<Option<Box<[u8]>>, String> {
    // ls-tree can distinguish a path the commit lacks from a git failure.
    let tree = git_ok(host, root, &["--literal-pathspecs", "ls-tree", "--full-tree", "-z", commit, "--", path])?;
    if tree.is_empty() {
        return Ok(None);
    }
    let invalid = || format!("unexpected tree entry for {commit}:{path}");
    let entry = text(&tree, "tree entry")?.strip_suffix('\0').ok_or_else(invalid)?;
    let (metadata, name) = entry.split_once('\t').ok_or_else(invalid)?;
    let fields: Vec<_> = metadata.split_ascii_whitespace().collect();
    let [_, kind, object] = fields.as_slice() else { return Err(invalid()) };
    if name != path || !hash(object) {
        return Err(invalid());
    }
    if *kind != "blob" {
        return Ok(None);
    }
    git_ok(host, root, &["cat-file", "blob", object]).map(Some)
}
