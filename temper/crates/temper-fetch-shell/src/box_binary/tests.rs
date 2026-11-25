use super::*;
use crate::testing::{digest, scratch, site, zstd};
use std::collections::BTreeMap;

fn trust() -> skein_shell::Trust {
    skein_shell::Trust::Der(PathBuf::from("no-trust-needed"))
}

const WANTED: &[u8] = b"#!/bin/sh\nprintf 'temper-box 0.0.0 (0123456789ab)\\n'\n";
const WRONG_BESIDE: &[u8] = b"#!/bin/sh\nprintf 'other\\n'\n";
const WRONG_CACHE: &[u8] = b"#!/bin/sh\nprintf 'site mistake\\n'\n";
const WRONG_STAGED: &[u8] = b"#!/bin/sh\nprintf 'wrong build\\n'\n";
const WANTED_LINE: &[u8] = b"temper-box 0.0.0 (0123456789ab)\n";
const WRONG_LINE: &[u8] = b"wrong build\n";
const STAGING: [&str; 3] = [".zst.part", ".part", ".sha256.part"];

#[test]
fn candidate_rows() {
    // box-binary.md, 2: a candidate's stdout is bounded to 256 bytes.
    const STDOUT_BOUND: usize = 256;
    let at_bound = [b'x'; STDOUT_BOUND];
    let past_bound = [at_bound.as_slice(), b"!"].concat();
    let executable_file = Some((true, 0o755));
    let file = Path::new("/fixture/temper-box");
    for (metadata, stdout, exited_zero, line, success) in [
        (executable_file, Some(WANTED_LINE), true, WANTED_LINE, true),
        (executable_file, Some(b"another\n"), true, b"another\n", true),
        (executable_file, Some(b"another\n"), false, b"another\n", false),
        (executable_file, Some(at_bound.as_slice()), true, at_bound.as_slice(), true),
        (executable_file, Some(past_bound.as_slice()), true, at_bound.as_slice(), false),
        (Some((true, 0o600)), None, false, b"", false),
        (Some((false, 0o755)), None, false, b"", false),
        (None, None, false, b"", false),
    ] {
        let got = identity_with(file, metadata, |_| {
            Some((stdout.expect("ineligible candidate must not run").to_vec(), exited_zero))
        });
        assert_eq!(got, Candidate { line: line.into(), success });
    }
}

