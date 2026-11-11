use super::{Machine, path_word, program_word};

type File<'a> = (&'a [u8], &'a [u8]);
fn program(executable: Option<&[u8]>, path: Option<&[u8]>, files: &[File<'_>]) -> Box<[u8]> {
    program_word(&Machine {
        executable,
        path,
        home: Some(b"/home/person"),
        canonical: &|path| files.iter().find(|(name, _)| *name == path).map(|(_, target)| Box::from(*target)),
    })
}

#[test]
fn only_the_first_executable_on_path_can_name_this_binary() {
    let files: &[File<'_>] = &[
        (b"/checkout/temper-box", b"/checkout/temper-box"),
        (b"/link/temper-box", b"/checkout/temper-box"),
        (b"/old/temper-box", b"/old/temper-box"),
    ];
    for (path, expected) in [
        (Some(b"/link:/old".as_slice()), b"temper-box".as_slice()),
        (Some(b"/old:/link".as_slice()), b"/checkout/temper-box".as_slice()),
        (Some(b"/absent".as_slice()), b"/checkout/temper-box".as_slice()),
        (None, b"/checkout/temper-box".as_slice()),
        // The fake returns None for a directory and a non-executable file.
        (Some(b"/directory:/non-executable:/link".as_slice()), b"temper-box".as_slice()),
    ] {
        assert_eq!(program(Some(b"/checkout/temper-box"), path, files).as_ref(), expected, "{path:?}");
    }
}

#[test]
fn empty_and_relative_path_entries_count_for_nothing() {
    let files: &[File<'_>] = &[
        (b"/checkout/temper-box", b"/checkout/temper-box"),
        (b"temper-box", b"/checkout/temper-box"),
        (b"/temper-box", b"/checkout/temper-box"),
        (b"./temper-box", b"/checkout/temper-box"),
        (b"relative/temper-box", b"/checkout/temper-box"),
    ];
    for path in [b"".as_slice(), b".", b"relative", b":.:relative:"] {
        assert_eq!(program(Some(b"/checkout/temper-box"), Some(path), files).as_ref(), b"/checkout/temper-box");
    }
}

#[test]
fn a_deleted_executable_uses_its_rebuilt_path() {
    let files: &[File<'_>] = &[(b"/bin/temper-box", b"/bin/temper-box")];
    for (path, expected) in [(Some(b"/bin".as_slice()), b"temper-box".as_slice()), (None, b"/bin/temper-box")] {
        assert_eq!(program(Some(b"/bin/temper-box (deleted)"), path, files).as_ref(), expected);
    }
}

#[test]
fn an_unreadable_executable_falls_back_to_temper() {
    assert_eq!(program(None, Some(b"/bin"), &[]).as_ref(), b"temper-box");
}

#[test]
fn home_leads_only_whole_components_after_slashes_collapse() {
    for home in [b"/home/person".as_slice(), b"/home/person/", b"/home//person"] {
        assert_eq!(path_word(b"/home/person/src//a///temper-box", Some(home)).as_ref(), b"~/src/a/temper-box");
        assert_eq!(path_word(b"/home/personal/temper-box", Some(home)).as_ref(), b"/home/personal/temper-box");
        assert_eq!(path_word(b"/home/person", Some(home)).as_ref(), b"~");
    }
    for home in [None, Some(b"".as_slice()), Some(b"home/person"), Some(b"/"), Some(b"///")] {
        assert_eq!(path_word(b"/home/person//temper-box", home).as_ref(), b"/home/person/temper-box");
    }
}

#[test]
fn shell_words_quote_only_what_needs_quotes_after_home() {
    for (path, expected) in [
        (b"/home/person/my dir/temper-box".as_slice(), b"~/'my dir/temper-box'".as_slice()),
        (b"/my dir/temper-box", b"'/my dir/temper-box'"),
        (b"/home/person/owner's/temper-box", b"~/'owner'\\''s/temper-box'"),
        (b"/owner's/temper-box", b"'/owner'\\''s/temper-box'"),
        (b"/home/person/\xff/temper-box", b"~/'\xff/temper-box'"),
        (b"/\xff/temper-box", b"'/\xff/temper-box'"),
        (b"/home/person/~", b"~/'~'"),
        (b"/home/person/a=b", b"~/'a=b'"),
        (b"/AZaz09_-./+,:@%", b"/AZaz09_-./+,:@%"),
        (b"", b"''"),
    ] {
        assert_eq!(path_word(path, Some(b"/home/person")).as_ref(), expected, "{path:?}");
    }
}

#[test]
fn the_executable_path_uses_the_same_home_word() {
    assert_eq!(program(Some(b"/home/person/my dir/temper-box"), None, &[]).as_ref(), b"~/'my dir/temper-box'");
}
