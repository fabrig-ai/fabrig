use super::*;
use crate::testing::{digest, gzip, scratch, script, site};

fn trust() -> skein_shell::Trust {
    skein_shell::Trust::Der(PathBuf::from("no-trust-needed"))
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
    assert_eq!(out.len(), 0);
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
    assert_eq!(out.len(), 0);
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

#[test]
fn gzip_and_cleanup_bounds() {
    let root = Path::new("/fixture/cache");
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
        let mut cache = std::collections::BTreeMap::from([
            (archive.clone(), bytes),
            (part.clone(), Vec::new()),
            (root.join("temper-box.sha256.part"), b"partial".to_vec()),
        ]);
        let bytes = cache.get(&archive).unwrap().as_slice();
        let mut output = Vec::new();
        assert_eq!(decode_gzip(bytes, &mut output, bound).unwrap(), expected);
        cache.insert(part.clone(), output);
        clean_with(root, |file| {
            cache.remove(file);
            Ok(())
        })
        .unwrap();
        assert!(!cache.contains_key(&archive));
        assert!(!cache.contains_key(&part));
        assert!(!cache.contains_key(&root.join("temper-box.sha256.part")));
    }
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
