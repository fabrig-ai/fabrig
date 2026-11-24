use super::*;

#[test]
fn git_lookup_skips_missing_nonexecutable_and_nonabsolute_entries() {
    let git = Path::new("/programs/git");
    let env = vec![(Box::from(b"PATH".as_slice()), Box::from(b"/nonexistent:/programs".as_slice()))];
    for (mode, expected) in [(0o700, Some(git.to_path_buf())), (0o600, None)] {
        let mut looked = Vec::new();
        let found = find_with(&env, b"git", |path| {
            looked.push(path.to_path_buf());
            if path == git { Ok((Kind::File, mode)) } else { Err(io::ErrorKind::NotFound.into()) }
        });
        assert_eq!(looked, [PathBuf::from("/nonexistent/git"), git.to_path_buf()]);
        assert_eq!(found, expected, "git at {mode:o}");
    }
    let env = vec![(Box::from(b"PATH".as_slice()), Box::from(b":programs".as_slice()))];
    assert_eq!(candidates(&env, b"git"), Vec::<PathBuf>::new());
    assert_eq!(find_with(&env, b"git", |_| panic!("no candidate to inspect")), None);
}
