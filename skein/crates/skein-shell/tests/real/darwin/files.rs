#[expect(unsafe_code, reason = "chflags changes the fixture's own scratch file")]
mod tests {
    use crate::system::{close, retry, status};
    use skein_scratch::Scratch;
    use skein_shell::open_root;
    use skein_shell::testing::{path_name, remove};

    // A directory's unlink answers EPERM on Darwin, as does a refusal that is
    // not a directory's: an immutable file's (chflags), and so a sticky
    // directory's other users' entries, which one user cannot make.
    #[test]
    fn only_a_directory_turns_an_unlinks_eperm_into_eisdir() {
        let scratch = Scratch::new("darwin-remove");
        std::fs::create_dir(scratch.path().join("dir")).expect("make the directory");
        std::fs::write(scratch.path().join("fixed"), b"x").expect("make the file");
        let fixed = path_name(&scratch.path().join("fixed")).expect("a path");
        let change = |flags: u32| {
            // SAFETY: fixed is terminated and lives across chflags.
            retry(|| status(unsafe { libc::chflags(fixed.as_ptr(), flags) })).expect("chflags the file");
        };
        change(libc::UF_IMMUTABLE);
        let parent = open_root(scratch.path()).expect("open the scratch directory");
        assert_eq!(remove(parent, b"dir", false), Err(libc::EISDIR), "a directory's unlink is EISDIR");
        assert_eq!(remove(parent, b"fixed", false), Err(libc::EPERM), "an immutable file stays EPERM");
        change(0);
        assert_eq!(remove(parent, b"fixed", false), Ok(()), "the file goes once it may");
        assert_eq!(remove(parent, b"dir", true), Ok(()), "the directory goes as a directory");
        assert_eq!(close(parent), Ok(()), "close the scratch directory");
    }
}
