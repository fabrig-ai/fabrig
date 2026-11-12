//! Whole fake-machine roots, including empty directories and their permissions.
use skein_fake_machine::{How, Is, Item, Machine, Opened};
use std::{fmt::Write as _, os::unix::fs::PermissionsExt, path::Path};

const ROOTS: [&str; 4] = ["state", "worker", "client", "setup"];

pub(super) fn export(disk: &mut Machine, directory: &Path, configuration: &[u8]) {
    std::fs::create_dir_all(directory).expect("temporary golden export");
    let mut modes = String::new();
    for (index, name) in ROOTS.iter().enumerate() {
        let root = disk.reopen_root(u32::try_from(index).expect("four roots"));
        save(disk, root, directory, name, &mut modes);
        disk.close(root);
    }
    std::fs::write(directory.join("modes.txt"), modes).expect("golden modes");
    std::fs::write(directory.join("configuration.json"), configuration).expect("golden configuration");
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

pub(super) fn transcripts(directory: &Path, store: &mut crate::referee::Store) {
    let modes = std::fs::read_to_string(directory.join("modes.txt")).expect("kept modes");
    for line in modes.lines() {
        let path = line.split(' ').nth(3).expect("manifest path");
        let Some(relative) = path.strip_prefix("state/transcripts/") else { continue };
        let host = host_path(directory, path);
        if host.is_file() {
            store.transcript_file(
                format!("transcripts/{relative}").into_bytes(),
                std::fs::read(host).expect("kept transcript"),
            );
        }
    }
}

pub(super) fn load(directory: &Path) -> Machine {
    // Reset the playground's per-run agent-root bindings as well as the disk.
    let _ = temper_playground::machine::fresh();
    let mut disk = Machine::for_user(1000);
    let modes = std::fs::read_to_string(directory.join("modes.txt")).expect("kept golden modes");
    for name in ROOTS {
        let mut items = Vec::new();
        for line in modes.lines() {
            let mut fields = line.split(' ');
            let kind = fields.next().expect("kind");
            let mode = u32::from_str_radix(fields.next().expect("mode"), 8).expect("octal permissions");
            let owner = fields.next().expect("owner").parse().expect("numeric owner");
            let path = fields.next().expect("path");
            assert!(fields.next().is_none());
            if path == name {
                assert_eq!((kind, mode, owner), ("d", 0o755, 1000), "world root metadata");
                continue;
            }
            let Some(relative) = path.strip_prefix(&format!("{name}/")) else { continue };
            let host = host_path(directory, path);
            let item = match kind {
                "d" => Item::directory(relative.as_bytes()),
                "f" => Item::file(relative.as_bytes(), &std::fs::read(host).expect("kept golden bytes")),
                other => panic!("golden node kind {other}"),
            };
            items.push(item.mode(mode).owner(owner));
        }
        let root = disk.lay(&items);
        disk.close(root);
    }
    disk
}
