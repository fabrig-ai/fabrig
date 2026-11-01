//! Seeded deployment files (temper/docs/plan/playground-reset-r6/notes/contracts.md, T1–T4).
use super::{API_TOKEN, FORGE, GIT_TOKEN, HOOK_SECRET};
use crate::world::Story;
use skein_fake_machine::Item;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use temper_engine_domain_forge::{ApiTransport, Bot, ForgeRecord, Mode};

pub(crate) fn forge(story: Story) -> ForgeRecord {
    let mut record = if crate::lower::github::active() {
        let mut record = temper_github_app::github(1, FORGE, b"localhost", None);
        record.api.transport = ApiTransport::Plaintext;
        record.git = format!("http://{FORGE}").into_bytes().into();
        record.web = Some(record.git.clone());
        record
    } else {
        temper_github_app::forgejo(1, FORGE, format!("http://{FORGE}").as_bytes())
    };
    record.bot = Bot {
        login: b"worker".as_slice().into(),
        name: b"Worker".as_slice().into(),
        email: b"worker@example.invalid".as_slice().into(),
    };
    if matches!(story, Story::HeldChange) {
        record.mode = Mode::Owned;
        record.setup = Some(
            serde_json::to_vec(&serde_json::json!({
                "bot":{"login":"worker","email":"worker@example.invalid"},
                "mode":{"owned":{"forgejo":"fetched","runner":null,
                    "listen":{"socket":FORGE.to_string()},
                    "person":{"login":"person","email":"person@example.invalid"}}}
            }))
            .expect("owned setup body")
            .into(),
        );
    }
    record
}

pub(crate) fn files(machine: &mut skein_fake_machine::Machine, story: Story) -> Vec<Item> {
    let scratch = skein_scratch::Scratch::new("temper-one-process-seed");
    fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700)).expect("private seed directory");
    let root = machine.reopen_root(0);
    export(machine, root, scratch.path());
    machine.close(root);
    // The owned fixture reconciles from its admin record; GitHub stories paste their App.
    let records = if matches!(story, Story::HeldChange) {
        use temper_protocol_records::{self as records, Item};
        Box::new([(
            records::number(1, Item::AdminToken),
            records::encode(Item::AdminToken, crate::restart_control::ADMIN_TOKEN)
                .expect("fixture admin record")
                .into_boxed_slice(),
        )])
    } else if crate::lower::github::active() {
        Box::default()
    } else {
        temper_github_app::forgejo_records(1, API_TOKEN, 1, Some(HOOK_SECRET), GIT_TOKEN, FORGE)
    };
    temper_github_app::seed_forges(scratch.path(), &[forge(story)], &records).expect("seed deployment forge");
    let mut items = Vec::new();
    copy(scratch.path(), Path::new(""), &mut items);
    items
}

fn export(machine: &mut skein_fake_machine::Machine, root: skein_fake_machine::Opened, path: &Path) {
    use skein_fake_machine::{How, Is};
    let entries = machine.list(root, 4096, 262_144).expect("initialized fixture entries");
    assert!(entries.len() < 4096, "whole initialized fixture directory");
    for (kind, name) in entries {
        let destination = path.join(std::ffi::OsStr::new(std::str::from_utf8(&name).expect("fixture filename")));
        match kind {
            Is::Directory => {
                fs::create_dir(&destination).expect("seed subdirectory");
                fs::set_permissions(&destination, fs::Permissions::from_mode(0o700)).expect("private subdirectory");
                let directory = machine.open(root, &name, How::Directory).expect("fixture directory");
                export(machine, directory, &destination);
                machine.close(directory);
            }
            Is::File => {
                let file = machine.open(root, &name, How::Read).expect("fixture file");
                let size = u32::try_from(machine.stat(file).size).expect("fixture file size");
                let bytes = machine.read(file, 0, size).expect("fixture file bytes");
                machine.close(file);
                fs::write(&destination, bytes).expect("seed fixture file");
                fs::set_permissions(&destination, fs::Permissions::from_mode(0o600)).expect("private fixture file");
            }
            Is::Link | Is::Fifo | Is::Device => panic!("fixture store contains only files and directories"),
        }
    }
}

// Only this harness's freshly generated fixture store crosses into the simulated disk.
fn copy(root: &Path, relative: &Path, items: &mut Vec<Item>) {
    let mut entries: Vec<_> = fs::read_dir(root.join(relative)).expect("seed directory").collect();
    entries.sort_by_key(|entry| entry.as_ref().expect("seed entry").file_name());
    for entry in entries {
        let entry = entry.expect("seed entry");
        let path = relative.join(entry.file_name());
        let metadata = entry.metadata().expect("seed metadata");
        let name = path.as_os_str().as_encoded_bytes();
        if metadata.is_dir() {
            items.push(Item::directory(name).mode(0o700));
            copy(root, &path, items);
        } else {
            assert!(metadata.is_file(), "seed contains only files and directories");
            items.push(Item::file(name, &fs::read(entry.path()).expect("seed fixture bytes")).mode(0o600));
        }
    }
}
