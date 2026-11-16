//! A tree's scripts name nextest profiles the root configuration defines
//! (skein/docs/design/ui/web/testing.md, 1). The check reads `.config/nextest.toml`'s
//! `[profile.NAME]` headers and every `--profile NAME` in the listed trees' `*.sh`. A
//! `--profile` with no name after it is a failure, never a skip. Each profile's own
//! wall is three times the gate's (`tools/tiers/design.md`, 11.7).

use std::collections::BTreeSet;
use std::fs;

const SCRIPTS: [&str; 1] = ["skein/scripts"];

fn timeout_seconds(profile: &str, table: &toml::Value, source: &str) -> u64 {
    let value = table
        .get("global-timeout")
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("{source}: profile {profile} needs a global-timeout string"));
    let (number, scale) = if let Some(number) = value.strip_suffix('s') {
        (number, 1)
    } else if let Some(number) = value.strip_suffix('m') {
        (number, 60)
    } else {
        panic!("{source}: profile {profile}: unsupported global-timeout {value:?}, expected Ns or Nm");
    };
    number
        .parse::<u64>()
        .ok()
        .and_then(|seconds| seconds.checked_mul(scale))
        .unwrap_or_else(|| panic!("{source}: profile {profile}: invalid global-timeout {value:?}"))
}

#[test]
fn a_profiles_own_wall_is_three_times_the_gates() {
    let root = skein_scratch::paths::workspace();
    let walls: toml::Value =
        toml::from_str(&fs::read_to_string(root.join(".config/gate/walls.toml")).expect("the gate walls are readable"))
            .expect("the gate walls are TOML");
    let config: toml::Value = toml::from_str(
        &fs::read_to_string(root.join(".config/nextest.toml")).expect("the nextest configuration is readable"),
    )
    .expect("the nextest configuration is TOML");
    let walls = walls.get("profile").and_then(toml::Value::as_table).expect("the gate walls name profiles");
    let profiles = config.get("profile").and_then(toml::Value::as_table).expect("nextest defines profiles");
    for name in ["default", "fuzzy"] {
        assert!(walls.contains_key(name), "the gate walls must name profile {name}");
    }
    for (name, table) in walls {
        let wall = timeout_seconds(name, table, ".config/gate/walls.toml");
        let profile =
            profiles.get(name).unwrap_or_else(|| panic!("gate wall profile {name} is not defined by nextest"));
        let own = timeout_seconds(name, profile, ".config/nextest.toml");
        assert_eq!(
            Some(own),
            wall.checked_mul(3),
            "profile {name}: gate wall {wall} s, own global-timeout {own} s; expected three times the gate wall"
        );
    }
}

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
