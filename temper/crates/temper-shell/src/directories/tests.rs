use super::*;
use crate::test_disk::{Disk, USER};
use skein_fake_machine::Item;
#[test]
fn roots_are_private_and_checked_at_their_own_names() {
    let target = Path::new("/parent/root");
    for (row, entries, private) in [
        ("absent", vec![], true),
        ("private", vec![Item::directory(b"parent"), Item::directory(b"parent/root").mode(0o700)], true),
        ("public", vec![Item::directory(b"parent"), Item::directory(b"parent/root").mode(0o755)], false),
        (
            "link",
            vec![
                Item::directory(b"parent"),
                Item::directory(b"private-target").mode(0o700),
                Item::link(b"parent/root", b"../private-target"),
            ],
            false,
        ),
        ("file", vec![Item::directory(b"parent"), Item::file(b"parent/root", b"").mode(0o600)], false),
        ("parent-link", vec![Item::directory(b"real"), Item::link(b"parent", b"real")], true),
    ] {
        let mut disk = Disk::new(&entries);
        let result = make_private_with(target, USER, &mut disk);
        if private {
            result.expect(row);
            assert_eq!(disk.symlink_stat(target).expect(row).mode, 0o700, "{row}");
        } else {
            assert_eq!(
                result.expect_err(row),
                format!("{}: must be a private directory of this user's, not reached through a link", target.display()),
                "{row}"
            );
        }
        if row == "absent" {
            assert_eq!(disk.symlink_stat(Path::new("/parent")).expect("made parent").mode, 0o700);
        }
        if row == "parent-link" {
            assert_eq!(disk.symlink_stat(Path::new("/real/root")).expect("made through link").mode, 0o700);
        }
    }
}

fn args(values: &[&str]) -> Vec<Argument> {
    values.iter().map(|s| Box::from(s.as_bytes())).collect()
}
#[test]
fn settings_resolve_with_their_origins_and_keep_the_remaining_arguments() {
    for (name, arguments, environment, config, origin, rest) in [
        ("flag", vec!["--config", "/flag"], vec![("TEMPER_CONFIG", "/variable")], "/flag", Origin::Flag, vec![]),
        ("variable", vec![], vec![("TEMPER_CONFIG", "/variable")], "/variable", Origin::Variable, vec![]),
        ("base", vec![], vec![("XDG_CONFIG_HOME", "/xdg")], "/xdg/temper", Origin::Base, vec![]),
        ("home", vec![], vec![], "/home/test/.config/temper", Origin::Home, vec![]),
        ("empty", vec![], vec![("TEMPER_CONFIG", "")], "/home/test/.config/temper", Origin::Home, vec![]),
        ("relative", vec!["--config", "../cfg"], vec![], "/work/../cfg", Origin::Flag, vec![]),
        (
            "positionals",
            vec!["one", "--config", "/cfg", "two", "--data", "/d", "three", "--cache", "/k"],
            vec![],
            "/cfg",
            Origin::Flag,
            vec!["one", "two", "three"],
        ),
        (
            "separator",
            vec!["--", "--config", "/cfg"],
            vec![],
            "/home/test/.config/temper",
            Origin::Home,
            vec!["--", "--config", "/cfg"],
        ),
    ] {
        let mut env: Vec<EnvironmentEntry> =
            environment.iter().map(|(k, v)| (Box::from(k.as_bytes()), Box::from(v.as_bytes()))).collect();
        env.push((Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice())));
        let (found, remaining) = resolve(&args(&arguments), &env, b"/work").expect(name);
        assert_eq!(found.config.as_ref(), config.as_bytes(), "{name}");
        assert_eq!(found.origins[0], origin, "{name}");
        assert_eq!(remaining.as_ref(), args(&rest), "{name}");
    }
}
#[test]
fn settings_refusals_name_the_flag_and_its_reason() {
    let long = "/".to_owned() + &"a".repeat(4096);
    for (arguments, expected) in [
        (vec!["--config", "/a", "--config", "/b"], "--config is given twice"),
        (vec!["--config"], "--config takes a value: CONFIG"),
        (
            vec![],
            "--config CONFIG is not given, TEMPER_CONFIG is not set, and its default $XDG_CONFIG_HOME/temper needs XDG_CONFIG_HOME or HOME as an absolute path",
        ),
        (vec!["--config", &long, "--data", "/d", "--cache", "/c"], "--config DIR: absolute path is past 4096 bytes"),
    ] {
        assert_eq!(resolve(&args(&arguments), &[], b"/work").expect_err("refused").to_string(), expected);
    }
}
