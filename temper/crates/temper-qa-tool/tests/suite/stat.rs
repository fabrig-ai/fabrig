//! The brief's path counts and generated-directory folding (qa/triage.md, 3).
use temper_qa_tool::{
    faces::{Changed, parse_numstat},
    stat_text,
};

fn text(path: &str, added: u64, deleted: u64) -> Changed {
    Changed { path: path.into(), added: Some(added), deleted: Some(deleted) }
}
fn binary(path: &str) -> Changed {
    Changed { path: path.into(), added: None, deleted: None }
}

#[test]
fn goldens_fold_once_and_all_paths_contribute_to_the_total() {
    assert_eq!(
        stat_text(&[
            text("x/golden/3", 10, 0),
            text("a/src/lib.rs", 3, 1),
            text("x/golden/1", 10, 0),
            text("x/golden/2", 10, 0),
        ]),
        "a/src/lib.rs | +3 -1\nx/golden/ | 3 files, +30 -0\n4 files changed, +33 -1\n"
    );
}

#[test]
fn directories_use_the_first_segment_and_keep_their_first_paths_place() {
    assert_eq!(
        stat_text(&[
            text("z/generated", 2, 1),
            text("temper/docs/help/reference/ctl-task.md", 4, 2),
            text("x/golden/generated/two", 5, 3),
            text("crates/c/src/generated/v1.rs", 3, 1),
            binary("x/golden/one"),
            text("x/golden.md", 1, 0),
            text("crates/c/src/generated/golden/v2.rs", 7, 4),
            binary("b/image.png"),
            binary("temper/docs/help/reference/image.png"),
            text("temper/docs/help/reference.md", 1, 0),
            text("x/golden0.rs", 1, 0),
            binary("y/generated/image.png"),
        ]),
        concat!(
            "b/image.png | binary\n",
            "crates/c/src/generated/ | 2 files, +10 -5\n",
            "temper/docs/help/reference.md | +1 -0\n",
            "temper/docs/help/reference/ | 2 files, +4 -2\n",
            "x/golden.md | +1 -0\n",
            "x/golden/ | 2 files, +5 -3\n",
            "x/golden0.rs | +1 -0\n",
            "y/generated/ | 1 files, +0 -0\n",
            "z/generated | +2 -1\n",
            "12 files changed, +24 -11\n",
        )
    );
    assert_eq!(stat_text(&[]), "0 files changed, +0 -0\n");
}

#[test]
fn numstat_reads_nul_records_without_splitting_or_unquoting_paths() {
    assert_eq!(
        parse_numstat("3\t1\ta space/\"quote\"\tand\nnewline\0-\t-\tx/golden/image.png\0").expect("numstat"),
        [text("a space/\"quote\"\tand\nnewline", 3, 1), binary("x/golden/image.png")]
    );
    assert_eq!(parse_numstat("").expect("empty"), []);
    for invalid in ["1\t0\tpath", "1\t0\t\0", "1\tpath\0", "-\t0\tpath\0", "1\t-\tpath\0", "x\t0\tpath\0"] {
        assert!(parse_numstat(invalid).is_err(), "{invalid:?}");
    }
}
