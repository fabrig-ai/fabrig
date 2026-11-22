use super::*;
use crate::testing::{digest, gzip, scratch, site};
use std::collections::BTreeMap;

fn trust() -> skein_shell::Trust {
    skein_shell::Trust::Der(PathBuf::from("no-trust-needed"))
}

const WANTED: &[u8] = b"#!/bin/sh\nprintf 'temper-box 0.0.0 (0123456789ab)\\n'\n";
const WRONG_BESIDE: &[u8] = b"#!/bin/sh\nprintf 'other\\n'\n";
const WRONG_CACHE: &[u8] = b"#!/bin/sh\nprintf 'site mistake\\n'\n";

#[derive(Default)]
struct CacheTree {
    files: BTreeMap<PathBuf, Vec<u8>>,
    held: Option<PathBuf>,
}

impl CacheTree {
    fn publish(&mut self, file: &Path, bytes: &[u8]) {
        let sum = [temper_fetch::fetch::hex(&digest(bytes)).as_ref(), b"\n"].concat();
        self.files.insert(file.to_path_buf(), bytes.to_vec());
        self.files.insert(suffix(file, ".sha256"), sum);
    }

    fn clean(&mut self, directory: &Path) -> Result<(), Failed> {
        clean_with(directory, |file| {
            self.files.remove(file);
            Ok(())
        })
    }

    fn answer(&mut self, request: Request) -> Result<Event, Failed> {
        match request {
            Request::Identity { file } => {
                let Some(bytes) = self.files.get(path(&file)) else { return Ok(Event::Identity(absent())) };
                let line: &[u8] = match bytes.as_slice() {
                    WANTED => b"temper-box 0.0.0 (0123456789ab)\n",
                    WRONG_BESIDE => b"other\n",
                    WRONG_CACHE => b"site mistake\n",
                    _ => panic!("unexpected candidate"),
                };
                Ok(Event::Identity(Candidate { line: line.into(), success: true }))
            }
            Request::Lock { directory } => {
                let directory = path(&directory);
                self.files.insert(directory.join("temper-box.lock"), Vec::new());
                assert!(self.held.replace(directory.to_path_buf()).is_none());
                Ok(Event::Done)
            }
            Request::Clean { directory } => self.clean(path(&directory)).map(|()| Event::Done),
            Request::Cached { file } => {
                let file = path(&file);
                assert_eq!(self.held.as_deref(), file.parent());
                Ok(Event::Cached {
                    file: self.files.get(file).map(|bytes| digest(bytes)),
                    sum: self.files.get(&suffix(file, ".sha256")).cloned().and_then(stored_sum),
                })
            }
            Request::RemoveCached { file } => {
                self.files.remove(path(&file));
                self.files.remove(&suffix(path(&file), ".sha256"));
                Ok(Event::Done)
            }
            Request::Get { .. } => Err(Failed { words: "scripted download refusal".into() }),
            Request::Open { .. }
            | Request::Read
            | Request::Write(_)
            | Request::Say(_)
            | Request::Unpack { .. }
            | Request::WriteSum { .. }
            | Request::RenameSum { .. }
            | Request::Publish { .. }
            | Request::End(_) => panic!("unexpected cache effect"),
        }
    }

    fn find(&mut self, find: &Find<'_>, out: &mut dyn Write) -> Result<PathBuf, Failed> {
        let build = build(find, |file| self.files.contains_key(file));
        let (result, downloading) = drive(build, out, |request| self.answer(request));
        let result = finish(find, result, downloading, || {
            if let Some(directory) = self.held.clone() {
                self.clean(&directory)?;
            }
            Ok(())
        });
        self.held = None;
        result
    }
}

#[test]
fn find_takes_beside_or_verified_cache_without_tls() {
    let directory = Path::new("/fixture");
    let beside = directory.join("beside");
    let cache = directory.join("cache");
    let site = site();
    let trust = trust();
    let mut input =
        Find { beside: &beside, cache: &cache, site: &site, trust: &trust, version: "0.0.0", commit: "0123456789ab" };
    let mut tree = CacheTree::default();
    tree.files.insert(beside.join("temper-box"), WANTED.to_vec());
    let mut out = Vec::new();
    assert_eq!(tree.find(&input, &mut out).unwrap(), beside.join("temper-box"));
    assert!(!tree.files.keys().any(|file| file.starts_with(&cache)));
    assert_eq!(out.len(), 0);
    tree.files.insert(beside.join("temper-box"), WRONG_BESIDE.to_vec());
    let cached = cache.join("bin/0123456789ab").join(platform()).join("temper-box");
    tree.publish(&cached, WANTED);
    for name in [".gz.part", ".part", ".sha256.part"] {
        tree.files.insert(suffix(&cached, name), b"stale".to_vec());
    }
    assert_eq!(tree.find(&input, &mut out).unwrap(), cached);
    for name in [".gz.part", ".part", ".sha256.part"] {
        assert!(!tree.files.contains_key(&suffix(&cached, name)));
    }
    assert_eq!(out.len(), 0);
    // A verified candidate of another identity is refused as a site error.
    tree.publish(&cached, WRONG_CACHE);
    let why = tree.find(&input, &mut out).unwrap_err().words();
    assert!(why.contains("site mistake\n"));
    assert!(!why.contains("Fetch it by hand"));
    assert!(!tree.files.contains_key(&cached));
    assert!(!tree.files.contains_key(&suffix(&cached, ".sha256")));
    input.commit = "unknown";
    let why = tree.find(&input, &mut out).unwrap_err().words();
    assert_eq!(
        why,
        format!(
            "no temper-box of this build (0.0.0 (unknown)) beside {}/temper; a build from a checkout downloads none: build it with cargo build -p temper-box",
            beside.display()
        )
    );
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
