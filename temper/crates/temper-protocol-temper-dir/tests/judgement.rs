#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary test fixtures drive and inspect the forge requests and judgement"
)]

extern crate alloc;

mod support;

use skein_forge::api::{EntryKind, Error};
use skein_lib::Writer;
use std::path::Path;
use support::*;
use temper_protocol_temper_dir::{Fault, Home, Reading, Touch, Unread, Verdict, touched};

const GOOD_OFFERS: &[u8] = b"offer = [\"pair\"]\n";
const GOOD_HOOKS: &[u8] = b"[close]\ntimeout_seconds = 1800\n";
const NEWER: &str = "format 2; this temper-engine reads format 1: upgrade it, or write the file at format 1";
const CUT: &str = "past what temper reads, 65,536 bytes";
const PAST: &str = ".temper/: past what temper reads: 64 files";
const BAD_FORMAT: &str = "format: a whole number from 1";
const FRONT_MATTER: &str = "front matter holds format = N alone";
const BAD_NAME: &str = "is no name: 1 to 32 lowercase letters, digits and `-`, from a letter";
const CHARTER: &str = ".temper/charters/pair.md";

fn shown(fault: &Fault) -> String {
    let mut measure = Writer::measure(usize::MAX);
    fault.write(&mut measure).expect("measure the fault");
    let mut out = Writer::new(measure.written());
    fault.write(&mut out).expect("the measured fault fits");
    String::from_utf8(out.finish().into_vec()).expect("this fault uses UTF-8 names and words")
}

