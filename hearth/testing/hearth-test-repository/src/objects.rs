//! A scratch repository's objects as `git add -A` and `git commit` would write them
//! (hearth/docs/design/testing/README.md, 4.1).
use sha1::{Digest, Sha1, digest::Output};
use std::{
    fmt::Write,
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};

pub(crate) fn lay_out(work_tree: &Path) {
    let git = work_tree.join(".git");
    fs::create_dir_all(git.join("objects")).expect("objects directory");
    fs::create_dir_all(git.join("refs/heads")).expect("heads directory");
    fs::write(git.join("HEAD"), b"ref: refs/heads/main\n").expect("symbolic HEAD");
    fs::write(
        git.join("config"),
        b"[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n",
    )
    .expect("repository config");
}

pub(crate) fn commit(work_tree: &Path, message: &str) -> String {
    let git = work_tree.join(".git");
    let root = tree(&git, work_tree).unwrap_or_else(|| object(&git, "tree", &[]));
    let mut body = format!("tree {:x}\n", Output::<Sha1>::from(root));
    let head = git.join("refs/heads/main");
    match fs::read_to_string(&head) {
        Ok(parent) => writeln!(body, "parent {}", parent.trim()).expect("parent line"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("read parent: {error}"),
    }
    for role in ["author", "committer"] {
        writeln!(body, "{role} {} <{}> {}", super::COMMIT_NAME, super::COMMIT_EMAIL, super::COMMIT_DATE)
            .expect("identity line");
    }
    writeln!(body, "\n{message}").expect("commit message");
    let id = format!("{:x}", Output::<Sha1>::from(object(&git, "commit", body.as_bytes())));
    fs::write(head, format!("{id}\n")).expect("commit ref");
    id
}

fn tree(git: &Path, directory: &Path) -> Option<[u8; 20]> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory).expect("work tree directory") {
        let entry = entry.expect("work tree entry");
        let path = entry.path();
        if path == git {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).expect("work tree metadata");
        let name = entry.file_name();
        let mut key = name.as_bytes().to_vec();
        let (mode, id) = if metadata.is_dir() {
            let Some(id) = tree(git, &path) else { continue };
            key.push(b'/');
            ("40000", id)
        } else if metadata.is_symlink() {
            let target = fs::read_link(&path).expect("link target");
            ("120000", object(git, "blob", target.as_os_str().as_bytes()))
        } else {
            assert!(metadata.is_file(), "fixture is a file, directory or link");
            let mode = if metadata.permissions().mode() & 0o100 != 0 { "100755" } else { "100644" };
            (mode, object(git, "blob", &fs::read(&path).expect("file bytes")))
        };
        let mut body = Vec::new();
        body.extend_from_slice(mode.as_bytes());
        body.push(b' ');
        body.extend_from_slice(name.as_bytes());
        body.push(0);
        body.extend_from_slice(&id);
        entries.push((key, body));
    }
    if entries.is_empty() {
        return None;
    }
    entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    let body: Vec<_> = entries.into_iter().flat_map(|(_, body)| body).collect();
    Some(object(git, "tree", &body))
}

fn object(git: &Path, kind: &str, body: &[u8]) -> [u8; 20] {
    let mut raw = format!("{kind} {}\0", body.len()).into_bytes();
    raw.extend_from_slice(body);
    let id = Sha1::digest(&raw);
    let name = format!("{id:x}");
    let (prefix, suffix) = name.split_at(2);
    let directory = git.join("objects").join(prefix);
    fs::create_dir_all(&directory).expect("loose object directory");
    let path = directory.join(suffix);
    if !path.exists() {
        fs::write(path, stored(&raw)).expect("loose object");
    }
    id.into()
}

fn stored(raw: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x78, 0x01];
    let mut blocks = raw.chunks(usize::from(u16::MAX)).peekable();
    if raw.is_empty() {
        bytes.extend_from_slice(&[1, 0, 0, 255, 255]);
    }
    while let Some(block) = blocks.next() {
        bytes.push(u8::from(blocks.peek().is_none()));
        let len = u16::try_from(block.len()).expect("stored block length");
        bytes.extend_from_slice(&len.to_le_bytes());
        bytes.extend_from_slice(&(!len).to_le_bytes());
        bytes.extend_from_slice(block);
    }
    let (mut low, mut high) = (1_u32, 0_u32);
    for byte in raw {
        low = (low + u32::from(*byte)) % 65_521;
        high = (high + low) % 65_521;
    }
    bytes.extend_from_slice(&((high << 16) | low).to_be_bytes());
    bytes
}
