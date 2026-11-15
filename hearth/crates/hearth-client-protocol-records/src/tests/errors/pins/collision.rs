use super::assert_prints;
use crate::tests::fixtures;
use hearth_domain_render_vocabulary as render;
use render::errors::{Collision, Error, Id, Kind, Subject};

use skein_lib::bytes::copy_of;
fn collision(collision: Collision) -> Error {
    let mut error = fixtures::path_collision();
    error.kind = Kind::Collision(collision);
    error
}

#[expect(clippy::too_many_lines, reason = "literal text and JSON pins keep each error beside its bytes")]
#[test]
fn collision_bytes() {
    let cases = [
        (
            collision(Collision::Given {
                subject: Subject::DataName,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: data item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"data_name","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::DataPath,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: data path item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"data_path","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Secret,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: secret item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"secret","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Identity,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: identity item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"identity","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Download,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: download item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"download","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::User,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: user item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"user","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Group,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: group item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"group","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Preset,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: preset item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"preset","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Tmpfiles,
                name: copy_of(b"d /run/a"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: tmpfiles d /run/a is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"tmpfiles","name":"d /run/a","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Preseed,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: preseed item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"preseed","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Parameter,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: boot parameter item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"parameter","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Given {
                subject: Subject::Range,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: subordinate ids item is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"range","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Tree,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"tree","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::DataName,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: data \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"data_name","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::DataPath,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: data path \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"data_path","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Secret,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: secret \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"secret","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Identity,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: identity \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"identity","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Download,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: download \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"download","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::User,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: user \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"user","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Group,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: group \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"group","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Preset,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: preset \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"preset","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Tmpfiles,
                name: copy_of(b"d /run/a"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: tmpfiles d /run/a differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"tmpfiles","name":"d /run/a","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Preseed,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: preseed \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"preseed","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Parameter,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: boot parameter \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"parameter","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Differs {
                subject: Subject::Range,
                name: copy_of(b"item"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: subordinate ids \"item\" differs between banner and motd (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"differs","subject":"range","name":"item","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap { subject: Subject::Tree, first: copy_of(b"alice"), second: copy_of(b"ci") }),
            "tasks/banner/task.json:1:12: x1-prod: alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"tree","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::DataName,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: data alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"data_name","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::DataPath,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: data path alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"data_path","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::Secret,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: secret alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"secret","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::Identity,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: identity alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"identity","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::Download,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: download alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"download","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap { subject: Subject::User, first: copy_of(b"alice"), second: copy_of(b"ci") }),
            "tasks/banner/task.json:1:12: x1-prod: user alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"user","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap { subject: Subject::Group, first: copy_of(b"alice"), second: copy_of(b"ci") }),
            "tasks/banner/task.json:1:12: x1-prod: group alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"group","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::Preset,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: preset alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"preset","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::Tmpfiles,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: tmpfiles alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"tmpfiles","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::Preseed,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: preseed alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"preseed","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap {
                subject: Subject::Parameter,
                first: copy_of(b"alice"),
                second: copy_of(b"ci"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: boot parameter alice overlaps ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"parameter","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Overlap { subject: Subject::Range, first: copy_of(b"alice"), second: copy_of(b"ci") }),
            "tasks/banner/task.json:1:12: x1-prod: subordinate ids of alice overlap ci's (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"overlap","subject":"range","first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Shared { id: Id::Gid, value: 1000, first: copy_of(b"alice"), second: copy_of(b"ci") }),
            "tasks/banner/task.json:1:12: x1-prod: gid 1000 is shared by alice and ci (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"shared","id":"gid","value":1000,"first":"alice","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
        (
            collision(Collision::Hidden {
                path: copy_of(b"/var/mail/a"),
                data: copy_of(b"/var/mail"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            "tasks/banner/task.json:1:12: x1-prod: /var/mail/a is under the data path /var/mail (a mount would hide it) (tasks/motd/tree/etc/motd)\n",
            concat!(
                r#"{"record":"error","kind":"collision","collision":{"kind":"hidden","path":"/var/mail/a","data":"/var/mail","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}
