use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::{
    Version,
    output::{self, Status},
};
use temper_worker_protocol_git_world::{fixtures::SIX, transcripts};

// The transcript's masked ids are literal bytes, independent of the parser.
const ONE: git::Commit =
    git::Commit::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const TWO: git::Commit =
    git::Commit::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const FOUR: git::Commit =
    git::Commit::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]);
const FETCHES: &[(&str, git::Commit)] = &[
    ("fetch-branch", ONE),
    ("fetch-commit", TWO),
    ("fetch-default", TWO),
    ("fetch-up-to-date", TWO),
    ("fetch-sha256-branch", FOUR),
    ("fetch-renamed", SIX),
];
const PUSHES: &[(&str, &[u8], Status)] = &[
    ("create-new-branch", b"created", Status::NewBranch),
    ("create-exists-at-commit", b"created", Status::UpToDate),
    ("create-exists-elsewhere", b"created", Status::StaleInfo),
    ("create-protected", b"blocked", Status::Declined),
    ("push-fast-forward", b"main", Status::FastForward),
    ("push-new-branch", b"pushed", Status::NewBranch),
    ("push-up-to-date", b"main", Status::UpToDate),
    ("push-non-fast-forward", b"main", Status::NonFastForward),
    ("push-lease-stale", b"main", Status::StaleInfo),
    ("push-no-permission", b"forbidden", Status::Declined),
    ("push-protected", b"main", Status::Declined),
];
#[test]
fn each_fetch_transcript_parses_to_its_fetched_commit() {
    for (case, commit) in FETCHES {
        let transcript = transcripts::read(case);
        assert_eq!(transcript.exit, 0, "{case}");
        assert_eq!(output::fetched(&transcript.stdout), Some(*commit), "{case}");
        assert!(transcript.argv.starts_with(b"git\n-c\ncore.hooksPath=/dev/null\n"), "{case}");
    }
}
#[test]
fn each_push_transcript_s_status_line_is_its_git_md_row() {
    for (case, branch, expected) in PUSHES {
        let transcript = transcripts::read(case);
        assert_eq!(output::status(&transcript.stdout, branch), Some(*expected), "{case}");
        assert_eq!(output::status(&transcript.stdout, b"absent"), None, "{case}");
    }
    for (bytes, expected) in [
        (
            b"!\t0000000000000000000000000000000000000001:refs/heads/main\t[rejected] (fetch first)\n".as_slice(),
            Status::FetchFirst,
        ),
        (b"+\t0000000000000000000000000000000000000001:refs/heads/main\tforced update\n", Status::Other),
    ] {
        assert_eq!(output::status(bytes, b"main"), Some(expected));
    }
}
#[test]
fn each_failure_transcript_classifies_as_section_six_says() {
    let rows = [
        ("clone-missing-repository", git::Kind::Clone, git::Fault::Missing { missing: git::Missing::Repository }),
        ("clone-stranger-private", git::Kind::Clone, git::Fault::Missing { missing: git::Missing::Repository }),
        ("fetch-missing-branch", git::Kind::Fetch, git::Fault::Missing { missing: git::Missing::Branch }),
        ("fetch-missing-commit", git::Kind::Fetch, git::Fault::Missing { missing: git::Missing::Commit }),
        ("fetch-wrong-token", git::Kind::Fetch, git::Fault::Refused),
        ("clone-wrong-token", git::Kind::Clone, git::Fault::Refused),
        ("clone-refused-connection", git::Kind::Clone, git::Fault::Unreachable),
    ];
    for (case, kind, fault) in rows {
        let transcript = transcripts::read(case);
        assert_ne!(transcript.exit, 0, "{case}");
        assert_eq!(output::classify(&transcript.stderr, kind), fault, "{case}");
    }
    // Section 6 reads status/stdout before stderr: these failures have a
    // declined status and only a generic failed-to-push stderr line.
    for (case, branch) in
        [("push-no-permission", b"forbidden".as_slice()), ("push-protected", b"main"), ("create-protected", b"blocked")]
    {
        let transcript = transcripts::read(case);
        assert_ne!(transcript.exit, 0, "{case}");
        assert_eq!(
            output::status(&transcript.stdout, branch),
            Some(Status::Declined),
            "{case}: Refused comes from stdout"
        );
        assert_eq!(
            output::classify(&transcript.stderr, git::Kind::Push),
            git::Fault::Broken,
            "{case}: stderr matches no row"
        );
    }
}
#[test]
fn the_version_transcript_parses_as_at_least_two_forty_one() {
    let transcript = transcripts::read("version");
    assert_eq!(transcript.exit, 0);
    assert_eq!(output::version(&transcript.stdout), Some(Version { major: 2, minor: 53, patch: 0 }));
    for (bytes, expected) in [
        (b"git version 2.41.1.windows.1\n".as_slice(), Some(Version { major: 2, minor: 41, patch: 1 })),
        (b"git version 2.41.1", None),
        (b"git version 2.41.x\n", None),
        (b"git version 4294967296.41.1\n", None),
    ] {
        assert_eq!(output::version(bytes), expected, "{bytes:?}");
    }
}
#[test]
fn mutated_outputs_parse_to_none_never_to_another_value() {
    for (case, expected) in FETCHES {
        for mutation in transcripts::mutations(&transcripts::read(case).stdout) {
            assert!(output::fetched(&mutation).is_none_or(|actual| actual == *expected), "{case}: {mutation:?}");
        }
    }
    for (case, branch, expected) in PUSHES {
        for mutation in transcripts::mutations(&transcripts::read(case).stdout) {
            assert!(output::status(&mutation, branch).is_none_or(|actual| actual == *expected), "{case}: {mutation:?}");
        }
    }
    for mutation in transcripts::mutations(&transcripts::read("version").stdout) {
        assert_eq!(output::version(&mutation), None, "{mutation:?}");
    }
    for case in ["merge-conflict-2", "commit-unmerged"] {
        let bytes = transcripts::read(case).stdout;
        let limits = temper_worker_domain_checkout::Limits {
            workspaces: 1,
            repositories: 1,
            name_bytes: 64,
            message_bytes: 64,
            conflicts: 2,
            path_bytes: 64,
            remote_timeout: skein_lib::Duration::from_secs(30),
            local_timeout: skein_lib::Duration::from_secs(5),
            facts: 2,
        };
        assert_eq!(
            output::paths(&bytes, &limits),
            Some(vec![Box::from(&b"conflict.txt"[..])].into_boxed_slice()),
            "{case}"
        );
        for mutation in transcripts::mutations(&bytes) {
            assert!(output::paths(&mutation, &limits).is_none_or(|paths| paths.is_empty()), "{case}: {mutation:?}");
        }
    }
}
