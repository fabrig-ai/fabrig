use std::os::unix::fs::MetadataExt;
use temper_shell::directories::make_private;

#[test]
fn roots_are_private_and_checked_at_their_own_names() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    for row in ["absent", "private", "public", "link", "file", "parent-link"] {
        let scratch = skein_scratch::Scratch::new(row);
        let root = scratch.path().to_path_buf();
        let target = root.join("parent/root");
        match row {
            "absent" => {}
            "parent-link" => {
                std::fs::create_dir(root.join("real")).expect("parent");
                symlink(root.join("real"), root.join("parent")).expect("parent link");
            }
            _ => {
                std::fs::create_dir(root.join("parent")).expect("parent");
                match row {
                    "link" => symlink(&root, &target).expect("link"),
                    "file" => std::fs::write(&target, b"").expect("file"),
                    _ => {
                        std::fs::create_dir(&target).expect("directory");
                        std::fs::set_permissions(
                            &target,
                            std::fs::Permissions::from_mode(if row == "public" { 0o755 } else { 0o700 }),
                        )
                        .expect("mode");
                    }
                }
            }
        }
        let result = make_private(&target, skein_shell::effective_user());
        if ["public", "link", "file"].contains(&row) {
            assert_eq!(
                result.expect_err(row),
                format!("{}: must be a private directory of this user's, not reached through a link", target.display())
            );
        } else {
            result.expect(row);
            assert_eq!(std::fs::metadata(&target).expect("made").mode() & 0o777, 0o700);
        }
        if row == "absent" {
            assert_eq!(std::fs::metadata(root.join("parent")).expect("made parent").mode() & 0o777, 0o700);
        }
    }
}
