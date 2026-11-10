//! Source ancestry selects the shipped product invocation and receiving bounds.
//! The registry supports only binaries descended from the first product face.
//! `face` reads only Git's ancestry, not an agent's private state
//! (../../../../docs/design/benchmarks/README.md, section 6.2; reliability-plan 00, Interface sketches).

use crate::{Bound, Fault, format, input, refuse_if};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::arm::BuiltArm;
use crate::isolate::SetupStep;

/// Receiving bounds pinned with the face that writes the stream.
/// The arm supplies semantic caps and the writer-derived aggregate line ceiling.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Receiving {
    pub string: u32,
    pub content: u32,
    pub items: u32,
    /// The writer's complete encoded line in bytes, including its newline.
    pub encoded_line: u32,
}

impl Receiving {
    /// Preserve semantic caps exactly; the separate aggregate ceiling must be nonzero.
    pub fn limits(self) -> Result<smith_events::Limits, Fault> {
        let limits = smith_events::Limits { string: self.string, content: self.content, items: self.items };
        if self.string == 0 || self.content == 0 || self.items == 0 || self.encoded_line == 0 {
            return Err(format("event receiving semantic caps and encoded-line ceiling must be positive"));
        }
        Ok(limits)
    }
}

/// A registry entry from which this product face applies.
#[derive(Clone, Debug, Serialize)]
pub struct FaceSince {
    pub commit: String,
    pub shape: super::events::Shape,
    pub receiving: Receiving,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    commit: String,
    shape: super::events::Shape,
    receiving: Receiving,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    face: Vec<Entry>,
}

/// Read the bounded, closed registry beside the configuration pin.
pub fn read(path: &Path) -> Result<Vec<FaceSince>, Fault> {
    read_registry(path).map_err(Fault::at(SetupStep::Configuration))
}

fn read_registry(path: &Path) -> Result<Vec<FaceSince>, Fault> {
    let bytes = std::fs::read(path)?;
    refuse_if(bytes.len() > 65_536, Bound, "Smith face registry exceeds its bound")?;
    let text = std::str::from_utf8(&bytes)?;
    let registry: Registry = toml::from_str(text)?;
    if registry.face.is_empty() || registry.face.len() > 64 {
        return Err(input("Smith face registry has an invalid inventory"));
    }
    for entry in &registry.face {
        if !crate::formats::lowercase_hex(&entry.commit, 40) {
            return Err(input("Smith face registry requires full source commits"));
        }
    }
    let mut faces = Vec::new();
    for entry in registry.face {
        entry.receiving.limits()?;
        faces.push(FaceSince { commit: entry.commit, shape: entry.shape, receiving: entry.receiving });
    }
    Ok(faces)
}

/// Choose the newest entry that is an ancestor of this verified built arm.
pub fn face(arm: &BuiltArm, faces: &[FaceSince], repository: &Path) -> Result<FaceSince, Fault> {
    select_face(arm, faces, repository).map_err(Fault::at(SetupStep::Configuration))
}

fn select_face(arm: &BuiltArm, faces: &[FaceSince], repository: &Path) -> Result<FaceSince, Fault> {
    let mut ancestry = Vec::new();
    for entry in faces.iter().rev() {
        let result = std::process::Command::new("git")
            .args(["-c", "core.hooksPath=/dev/null", "-C"])
            .arg(repository)
            .args(["merge-base", "--is-ancestor", &entry.commit, &arm.commit])
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_OBJECT_DIRECTORY")
            .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
            .stdin(std::process::Stdio::null())
            .output()?;
        let status = result.status.code();
        ancestry.push(status);
        if status != Some(1) {
            break;
        }
    }
    select(arm, faces, &ancestry)
}

// Git results arrive newest first; only a proved ancestor admits a product face.
fn select(arm: &BuiltArm, faces: &[FaceSince], ancestry: &[Option<i32>]) -> Result<FaceSince, Fault> {
    for (entry, status) in faces.iter().rev().zip(ancestry) {
        match status {
            Some(0) => return Ok(entry.clone()),
            Some(1) => {}
            Some(_) | None => return Err(input("cannot establish Smith arm source ancestry")),
        }
    }
    Err(input(format!("Smith arm {} predates the first supported product face", arm.commit)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::smith::events::Shape;

    fn registry() -> Vec<FaceSince> {
        const RECEIVING: Receiving = Receiving { string: 128, content: 1024, items: 16, encoded_line: 16384 };
        vec![
            FaceSince { commit: "1".repeat(40), shape: Shape::Chat, receiving: RECEIVING },
            FaceSince { commit: "2".repeat(40), shape: Shape::Exec, receiving: RECEIVING },
        ]
    }

    fn arm() -> BuiltArm {
        BuiltArm {
            version: 1,
            commit: "a".repeat(40),
            sha256: "b".repeat(64),
            toolchain: "supplied verified identity".into(),
            binaries: Vec::new(),
        }
    }

    #[test]
    fn all_false_ancestry_refuses_an_unsupported_source() {
        let refusal = select(&arm(), &registry(), &[Some(1), Some(1)]).expect_err("no supported ancestor");
        assert_eq!(refusal.detail, format!("Smith arm {} predates the first supported product face", "a".repeat(40)));
    }

    #[test]
    fn newest_proved_ancestor_selects_its_face_and_receiving_bounds() {
        let selected = select(&arm(), &registry(), &[Some(0), Some(0)]).expect("newest matching face");
        assert_eq!(selected.shape, Shape::Exec);
        assert_eq!(selected.commit, "2".repeat(40));
        assert_eq!(selected.receiving.encoded_line, 16384);
    }

    #[test]
    fn nonancestor_newer_face_falls_back_to_the_older_ancestor() {
        let selected = select(&arm(), &registry(), &[Some(1), Some(0)]).expect("older matching face");
        assert_eq!(selected.shape, Shape::Chat);
        assert_eq!(selected.commit, "1".repeat(40));
    }

    #[test]
    fn failed_ancestry_queries_never_admit_an_older_face() {
        for status in [None, Some(2), Some(128)] {
            assert_eq!(
                select(&arm(), &registry(), &[status, Some(0)]).expect_err("unknown ancestry").detail,
                "cannot establish Smith arm source ancestry"
            );
        }
    }
}
