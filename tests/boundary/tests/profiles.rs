//! A tree's scripts name nextest profiles the root configuration defines
//! (skein/docs/design/ui/web/testing.md, 1). The check reads `.config/nextest.toml`'s
//! `[profile.NAME]` headers and every `--profile NAME` in the listed trees' `*.sh`. A
//! `--profile` with no name after it is a failure, never a skip.

use std::collections::BTreeSet;
use std::fs;

const SCRIPTS: [&str; 1] = ["skein/scripts"];

#[test]
fn a_scripts_nextest_profile_is_defined() {
    let root = skein_scratch::paths::workspace();
    let config = fs::read_to_string(root.join(".config/nextest.toml")).expect("the nextest configuration is readable");
    let profiles: BTreeSet<&str> = config
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("[profile.").and_then(|rest| rest.strip_suffix(']')))
        .filter(|name| !name.is_empty() && !name.contains('.'))
        .collect();
    assert!(!profiles.is_empty(), "the check reads nextest profile definitions");

    let mut read = 0;
    for directory in SCRIPTS {
        let mut scripts: Vec<_> = fs::read_dir(root.join(directory))
            .expect("the scripts directory is readable")
            .map(|entry| entry.expect("the script entry is readable").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "sh"))
            .collect();
        scripts.sort();
        for path in scripts {
            let source = fs::read_to_string(&path).expect("the script is readable");
            let relative = path.strip_prefix(&root).expect("the script is in the repository");
            for (line, source) in source.lines().enumerate() {
                if source.trim_start().starts_with('#') {
                    continue;
                }
                let mut words = source.split_whitespace();
                while let Some(word) = words.next() {
                    if word == "--profile" {
                        let name = words
                            .next()
                            .unwrap_or_else(|| panic!("{}:{}: --profile has no name", relative.display(), line + 1));
                        assert!(
                            profiles.contains(name),
                            "{}:{}: --profile {name} is not defined in .config/nextest.toml",
                            relative.display(),
                            line + 1
                        );
                        read += 1;
                    }
                }
            }
        }
    }
    assert!(read > 0, "the check reads at least one --profile argument");
}
