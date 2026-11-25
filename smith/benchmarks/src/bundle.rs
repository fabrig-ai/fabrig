//! Repository seeds name frozen, vendored directories kept outside the tree.
//! The manifest binds their source pins and digest; an absent bundle is said and
//! skipped offline, and refuses an attempt (../../docs/design/benchmarks/README.md, section 5.2, and ../../docs/design/benchmarks/isolation.md, section 1).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::files::{Files, Native};
use crate::isolate::SetupStep;
use crate::{Fault, Kind, Refusal, Task, input};

/// A repository seed's bundle contents and digest, supplied by the task author.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    /// Directory under `$XDG_STATE_HOME/smith-bench/bundles/`.
    pub name: String,
    /// Frozen source commit.
    pub source: String,
    pub pins: Vec<Pin>,
    /// `seed_digest` of the complete bundle directory.
    pub sha256: String,
}

/// One vendored git dependency's frozen commit, supplied by the task author.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub name: String,
    pub commit: String,
}

/// Offline bundle checking either accepts the directory or reports its absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundleCheck {
    /// The present bundle matches its manifest.
    Matches,
    /// The machine has no bundle: the offline check is skipped and said so.
    Absent,
}

/// Read a strict bundle manifest with directory names, git pins and SHA-256 checked.
pub fn read_bundle_manifest(file: &Path) -> Result<BundleManifest, Refusal> {
    read_bundle_manifest_with(&Native, file)
}

/// Read and check a bundle manifest on the supplied filesystem.
pub fn read_bundle_manifest_with(files: &dyn Files, file: &Path) -> Result<BundleManifest, Refusal> {
    let manifest: BundleManifest = crate::formats::read_document_with(files, file)?;
    crate::check::check_path(file, "name", Path::new(&manifest.name))?;
    if manifest.name.contains('/')
        || !crate::formats::lowercase_hex(&manifest.source, 40)
        || !crate::formats::lowercase_hex(&manifest.sha256, 64)
        || manifest.pins.iter().any(|pin| pin.name.is_empty() || !crate::formats::lowercase_hex(&pin.commit, 40))
    {
        return Err(Refusal::new(file, "bundle", "one directory name, exact git commits and SHA-256 required"));
    }
    Ok(manifest)
}

/// Accept a present matching bundle; absence is skipped, and any changed bytes refuse.
pub fn check_bundle(manifest: &BundleManifest, bundles: &Path) -> Result<BundleCheck, Refusal> {
    check_bundle_with(&Native, manifest, bundles)
}

/// Verify a bundle's presence and digest on the supplied filesystem.
pub fn check_bundle_with(files: &dyn Files, manifest: &BundleManifest, bundles: &Path) -> Result<BundleCheck, Refusal> {
    let directory = bundles.join(&manifest.name);
    if !files.try_exists(&directory).map_err(|error| Refusal::new(&directory, "bundle", error.to_string()))? {
        return Ok(BundleCheck::Absent);
    }
    let digest = crate::seed_digest_with(files, &directory)?;
    if digest != manifest.sha256 {
        return Err(Refusal::new(&directory, "sha256", "bundle digest differs"));
    }
    Ok(BundleCheck::Matches)
}

/// The copy source and its digest: an ordinary seed, or a repository's checked bundle.
pub fn seed_source(task: &Task, task_directory: &Path, bundles: &Path) -> Result<(PathBuf, String), Fault> {
    seed_source_with(&Native, task, task_directory, bundles)
}

/// Resolve a seed or bundle on the supplied filesystem.
pub fn seed_source_with(
    files: &dyn Files,
    task: &Task,
    task_directory: &Path,
    bundles: &Path,
) -> Result<(PathBuf, String), Fault> {
    resolve_seed(files, task, task_directory, bundles).map_err(Fault::at(SetupStep::Seed))
}

fn resolve_seed(
    files: &dyn Files,
    task: &Task,
    task_directory: &Path,
    bundles: &Path,
) -> Result<(PathBuf, String), Fault> {
    let seed = task_directory.join("seed");
    if task.kind != Kind::Repository {
        return Ok((seed, task.seed_sha256.clone()));
    }
    let manifest = read_bundle_manifest_with(files, &seed.join("bundle.toml"))?;
    match check_bundle_with(files, &manifest, bundles)? {
        BundleCheck::Matches => Ok((bundles.join(manifest.name), manifest.sha256)),
        BundleCheck::Absent => Err(input(format!("repository bundle {} is absent", manifest.name))),
    }
}

pub(crate) fn check_repository_bundles(
    files: &dyn Files,
    tasks: &[crate::CatalogueTask],
    bundles: &Path,
) -> Result<(), Refusal> {
    for entry in tasks {
        if entry.task.kind != Kind::Repository {
            continue;
        }
        let file = entry.manifest.parent().expect("catalogue manifest parent").join("seed/bundle.toml");
        let manifest = read_bundle_manifest_with(files, &file)?;
        if check_bundle_with(files, &manifest, bundles)? == BundleCheck::Absent {
            eprintln!("{} bundle {} absent: skipped", entry.name, manifest.name);
        }
    }
    Ok(())
}
