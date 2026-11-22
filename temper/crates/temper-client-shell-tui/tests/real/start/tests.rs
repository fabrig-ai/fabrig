use std::{fs, os::unix::fs::PermissionsExt};
use temper_client_shell_tui::testing::origin;

#[test]
fn origin_directories_keep_components_private_and_unshortened() {
    use temper_client_tui::Origin;
    let longest = ["a".repeat(63), "b".repeat(63), "c".repeat(63), "d".repeat(61)].join(".");
    for (origin, relative) in [
        (origin(), "at/127.0.0.1/8367".to_owned()),
        (Origin::Http("[::1]:8367".parse().expect("IPv6")), "at/::1/8367".to_owned()),
        (Origin::Https { name: b"dev.fabrig.ai".as_slice().into(), port: 443 }, "at/dev.fabrig.ai/443".to_owned()),
        (Origin::Https { name: longest.as_bytes().into(), port: 443 }, format!("at/{longest}/443")),
    ] {
        let scratch = skein_scratch::Scratch::new("tui-origin");
        let user = skein_shell::effective_user();
        let (directory, made) = super::deployment::prepare(scratch.path(), &origin, user).expect("prepare");
        assert!(made);
        assert_eq!(directory, scratch.path().join(relative));
        for path in [directory.clone(), directory.parent().expect("host").into(), scratch.path().join("at")] {
            assert_eq!(fs::metadata(&path).expect("metadata").permissions().mode() & 0o777, 0o700);
        }
        assert!(!super::deployment::prepare(scratch.path(), &origin, user).expect("again").1);
    }
}

#[test]
fn origin_directories_refuse_a_public_or_linked_parent() {
    for linked in [false, true] {
        let scratch = skein_scratch::Scratch::new("tui-parent");
        let at = scratch.path().join("at");
        if linked {
            std::os::unix::fs::symlink(scratch.path(), &at).expect("link");
        } else {
            fs::create_dir(&at).expect("public parent");
            fs::set_permissions(&at, fs::Permissions::from_mode(0o755)).expect("mode");
        }
        assert!(super::deployment::prepare(scratch.path(), &origin(), skein_shell::effective_user()).is_err());
    }
}
