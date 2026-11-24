//! Protocol-world checks for reviewed goldens.
use super::{File, Laid, Scenario};
use hearth_test_render::golden::{self, Kind, Leaf};
use skein_scratch::paths;
use std::{fs, path::Path};
#[must_use]
pub fn files_of(leaves: &[Leaf]) -> Vec<File> {
    leaves
        .iter()
        .map(|leaf| File {
            path: String::from_utf8(leaf.path.clone()).expect("golden path").leak(),
            body: leaf.bytes.clone().leak(),
            mode: match leaf.kind {
                Kind::File => 0o644,
                Kind::Executable => 0o755,
                Kind::Link => 0o120_000,
            },
        })
        .collect()
}
#[must_use]
pub fn scenario_of<'a>(files: &'a [File], laid: &'a [Laid], fault: Option<super::Fault>) -> Scenario<'a> {
    Scenario { files, laid, fault, ..super::thin() }
}
#[must_use]
pub fn text(run: &crate::protocol::Run) -> String {
    String::from_utf8(run.stdout.clone()).expect("client text")
}
#[must_use]
pub fn check_golden(name: &str) -> String {
    check_loaded_golden(&golden::load(name), "", &[])
}
/// Check the whole output, exit and witnesses, returning the text for any fixture-specific checks.
#[must_use]
pub fn check_loaded_golden(golden: &golden::Golden, variant: &str, laid: &[Laid]) -> String {
    let files = files_of(&golden.tree);
    let run = crate::protocol::run(&scenario_of(&files, laid, None), super::SIMULATION_SEED);
    let stdout = text(&run);
    check_output(golden, variant, &stdout, run.code);
    stdout
}
/// Deterministic schedule for the tool and resolution fault cases.
const PROTOCOL_ERROR_SEED: u64 = 0;
/// A production-only hostname edit, leaving the shared carbon base intact.
pub const CARBON_PRODUCTION_EDIT: File = File {
    path: "hosts/carbon/carbon-prod.json",
    body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"provider-a"},"core":{"hostname":"carbon-production"}}"#,
    mode: 0o644,
};

/// Repository configuration failures, followed by the multiple-fault ordering case.
pub const ERROR_CASES: &[&str] = &[
    "errors/decode",
    "errors/repository",
    "errors/composition",
    "errors/schema",
    "errors/evaluation",
    "errors/guard",
    "errors/reserved",
    "errors/ids",
    "errors/collision",
    "errors/exported",
    "errors/sorted",
];
#[derive(Clone, Copy)]
pub enum GoldenCall {
    KilledGit,
    Resolve,
}
pub const PROTOCOL_ERRORS: &[(&str, GoldenCall)] =
    &[("errors/tool", GoldenCall::KilledGit), ("errors/resolution", GoldenCall::Resolve)];

pub fn check_protocol_golden(name: &str, call: GoldenCall) {
    let root = paths::tree().join("tests/render");
    let files = fixture_files(&root.join(name), &[]);
    let scenario = match call {
        GoldenCall::KilledGit => {
            scenario_of(&files, &[], Some(super::Fault::KilledAt { stdout_bytes: super::KILLED_BYTES }))
        }
        GoldenCall::Resolve => super::resolving(&files),
    };
    let run = crate::protocol::run(&scenario, PROTOCOL_ERROR_SEED);
    // Resolution keeps only expected output here. Its repository is under testing/repos/test.
    let golden = golden::load_from(name, &root.join("resolution"));
    let verdict = golden.check("", &text(&run), run.code);
    assert!(verdict.is_clean(), "{}: {}", golden.name, verdict.report());
}

pub(super) fn fixture_files(root: &Path, modes: &[(&str, u32)]) -> Vec<File> {
    let mut files = Vec::new();
    read_fixture_files(root, root, modes, &mut files);
    files
}

fn read_fixture_files(root: &Path, directory: &Path, modes: &[(&str, u32)], files: &mut Vec<File>) {
    let mut entries: Vec<_> =
        fs::read_dir(directory).expect("repository fixture").map(|entry| entry.expect("fixture entry")).collect();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name();
        if directory == root
            && (name.as_encoded_bytes().starts_with(b"expected") || name == "rendered" || name == ".git")
        {
            continue;
        }
        let path = entry.path();
        if entry.file_type().expect("fixture entry type").is_dir() {
            read_fixture_files(root, &path, modes, files);
            continue;
        }
        let relative = path.strip_prefix(root).expect("repository fixture path").to_str().expect("fixture path text");
        let mode = modes.iter().find(|(path, _)| *path == relative).map_or(0o644, |&(_, mode)| mode);
        let body = if mode == 0o120_000 {
            fs::read_link(&path).expect("fixture link").into_os_string().into_encoded_bytes()
        } else {
            fs::read(&path).expect("fixture file")
        };
        files.push(File { path: relative.to_owned().leak(), body: body.leak(), mode });
    }
}

fn check_output(golden: &golden::Golden, variant: &str, stdout: &str, code: Option<u8>) {
    let verdict = golden.check(variant, stdout, code);
    if golden::bless() {
        golden.bless(variant, stdout, code);
        panic!("blessed: {}", verdict.changed());
    }
    assert!(verdict.is_clean(), "{}: {}", golden.name, verdict.report());
}

/// hearth/docs/design/domain/render/errors.md 2's repository-caused kinds plus the two builder-world kinds.
pub const ERROR_KINDS: &[&str] = &[
    "collision",
    "composition",
    "decode",
    "evaluation",
    "exported",
    "guard",
    "ids",
    "repository",
    "reserved",
    "resolution",
    "schema",
    "tool",
];

/// The pinned test repository supplies files; the golden holds expectations alone.
#[must_use]
pub fn resolution_golden() -> golden::Golden {
    let files = skein_scratch::paths::tree().join("testing/repos/test");
    golden::load_from("resolution", &files)
}

/// Whole digests from client JSON-lines spec records (hearth/docs/design/client/README.md, 3.2).
#[must_use]
pub fn digests_of(text: &str) -> Vec<(String, String)> {
    let mut digests = Vec::new();
    for line in text.lines().filter(|line| line.contains("\"record\":\"spec\"")) {
        let field = |name| {
            let prefix = format!("\"{name}\":\"");
            line.split_once(&prefix).expect("spec field").1.split('"').next().expect("field value").to_owned()
        };
        digests.push((field("target"), field("digest")));
    }
    digests.sort();
    digests
}
