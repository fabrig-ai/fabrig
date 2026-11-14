use super::*;
use hearth_domain_render_vocabulary as render;
use render::{
    diff::*,
    evaluate::Value,
    parts::{
        data::*,
        exports::{Change as ExportsChange, ExportChange},
        live,
        meta::{Change as MetaChange, MemberPair, PlacementPair, *},
        secrets, software,
    },
    revision::{Firmware, Runs as PlacementKind},
};
use skein_lib::bytes::copy_of;

#[test]
fn target_and_rebuild_forms() {
    check(&[
        (
            fixtures::diff_change(
                b"west-prod",
                TargetChange::Added { spec_digest: hearth_lib::encode::Digest([1; 32]) },
            ),
            "west-prod                                           added spec 010101010101\n",
            "{\"record\":\"change\",\"target\":\"west-prod\",\"kind\":\"added\",\"added\":{\"spec_digest\":\"0101010101010101010101010101010101010101010101010101010101010101\"}}\n",
        ),
        (
            fixtures::diff_change(b"west-prod", TargetChange::Removed),
            "west-prod                                           removed\n",
            "{\"record\":\"change\",\"target\":\"west-prod\",\"kind\":\"removed\"}\n",
        ),
        (
            fixtures::diff_change(b"west-prod", TargetChange::Failed { side: Side::Both }),
            "west-prod                                           failed (both)\n",
            "{\"record\":\"change\",\"target\":\"west-prod\",\"kind\":\"failed\",\"failed\":{\"side\":\"both\"}}\n",
        ),
        (
            fixtures::diff_change(
                b"west-prod",
                TargetChange::Changed { parts: Box::new([]), rebuild: Rebuild::FromSoftware },
            ),
            "west-prod                                           rebuild from software\n",
            "{\"record\":\"change\",\"target\":\"west-prod\",\"kind\":\"changed\",\"changed\":{\"parts\":[],\"rebuild\":{\"kind\":\"from_software\"}}}\n",
        ),
        (
            fixtures::diff_change(
                b"west-prod",
                TargetChange::Changed {
                    parts: Box::new([]),
                    rebuild: Rebuild::FromPackages { resolution: Needs::Needed },
                },
            ),
            "west-prod                                           rebuild from packages, resolution needed\n",
            "{\"record\":\"change\",\"target\":\"west-prod\",\"kind\":\"changed\",\"changed\":{\"parts\":[],\"rebuild\":{\"kind\":\"from_packages\",\"resolution\":\"needed\"}}}\n",
        ),
    ]);
}
#[test]
fn meta_member_and_placement_changes() {
    let member = MetaChange {
        base: None,
        member: Some(MemberPair { before: Some(copy_of(b"a")), after: Some(copy_of(b"b")) }),
        environment: None,
        placement: None,
    };
    let vm = MetaChange {
        base: None,
        member: None,
        environment: None,
        placement: Some(PlacementPair {
            before: Placement {
                kind: PlacementKind::Metal,
                firmware: Some(Firmware::Efi),
                hardware: Some(copy_of(b"old")),
                on: None,
            },
            after: Placement {
                kind: PlacementKind::Vm,
                firmware: Some(Firmware::Bios),
                hardware: Some(copy_of(b"new")),
                on: Some(copy_of(b"host-prod")),
            },
        }),
    };
    check(&[
        (
            change(PartChange::Meta(member)),
            change_text!("  meta     ~ member  a → b\n"),
            change_json!(r#"{"kind":"meta","member":{"before":"a","after":"b"}}"#),
        ),
        (
            change(PartChange::Meta(vm)),
            change_text!("  meta     ~ placement  metal efi old → vm bios new on host-prod\n"),
            change_json!(
                r#"{"kind":"meta","placement":{"before":{"firmware":"efi","hardware":"old","kind":"metal"},"after":{"firmware":"bios","hardware":"new","kind":"vm","on":"host-prod"}}}"#
            ),
        ),
    ]);
}
#[test]
fn secrets_password_and_identity_names() {
    check(&[
        (
            change(PartChange::Secrets(secrets::Change {
                files: Box::new([]),
                passwords: Box::new([secrets::Named::Added(copy_of(b"alice"))]),
                identity: Box::new([secrets::Named::Added(copy_of(b"ssh"))]),
            })),
            change_text!("  secrets  + password alice\n  secrets  + identity ssh\n"),
            change_json!(
                r#"{"kind":"secrets","secrets":{"files":[],"passwords":[{"key":"alice","kind":"added"}],"identity":[{"key":"ssh","kind":"added"}]}}"#
            ),
        ),
        (
            change(PartChange::Secrets(secrets::Change {
                files: Box::new([]),
                passwords: Box::new([secrets::Named::Removed(copy_of(b"alice"))]),
                identity: Box::new([secrets::Named::Removed(copy_of(b"ssh"))]),
            })),
            change_text!("  secrets  - password alice\n  secrets  - identity ssh\n"),
            change_json!(
                r#"{"kind":"secrets","secrets":{"files":[],"passwords":[{"key":"alice","kind":"removed"}],"identity":[{"key":"ssh","kind":"removed"}]}}"#
            ),
        ),
    ]);
}
fn download(digest: [u8; 32]) -> software::Download {
    software::Download {
        path: copy_of(b"/opt/tool"),
        sha256: hearth_lib::encode::Digest(digest),
        owner: copy_of(b"root"),
        group: copy_of(b"root"),
        how: software::How::File { mode: 0o755 },
    }
}
fn software(entry: software::DownloadKeyed) -> Record {
    change(PartChange::Software(software::Change { downloads: Box::new([entry]), scripts: None }))
}
#[test]
fn download_keyed_file_forms() {
    check(&[
        (
            software(software::DownloadKeyed::Added { key: copy_of(b"/opt/tool"), value: download([1; 32]) }),
            change_text!("  software + /opt/tool                              file 0755 root:root 010101010101\n"),
            change_json!(
                r#"{"kind":"software","software":{"downloads":[{"kind":"added","value":{"group":"root","mode":"0755","owner":"root","sha256":"0101010101010101010101010101010101010101010101010101010101010101","unpack":"file"},"key":"/opt/tool"}]}}"#
            ),
        ),
        (
            software(software::DownloadKeyed::Removed { key: copy_of(b"/opt/tool"), value: download([1; 32]) }),
            change_text!("  software - /opt/tool                              file 0755 root:root 010101010101\n"),
            change_json!(
                r#"{"kind":"software","software":{"downloads":[{"kind":"removed","value":{"group":"root","mode":"0755","owner":"root","sha256":"0101010101010101010101010101010101010101010101010101010101010101","unpack":"file"},"key":"/opt/tool"}]}}"#
            ),
        ),
        (
            software(software::DownloadKeyed::Changed {
                key: copy_of(b"/opt/tool"),
                before: download([1; 32]),
                after: download([2; 32]),
            }),
            change_text!("  software ~ /opt/tool                              file 0755 root:root 020202020202\n"),
            change_json!(
                r#"{"kind":"software","software":{"downloads":[{"after":{"group":"root","mode":"0755","owner":"root","sha256":"0202020202020202020202020202020202020202020202020202020202020202","unpack":"file"},"before":{"group":"root","mode":"0755","owner":"root","sha256":"0101010101010101010101010101010101010101010101010101010101010101","unpack":"file"},"kind":"changed","key":"/opt/tool"}]}}"#
            ),
        ),
    ]);
}
fn live(entry: live::ConsumersKeyed) -> Record {
    change(PartChange::Live(live::Change { paths: Box::new([entry]) }))
}
#[test]
fn live_keyed_actions_and_units() {
    use super::specs::consumers;
    check(&[
        (
            live(live::ConsumersKeyed::Added {
                key: copy_of(b"/etc/mailname"),
                value: consumers(live::Action::Reload),
            }),
            change_text!("  live     + /etc/mailname                          mail mail.service reload\n"),
            change_json!(
                r#"{"kind":"live","live":{"paths":[{"kind":"added","value":[{"action":"reload","task":"mail","unit":"mail.service"}],"key":"/etc/mailname"}]}}"#
            ),
        ),
        (
            live(live::ConsumersKeyed::Removed {
                key: copy_of(b"/etc/mailname"),
                value: consumers(live::Action::Restart),
            }),
            change_text!("  live     - /etc/mailname                          mail mail.service restart\n"),
            change_json!(
                r#"{"kind":"live","live":{"paths":[{"kind":"removed","value":[{"action":"restart","task":"mail","unit":"mail.service"}],"key":"/etc/mailname"}]}}"#
            ),
        ),
        (
            live(live::ConsumersKeyed::Changed {
                key: copy_of(b"/etc/mailname"),
                before: consumers(live::Action::Restart),
                after: consumers(live::Action::ReloadOrRestart),
            }),
            change_text!("  live     ~ /etc/mailname                          mail mail.service reload-or-restart\n"),
            change_json!(
                r#"{"kind":"live","live":{"paths":[{"after":[{"action":"reload-or-restart","task":"mail","unit":"mail.service"}],"before":[{"action":"restart","task":"mail","unit":"mail.service"}],"kind":"changed","key":"/etc/mailname"}]}}"#
            ),
        ),
        (
            live(live::ConsumersKeyed::Changed {
                key: copy_of(b"/etc/mailname"),
                before: consumers(live::Action::Reload),
                after: consumers(live::Action::Start),
            }),
            change_text!("  live     ~ /etc/mailname                          mail mail.service start\n"),
            change_json!(
                r#"{"kind":"live","live":{"paths":[{"after":[{"action":"start","task":"mail","unit":"mail.service"}],"before":[{"action":"reload","task":"mail","unit":"mail.service"}],"kind":"changed","key":"/etc/mailname"}]}}"#
            ),
        ),
    ]);
}
fn data(entry: DataKeyed) -> Record {
    change(PartChange::Data(DataChange { entries: Box::new([entry]) }))
}
#[test]
fn data_keyed_cache_forms() {
    check(&[
        (
            data(DataKeyed::Added { key: copy_of(b"iwd"), value: fixtures::data::value(DataClass::Cache) }),
            change_text!("  data     + iwd                                    cache /var/lib/iwd\n"),
            change_json!(
                r#"{"kind":"data","data":{"entries":[{"key":"iwd","kind":"added","value":{"class":{"kind":"cache"},"path":"/var/lib/iwd"}}]}}"#
            ),
        ),
        (
            data(DataKeyed::Removed { key: copy_of(b"iwd"), value: fixtures::data::value(DataClass::Cache) }),
            change_text!("  data     - iwd                                    cache /var/lib/iwd\n"),
            change_json!(
                r#"{"kind":"data","data":{"entries":[{"key":"iwd","kind":"removed","value":{"class":{"kind":"cache"},"path":"/var/lib/iwd"}}]}}"#
            ),
        ),
        (
            data(DataKeyed::Changed {
                key: copy_of(b"iwd"),
                before: fixtures::data::value(DataClass::Data),
                after: fixtures::data::value(DataClass::Cache),
            }),
            change_text!("  data     ~ iwd                                    cache /var/lib/iwd\n"),
            change_json!(
                r#"{"kind":"data","data":{"entries":[{"after":{"class":{"kind":"cache"},"path":"/var/lib/iwd"},"before":{"class":{"kind":"data"},"path":"/var/lib/iwd"},"key":"iwd","kind":"changed"}]}}"#
            ),
        ),
    ]);
}
fn export(before: Value, after: Value) -> Record {
    change(PartChange::Exports(ExportsChange {
        entries: Box::new([ExportChange {
            task: copy_of(b"vpn"),
            name: copy_of(b"endpoint"),
            before: Some(before),
            after: Some(after),
        }]),
    }))
}
#[test]
fn export_change_value_forms() {
    check(&[
        (
            export(Value::Bool(false), Value::Bool(true)),
            change_text!("  exports  ~ vpn.endpoint   false → true\n"),
            change_json!(
                r#"{"kind":"exports","exports":[{"after":true,"before":false,"name":"endpoint","task":"vpn"}]}"#
            ),
        ),
        (
            export(Value::Int(-7), Value::Int(8)),
            change_text!("  exports  ~ vpn.endpoint   -7 → 8\n"),
            change_json!(r#"{"kind":"exports","exports":[{"after":8,"before":-7,"name":"endpoint","task":"vpn"}]}"#),
        ),
        (
            export(Value::List(Box::new([Value::Int(1)])), Value::List(Box::new([Value::Bool(false), Value::Int(2)]))),
            change_text!("  exports  ~ vpn.endpoint   [1] → [false,2]\n"),
            change_json!(
                r#"{"kind":"exports","exports":[{"after":[false,2],"before":[1],"name":"endpoint","task":"vpn"}]}"#
            ),
        ),
        (
            export(fixtures::export_map(false), fixtures::export_map(true)),
            change_text!("  exports  ~ vpn.endpoint   {\"up\":false} → {\"up\":true}\n"),
            change_json!(
                r#"{"kind":"exports","exports":[{"after":{"up":true},"before":{"up":false},"name":"endpoint","task":"vpn"}]}"#
            ),
        ),
    ]);
}
