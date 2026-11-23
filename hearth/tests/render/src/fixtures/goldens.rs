//! Four arrival orders per repository, as session 29 prescribes.
use crate::golden::{self, Golden, Kind, Leaf};
use skein_scratch::paths;
use std::{fs, path::Path};

pub const GOLDEN_SEEDS: core::ops::Range<u64> = 0..4;

#[must_use]
pub fn expectations(name: &str) -> Golden {
    // Resolution keeps only expected output here. Its repository is under testing/repos/test.
    golden::load_from(name, &paths::tree().join("tests/render/resolution"))
}

pub fn check_digests(name: &str) {
    let golden = expectations(name);
    let root = paths::tree().join("tests/render").join(name);
    let mut leaves = Vec::new();
    let kinds = if name == "fleet" { FLEET_KINDS } else { &[] };
    read_leaves(&root, &root, kinds, &mut leaves);
    let repository = golden::repository_of(&leaves);
    for seed in GOLDEN_SEEDS {
        skein_checks::count("seed", 1);
        let run = crate::parent::run(&repository, &[], seed);
        let digests: Vec<_> = run
            .answers
            .iter()
            .map(|answer| {
                let hearth_domain_render::vocabulary::Answer::Spec(spec) = answer else {
                    panic!("seed {seed}: {answer:?}")
                };
                let target = std::str::from_utf8(&spec.meta.target).expect("target name");
                (target, spec.digest.0.iter().take(6).copied().collect::<Vec<_>>())
            })
            .collect();
        let targets: Vec<_> =
            golden.target_digests("").into_iter().map(|(name, digest)| (name, digest.to_vec())).collect();
        let digests: Vec<_> = digests.into_iter().map(|(name, digest)| (name.to_owned(), digest)).collect();
        assert_eq!(digests, targets, "{name}: seed {seed}");
    }
}

const FLEET_KINDS: &[(&[u8], Kind)] = &[
    (b"tasks/desktop/tree/usr/local/bin/desktop-user.j2", Kind::Executable),
    (b"tasks/locale/tree/etc/localtime", Kind::Link),
];

fn read_leaves(root: &Path, directory: &Path, kinds: &[(&[u8], Kind)], leaves: &mut Vec<Leaf>) {
    let mut entries: Vec<_> =
        fs::read_dir(directory).expect("golden directory").map(|entry| entry.expect("golden entry")).collect();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name();
        if directory == root
            && (name.as_encoded_bytes().starts_with(b"expected") || name == "rendered" || name == ".git")
        {
            continue;
        }
        let path = entry.path();
        if entry.file_type().expect("golden entry type").is_dir() {
            read_leaves(root, &path, kinds, leaves);
            continue;
        }
        let relative = path.strip_prefix(root).expect("below golden").as_os_str().as_encoded_bytes();
        let kind = kinds.iter().find(|(name, _)| *name == relative).map_or(Kind::File, |&(_, kind)| kind);
        let bytes = match kind {
            Kind::File | Kind::Executable => fs::read(&path).expect("golden file"),
            Kind::Link => fs::read_link(&path).expect("golden link").into_os_string().into_encoded_bytes(),
        };
        leaves.push(Leaf { path: relative.to_vec(), bytes, kind });
    }
}

pub use hearth_test_render::{JSON_QUEUE, JSON_TURN_GRACE, JSON_TURNS_PER_BYTE};