#[derive(Clone, Copy)]
enum At<'a> {
    Listing(&'a str),
    CutListing(&'a str),
    Entry(&'a str, EntryKind),
    File(&'a str, &'a [u8]),
    CutFile(&'a str),
}

fn tree(parts: &[At<'_>]) -> Tree {
    use At::{CutFile, CutListing, Entry, File, Listing};
    let mut tree = Tree::default();
    for part in parts {
        match part {
            Listing(path) => tree.dir(path),
            Entry(path, kind) => tree.entry(path, *kind),
            File(path, body) => tree.file(path, body),
            CutListing(path) | CutFile(path) => tree.cut(path),
        }
    }
    tree
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table of trees checks all faults with the same assertion")]
fn faults_name_the_entry_or_file_and_line() {
    use At::{CutFile, CutListing, Entry, File, Listing};
    use EntryKind::{Directory, Link, Submodule};
    // A row describes a complete forge tree; only its expected fault is synthesized here.
    type Case<'a> = (&'a [At<'a>], Home, (&'a str, u32, &'a str));
    let cases: &[Case<'_>] = &[
        (
            &[File(OFFERS, GOOD_OFFERS)],
            Home::Here,
            (OFFERS, 1, "`offer` names `pair`, and `.temper/ways/pair.toml` is missing"),
        ),
        (
            &[Listing(WAYS), Entry(PAIR, EntryKind::File), File(OFFERS, GOOD_OFFERS)],
            Home::Here,
            (OFFERS, 1, "`offer` names `pair`, and `.temper/ways/pair.toml` is missing"),
        ),
        (&[File(OFFERS, b"format = 2\nfuture_key = true\n")], Home::Here, (OFFERS, 1, NEWER)),
        (&[File(OFFERS, b"future_key = true\nformat = 0\n")], Home::Here, (OFFERS, 2, BAD_FORMAT)),
        (&[File(OFFERS, b"format = 1.0\n")], Home::Here, (OFFERS, 1, BAD_FORMAT)),
        (&[File(OFFERS, b"format = 0x1\n")], Home::Here, (OFFERS, 1, BAD_FORMAT)),
        (&[Listing(WAYS), File(PAIR, b"other = 3\nformat = 2\n")], Home::Here, (PAIR, 2, NEWER)),
        (&[Listing(WAYS), File(PAIR, b"format = \"1\"\n")], Home::Here, (PAIR, 1, BAD_FORMAT)),
        (&[Listing(WAYS), File(PAIR, b"# way\nnot toml\n")], Home::Here, (PAIR, 2, "does not parse as TOML")),
        (&[Listing(WAYS), File(PAIR, b"# way\n\xff\n")], Home::Here, (PAIR, 2, "not UTF-8")),
        (&[Listing(WAYS), File(PAIR, b"name = 2\nname = 3\n")], Home::Here, (PAIR, 2, "a TOML key is defined twice")),
        (&[Listing(WAYS), File(PAIR, b"[task]\n[task]\n")], Home::Here, (PAIR, 2, "a TOML table is defined twice")),
        (
            &[Listing(WAYS), File(PAIR, b"name = 9223372036854775808\n")],
            Home::Here,
            (PAIR, 1, "a TOML value is outside its range"),
        ),
        (&[Entry(OFFERS, Link)], Home::Here, (OFFERS, 0, "a symbolic link; temper reads a regular file here")),
        (&[Entry(OFFERS, Directory)], Home::Here, (OFFERS, 0, "a directory; temper reads a regular file here")),
        (&[Entry(CHARTERS, Submodule)], Home::Here, (CHARTERS, 0, "a submodule; temper reads a directory here")),
        (&[Entry(WAYS, EntryKind::File)], Home::Here, (WAYS, 0, "a regular file; temper reads a directory here")),
        (&[Entry(HOOKS, Link)], Home::Here, (HOOKS, 0, "a symbolic link; temper reads a directory here")),
        (
            &[Listing(WAYS), Entry(PAIR, Link)],
            Home::Here,
            (PAIR, 0, "a symbolic link; temper reads a regular file here"),
        ),
        (
            &[Listing(CHARTERS), Entry(CHARTER, Directory)],
            Home::Here,
            (CHARTER, 0, "a directory; temper reads a regular file here"),
        ),
        (
            &[Listing(WAYS), Entry(PAIR, Submodule)],
            Home::Here,
            (PAIR, 0, "a submodule; temper reads a regular file here"),
        ),
        (
            &[Listing(WAYS), File(".temper/ways/pair.txt", b"")],
            Home::Here,
            (".temper/ways/pair.txt", 0, "a regular file; temper reads a NAME.toml regular file here"),
        ),
        (
            &[Listing(CHARTERS), File(".temper/charters/pair.txt", b"")],
            Home::Here,
            (".temper/charters/pair.txt", 0, "a regular file; temper reads a NAME.md regular file here"),
        ),
        (
            &[Listing(WAYS), File(".temper/ways/Hand-over.toml", b"")],
            Home::Here,
            (
                ".temper/ways/Hand-over.toml",
                0,
                "`Hand-over` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter",
            ),
        ),
        (
            &[Listing(CHARTERS), File(".temper/charters/9.md", b"")],
            Home::Here,
            (".temper/charters/9.md", 0, "`9` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter"),
        ),
        (&[File(OFFERS, b"# offer not given\n")], Home::Here, (OFFERS, 0, "missing key `offer`")),
        (&[File(OFFERS, b"# wrong type\noffer = 3\n")], Home::Here, (OFFERS, 2, "`offer`: an array of way names")),
        (&[File(OFFERS, b"offer = []\n")], Home::Here, (OFFERS, 1, "`offer`: at least one way name")),
        (&[File(OFFERS, b"offer = [\n 7,\n]\n")], Home::Here, (OFFERS, 2, "`offer`: an array of way names")),
        (
            &[File(OFFERS, b"offer = [\"Bad\"]\n")],
            Home::Here,
            (OFFERS, 1, "`Bad` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter"),
        ),
        (
            &[Listing(WAYS), File(PAIR, b""), File(OFFERS, b"offer = [\"pair\",\n\"pair\"]\n")],
            Home::Here,
            (OFFERS, 2, "`offer` names `pair` twice"),
        ),
        (
            &[Listing(WAYS), File(PAIR, b""), File(OFFERS, b"offer = [\"pair\"]\n[default]\nchange = \"x\"\n")],
            Home::Here,
            (OFFERS, 3, "`default.change` names `x`, which `offer` doesn't"),
        ),
        (
            &[Listing(WAYS), File(PAIR, b""), File(OFFERS, b"offer = [\"pair\"]\ndefault = 3\n")],
            Home::Here,
            (OFFERS, 2, "`default`: a table of start commands"),
        ),
        (
            &[Listing(WAYS), File(PAIR, b""), File(OFFERS, b"offer = [\"pair\"]\ndefault.change = 7\n")],
            Home::Here,
            (OFFERS, 2, "`default.change`: a way name"),
        ),
        (
            &[Listing(WAYS), File(PAIR, b""), File(OFFERS, b"offer = [\"pair\"]\ndefault.change = \"Bad\"\n")],
            Home::Here,
            (OFFERS, 2, "`Bad` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter"),
        ),
        (
            &[Listing(WAYS), File(PAIR, b""), File(OFFERS, b"offer = [\"pair\"]\ndefault.repair = \"pair\"\n")],
            Home::Here,
            (OFFERS, 2, "unknown key `default.repair`"),
        ),
        (
            &[Listing(WAYS), File(PAIR, b""), File(OFFERS, b"offer = [\"pair\"]\ndefaults = true\n")],
            Home::Here,
            (OFFERS, 2, "unknown key `defaults`"),
        ),
        (
            &[Listing(CHARTERS)],
            Home::Elsewhere { name: b"owner/repo".as_slice().into() },
            (CHARTERS, 0, "a project's ways live in its home repository, owner/repo"),
        ),
        (
            &[Listing(CHARTERS)],
            Home::None,
            (CHARTERS, 0, "a project's ways live in its home repository, and it has none"),
        ),
        (
            &[Listing(HOOKS), Entry(CONFIG, Link)],
            Home::Here,
            (CONFIG, 0, "a symbolic link; temper reads a regular file here"),
        ),
        (
            &[Listing(HOOKS), File(CONFIG, b"# hook config\nformat = 2\n[close]\ntimeout_seconds = 2\n")],
            Home::Here,
            (CONFIG, 2, NEWER),
        ),
        (&[Listing(HOOKS), File(CONFIG, b"format = 0\n")], Home::Here, (CONFIG, 1, BAD_FORMAT)),
        (&[Listing(HOOKS), File(CONFIG, b"[merge]\ntimeout_seconds = 0\n")], Home::Here, (CONFIG, 2, "a value of 0")),
        (
            &[Listing(HOOKS), File(CONFIG, b"[merge]\ndeadline = 2\n")],
            Home::Here,
            (CONFIG, 2, "unknown key `deadline`"),
        ),
        (&[Listing(CHARTERS), File(CHARTER, b"# Pair\n\xff\n")], Home::Here, (CHARTER, 2, "not UTF-8")),
        (
            &[Listing(CHARTERS), File(CHARTER, b"+++\nformat = 1\nextra = 1\n+++\n")],
            Home::Here,
            (CHARTER, 3, FRONT_MATTER),
        ),
        (&[Listing(CHARTERS), File(CHARTER, b"+++\nextra = 1\n+++\n")], Home::Here, (CHARTER, 2, FRONT_MATTER)),
        (&[Listing(CHARTERS), File(CHARTER, b"+++\nformat = 1\n")], Home::Here, (CHARTER, 3, FRONT_MATTER)),
        (&[Listing(CHARTERS), File(CHARTER, b"+++\nformat = 0\n+++\n")], Home::Here, (CHARTER, 2, BAD_FORMAT)),
        (&[Listing(CHARTERS), File(CHARTER, b"+++\nformat = 2\n+++\n")], Home::Here, (CHARTER, 2, NEWER)),
        (&[File(OFFERS, b"this would not parse"), CutFile(OFFERS)], Home::Here, (OFFERS, 0, CUT)),
        (&[Listing(ROOT), CutListing(ROOT)], Home::Here, (ROOT, 0, CUT)),
        (&[Listing(WAYS), CutListing(WAYS), File(OFFERS, GOOD_OFFERS)], Home::Here, (WAYS, 0, CUT)),
    ];
    for (parts, home, (path, line, why)) in cases {
        let (reading, _) = tree(parts).read(FILE_BYTES);
        let home = match home {
            Home::Here => Home::Here,
            Home::None => Home::None,
            Home::Elsewhere { name } => Home::Elsewhere { name: name.clone() },
        };
        let verdict = reading.judge(home);
        let expected_fault = Fault { path: path.as_bytes().into(), line: *line, why: why.as_bytes().into() };
        assert_eq!(verdict.faults.as_ref(), [expected_fault], "{path}: {why}");
        assert_eq!(verdict.more, 0);
    }
}

#[test]
fn method_means_any_listed_method_entry_not_hooks() {
    use At::{Entry, File, Listing};
    let cases: &[(&[At<'_>], bool)] = &[
        (&[Listing(CHARTERS)], true),
        (&[Entry(OFFERS, EntryKind::Link)], true),
        (&[Listing(WAYS), File(PAIR, b"")], true),
        (&[Listing(HOOKS), File(CONFIG, GOOD_HOOKS)], false),
        (&[Listing(HOOKS), Entry(".temper/hooks/pre-pr", EntryKind::Link)], false),
        (&[Entry(".temper/README.md", EntryKind::File)], false),
        (&[Entry(WAYS, EntryKind::Directory)], false), // The listed directory is Missing.
        (&[], false),
    ];
    for (parts, method) in cases {
        let (reading, _) = tree(parts).read(FILE_BYTES);
        let verdict = reading.judge(Home::Here);
        assert_eq!(verdict.method, *method);
        if !method {
            assert!(verdict.faults.is_empty() && verdict.more == 0, "{verdict:?}");
        }
    }
}

#[test]
fn reading_requests_sorted_listings_then_sorted_regular_files() {
    use At::{Entry, File, Listing};
    const ANSWER_BUDGET: u32 = 8_192;
    let tree = tree(&[
        File(OFFERS, GOOD_OFFERS),
        Listing(WAYS),
        File(PAIR, b""),
        File(".temper/ways/hand-it-over.toml", b""),
        Entry(".temper/ways/link.toml", EntryKind::Link),
        Listing(HOOKS),
        File(CONFIG, GOOD_HOOKS),
        Entry(".temper/hooks/pre-pr", EntryKind::Link),
        Listing(CHARTERS),
        File(CHARTER, b"Charter"),
        Entry(".temper/ignored", EntryKind::Directory),
    ]);
    for budget in [HOOK_BYTES / 2, ANSWER_BUDGET, FILE_BYTES + 1] {
        let mut reading = Reading::new(HEAD, budget);
        for (path, limit) in [
            (ROOT, FILE_BYTES),
            (CHARTERS, FILE_BYTES),
            (HOOKS, FILE_BYTES),
            (WAYS, FILE_BYTES),
            (CHARTER, FILE_BYTES),
            (CONFIG, HOOK_BYTES),
            // '.' sorts before '/'.
            (OFFERS, FILE_BYTES),
            (".temper/ways/hand-it-over.toml", FILE_BYTES),
            (PAIR, FILE_BYTES),
        ] {
            let request = reading.next().expect("more paths to read");
            assert_eq!(request_file(request), (HEAD, path.to_owned(), limit.min(budget)));
            reading.take(&tree.answer(HEAD, path, limit.min(budget))).expect("matching answer");
        }
        assert!(reading.next().is_none());
        let method = reading.method_files();
        let files: Vec<_> = method.iter().map(|item| (item.path.as_ref(), item.bytes.as_ref())).collect();
        assert_eq!(
            files,
            [
                (CHARTER.as_bytes(), b"Charter".as_slice()),
                (OFFERS.as_bytes(), GOOD_OFFERS),
                (b".temper/ways/hand-it-over.toml", b""),
                (PAIR.as_bytes(), b""),
            ]
        );
    }
}

#[test]
fn wrong_answer_does_not_advance_the_pending_read() {
    const OTHER_HEAD: [u8; 32] = [0x37; 32];
    let mut reading = Reading::new(HEAD, FILE_BYTES);
    let initial = reading.next();
    for response in [
        Ok(directory(OTHER_HEAD, ROOT, &[], false)),
        Ok(directory(HEAD, CHARTERS, &[], false)),
        Ok(file(HEAD, ROOT, b"", false)),
        Ok(skein_forge::api::Answer::Done),
        Err(Error::Forbidden),
    ] {
        assert_eq!(reading.take(&response), Err(Unread));
        assert_eq!(reading.next(), initial);
    }
    let tree = tree(&[At::File(OFFERS, GOOD_OFFERS)]);
    reading.take(&tree.answer(HEAD, ROOT, FILE_BYTES)).expect("matching listing");
    let pending = reading.next();
    for response in [
        Ok(file(OTHER_HEAD, OFFERS, b"", false)),
        Ok(file(HEAD, PAIR, b"", false)),
        Ok(directory(HEAD, OFFERS, &[], false)),
    ] {
        assert_eq!(reading.take(&response), Err(Unread));
        assert_eq!(reading.next(), pending);
    }
    reading.take(&Err(Error::Missing)).expect("Missing answers a pending file");
    assert!(reading.next().is_none());
    assert_eq!(reading.take(&Err(Error::Missing)), Err(Unread));
}

#[test]
fn exact_file_count_holds_and_next_does_not_overread() {
    for (count, expected) in [(MAX_FILES, &[][..]), (MAX_FILES + 1, &[PAST][..])] {
        let mut tree = tree(&[At::Listing(WAYS)]);
        for index in (0..count).rev() {
            tree.file(&format!("{WAYS}/way-{index:02}.toml"), b"");
        }
        let (reading, paths) = tree.read(FILE_BYTES);
        assert_eq!(paths.len(), MAX_FILES + 2, "64 files and two listings");
        assert_eq!(paths.last().expect("last requested file"), ".temper/ways/way-63.toml");
        let faults: Vec<_> = reading.judge(Home::Here).faults.iter().map(shown).collect();
        assert_eq!(faults, expected);
    }
}

#[test]
fn total_bytes_hold_at_the_bound_but_not_past() {
    const FULL_FILE: usize = 65_536;
    const FULL_FILES: usize = MAX_TOTAL / FULL_FILE;
    let body = vec![b' '; FULL_FILE];
    for (last, more, requests, kept, expected) in [
        (FULL_FILE, &[][..], FULL_FILES, FULL_FILES, &[][..]),
        (FULL_FILE, &[b"".as_slice()][..], FULL_FILES + 1, FULL_FILES + 1, &[][..]),
        (FULL_FILE, &[b"x".as_slice(), b"later"][..], FULL_FILES + 1, FULL_FILES, &[PAST][..]),
        (FULL_FILE - 1, &[b"xy".as_slice(), b"later"][..], FULL_FILES + 1, FULL_FILES, &[PAST][..]),
    ] {
        let mut tree = tree(&[At::Listing(CHARTERS)]);
        for index in 0..FULL_FILES - 1 {
            tree.file(&format!("{CHARTERS}/a-{index}.md"), &body);
        }
        tree.file(&format!("{CHARTERS}/b.md"), &body[..last]);
        for (index, bytes) in more.iter().enumerate() {
            tree.file(&format!("{CHARTERS}/c-{index}.md"), bytes);
        }
        let (reading, paths) = tree.read(FILE_BYTES);
        assert_eq!(paths.len(), requests + 2, "requests, including two listings");
        assert_eq!(reading.method_files().len(), kept);
        let faults: Vec<_> = reading.judge(Home::Here).faults.iter().map(shown).collect();
        assert_eq!(faults, expected);
    }
}

#[test]
fn first_sixteen_faults_are_kept_in_judgement_order() {
    const NAME_FAULTS: usize = 12;
    const FILE_FAULTS: usize = 6;
    const HOME_FAULTS: usize = 2;
    let mut tree = tree(&[At::Listing(CHARTERS), At::Listing(WAYS)]);
    for index in (0..NAME_FAULTS).rev() {
        tree.entry(&format!("{WAYS}/Not-{index:02}.toml"), EntryKind::File);
    }
    for index in (0..FILE_FAULTS).rev() {
        tree.file(&format!("{CHARTERS}/a-{index}.md"), b"+++\nformat = 0\n+++\n");
    }
    let (reading, _) = tree.read(FILE_BYTES);
    let verdict = reading.judge(Home::None);
    let expected: Vec<_> = (0..NAME_FAULTS)
        .map(|i| format!("{WAYS}/Not-{i:02}.toml: `Not-{i:02}` {BAD_NAME}"))
        .chain((0..MAX_FAULTS - NAME_FAULTS).map(|i| format!("{CHARTERS}/a-{i}.md:2: {BAD_FORMAT}")))
        .collect();
    assert_eq!(verdict.faults.iter().map(shown).collect::<Vec<_>>(), expected);
    assert_eq!(verdict.more, u32::try_from(NAME_FAULTS + FILE_FAULTS + HOME_FAULTS - MAX_FAULTS).expect("small table"));
}

#[test]
fn faults_within_offers_follow_lines_then_home_is_last() {
    use At::{File, Listing};
    let (reading, _) = tree(&[
        Listing(WAYS),
        File(OFFERS, b"unknown = true\noffer = [\n  \"pair\",\n  \"Wrong\",\n]\n[default]\nchange = \"x\"\n"),
    ])
    .read(FILE_BYTES);
    let verdict = reading.judge(Home::None);
    assert_eq!(
        verdict.faults.iter().map(shown).collect::<Vec<_>>(),
        [
            ".temper/ways.toml:1: unknown key `unknown`",
            ".temper/ways.toml:3: `offer` names `pair`, and `.temper/ways/pair.toml` is missing",
            ".temper/ways.toml:4: `Wrong` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter",
            ".temper/ways.toml:7: `default.change` names `x`, which `offer` doesn't",
            ".temper/ways: a project's ways live in its home repository, and it has none",
            ".temper/ways.toml: a project's ways live in its home repository, and it has none",
        ]
    );
}

#[test]
fn touched_uses_paths_and_the_comparisons_completeness() {
    for (paths, whole, want) in [
        (&[CONFIG, OFFERS][..], false, Touch::Method),
        (&[PAIR][..], true, Touch::Method),
        (&[CHARTER][..], false, Touch::Method),
        (&[CONFIG][..], true, Touch::Temper),
        (&[".temper/ways-other/x.toml"][..], false, Touch::Temper),
        (&["src/.temper/ways.toml", ".temper-other/ways.toml"][..], true, Touch::None),
        (&["README.md"][..], false, Touch::Unknown),
        (&[][..], false, Touch::Unknown),
        (&[][..], true, Touch::None),
    ] {
        let input: Vec<_> = paths.iter().map(|path| path.as_bytes().into()).collect();
        assert_eq!(touched(&input, whole), want, "{paths:?}, whole: {whole}");
    }
}

fn load_fixture(tree: &mut Tree, path: &str, dir: &Path) {
    tree.dir(path);
    for entry in dir.read_dir().expect("open fixture directory") {
        let entry = entry.expect("read fixture directory entry");
        let path = format!("{path}/{}", entry.file_name().to_str().expect("frozen fixture name is UTF-8"));
        let full = entry.path();
        if entry.file_type().expect("read fixture entry kind").is_dir() {
            load_fixture(tree, &path, &full);
        } else {
            tree.file(&path, &std::fs::read(full).expect("read fixture bytes"));
        }
    }
}

#[test]
fn frozen_format_one_tree_reads_with_no_faults() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples/formats/temper-dir/1");
    let mut tree = Tree::default();
    load_fixture(&mut tree, ROOT, &root);
    let (reading, _) = tree.read(FILE_BYTES);
    assert_eq!(reading.judge(Home::Here), Verdict { method: true, faults: Box::new([]), more: 0 });
}

mod fuzzy {
    use super::*;
    use skein_lib::Rng;

    const SEEDS: u64 = 128;
    const BODY_BYTES: u64 = 192;
    const KINDS: &[EntryKind] = &[EntryKind::File, EntryKind::Directory, EntryKind::Link, EntryKind::Submodule];
    const NAMES: &[&str] = &["way", "a-9", "Way", "3", "-", "é", "", "long-and-longer-than-thirty-two-bytes"];

    fn pick<T: Copy>(rng: &mut Rng, items: &[T]) -> T {
        let count = u64::try_from(items.len()).expect("small fixture table");
        items[usize::try_from(rng.below(count)).expect("index in fixture table")]
    }

    fn random_tree(seed: u64) -> Tree {
        let mut rng = Rng::new(seed);
        let mut tree = Tree::default();
        for (directory, suffix) in [(CHARTERS, "md"), (HOOKS, "sh"), (WAYS, "toml")] {
            let kind = pick(&mut rng, KINDS);
            if kind != EntryKind::Directory {
                tree.entry(directory, kind);
                continue;
            }
            tree.dir(directory);
            for index in 0..rng.below(u64::try_from(MAX_FILES / 3).expect("small file count")) {
                let name = pick(&mut rng, NAMES);
                let path = format!("{directory}/{name}-{index}.{suffix}");
                let kind = pick(&mut rng, KINDS);
                if kind == EntryKind::File {
                    tree.file(&path, &random_body(&mut rng));
                } else {
                    tree.entry(&path, kind);
                }
            }
            if directory == HOOKS {
                tree.file(CONFIG, &random_body(&mut rng));
            }
        }
        if rng.chance(500) {
            tree.file(OFFERS, &random_body(&mut rng));
        }
        tree
    }

    fn random_body(rng: &mut Rng) -> Vec<u8> {
        match rng.below(6) {
            0 => GOOD_OFFERS.to_vec(),
            1 => b"format = 2\n".to_vec(),
            2 => b"+++\nformat = 1\n+++\n".to_vec(),
            3 => GOOD_HOOKS.to_vec(),
            4 => Vec::new(),
            choice => {
                assert_eq!(choice, 5, "choice is the random-body case");
                (0..rng.below(BODY_BYTES)).map(|_| rng.next_u64().to_le_bytes()[0]).collect()
            }
        }
    }

    #[test]
    fn bounded_trees_and_verdicts_replay_from_the_seed() {
        for seed in 0..SEEDS {
            let (reading, reads) = random_tree(seed).read(FILE_BYTES);
            let (again, read_again) = random_tree(seed).read(FILE_BYTES);
            let verdict = reading.judge(Home::None);
            assert_eq!(reads, read_again, "seed {seed}");
            assert_eq!(verdict, reading.judge(Home::None), "seed {seed}: judgement changed");
            assert_eq!(verdict, again.judge(Home::None), "seed {seed}: tree did not replay");
            assert!(verdict.faults.len() <= MAX_FAULTS, "seed {seed}: too many faults");
            assert!(reads.len() <= MAX_FILES + 4, "seed {seed}: too many reads");
            let files = reading.method_files();
            let retained: usize = files.iter().map(|file| file.bytes.len()).sum();
            assert!(retained <= MAX_TOTAL, "seed {seed}: retained too many bytes");
        }
    }

    fn good_tree_with_way(seed: u64, kind: EntryKind) -> (Tree, String) {
        let path = format!("{WAYS}/way-{:02x}.toml", Rng::new(seed).next_u64());
        let mut tree =
            tree(&[At::Listing(CHARTERS), At::Listing(WAYS), At::File(PAIR, b""), At::File(OFFERS, GOOD_OFFERS)]);
        if kind == EntryKind::File {
            tree.file(&path, b"description = \"Not offered but still judged\"\n");
        } else {
            tree.entry(&path, kind);
        }
        (tree, path)
    }

    #[test]
    fn changing_one_entry_adds_its_fault_to_a_good_tree() {
        for seed in 0..SEEDS {
            let (good, _) = good_tree_with_way(seed, EntryKind::File);
            let (reading, _) = good.read(FILE_BYTES);
            assert_eq!(reading.judge(Home::Here), Verdict { method: true, faults: Box::new([]), more: 0 });

            let (kind, why) = pick(
                &mut Rng::new(seed),
                &[
                    (EntryKind::Directory, "a directory; temper reads a regular file here"),
                    (EntryKind::Link, "a symbolic link; temper reads a regular file here"),
                    (EntryKind::Submodule, "a submodule; temper reads a regular file here"),
                ],
            );
            let (bad, path) = good_tree_with_way(seed, kind);
            let (reading, _) = bad.read(FILE_BYTES);
            let verdict = reading.judge(Home::Here);
            assert_eq!(verdict.faults.iter().map(shown).collect::<Vec<_>>(), [format!("{path}: {why}")], "seed {seed}");
            assert_eq!(verdict.more, 0);
        }
    }
}