#[derive(Default)]
struct CacheTree {
    files: BTreeMap<PathBuf, Vec<u8>>,
    modes: BTreeMap<PathBuf, u32>,
    held: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl CacheTree {
    fn publish(&mut self, file: &Path, bytes: &[u8]) {
        let sum = [temper_fetch::fetch::hex(&digest(bytes)).as_ref(), b"\n"].concat();
        self.files.insert(file.to_path_buf(), bytes.to_vec());
        self.files.insert(suffix(file, ".sha256"), sum);
    }

    fn clean(&mut self, directory: &Path) -> Result<(), Failed> {
        self.output = None;
        clean_with(directory, |file| {
            self.files.remove(file);
            self.modes.remove(file);
            Ok(())
        })
    }

    fn answer(&mut self, request: Request) -> Result<Event, Failed> {
        match request {
            Request::Identity { file } => {
                let Some(bytes) = self.files.get(path(&file)) else { return Ok(Event::Identity(absent())) };
                let line: &[u8] = match bytes.as_slice() {
                    WANTED => WANTED_LINE,
                    WRONG_BESIDE => b"other\n",
                    WRONG_CACHE => b"site mistake\n",
                    WRONG_STAGED => WRONG_LINE,
                    _ => panic!("unexpected candidate"),
                };
                Ok(Event::Identity(identity_with(path(&file), Some((true, 0o755)), |_| Some((line.to_vec(), true)))))
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
                self.modes.remove(path(&file));
                self.files.remove(&suffix(path(&file), ".sha256"));
                Ok(Event::Done)
            }
            Request::Open { file } => {
                let file = path(&file).to_path_buf();
                assert!(self.files.insert(file.clone(), Vec::new()).is_none(), "open a fresh part");
                self.output = Some(file);
                Ok(Event::Done)
            }
            Request::Write(bytes) => {
                self.files.get_mut(self.output.as_ref().expect("open before write")).unwrap().extend_from_slice(&bytes);
                Ok(Event::Done)
            }
            Request::Unpack { archive, part, bound } => {
                self.output = None;
                let mut output = Vec::new();
                let part = path(&part);
                let unpacked = unpack_with(self.files[path(&archive)].as_slice(), &mut output, bound, |_, mode| {
                    self.modes.insert(part.to_path_buf(), mode);
                    Ok(())
                })
                .unwrap();
                assert!(self.files.insert(part.to_path_buf(), output).is_none(), "unpack to a fresh part");
                Ok(Event::Unpacked(unpacked))
            }
            Request::WriteSum { file, digest } => {
                let part = suffix(path(&file), ".sha256.part");
                let bytes = [temper_fetch::fetch::hex(&digest).as_ref(), b"\n"].concat();
                assert!(self.files.insert(part, bytes).is_none(), "write a fresh staged sum");
                Ok(Event::Done)
            }
            Request::RenameSum { file } => {
                let file = path(&file);
                let bytes = self.files.remove(&suffix(file, ".sha256.part")).expect("staged sum");
                self.files.insert(suffix(file, ".sha256"), bytes);
                Ok(Event::Done)
            }
            Request::Publish { file } => {
                let file = path(&file);
                let part = suffix(file, ".part");
                let bytes = self.files.remove(&part).expect("staged binary");
                let mode = self.modes.remove(&part).expect("staged binary mode");
                self.files.insert(file.to_path_buf(), bytes);
                self.modes.insert(file.to_path_buf(), mode);
                self.files.remove(&suffix(file, ".zst.part")).expect("staged archive");
                Ok(Event::Done)
            }
            Request::Get { .. } => Err(Failed { words: "scripted download refusal".into() }),
            Request::Read | Request::Say(_) | Request::End(_) => panic!("unexpected cache effect"),
        }
    }

    fn find(&mut self, find: &Find<'_>, out: &mut dyn Write) -> Result<PathBuf, Failed> {
        self.find_with(find, out, Self::answer)
    }

    fn find_with(
        &mut self,
        find: &Find<'_>,
        out: &mut dyn Write,
        mut answer: impl FnMut(&mut Self, Request) -> Result<Event, Failed>,
    ) -> Result<PathBuf, Failed> {
        let build = build(find, |file| self.files.contains_key(file));
        let (result, downloading) = drive(build, out, |request| answer(self, request));
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
    for name in STAGING {
        tree.files.insert(suffix(&cached, name), b"stale".to_vec());
    }
    assert_eq!(tree.find(&input, &mut out).unwrap(), cached);
    for name in STAGING {
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

fn download_input<'a>(site: &'a Origin, trust: &'a skein_shell::Trust) -> Find<'a> {
    Find {
        beside: Path::new("/fixture/beside"),
        cache: Path::new("/fixture/cache"),
        site,
        trust,
        version: "0.0.0",
        commit: "0123456789ab",
    }
}

fn cache_file(input: &Find<'_>) -> PathBuf {
    input.cache.join("bin").join(input.commit).join(platform()).join("temper-box")
}

fn sums_for(archive: &[u8], artifact: &str) -> String {
    let hex = temper_fetch::fetch::hex(&digest(archive));
    format!("{}  {}/{artifact}\n", str::from_utf8(&hex).unwrap(), platform())
}

fn from_site(
    tree: &mut CacheTree,
    input: &Find<'_>,
    sums: &[u8],
    archive: &[u8],
    length: u64,
) -> Result<PathBuf, Failed> {
    let file = cache_file(input);
    let directory = file.parent().unwrap();
    let root = format!("/temper/v0/{}/", input.commit);
    let url = format!("{root}{}/temper-box.zst", platform());
    let mut response = Vec::new();
    tree.find_with(input, &mut Vec::new(), |tree, request| {
        if !matches!(request, Request::Lock { .. }) {
            assert_eq!(tree.held.as_deref(), Some(directory), "every effect holds the lock");
        }
        if let Request::Cached { .. } = &request {
            assert_no_staging(tree, &file);
        }
        if let Request::Identity { file: staged } = &request {
            assert_eq!(path(staged), suffix(&file, ".part"));
        }
        if let Request::Get { path, .. } = &request {
            assert!(response.is_empty(), "previous response drained");
            let (body, length) = if path.ends_with(b"SHA256SUMS") {
                assert_eq!(path.as_ref(), format!("{root}SHA256SUMS").as_bytes());
                (sums, u64::try_from(sums.len()).unwrap())
            } else {
                assert_eq!(path.as_ref(), url.as_bytes());
                (archive, length)
            };
            response = vec![Event::Ended, Event::Chunk(body.into())];
            return Ok(Event::Answered { status: 200, length: Some(length), location: None });
        }
        if let Request::Read = request {
            return Ok(response.pop().unwrap());
        }
        tree.answer(request)
    })
}

fn assert_no_staging(tree: &CacheTree, file: &Path) {
    for name in STAGING {
        assert!(!tree.files.contains_key(&suffix(file, name)), "leftover {name}");
    }
}

fn assert_no_publication(tree: &CacheTree, file: &Path) {
    assert!(!tree.files.contains_key(file));
    assert!(!tree.files.contains_key(&suffix(file, ".sha256")));
    assert_no_staging(tree, file);
}

#[test]
fn a_staged_build_of_another_identity_is_refused_under_the_lock() {
    let site = site();
    let trust = trust();
    let input = download_input(&site, &trust);
    let file = cache_file(&input);
    let mut tree = CacheTree::default();
    for name in STAGING {
        tree.files.insert(suffix(&file, name), b"stale".to_vec());
    }
    let archive = zstd(WRONG_STAGED);
    let sums = sums_for(&archive, "temper-box.zst");
    let length = u64::try_from(archive.len()).unwrap();
    let why = from_site(&mut tree, &input, sums.as_bytes(), &archive, length).unwrap_err().words();
    assert!(why.contains("temper-box identity differs: wrong build\n"), "{why}");
    assert_no_publication(&tree, &file);
}

#[test]
fn a_verified_archive_publishes_only_the_executable_and_sum() {
    let site = site();
    let trust = trust();
    let input = download_input(&site, &trust);
    let file = cache_file(&input);
    let mut tree = CacheTree::default();
    let archive = zstd(WANTED);
    let sums = sums_for(&archive, "temper-box.zst");
    let length = u64::try_from(archive.len()).unwrap();
    assert_eq!(from_site(&mut tree, &input, sums.as_bytes(), &archive, length).unwrap(), file);
    assert_eq!(tree.files[&file], WANTED);
    assert_eq!(tree.modes[&file], 0o755);
    let expected_sum = [temper_fetch::fetch::hex(&digest(WANTED)).as_ref(), b"\n"].concat();
    assert_eq!(tree.files[&suffix(&file, ".sha256")], expected_sum);
    assert_no_staging(&tree, &file);
}

const WRONG_MAGIC: &[u8] = b"not a frame";
// A single-segment header declaring 512 MiB + 1, with no blocks to read.
const OVERSIZED_HEADER: &[u8] = b"\x28\xb5\x2f\xfd\xa0\x01\x00\x00\x20";

#[test]
fn download_refusals_leave_no_cache_or_staging_files() {
    let site = site();
    let trust = trust();
    let input = download_input(&site, &trust);
    let file = cache_file(&input);
    let packed = zstd(WANTED);
    let sums = sums_for(&packed, "temper-box.zst");
    let length = u64::try_from(packed.len()).unwrap();
    let truncated = &packed[..packed.len() - 1];
    let wrong_digest = b"different bytes";
    for (sums, archive, length, words) in [
        (sums.clone(), wrong_digest.as_slice(), u64::try_from(wrong_digest.len()).unwrap(), "digest differs: wanted"),
        (sums_for(&packed, "temper-box"), packed.as_slice(), length, "missing temper-box line"),
        (sums.repeat(2), &packed, length, "doubled temper-box line"),
        (sums_for(truncated, "temper-box.zst"), truncated, length - 1, "truncated zstd frame"),
        (
            sums_for(WRONG_MAGIC, "temper-box.zst"),
            WRONG_MAGIC,
            u64::try_from(WRONG_MAGIC.len()).unwrap(),
            "damaged zstd frame",
        ),
        (sums, &packed, machine::SERVED_BYTES + 1, "Content-Length past byte bound"),
        (
            sums_for(OVERSIZED_HEADER, "temper-box.zst"),
            OVERSIZED_HEADER,
            u64::try_from(OVERSIZED_HEADER.len()).unwrap(),
            "unpacked file past 512 MiB",
        ),
    ] {
        let mut tree = CacheTree::default();
        let why = from_site(&mut tree, &input, sums.as_bytes(), archive, length).unwrap_err().words();
        assert!(why.contains(words), "wanted {words}: {why}");
        assert_no_publication(&tree, &file);
    }
}

// `zstd -19 -c --stream-size=384` with 128 repetitions of `abc` on stdin.
// This is one compressed block, a declared size and a four-byte checksum.
const CLI_FRAME: &[u8] = b"\x28\xb5\x2f\xfd\x64\x80\x00\x55\x00\x00\x18abc\x01\x00\x7a\xaa\x6e\x08\x86\xa5\x41\xc7";
const CLI_FRAME_BYTES: u64 = 384;

#[test]
fn a_checksummed_cli_frame_unpacks_at_its_declared_size() {
    let expected = b"abc".repeat(128);
    let mut output = Vec::new();
    let unpacked = decode_zstd(CLI_FRAME, &mut output, CLI_FRAME_BYTES).unwrap();
    assert_eq!(unpacked, Unpacked::File { digest: digest(&expected), bytes: CLI_FRAME_BYTES });
    assert_eq!(output, expected);
}

#[test]
fn an_incomplete_frame_is_truncated_at_every_prefix() {
    for end in 0..CLI_FRAME.len() {
        assert_eq!(
            decode_zstd(&CLI_FRAME[..end], &mut Vec::new(), CLI_FRAME_BYTES).unwrap(),
            Unpacked::Truncated,
            "prefix of {end} bytes"
        );
    }
}

#[test]
fn an_invalid_header_or_block_is_damaged() {
    const RESERVED_BIT: &[u8] = b"\x28\xb5\x2f\xfd\x08";
    const RESERVED_BLOCK: &[u8] = b"\x28\xb5\x2f\xfd\x00\x00\x07\x00\x00";
    const CORRUPT_COMPRESSED_BLOCK: &[u8] = b"\x28\xb5\x2f\xfd\x00\x00\x0d\x00\x00\x08";
    const UNSUPPORTED_WINDOW: &[u8] = b"\x28\xb5\x2f\xfd\x00\xff";
    const UNKNOWN_DICTIONARY: &[u8] = b"\x28\xb5\x2f\xfd\x01\x00\x01";
    for frame in
        [WRONG_MAGIC, RESERVED_BIT, RESERVED_BLOCK, CORRUPT_COMPRESSED_BLOCK, UNSUPPORTED_WINDOW, UNKNOWN_DICTIONARY]
    {
        assert_eq!(decode_zstd(frame, &mut Vec::new(), CLI_FRAME_BYTES).unwrap(), Unpacked::Damaged, "{frame:02x?}");
    }
}

// No declared size, a 1 KiB window, and 1 KiB from RLE. A second block adds one byte.
const RLE_BOUND: u64 = 1024;
const RLE_AT_BOUND: &[u8] = b"\x28\xb5\x2f\xfd\x00\x00\x03\x20\x00a";
const RLE_PAST_BOUND: &[u8] = b"\x28\xb5\x2f\xfd\x00\x00\x02\x20\x00a\x09\x00\x00b";

#[test]
fn a_frame_without_a_declared_size_holds_at_bound() {
    let expected = vec![b'a'; usize::try_from(RLE_BOUND).unwrap()];
    let mut output = Vec::new();
    let unpacked = decode_zstd(RLE_AT_BOUND, &mut output, RLE_BOUND).unwrap();
    assert_eq!(unpacked, Unpacked::File { digest: digest(&expected), bytes: RLE_BOUND });
    assert_eq!(output, expected);
}

#[test]
fn an_over_bound_block_is_not_written_or_read_past() {
    // The second frame is missing a last block; the third is missing its checksum.
    // Both must refuse at the over-bound block, before finding truncation.
    const NO_LAST_BLOCK: &[u8] = b"\x28\xb5\x2f\xfd\x00\x00\x02\x20\x00a\x08\x00\x00b";
    const NO_CHECKSUM: &[u8] = b"\x28\xb5\x2f\xfd\x04\x00\x02\x20\x00a\x09\x00\x00b";
    for frame in [RLE_PAST_BOUND, NO_LAST_BLOCK, NO_CHECKSUM] {
        let mut output = Vec::new();
        assert_eq!(decode_zstd(frame, &mut output, RLE_BOUND).unwrap(), Unpacked::TooLong);
        assert!(output.is_empty(), "no byte from an over-bound block is written");
    }
}

#[test]
fn an_over_bound_declaration_refuses_before_decoding() {
    const CLI_HEADER: usize = 7;
    let mut output = Vec::new();
    assert_eq!(decode_zstd(&CLI_FRAME[..CLI_HEADER], &mut output, CLI_FRAME_BYTES - 1).unwrap(), Unpacked::TooLong);
    assert_eq!(output, [] as [u8; 0]);
}

#[test]
fn any_byte_after_the_frame_is_trailing() {
    const SKIPPABLE: &[u8] = b"\x50\x2a\x4d\x18\x00\x00\x00\x00";
    for suffix in [b"\0".as_slice(), b"x", CLI_FRAME, SKIPPABLE] {
        let frame = [CLI_FRAME, suffix].concat();
        assert_eq!(decode_zstd(frame.as_slice(), &mut Vec::new(), CLI_FRAME_BYTES).unwrap(), Unpacked::Trailing);
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
    let target = format!("{}/it'\\''s here/temper-box", root.display());
    assert_eq!(
        why.lines().last().unwrap(),
        format!(
            "Fetch it by hand: curl -fL https://site.test/temper/v0/0123456789ab/{}/temper-box.zst | zstd -dc > '{target}' && chmod +x '{target}'",
            platform()
        )
    );
    assert!(!why.starts_with("temper: "));
    let file = root.join("bin/0123456789ab").join(platform()).join("temper-box");
    for name in STAGING {
        assert!(!suffix(&file, name).exists());
    }
    fs::remove_dir_all(root).unwrap();
}
