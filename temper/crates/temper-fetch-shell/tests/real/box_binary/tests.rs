use skein_lib::Queue;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
use temper_fetch::box_binary::{self as machine, Build, Candidate, Event, Request};
use temper_fetch_shell::testing::{Files, cached, digest, gzip, identity, scratch, script, site, suffix};

fn next(machine: &mut machine::Find, event: Event) -> Request {
    let mut out = Queue::uncapped();
    machine::step(machine, event, &mut out);
    assert_eq!(out.len(), 1);
    out.pop().unwrap()
}

#[test]
fn candidate_is_run_and_bounded() {
    let directory = scratch("identity");
    let file = directory.join("temper-box");
    script(&file, b"#!/bin/sh\n[ \"$1\" = --version ] || exit 1\n[ -z \"$HOME\" ] || exit 1\nprintf 'checked\\n'\n");
    assert_eq!(identity(&file), Candidate { line: b"checked\n".as_slice().into(), success: true });
    fs::remove_dir_all(directory).unwrap();
}

/// The production file interpreter is driven by a fake HTTP body, not a TLS server.
#[test]
fn lock_cleanup_and_publication_order() {
    let root = scratch("publish");
    let body = b"#!/bin/sh\nprintf 'temper-box 0.0.0 (0123456789ab)\\n'\n".as_slice();
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
            assert!(files.is_held());
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
    assert!(result.is_ok(), "{result:?}");
    assert!(cache_read);
    for suffix_name in [".gz.part", ".part", ".sha256.part"] {
        assert!(!suffix(&file, suffix_name).exists());
    }
    assert_eq!(publication, ["sum", "rename sum", "publish"]);
    assert_eq!(fs::read(&file).unwrap(), body);
    assert_eq!(cached(&file).unwrap(), Event::Cached { file: Some(digest(body)), sum: Some(digest(body)) });
    assert_eq!(fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o755);
    assert_eq!(fs::metadata(suffix(&file, ".sha256")).unwrap().permissions().mode() & 0o777, 0o600);
    drop(files);
    fs::remove_dir_all(root).unwrap();
}
