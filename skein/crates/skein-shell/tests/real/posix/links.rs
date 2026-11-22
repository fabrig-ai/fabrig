//! The walk's owner argument stands in for root on either system.

mod tests {
    use skein_scratch::Scratch;
    use skein_shell::testing::links::{configure, pass_links, walk};
    use std::fs;
    use std::os::unix::{ffi::OsStrExt, fs::symlink};
    use std::path::PathBuf;

    fn layout(mode: u32) -> Scratch {
        scratch(mode, false)
    }

    fn parent_link(mode: u32) -> Scratch {
        scratch(mode, true)
    }

    fn scratch(mode: u32, parent: bool) -> Scratch {
        let scratch = Scratch::new("system-links");
        configure(scratch.path(), mode, parent);
        scratch
    }

    #[test]
    fn owned_parent_links_are_passed_without_leaving_a_link() {
        let scratch = parent_link(0o755);
        let expected = scratch.path().join("real/f");
        let answer = walk(&scratch.path().join("sys/f"), crate::effective_user()).expect("owned parent passed");
        assert_eq!(&*answer, expected.as_os_str().as_bytes(), "relative link target replaces its name");
        let mut prefix = PathBuf::new();
        for component in expected.components() {
            prefix.push(component);
            assert!(
                !fs::symlink_metadata(&prefix).expect("walked prefix").file_type().is_symlink(),
                "no remaining link"
            );
        }
    }

    #[test]
    fn a_parent_link_is_passed_beneath_a_descriptor() {
        let scratch = parent_link(0o755);
        let directory = crate::open_root(scratch.path()).expect("borrowed directory descriptor");
        let answer = pass_links(directory.raw(), b"sys/f", crate::effective_user()).expect("owned parent passed");
        crate::close_startup_fd(directory);
        assert_eq!(&*answer, b"real/f", "the answer stays relative to the descriptor");
    }

    #[test]
    fn another_owners_link_is_not_passed() {
        let scratch = parent_link(0o755);
        let user = crate::effective_user();
        let other = u32::from(user == 0);
        assert_eq!(walk(&scratch.path().join("sys/f"), other), None, "the link must belong to the named owner");
    }

    #[test]
    fn links_in_writable_directories_are_not_passed() {
        for mode in [0o777, 0o775] {
            let scratch = parent_link(mode);
            assert_eq!(
                walk(&scratch.path().join("sys/f"), crate::effective_user()),
                None,
                "mode {mode:o} permits other writers"
            );
        }
    }

    #[test]
    fn a_leaf_link_is_not_passed() {
        let scratch = layout(0o755);
        symlink("f", scratch.path().join("real/leaf")).expect("leaf link");
        assert_eq!(
            walk(&scratch.path().join("real/leaf"), crate::effective_user()),
            None,
            "the last component is left for the open"
        );
    }

    #[test]
    fn resolving_system_links_preserves_a_users_link_in_a_writable_directory() {
        let scratch = parent_link(0o777);
        let path = scratch.path().join("sys/f");
        assert_eq!(crate::resolve_system_links(&path), path, "a caller's check still sees the unsafe link");
    }
}
