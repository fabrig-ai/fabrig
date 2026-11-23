//! Whole fake-machine roots, including empty directories and their permissions.
use skein_fake_machine::{How, Is, Item, Machine, Opened};
use std::{fmt::Write as _, os::unix::fs::PermissionsExt, path::Path};

const ROOTS: [&str; 4] = ["state", "worker", "client", "setup"];

pub(crate) fn export(
    disk: &mut Machine,
    directory: &Path,
    configuration: &temper_playground::documents::Configuration,
) {
    std::fs::create_dir_all(directory).expect("temporary golden export");
    let mut modes = String::new();
    for (index, name) in ROOTS.iter().enumerate() {
        let root = disk.reopen_root(u32::try_from(index).expect("four roots"));
        save(disk, root, directory, name, &mut modes);
        disk.close(root);
    }
    std::fs::write(directory.join("modes.txt"), modes).expect("golden modes");
    std::fs::write(
        directory.join("configuration.json"),
        serde_json::to_vec(configuration).expect("configuration document"),
    )
    .expect("golden configuration");
}

fn save(disk: &mut Machine, opened: Opened, directory: &Path, path: &str, modes: &mut String) {
    let facts = disk.stat(opened);
    assert_eq!(facts.owner, 1000, "all roots belong to the service user");
    let kind = match facts.is {
        Is::Directory => "d",
        Is::File => "f",
        Is::Link | Is::Fifo | Is::Device => panic!("unexpected golden node {:?}", facts.is),
    };
    writeln!(modes, "{kind} {:o} {} {path}", facts.mode, facts.owner).expect("mode line");
    let target = host_path(directory, path);
    match facts.is {
        Is::Directory => {
            std::fs::create_dir_all(&target).expect("export directory");
            loop {
                let entries = disk.list(opened, 32, 4096).expect("export list");
                if entries.is_empty() {
                    break;
                }
                for (kind, name) in entries {
                    let how = match kind {
                        Is::Directory => How::DirectoryNoFollow,
                        Is::File => How::ReadNoFollow,
                        Is::Link | Is::Fifo | Is::Device => panic!("unexpected golden node {kind:?}"),
                    };
                    let child = disk.open(opened, &name, how).expect("export child");
                    let name = std::str::from_utf8(&name).expect("world's file names");
                    save(disk, child, directory, &format!("{path}/{name}"), modes);
                    disk.close(child);
                }
            }
        }
        Is::File => {
            let bytes =
                disk.read(opened, 0, u32::try_from(facts.size).expect("small golden file")).expect("export bytes");
            assert_eq!(u64::try_from(bytes.len()).expect("file size"), facts.size);
            std::fs::write(&target, bytes).expect("export file");
        }
        Is::Link | Is::Fifo | Is::Device => panic!("unexpected golden node {:?}", facts.is),
    }
    std::fs::set_permissions(target, std::fs::Permissions::from_mode(facts.mode)).expect("export mode");
}

fn host_path(directory: &Path, path: &str) -> std::path::PathBuf {
    // Git does not keep a fixture directory named .git. The manifest keeps the
    // machine's names; only the host representation escapes this component.
    let mut host = directory.to_path_buf();
    for part in path.split('/') {
        host.push(if part == ".git" { "dot-git" } else { part });
    }
    host
}

pub(crate) fn transcripts(directory: &Path, store: &mut crate::referee::Store) {
    let modes = std::fs::read_to_string(directory.join("modes.txt")).expect("kept modes");
    for line in modes.lines() {
        let path = line.split(' ').nth(3).expect("manifest path");
        let Some(relative) = path.strip_prefix("state/") else { continue };
        if !relative.starts_with("transcripts/")
            && jig_channel_kit::world::store::logs::path(relative.as_bytes()).is_none()
        {
            continue;
        }
        let host = host_path(directory, path);
        if host.is_file() {
            store.transcript_file(relative.as_bytes().to_vec(), std::fs::read(host).expect("kept payload"));
        }
    }
}

pub(crate) fn load(directory: &Path) -> Machine {
    // Reset the playground's per-run agent-root bindings as well as the disk.
    let _ = temper_playground::machine::fresh();
    let mut disk = Machine::for_user(1000);
    let modes = std::fs::read_to_string(directory.join("modes.txt")).expect("kept golden modes");
    let mut roots: [Vec<Item>; ROOTS.len()] = std::array::from_fn(|_| Vec::new());
    for line in modes.lines() {
        let mut fields = line.split(' ');
        let kind = fields.next().expect("kind");
        let mode = u32::from_str_radix(fields.next().expect("mode"), 8).expect("octal permissions");
        let owner = fields.next().expect("owner").parse().expect("numeric owner");
        let path = fields.next().expect("path");
        assert!(fields.next().is_none());
        let (name, relative) = path.split_once('/').unwrap_or((path, ""));
        let Some(index) = ROOTS.iter().position(|root| *root == name) else { continue };
        if path == name {
            assert_eq!((kind, mode, owner), ("d", 0o755, 1000), "world root metadata");
            continue;
        }
        let item = match kind {
            "d" => Item::directory(relative.as_bytes()),
            "f" => {
                Item::file(relative.as_bytes(), &std::fs::read(host_path(directory, path)).expect("kept golden bytes"))
            }
            other => panic!("golden node kind {other}"),
        };
        roots[index].push(item.mode(mode).owner(owner));
    }
    for items in roots {
        let root = disk.lay(&items);
        disk.close(root);
    }
    disk
}

