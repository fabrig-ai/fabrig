//! A tree's scripts name nextest profiles the root configuration defines
//! (skein/docs/design/ui/web/testing.md, 1). The check reads `.config/nextest.toml`'s
//! `[profile.NAME]` headers and every `--profile NAME` in the listed trees' `*.sh`. A
//! `--profile` with no name after it is a failure, never a skip. Each profile's own
//! wall is three times the gate's (`tools/tiers/design.md`, 11.7). Every profile doubles
//! its slow timeouts on macOS; only the helper's `browser-temper` global timeout
//! exceeds the root profile's (`temper/docs/design/testing/macos.md`, 7).

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
fn macos_global_timeouts_name_browser_temper_alone() {
    const MACOS: &str = "tools/macos/global-timeouts.toml";
    const NEXTEST: &str = ".config/nextest.toml";
    let root = skein_scratch::paths::workspace();
    let overrides: toml::Table =
        toml::from_str(&fs::read_to_string(root.join(MACOS)).expect("macOS timeouts are readable"))
            .expect("macOS timeouts are TOML");
    assert_eq!(overrides.len(), 1, "{MACOS}: only the profile table is allowed");
    let overrides = overrides.get("profile").and_then(toml::Value::as_table).expect("macOS timeouts name profiles");
    assert!(overrides.len() == 1 && overrides.contains_key("browser-temper"), "{MACOS}: only browser-temper is named");
    let config: toml::Value =
        toml::from_str(&fs::read_to_string(root.join(NEXTEST)).expect("nextest config is readable"))
            .expect("nextest config is TOML");
    let profiles = config.get("profile").and_then(toml::Value::as_table).expect("nextest defines profiles");
    for (name, table) in overrides {
        assert_eq!(table.as_table().map(toml::Table::len), Some(1), "{MACOS}: {name} must hold only global-timeout");
        let profile = profiles.get(name).unwrap_or_else(|| panic!("{MACOS}: nextest does not define profile {name}"));
        let macos = timeout_seconds(name, table, MACOS);
        let linux = timeout_seconds(name, profile, NEXTEST);
        assert!(macos > linux, "profile {name}: macOS global-timeout {macos} s must exceed Linux's {linux} s");
    }
}

fn slow_timeout(value: &toml::Value, context: &str) -> (u64, Option<i64>) {
    let period = value
        .as_str()
        .or_else(|| value.get("period").and_then(toml::Value::as_str))
        .unwrap_or_else(|| panic!("{context}: slow-timeout needs a period"));
    let milliseconds = [("ms", 1), ("s", 1000), ("m", 60_000), ("h", 3_600_000)]
        .into_iter()
        .find_map(|(suffix, scale)| period.strip_suffix(suffix).map(|number| (number, scale)))
        .and_then(|(number, scale)| number.parse::<u64>().ok()?.checked_mul(scale))
        .unwrap_or_else(|| panic!("{context}: unsupported slow-timeout period {period:?}"));
    let terminate_after = value
        .get("terminate-after")
        .map(|count| count.as_integer().unwrap_or_else(|| panic!("{context}: terminate-after must be an integer")));
    (milliseconds, terminate_after)
}

#[test]
fn every_profiles_macos_overrides_double_its_linux_slow_timeouts() {
    let root = skein_scratch::paths::workspace();
    let config: toml::Value = toml::from_str(
        &fs::read_to_string(root.join(".config/nextest.toml")).expect("the nextest configuration is readable"),
    )
    .expect("the nextest configuration is TOML");
    let profiles = config.get("profile").and_then(toml::Value::as_table).expect("nextest defines profiles");
    let default = profiles.get("default").expect("nextest defines the default profile");
    let overrides =
        |table: &toml::Value| table.get("overrides").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let macos = "cfg(target_os = \"macos\")";
    for (name, profile) in profiles {
        let own = overrides(profile);
        let inherited = if name == "default" { Vec::new() } else { overrides(default) };
        let mut seen = BTreeSet::new();
        let mut expected = Vec::new();
        for entry in own.iter().chain(&inherited) {
            if entry.get("platform").is_some() {
                continue;
            }
            if let Some(timeout) = entry.get("slow-timeout") {
                let filter = entry.get("filter").and_then(toml::Value::as_str);
                if seen.insert(filter) {
                    expected
                        .push((filter, slow_timeout(timeout, &format!("profile {name}: Linux override {filter:?}"))));
                }
            }
        }
        let fallback = profile
            .get("slow-timeout")
            .or_else(|| default.get("slow-timeout"))
            .unwrap_or_else(|| panic!("profile {name}: needs a slow-timeout fallback"));
        expected.push((None, slow_timeout(fallback, &format!("profile {name}: fallback"))));
        let leading =
            own.iter().take_while(|entry| entry.get("platform").and_then(toml::Value::as_str) == Some(macos)).count();
        assert_eq!(
            leading,
            expected.len(),
            "profile {name}: macOS override prefix must cover every Linux slow-timeout override and fallback: {expected:?}"
        );
        for (index, (entry, (filter, (period, terminate_after)))) in own.iter().zip(expected).enumerate() {
            let context = format!("profile {name}: macOS override {index}, expected filter {filter:?}");
            let table = entry.as_table().unwrap_or_else(|| panic!("{context}: expected a table"));
            assert!(
                table.keys().all(|key| ["platform", "filter", "slow-timeout"].contains(&key.as_str())),
                "{context}: unexpected key"
            );
            assert_eq!(entry.get("filter").and_then(toml::Value::as_str), filter, "{context}: filter or order differs");
            let actual = slow_timeout(
                entry.get("slow-timeout").unwrap_or_else(|| panic!("{context}: needs slow-timeout")),
                &context,
            );
            assert_eq!(Some(actual.0), period.checked_mul(2), "{context}: period must double");
            assert_eq!(actual.1, terminate_after, "{context}: terminate-after must stay unchanged");
        }
        assert!(
            own.iter().skip(leading).all(|entry| entry.get("platform").and_then(toml::Value::as_str) != Some(macos)),
            "profile {name}: macOS override must be in the prefix"
        );
    }
}

#[test]
fn macos_nextest_profiles_name_their_junit_path() {
    let root = skein_scratch::paths::workspace();
    let config: toml::Value = toml::from_str(
        &fs::read_to_string(root.join(".config/nextest.toml")).expect("the nextest configuration is readable"),
    )
    .expect("the nextest configuration is TOML");
    for name in ["process-temper", "browser-temper", "browser-jig", "browser-skein", "live-temper", "live-skein"] {
        let path = config
            .get("profile")
            .and_then(|profiles| profiles.get(name))
            .and_then(|profile| profile.get("junit"))
            .and_then(|junit| junit.get("path"))
            .and_then(toml::Value::as_str);
        assert_eq!(path, Some("junit.xml"), "profile {name}: macOS nextest runs need a JUnit path");
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
