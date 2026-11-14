use crate::{
    diff::{self, Layer},
    parts::{data, live, packages, secrets, software},
    tests::fixtures,
};
use hearth_lib::encode::Digest;
use skein_lib::bytes::copy_of;
#[test]
fn a_changed_download_is_keyed_by_path() {
    let (before, after) = fixtures::changed_download();
    assert_eq!(
        software::change(&before, &after),
        software::Change {
            downloads: Box::new([software::DownloadKeyed::Changed {
                key: copy_of(b"/opt/tool"),
                before: software::Download {
                    path: copy_of(b"/opt/tool"),
                    sha256: fixtures::ABC_SHA256,
                    owner: copy_of(b"root"),
                    group: copy_of(b"root"),
                    how: software::How::File { mode: 0o755 }
                },
                after: software::Download {
                    path: copy_of(b"/opt/tool"),
                    sha256: Digest([2; 32]),
                    owner: copy_of(b"root"),
                    group: copy_of(b"root"),
                    how: software::How::File { mode: 0o755 }
                },
            }]),
            scripts: None,
        }
    );
}
#[test]
fn secrets_change_by_name_only() {
    let (before, after) = fixtures::changed_secrets();
    assert_eq!(
        secrets::change(&before, &after),
        secrets::Change {
            files: Box::new([secrets::Named::Changed(copy_of(b"key"))]),
            passwords: Box::new([secrets::Named::Changed(copy_of(b"alice"))]),
            identity: Box::new([secrets::Named::Changed(copy_of(b"ssh"))]),
        }
    );
}
#[test]
fn a_live_path_gaining_a_consumer_is_changed() {
    let (before, after) = fixtures::changed_live();
    assert_eq!(
        live::change(&before, &after),
        live::Change {
            paths: Box::new([live::ConsumersKeyed::Changed {
                key: copy_of(b"/etc/mailname"),
                before: Box::new([live::Consumer {
                    task: copy_of(b"motd"),
                    unit: Some(copy_of(b"postfix.service")),
                    action: live::Action::Reload
                }]),
                after: Box::new([
                    live::Consumer {
                        task: copy_of(b"motd"),
                        unit: Some(copy_of(b"postfix.service")),
                        action: live::Action::Reload
                    },
                    live::Consumer { task: copy_of(b"other"), unit: None, action: live::Action::None },
                ]),
            }])
        }
    );
}
#[test]
fn data_entries_are_removed_added_and_changed_in_key_order() {
    use data::{Data, DataChange, DataClass, DataEntry, DataKeyed, DataValue};
    let row =
        |name: &[u8], class| DataEntry { name: copy_of(name), value: DataValue { class, path: copy_of(b"/var/data") } };
    let removed = row(b"a", DataClass::Data);
    let added = row(b"b", DataClass::Cache);
    let was = row(b"c", DataClass::Core);
    let now = row(b"c", DataClass::Data);
    let same = row(b"d", DataClass::Cache);
    let before = Data { entries: Box::new([was.clone(), same.clone(), removed.clone()]) };
    let after = Data { entries: Box::new([same, now.clone(), added.clone()]) };
    assert_eq!(
        data::change(Some(&before), Some(&after)),
        DataChange {
            entries: Box::new([
                DataKeyed::Removed { key: copy_of(b"a"), value: removed.value },
                DataKeyed::Added { key: copy_of(b"b"), value: added.value },
                DataKeyed::Changed { key: copy_of(b"c"), before: was.value, after: now.value },
            ])
        }
    );
}
#[test]
fn live_paths_are_removed_added_and_changed_in_key_order() {
    let old = Box::new([live::Consumer { task: copy_of(b"task"), unit: None, action: live::Action::None }]);
    let new = Box::new([live::Consumer { task: copy_of(b"task"), unit: None, action: live::Action::Restart }]);
    let before = live::Live {
        paths: Box::new([
            live::Covered { path: copy_of(b"c"), consumers: old.clone() },
            live::Covered { path: copy_of(b"d"), consumers: old.clone() },
            live::Covered { path: copy_of(b"a"), consumers: old.clone() },
        ]),
    };
    let after = live::Live {
        paths: Box::new([
            live::Covered { path: copy_of(b"d"), consumers: old.clone() },
            live::Covered { path: copy_of(b"c"), consumers: new.clone() },
            live::Covered { path: copy_of(b"b"), consumers: new.clone() },
        ]),
    };
    assert_eq!(
        live::change(&before, &after),
        live::Change {
            paths: Box::new([
                live::ConsumersKeyed::Removed { key: copy_of(b"a"), value: old.clone() },
                live::ConsumersKeyed::Added { key: copy_of(b"b"), value: new.clone() },
                live::ConsumersKeyed::Changed { key: copy_of(b"c"), before: old, after: new },
            ])
        }
    );
}
#[test]
fn downloads_are_removed_added_and_changed_in_key_order() {
    let (mut before, mut after) = fixtures::changed_download();
    let mut was = fixtures::only(&before.downloads).clone();
    was.path = copy_of(b"c");
    let mut now = fixtures::only(&after.downloads).clone();
    now.path = copy_of(b"c");
    let mut removed = was.clone();
    removed.path = copy_of(b"a");
    let mut added = now.clone();
    added.path = copy_of(b"b");
    let mut same = was.clone();
    same.path = copy_of(b"d");
    before.downloads = Box::new([was.clone(), same.clone(), removed.clone()]);
    after.downloads = Box::new([same, now.clone(), added.clone()]);
    assert_eq!(
        software::change(&before, &after),
        software::Change {
            downloads: Box::new([
                software::DownloadKeyed::Removed { key: copy_of(b"a"), value: removed },
                software::DownloadKeyed::Added { key: copy_of(b"b"), value: added },
                software::DownloadKeyed::Changed { key: copy_of(b"c"), before: was, after: now },
            ]),
            scripts: None,
        }
    );
}
#[test]
fn secret_files_are_removed_added_and_changed_in_key_order() {
    let before = secrets::Secrets {
        files: Box::new([
            fixtures::secret_file(b"c", b"/etc/c", 0o600, b"root", b"root"),
            fixtures::secret_file(b"a", b"/etc/a", 0o600, b"root", b"root"),
        ]),
        passwords: Box::new([]),
        identity: Box::new([]),
    };
    let after = secrets::Secrets {
        files: Box::new([
            fixtures::secret_file(b"c", b"/etc/c", 0o640, b"root", b"root"),
            fixtures::secret_file(b"b", b"/etc/b", 0o600, b"root", b"root"),
        ]),
        passwords: Box::new([]),
        identity: Box::new([]),
    };
    assert_eq!(
        secrets::change(&before, &after),
        secrets::Change {
            files: Box::new([
                secrets::Named::Removed(copy_of(b"a")),
                secrets::Named::Added(copy_of(b"b")),
                secrets::Named::Changed(copy_of(b"c")),
            ]),
            passwords: Box::new([]),
            identity: Box::new([]),
        }
    );
}
#[test]
fn seeds_change_as_users_and_groups_added_removed_changed_by_name() {
    use packages::{SeedIds, SeedUserKeyed};
    let (before, after) = fixtures::changed_seeds();
    assert_eq!(
        packages::change(&before, &after).seeds.users.as_ref(),
        &[
            SeedUserKeyed::Added { key: copy_of(b"added"), value: SeedIds { uid: 122, gid: Some(123) } },
            SeedUserKeyed::Changed {
                key: copy_of(b"changed"),
                before: SeedIds { uid: 120, gid: None },
                after: SeedIds { uid: 121, gid: Some(121) }
            },
            SeedUserKeyed::Removed { key: copy_of(b"removed"), value: SeedIds { uid: 120, gid: Some(121) } },
        ]
    );
}
#[test]
fn seed_groups_change_by_name_in_byte_order() {
    use packages::SeedGroupKeyed;
    let (before, after) = fixtures::changed_seeds();
    assert_eq!(
        packages::change(&before, &after).seeds.groups.as_ref(),
        &[
            SeedGroupKeyed::Added { key: copy_of(b"added"), value: 122 },
            SeedGroupKeyed::Changed { key: copy_of(b"changed"), before: 120, after: 121 },
            SeedGroupKeyed::Removed { key: copy_of(b"removed"), value: 120 },
        ]
    );
}
#[test]
fn the_three_layers_are_slot_software_other() {
    use crate::parts::PartRecord;
    let (secrets, _) = fixtures::changed_secrets();
    let (software, _) = fixtures::changed_download();
    let (live, _) = fixtures::changed_live();
    assert_eq!(
        [
            diff::layer(&PartRecord::Secrets(secrets)),
            diff::layer(&PartRecord::Software(software)),
            diff::layer(&PartRecord::Live(live))
        ],
        [Layer::Slot, Layer::Software, Layer::Other]
    );
}

#[test]
fn a_fetch_hint_changed_alone_is_no_software_change() {
    assert_eq!(
        software::change(
            &fixtures::mirrored(b"https://a.example/tool"),
            &fixtures::mirrored(b"https://b.example/tool")
        ),
        software::Change { downloads: Box::new([]), scripts: None }
    );
}