pub(crate) fn active(disk: &mut Machine) -> Vec<u8> {
    let root = disk.reopen_root(0);
    let opened = disk.open(root, b"active", How::ReadNoFollow).expect("active file");
    let bytes = disk.read(opened, 0, 64).expect("active generation");
    disk.close(opened);
    disk.close(root);
    bytes
}

/// Append one valid map commit only in the loaded copy; the fixture stays byte-for-byte.
pub(crate) fn change_record(disk: &mut Machine, key: &[u8], value: Option<&[u8]>) {
    let active = active(disk);
    let active = std::str::from_utf8(&active).expect("active text").trim();
    let root = disk.reopen_root(0);
    let path = format!("generations/{active}/store/log-00000000000000000001");
    let log = disk.open(root, path.as_bytes(), How::WriteNoFollow).expect("floor log");
    let size = u32::try_from(disk.stat(log).size).expect("small log");
    let bytes = disk.read(log, 0, size).expect("log bytes");
    let (frames, used) = crate::referee::durable::frames(&bytes, 0);
    assert_eq!(used, bytes.len(), "whole golden log");
    let number = frames.last().expect("kept commits").0 + 1;
    let mut body = vec![if value.is_some() { 1 } else { 2 }];
    body.extend(u32::try_from(key.len()).expect("key length").to_be_bytes());
    body.extend(key);
    if let Some(value) = value {
        body.extend(u32::try_from(value.len()).expect("value length").to_be_bytes());
        body.extend(value);
    }
    let mut frame = b"SKVC".to_vec();
    frame.extend(1_u16.to_be_bytes());
    frame.extend(number.to_be_bytes());
    frame.extend(1_u32.to_be_bytes());
    frame.extend(u32::try_from(body.len()).expect("body length").to_be_bytes());
    frame.extend(body);
    frame.extend(jig_protocol_store::transcripts::frame::crc32c(&frame).to_be_bytes());
    disk.append(log, &frame).expect("copy's test commit");
    disk.sync(log);
    disk.close(log);
    disk.close(root);
}

type Image = std::collections::BTreeMap<(u32, Vec<u8>), (skein_fake_machine::Facts, Vec<u8>)>;
pub(crate) fn image(disk: &mut Machine) -> Image {
    let mut image = Image::new();
    for index in 0..4 {
        image.extend(image_root(disk, index));
    }
    image
}
pub(crate) fn image_root(disk: &mut Machine, index: u32) -> Image {
    let root = disk.reopen_root(index);
    let mut image = Image::new();
    collect(disk, root, index, &[], &mut image);
    disk.close(root);
    image
}
fn collect(disk: &mut Machine, opened: Opened, index: u32, path: &[u8], image: &mut Image) {
    let facts = disk.stat(opened);
    let bytes = if facts.is == Is::File {
        disk.read(opened, 0, u32::try_from(facts.size).expect("small file")).expect("image bytes")
    } else {
        Vec::new()
    };
    image.insert((index, path.to_vec()), (facts, bytes));
    if facts.is != Is::Directory {
        return;
    }
    loop {
        let entries = disk.list(opened, 32, 4096).expect("image directory");
        if entries.is_empty() {
            break;
        }
        for (kind, name) in entries {
            let how = if kind == Is::Directory { How::DirectoryNoFollow } else { How::ReadNoFollow };
            let child = disk.open(opened, &name, how).expect("image child");
            let prefix = if path.is_empty() { Vec::new() } else { [path, b"/"].concat() };
            collect(disk, child, index, &[prefix.as_slice(), &name].concat(), image);
            disk.close(child);
        }
    }
}

/// Compare without printing credentials or other payloads on a failure.
pub(crate) fn unchanged(before: &Image, after: &Image) {
    let missing: Vec<_> = before
        .keys()
        .filter(|path| !after.contains_key(*path))
        .map(|(root, path)| (*root, String::from_utf8_lossy(path).into_owned()))
        .collect();
    let added: Vec<_> = after
        .keys()
        .filter(|path| !before.contains_key(*path))
        .map(|(root, path)| (*root, String::from_utf8_lossy(path).into_owned()))
        .collect();
    assert!(missing.is_empty() && added.is_empty(), "paths changed: missing {missing:?}, added {added:?}");
    for (path, (facts, bytes)) in before {
        let (found_facts, found_bytes) = &after[path];
        assert_eq!(facts, found_facts, "metadata at {path:?}");
        assert!(bytes == found_bytes, "bytes at {path:?}");
    }
}

/// A refusal preserves the copy, apart from recovery's two empty next segments.
pub(crate) fn refused(before: &Image, disk: &mut Machine) {
    let mut after = image(disk);
    for path in ["generations/1/store/log-00000000000000000102", "generations/1/secrets/log-00000000000000000014"] {
        let (facts, bytes) = after.remove(&(0, path.as_bytes().to_vec())).expect("recovery's next empty log");
        assert_eq!(facts.is, Is::File);
        assert_eq!(facts.size, 0, "{path}");
        assert!(bytes.is_empty(), "{path}");
    }
    unchanged(before, &after);
}
