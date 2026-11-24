use super::*;
use std::cell::RefCell;

#[test]
fn claude_cache_hit_and_refusals_request_only_metadata() {
    let file = Path::new("/cache/claude-code/2.1.311/claude");
    let size = 200;
    let present = "/cache/claude-code/2.1.311/claude: does not match the pinned digest";
    for (row, metadata, expected) in [
        ("sized hit", (true, size), Ok(())),
        ("wrong size", (true, size - 1), Err(present)),
        ("not a file", (false, size), Err(present)),
    ] {
        let requests = RefCell::new(Vec::new());
        let mut out = Vec::new();
        let result = claude_code_with(
            file,
            size,
            |candidate| {
                assert_eq!(candidate, file, "{row}");
                requests.borrow_mut().push("metadata");
                Ok(metadata)
            },
            |_| {
                requests.borrow_mut().push("prepare cache directory");
                Ok(())
            },
            |_| {
                requests.borrow_mut().push("lock");
                Ok(())
            },
            |_| {
                requests.borrow_mut().push("fetch");
                Ok(())
            },
            &mut out,
        )
        .map_err(|why| why.words());
        assert_eq!(result.as_ref().map_err(String::as_str), expected.as_ref().map_err(|why| *why), "{row}");
        assert_eq!(requests.into_inner(), ["metadata"], "{row}");
        assert!(out.is_empty(), "{row}: no fetching notice");
    }
}

#[test]
fn a_claude_cache_miss_is_checked_again_under_the_lock() {
    let file = Path::new("/cache/claude-code/2.1.311/claude");
    let size = 200;
    for found_under_lock in [true, false] {
        let requests = RefCell::new(Vec::new());
        let mut out = Vec::new();
        claude_code_with(
            file,
            size,
            |candidate| {
                assert_eq!(candidate, file);
                let mut requests = requests.borrow_mut();
                let under_lock = requests.last() == Some(&"lock");
                requests.push("metadata");
                if under_lock && found_under_lock { Ok((true, size)) } else { Err(io::ErrorKind::NotFound.into()) }
            },
            |candidate| {
                assert_eq!(candidate, file);
                requests.borrow_mut().push("prepare cache directory");
                Ok(())
            },
            |candidate| {
                assert_eq!(candidate, file);
                requests.borrow_mut().push("lock");
                Ok(())
            },
            |_| {
                requests.borrow_mut().push("fetch");
                Ok(())
            },
            &mut out,
        )
        .expect("cache hit or fetch");
        let mut expected = vec!["metadata", "prepare cache directory", "lock", "metadata"];
        if found_under_lock {
            assert_eq!(out, b"");
        } else {
            expected.push("fetch");
            assert_eq!(out, b"fetching Claude Code\n");
        }
        assert_eq!(requests.into_inner(), expected);
    }
}
