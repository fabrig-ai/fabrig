#[test]
fn an_unset_forgejo_variable_is_refused_naming_it() {
    let tmp = skein_scratch::paths::target_tmp();
    let output = std::process::Command::new(skein_scratch::paths::binary("git-transcripts"))
        .args(["--git", "/usr/bin/git", "--scratch"])
        .arg(tmp.join("refusal-scratch"))
        .arg("--out")
        .arg(tmp.join("refusal-out"))
        .env_remove("TEMPER_FORGEJO")
        .output()
        .expect("the tool runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"Error: \"TEMPER_FORGEJO must name the pinned Forgejo\"\n");
}
