//! Bounded discovery retains paths and parent evidence from rollout openings alone.
//! It walks without following links and never reads credentials or a state database.
//! Contract: smith/docs/design/benchmarks/README.md, 6.3.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use super::{ParseError, bad, parent_value};

#[derive(Clone, Copy, Debug)]
pub(super) struct Bounds {
    pub entries: usize,
    pub opening_bytes: usize,
}

pub(super) const BOUNDS: Bounds = Bounds { entries: 8192, opening_bytes: 1024 * 1024 };

#[derive(Debug)]
pub(super) struct Candidate {
    pub path: PathBuf,
    pub parent: Option<String>,
}

fn fault(home: &Path, path: &Path, reason: &str) -> ParseError {
    bad("discovery", &path.strip_prefix(home).unwrap_or(path).display().to_string(), reason)
}

fn inside(path: &Path, roots: &[PathBuf; 2]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn opening(home: &Path, path: &Path, bound: usize) -> Result<Option<(String, Candidate)>, ParseError> {
    let file = std::fs::File::open(path).map_err(|_| fault(home, path, "cannot open rollout opening"))?;
    let mut bytes = Vec::new();
    BufReader::new(file.take(u64::try_from(bound).expect("bounded opening")))
        .read_until(b'\n', &mut bytes)
        .map_err(|_| fault(home, path, "cannot read rollout opening"))?;
    if bytes.len() >= bound {
        return Ok(None);
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else { return Ok(None) };
    if value.get("type").and_then(serde_json::Value::as_str) != Some("session_meta") {
        return Ok(None);
    }
    let Some(payload) = value.get("payload") else { return Ok(None) };
    let Some(id) = payload.get("id").and_then(serde_json::Value::as_str).filter(|id| !id.is_empty()) else {
        return Ok(None);
    };
    let parent =
        parent_value(payload).and_then(serde_json::Value::as_str).filter(|id| !id.is_empty()).map(str::to_owned);
    Ok(Some((id.into(), Candidate { path: path.into(), parent })))
}

pub(super) fn discover(home: &Path, bounds: Bounds) -> Result<BTreeMap<String, Candidate>, ParseError> {
    let home = home.canonicalize().map_err(|_| bad("discovery", "home", "cannot resolve Codex home"))?;
    let roots = [home.join("sessions"), home.join("archived_sessions")];
    let mut pending: Vec<_> = roots.iter().rev().cloned().collect();
    let mut entries = 0_usize;
    let mut candidates = BTreeMap::new();
    while let Some(path) = pending.pop() {
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if roots.contains(&path) && error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return Err(fault(&home, &path, "cannot inspect entry")),
        };
        if metadata.is_symlink() {
            let target = path.canonicalize().map_err(|_| fault(&home, &path, "dangling or unresolved link"))?;
            if !inside(&target, &roots) {
                return Err(fault(&home, &path, "link outside session directories"));
            }
            continue;
        }
        if metadata.is_dir() {
            let directory = std::fs::read_dir(&path).map_err(|_| fault(&home, &path, "cannot list directory"))?;
            let mut children = Vec::new();
            for entry in directory {
                let entry = entry.map_err(|_| fault(&home, &path, "cannot list entry"))?;
                entries = entries.checked_add(1).ok_or_else(|| fault(&home, &path, "entry bound exceeded"))?;
                if entries > bounds.entries {
                    return Err(fault(&home, &entry.path(), "entry bound exceeded"));
                }
                children.push(entry.path());
            }
            children.sort();
            pending.extend(children.into_iter().rev());
        } else if metadata.is_file() {
            let resolved = path.canonicalize().map_err(|_| fault(&home, &path, "cannot resolve rollout path"))?;
            if !inside(&resolved, &roots) {
                return Err(fault(&home, &path, "file outside session directories"));
            }
            if let Some((id, candidate)) = opening(&home, &path, bounds.opening_bytes)?
                && candidates.insert(id, candidate).is_some()
            {
                return Err(fault(&home, &path, "duplicate thread opening"));
            }
        }
    }
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::{Bounds, discover};

    #[test]
    fn discovery_bounds_count_entries_and_cut_only_the_opening() {
        let scratch = skein_scratch::Scratch::new("discovery-bounds");
        let home = scratch.path();
        std::fs::create_dir_all(home.join("sessions/nested")).expect("session directories");
        std::fs::create_dir(home.join("archived_sessions")).expect("archive directory");
        let metadata = b"{\"type\":\"session_meta\",\"payload\":{\"id\":\"root\"}}";
        let mut opening = metadata.to_vec();
        opening.resize(63, b' ');
        std::fs::write(home.join("sessions/nested/root"), &opening).expect("opening below bound, no newline");
        let mut overlong = metadata.to_vec();
        overlong.resize(65, b' ');
        overlong.push(b'\n');
        std::fs::write(home.join("archived_sessions/long"), &overlong).expect("opening past bound");
        std::os::unix::fs::symlink(home.join("sessions/nested/root"), home.join("sessions/alias"))
            .expect("internal link also counts");
        let bounds = Bounds { entries: 4, opening_bytes: 64 };
        let found = discover(home, bounds).expect("four entries, long opening unrelated");
        assert_eq!(found.keys().map(String::as_str).collect::<Vec<_>>(), ["root"]);
        // Only the opening counts: a short first line may precede an arbitrarily long body.
        let mut body = metadata.to_vec();
        body.push(b'\n');
        body.extend_from_slice(&overlong);
        std::fs::write(home.join("sessions/nested/root"), body).expect("short opening, long body");
        assert_eq!(discover(home, bounds).expect("only opening read").len(), 1);
        std::fs::write(home.join("archived_sessions/extra"), []).expect("fifth entry");
        let error = discover(home, bounds).expect_err("one past entry bound").to_string();
        assert!(error.contains("entry bound"), "{error}");
        assert!(error.contains("sessions/"), "fault names entry under home: {error}");
    }
}
