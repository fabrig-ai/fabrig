use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::{Change, Host, Journey, Side, TUI, WEB, git_ok, text, valid_name, version_two};

pub(super) struct Request {
    pub(super) parent: String,
    pub(super) commit: String,
}

pub(super) fn read(arguments: &[Box<[u8]>], host: &mut Host) -> Result<Request, String> {
    let [name, prefix] = arguments else { return Err("expected NAME and C".into()) };
    let name =
        std::str::from_utf8(name).ok().filter(|name| valid_name(name)).ok_or("NAME must be [a-z0-9][a-z0-9-]*")?;
    let prefix = std::str::from_utf8(prefix)
        .ok()
        .filter(|prefix| (7..=64).contains(&prefix.len()) && hex(prefix))
        .ok_or("C must be 7 to 64 lowercase hexadecimal digits")?;
    let branch = format!("refs/heads/docs/look/{name}");
    git_ok(host, None, &["fetch", "--no-tags", "--no-recurse-submodules", "--refmap=", "origin", &branch])?;
    let result = git_ok(host, None, &["rev-parse", "--verify", "FETCH_HEAD^{commit}"])?;
    let commit = text(&result, "fetched tip")?.trim_end_matches('\n');
    if !hash(commit) || !commit.starts_with(prefix) {
        return Err(format!("docs/look/{name} fetched {commit}, not C ({prefix})"));
    }
    let result = git_ok(host, None, &["show", "-s", "--format=%P", commit])?;
    let parent = text(&result, "request parent")?.trim_end_matches('\n');
    if !hash(parent) {
        return Err(format!("{commit} must have exactly one parent"));
    }
    let result = git_ok(host, None, &["show", "-s", "--format=%(trailers:key=Look-request,unfold)", commit])?;
    let trailers = text(&result, "request trailer")?.trim_end_matches('\n');
    let trailer = trailers.split_once(':');
    if trailers.contains('\n')
        || !trailer.is_some_and(|(key, value)| {
            key.eq_ignore_ascii_case("Look-request") && value.trim_start_matches([' ', '\t']) == name
        })
    {
        return Err(format!("{commit} must carry exactly one Look-request: {name} trailer"));
    }
    Ok(Request { parent: parent.into(), commit: commit.into() })
}

fn hex(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn hash(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && hex(value)
}

pub(super) struct Changes {
    pub(super) journeys: BTreeMap<Journey, Change>,
    pub(super) overlays: BTreeSet<Side>,
}

impl Request {
    pub(super) fn changes(&self, host: &mut Host, root: &Path) -> Result<Changes, String> {
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

pub(super) fn blob(host: &mut Host, commit: &str, path: &str) -> Result<Option<Box<[u8]>>, String> {
    // ls-tree can distinguish a path the commit lacks from a git failure.
    let tree = git_ok(host, None, &["--literal-pathspecs", "ls-tree", "--full-tree", "-z", commit, "--", path])?;
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
    git_ok(host, None, &["cat-file", "blob", object]).map(Some)
}
