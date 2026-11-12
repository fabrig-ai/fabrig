use super::*;
use std::os::unix::fs::PermissionsExt;
fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("temper-fetch-box-{name}-{}", std::process::id()));
    match fs::remove_dir_all(&path) {
        Ok(()) | Err(_) => {}
    }
    fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
    path
}
fn script(file: &Path, body: &[u8]) {
    fs::write(file, body).unwrap();
    fs::set_permissions(file, fs::Permissions::from_mode(0o755)).unwrap();
}
fn digest(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.finish()
}
fn trust() -> skein_shell::Trust {
    skein_shell::Trust::Der(PathBuf::from("no-trust-needed"))
}
fn site() -> Origin {
    Origin { host: b"site.test".as_slice().into(), port: 443 }
}
fn next(machine: &mut machine::Find, event: Event) -> Request {
    let mut out = Queue::uncapped();
    machine::step(machine, event, &mut out);
    assert_eq!(out.len(), 1);
    out.pop().unwrap()
}
fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}
#[test]
fn candidate_is_run_and_bounded() {
    let directory = scratch("identity");
    let file = directory.join("temper-box");
    for (body, line, success) in [
        (
            "#!/bin/sh\nprintf 'temper-box 0.0.0 (0123456789ab)\\n'\n",
            b"temper-box 0.0.0 (0123456789ab)\n".as_slice(),
            true,
        ),
        ("#!/bin/sh\nprintf 'another\\n'\n", b"another\n", true),
        ("#!/bin/sh\nprintf 'another\\n'; exit 1\n", b"another\n", false),
        (
            "#!/bin/sh\n[ \"$1\" = --version ] || exit 1\n[ -z \"$HOME\" ] || exit 1\nprintf 'checked\\n'\n",
            b"checked\n",
            true,
        ),
    ] {
        script(&file, body.as_bytes());
        assert_eq!(identity(&file), Candidate { line: line.into(), success });
    }
    script(&file, format!("#!/bin/sh\nprintf '{}'\n", "x".repeat(257)).as_bytes());
    let candidate = identity(&file);
    assert!(!candidate.success);
    assert_eq!(candidate.line.len(), 256);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(identity(&file), absent());
    assert_eq!(identity(&directory), absent());
    assert_eq!(identity(&directory.join("missing")), absent());
    fs::remove_dir_all(directory).unwrap();
}
#[test]
fn find_takes_beside_or_verified_cache_without_tls() {
    let directory = scratch("find");
    let beside = directory.join("beside");
    fs::create_dir(&beside).unwrap();
    let cache = directory.join("cache");
    let site = site();
    let trust = trust();
    let mut input =
        Find { beside: &beside, cache: &cache, site: &site, trust: &trust, version: "0.0.0", commit: "0123456789ab" };
    let script_bytes = b"#!/bin/sh\nprintf 'temper-box 0.0.0 (0123456789ab)\\n'\n";
    script(&beside.join("temper-box"), script_bytes);
    let mut out = Vec::new();
    assert_eq!(find(&skein_io::Limits::TEST, &input, &mut out).unwrap(), beside.join("temper-box"));
    assert!(!cache.exists());
    assert!(out.is_empty());
    script(&beside.join("temper-box"), b"#!/bin/sh\nprintf 'other\\n'\n");
    let cached = cache.join("bin/0123456789ab").join(platform()).join("temper-box");
    fs::create_dir_all(cached.parent().unwrap()).unwrap();
    script(&cached, script_bytes);
    write_sum(&cached, &digest(script_bytes)).unwrap();
    rename_sum(&cached).unwrap();
    for name in [".gz.part", ".part", ".sha256.part"] {
        fs::write(suffix(&cached, name), b"stale").unwrap();
    }
    assert_eq!(find(&skein_io::Limits::TEST, &input, &mut out).unwrap(), cached);
    for name in [".gz.part", ".part", ".sha256.part"] {
        assert!(!suffix(&cached, name).exists());
    }
    assert!(out.is_empty());
    // A verified but wrong identity is a site error, not a second download.
    script(&cached, b"#!/bin/sh\nprintf 'site mistake\\n'\n");
    let bytes = fs::read(&cached).unwrap();
    write_sum(&cached, &digest(&bytes)).unwrap();
    rename_sum(&cached).unwrap();
    let why = find(&skein_io::Limits::TEST, &input, &mut out).unwrap_err().words();
    assert!(why.contains("site mistake\n"));
    assert!(!why.contains("Fetch it by hand"));
    assert!(!cached.exists());
    assert!(!suffix(&cached, ".sha256").exists());
    input.commit = "unknown";
    let why = find(&skein_io::Limits::TEST, &input, &mut out).unwrap_err().words();
    assert_eq!(
        why,
        format!(
            "no temper-box of this build (0.0.0 (unknown)) beside {}/temper; a build from a checkout downloads none: build it with cargo build -p temper-box",
            beside.display()
        )
    );
    fs::remove_dir_all(directory).unwrap();
}
/// The production file interpreter is driven by a fake HTTP body, not a TLS server.
#[expect(clippy::too_many_lines, reason = "the scratch replay observes each publication effect before answering it")]
fn staged(name: &str, body: &[u8], success: bool) {
    let root = scratch(name);
    let mut machine = machine::Find::new(Build {
        beside: root.as_os_str().as_bytes().into(),
        cache: root.as_os_str().as_bytes().into(),
        site: site(),
        version: "0.0.0".into(),
        commit: "0123456789ab".into(),
        platform: b"Linux-x86_64".as_slice().into(),
        beside_exists: false,
    });
    let file = root.join("bin/0123456789ab/Linux-x86_64/temper-box");
    let mut files = Files::default();
    let mut event = Event::Begin;
    let compressed = gzip(body);
    let sums = format!(
        "{}  Linux-x86_64/temper-box\n",
        String::from_utf8(temper_fetch::fetch::hex(&digest(&compressed)).into()).unwrap()
    )
    .into_bytes();
    let mut response = Vec::new();
    let mut publication = Vec::new();
    let mut cache_read = false;
    let result = loop {
        let request = next(&mut machine, event);
        if let Request::Lock { .. } = &request {
            assert!(!file.parent().unwrap().exists());
        }
        if let Request::Clean { .. } = &request {
            assert!(files.held.is_some());
            if !cache_read {
                for suffix_name in [".gz.part", ".part", ".sha256.part"] {
                    fs::write(suffix(&file, suffix_name), b"stale").unwrap();
                }
            }
        }
        if let Request::Cached { .. } = &request {
            cache_read = true;
            let other = fs::OpenOptions::new().write(true).open(suffix(&file, ".lock")).unwrap();
            assert!(other.try_lock().is_err(), "lock held before cache is read");
            for suffix_name in [".gz.part", ".part", ".sha256.part"] {
                assert!(!suffix(&file, suffix_name).exists());
            }
            for parent in [root.join("bin"), root.join("bin/0123456789ab"), file.parent().unwrap().to_path_buf()] {
                assert_eq!(fs::metadata(parent).unwrap().permissions().mode() & 0o777, 0o700);
            }
            assert_eq!(fs::metadata(suffix(&file, ".lock")).unwrap().permissions().mode() & 0o777, 0o600);
        }
        if let Request::WriteSum { .. } = &request {
            assert!(!file.exists());
            assert!(!suffix(&file, ".sha256").exists());
            publication.push("sum");
        }
        if let Request::RenameSum { .. } = &request {
            assert!(!file.exists());
            assert!(suffix(&file, ".sha256.part").exists());
            assert!(!suffix(&file, ".sha256").exists());
            publication.push("rename sum");
        }
        if let Request::Publish { .. } = &request {
            assert!(!file.exists());
            assert!(suffix(&file, ".sha256").exists());
            assert!(!suffix(&file, ".sha256.part").exists());
            publication.push("publish");
        }
        event = match request {
            Request::Get { path, .. } => {
                let bytes = if path.ends_with(b"SHA256SUMS") { sums.clone() } else { compressed.clone() };
                let length = u64::try_from(bytes.len()).unwrap();
                response = vec![Event::Ended, Event::Chunk(bytes.into_boxed_slice())];
                Event::Answered { status: 200, length: Some(length), location: None }
            }
            Request::Read => response.pop().unwrap(),
            Request::Say(_) => Event::Done,
            Request::End(result) => break result,
            request @ (Request::Identity { .. }
            | Request::Lock { .. }
            | Request::Clean { .. }
            | Request::Cached { .. }
            | Request::RemoveCached { .. }
            | Request::Open { .. }
            | Request::Write(_)
            | Request::Unpack { .. }
            | Request::WriteSum { .. }
            | Request::RenameSum { .. }
            | Request::Publish { .. }) => files.answer(request).unwrap(),
        };
    };
    assert_eq!(result.is_ok(), success, "{result:?}");
    assert!(cache_read);
    for suffix_name in [".gz.part", ".part", ".sha256.part"] {
        assert!(!suffix(&file, suffix_name).exists());
    }
    if success {
        assert_eq!(publication, ["sum", "rename sum", "publish"]);
        assert_eq!(fs::read(&file).unwrap(), body);
        assert_eq!(cached(&file).unwrap(), Event::Cached { file: Some(digest(body)), sum: Some(digest(body)) });
        assert_eq!(fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o755);
        assert_eq!(fs::metadata(suffix(&file, ".sha256")).unwrap().permissions().mode() & 0o777, 0o600);
    } else {
        assert!(publication.is_empty());
        assert!(!file.exists());
    }
    drop(files);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn lock_cleanup_and_publication_order() {
    staged("publish", b"#!/bin/sh\nprintf 'temper-box 0.0.0 (0123456789ab)\\n'\n", true);
    staged("refuse", b"#!/bin/sh\nprintf 'wrong build\\n'\n", false);
}
#[test]
fn gzip_and_cleanup_bounds() {
    let root = scratch("gzip");
    let archive = root.join("temper-box.gz.part");
    let part = root.join("temper-box.part");
    let compressed = gzip(b"abc");
    for (bytes, bound, expected) in [
        (compressed.clone(), 3, Unpacked::File { digest: digest(b"abc"), bytes: 3 }),
        (compressed.clone(), 2, Unpacked::TooLong),
        (compressed[..compressed.len() - 1].to_vec(), 3, Unpacked::Invalid),
        ([compressed.as_slice(), b"x"].concat(), 3, Unpacked::Invalid),
        ([compressed.as_slice(), compressed.as_slice()].concat(), 6, Unpacked::Invalid),
        ([compressed.as_slice(), b"\0\0"].concat(), 3, Unpacked::File { digest: digest(b"abc"), bytes: 3 }),
    ] {
        fs::write(&archive, bytes).unwrap();
        assert_eq!(unpack(&archive, &part, bound).unwrap(), expected);
        let mut files = Files { held: Some(lock(&root).unwrap()), directory: Some(root.clone()), output: None };
        fs::write(root.join("temper-box.sha256.part"), b"partial").unwrap();
        files.cleanup().unwrap();
        assert!(!archive.exists());
        assert!(!part.exists());
        assert!(!root.join("temper-box.sha256.part").exists());
    }
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn download_failure_words_and_shell_quote() {
    let root = scratch("failure");
    let beside = root.join("it's here");
    let site = site();
    let trust = trust();
    let input =
        Find { beside: &beside, cache: &root, site: &site, trust: &trust, version: "0.0.0", commit: "0123456789ab" };
    let why = find(&skein_io::Limits::TEST, &input, &mut Vec::new()).unwrap_err().words();
    assert!(why.starts_with("temper-box: https://site.test/temper/v0/0123456789ab/SHA256SUMS:"), "{why}");
    assert!(why.contains("Fetch it by hand: curl -fL https://site.test/temper/v0/0123456789ab/"));
    assert!(why.contains(" | gunzip > '"));
    assert!(why.contains("it'\\''s here/temper-box' && chmod +x '"));
    assert!(!why.starts_with("temper: "));
    let file = root.join("bin/0123456789ab").join(platform()).join("temper-box");
    for name in [".gz.part", ".part", ".sha256.part"] {
        assert!(!suffix(&file, name).exists());
    }
    fs::remove_dir_all(root).unwrap();
}
