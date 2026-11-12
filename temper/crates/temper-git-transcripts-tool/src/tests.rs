use crate::{
    invocation::{base64, command},
    mask::Mask,
};
use std::{collections::BTreeMap, path::Path};
#[test]
fn the_environment_is_given_whole_and_holds_the_header_only_for_remote_operations() {
    for (name, identity, author) in [("fetch-branch", Some(("writer", "t")), "writer"), ("checkout-1", None, "temper")]
    {
        let cmd = command(
            Path::new("/usr/bin/git"),
            Path::new("/scratch/worker/home"),
            Path::new("/scratch/worker/workspaces/1/specimen"),
            identity,
            false,
        );
        let seen: BTreeMap<_, _> = cmd
            .get_envs()
            .map(|(k, v)| (k.to_str().expect("key"), v.expect("value").to_str().expect("text")))
            .collect();
        let mut expected = BTreeMap::from([
            ("HOME", "/scratch/worker/home"),
            ("LC_ALL", "C"),
            ("TZ", "UTC"),
            ("GIT_CONFIG_NOSYSTEM", "1"),
            ("GIT_CONFIG_GLOBAL", "/dev/null"),
            ("GIT_TERMINAL_PROMPT", "0"),
            ("GIT_ASKPASS", ""),
            ("GIT_COMMITTER_NAME", author),
            ("GIT_COMMITTER_EMAIL", if identity.is_some() { "writer@temper.invalid" } else { "temper@temper.invalid" }),
        ]);
        if identity.is_some() {
            expected.extend([
                ("GIT_CONFIG_COUNT", "1"),
                ("GIT_CONFIG_KEY_0", "http.extraHeader"),
                ("GIT_CONFIG_VALUE_0", "Authorization: Basic d3JpdGVyOnQ="),
            ]);
        }
        assert_eq!(seen, expected, "{name}");
    }
}
#[test]
fn masking_rewrites_ids_ports_and_the_scratch_path() {
    let mut mask = Mask::new(Path::new("/home/someone/temper/target/git-transcripts"), 41_823);
    let first = mask.bytes(b"To http://127.0.0.1:41823/fixture/specimen.git\n *\t9fceb02d0ae598e95dc970b74767f19372d61af8:refs/heads/topic\t[new branch]\n");
    let again =
        mask.bytes(b"/home/someone/temper/target/git-transcripts/worker 9fceb02d0ae598e95dc970b74767f19372d61af8\n");
    assert_eq!(first,b"To http://127.0.0.1:3000/fixture/specimen.git\n *\t0000000000000000000000000000000000000001:refs/heads/topic\t[new branch]\n".to_vec());
    assert_eq!(again, b"/scratch/worker 0000000000000000000000000000000000000001\n".to_vec(), "one map for the run");
    for (input, expected) in [
        (&b"0000000000000000000000000000000000000000"[..], &b"0000000000000000000000000000000000000000"[..]),
        (
            &b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"[..],
            &b"0000000000000000000000000000000000000000000000000000000000000001"[..],
        ),
        (&b"\xff port 41823 9fceb02"[..], &b"\xff port 3000 0000001"[..]),
        (&b"after 21 ms"[..], &b"after 0 ms"[..]),
    ] {
        let mut mask = Mask::new(Path::new("/scratch"), 41_823);
        assert_eq!(mask.bytes(input), expected, "{input:?}");
    }
}
#[test]
fn no_argument_holds_a_token_or_its_base64() {
    let token = "fixture-secret-token";
    let encoded = base64(&format!("writer:{token}"));
    let mut cmd = command(
        Path::new("/usr/bin/git"),
        Path::new("/scratch/home"),
        Path::new("/scratch/workspaces/1"),
        Some(("writer", token)),
        false,
    );
    cmd.args([
        "clone",
        "--no-checkout",
        "--no-tags",
        "--origin",
        "origin",
        "--",
        "http://127.0.0.1:3000/fixture/specimen.git",
        "specimen",
    ]);
    for arg in cmd.get_args() {
        let arg = arg.to_string_lossy();
        assert!(!arg.contains(token) && !arg.contains(&encoded), "argument holds a credential");
    }
}

#[test]
fn a_renamed_fetch_warned_of_its_redirect_holds_the_fact() {
    assert!(crate::cases::followed(
        b"0\n",
        b"warning: redirecting to http://127.0.0.1:3000/fixture/renamed.git/\nPOST git-upload-pack (285 bytes)\n"
    ));
}
#[test]
fn a_renamed_fetch_without_git_s_redirect_warning_fails_the_fact() {
    assert!(!crate::cases::followed(b"0\n", b"POST git-upload-pack (285 bytes)\n"));
}
#[test]
fn a_renamed_fetch_exiting_non_zero_fails_the_redirect_fact() {
    assert!(!crate::cases::followed(
        b"128\n",
        b"warning: redirecting to http://127.0.0.1:3000/fixture/renamed.git/\nPOST git-upload-pack (285 bytes)\n"
    ));
}
#[test]
fn a_redirect_to_the_old_name_fails_the_redirect_fact() {
    assert!(!crate::cases::followed(b"0\n", b"warning: redirecting to http://127.0.0.1:3000/fixture/specimen.git/\n"));
}
