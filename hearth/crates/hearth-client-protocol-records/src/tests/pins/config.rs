use super::*;
use crate::{diff::*, parts::config::*};
use skein_lib::bytes::copy_of;

macro_rules! config_diff_json {
    ($($body:literal),* $(,)?) => { change_json!(r#"{"kind":"config","groups":[],"users":[],"members":[],"presets":[],"tmpfiles":[],"tree":["#, $($body),*, "]}") };
}

#[test]
fn config_nodes_and_optional_account_ids() {
    let mut nodes = config();
    nodes.tree = fixtures::config_mut(&mut fixtures::links_and_directories()).tree.clone();
    let mut accounts = config();
    accounts.groups = Box::new([Group { name: copy_of(b"ops"), kind: GroupKind::Sysusers { gid: None } }]);
    accounts.users = Box::new([User { name: copy_of(b"alice"), kind: user() }]);
    check(&[
        (
            spec(PartRecord::Config(nodes)),
            spec_text!(
                "config       part 000000000000\n  /etc/mtab                link root:root /proc/self/mounts\n  /var/lib/motd            directory 0750 root:adm\n"
            ),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"config","digest":"0000000000000000000000000000000000000000000000000000000000000000","config":{"groups":{},"members":[],"presets":{},"scripts":[],"tmpfiles":{},"tree":{"/etc/mtab":{"group":"root","kind":"link","owner":"root","target":"/proc/self/mounts"},"/var/lib/motd":{"group":"adm","kind":"directory","mode":"0750","owner":"root"}},"users":{}}}"#,
                "\n"
            ),
        ),
        (
            spec(PartRecord::Config(accounts)),
            spec_text!(
                "config       part 000000000000\n  groups     ops -\n  users      alice - ops /home/alice /bin/sh\n"
            ),
            spec_json!(
                r#"{"record":"part","target":"x1-prod","name":"config","digest":"0000000000000000000000000000000000000000000000000000000000000000","config":{"groups":{"ops":{"kind":"sysusers"}},"members":[],"presets":{},"scripts":[],"tmpfiles":{},"tree":{},"users":{"alice":{"gecos":"","group":"ops","home":"/home/alice","kind":"sysusers","shell":"/bin/sh"}}}}"#,
                "\n"
            ),
        ),
    ]);
}
fn user() -> UserKind {
    UserKind::Sysusers {
        uid: None,
        group: copy_of(b"ops"),
        gecos: copy_of(b""),
        home: copy_of(b"/home/alice"),
        shell: copy_of(b"/bin/sh"),
    }
}
fn line(kind: crate::parts::config::LineKind) -> Line {
    Line { kind, mode: None, owner: None, group: None, age: Some(copy_of(b"7d")), argument: Some(copy_of(b"target")) }
}
fn tmpfile(kind: crate::parts::config::LineKind) -> Record {
    let mut config = config();
    config.tmpfiles = Box::new([Tmpfile { path: copy_of(b"/tmp/item"), line: line(kind) }]);
    spec(PartRecord::Config(config))
}
macro_rules! tmp_json {
    ($body:literal) => { spec_json!(r#"{"record":"part","target":"x1-prod","name":"config","digest":"0000000000000000000000000000000000000000000000000000000000000000","config":{"groups":{},"members":[],"presets":{},"scripts":[],"tmpfiles":{"/tmp/item":{"age":"7d","argument":"target","type":""#,
        $body, r#""}},"tree":{},"users":{}}}"#, "\n") };
}
#[test]
fn tmpfile_line_kinds_with_optional_fields() {
    use crate::parts::config::LineKind;
    check(&[
        (
            tmpfile(LineKind::CapitalD),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item D - - - 7d target\n"),
            tmp_json!("D"),
        ),
        (
            tmpfile(LineKind::E),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item e - - - 7d target\n"),
            tmp_json!("e"),
        ),
        (
            tmpfile(LineKind::F),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item f - - - 7d target\n"),
            tmp_json!("f"),
        ),
        (
            tmpfile(LineKind::FPlus),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item f+ - - - 7d target\n"),
            tmp_json!("f+"),
        ),
        (
            tmpfile(LineKind::W),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item w - - - 7d target\n"),
            tmp_json!("w"),
        ),
        (
            tmpfile(LineKind::WPlus),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item w+ - - - 7d target\n"),
            tmp_json!("w+"),
        ),
        (
            tmpfile(LineKind::CapitalL),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item L - - - 7d target\n"),
            tmp_json!("L"),
        ),
        (
            tmpfile(LineKind::CapitalLPlus),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item L+ - - - 7d target\n"),
            tmp_json!("L+"),
        ),
        (
            tmpfile(LineKind::CapitalC),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item C - - - 7d target\n"),
            tmp_json!("C"),
        ),
        (
            tmpfile(LineKind::Z),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item z - - - 7d target\n"),
            tmp_json!("z"),
        ),
        (
            tmpfile(LineKind::CapitalZ),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item Z - - - 7d target\n"),
            tmp_json!("Z"),
        ),
        (
            tmpfile(LineKind::R),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item r - - - 7d target\n"),
            tmp_json!("r"),
        ),
        (
            tmpfile(LineKind::CapitalR),
            spec_text!("config       part 000000000000\n  tmpfiles   /tmp/item R - - - 7d target\n"),
            tmp_json!("R"),
        ),
    ]);
}
fn entry(entry: EntryChange) -> Record {
    let mut config = config_change();
    config.tree = Box::new([entry]);
    change(PartChange::Config(config))
}
fn link(target: &[u8]) -> Node {
    Node::Link { target: copy_of(target), owner: copy_of(b"root"), group: copy_of(b"root") }
}
fn directory(mode: u16) -> Node {
    Node::Directory { mode, owner: copy_of(b"root"), group: copy_of(b"root") }
}
#[test]
fn tree_entry_change_forms() {
    check(&[
        (
            entry(EntryChange::Added { path: copy_of(b"/etc/item"), node: fixtures::file([1; 32]) }),
            change_text!("  config   + /etc/item                              0644 root:root\n"),
            config_diff_json!(
                r#"{"kind":"added","node":{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","group":"root","kind":"file","mode":"0644","owner":"root"},"path":"/etc/item"}"#
            ),
        ),
        (
            entry(EntryChange::Removed { path: copy_of(b"/etc/item"), node: fixtures::file([1; 32]) }),
            change_text!("  config   - /etc/item                              0644 root:root\n"),
            config_diff_json!(
                r#"{"kind":"removed","node":{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","group":"root","kind":"file","mode":"0644","owner":"root"},"path":"/etc/item"}"#
            ),
        ),
        (
            entry(EntryChange::Added { path: copy_of(b"/etc/item"), node: link(b"old") }),
            change_text!("  config   + /etc/item                              link root:root old\n"),
            config_diff_json!(
                r#"{"kind":"added","node":{"group":"root","kind":"link","owner":"root","target":"old"},"path":"/etc/item"}"#
            ),
        ),
        (
            entry(EntryChange::Removed { path: copy_of(b"/etc/item"), node: link(b"old") }),
            change_text!("  config   - /etc/item                              link root:root old\n"),
            config_diff_json!(
                r#"{"kind":"removed","node":{"group":"root","kind":"link","owner":"root","target":"old"},"path":"/etc/item"}"#
            ),
        ),
        (
            entry(EntryChange::Added { path: copy_of(b"/etc/item"), node: directory(0o750) }),
            change_text!("  config   + /etc/item                              directory 0750 root:root\n"),
            config_diff_json!(
                r#"{"kind":"added","node":{"group":"root","kind":"directory","mode":"0750","owner":"root"},"path":"/etc/item"}"#
            ),
        ),
        (
            entry(EntryChange::Removed { path: copy_of(b"/etc/item"), node: directory(0o750) }),
            change_text!("  config   - /etc/item                              directory 0750 root:root\n"),
            config_diff_json!(
                r#"{"kind":"removed","node":{"group":"root","kind":"directory","mode":"0750","owner":"root"},"path":"/etc/item"}"#
            ),
        ),
        (
            entry(EntryChange::Changed {
                path: copy_of(b"/etc/item"),
                before: link(b"old"),
                after: link(b"new"),
                sizes: None,
                hunks: None,
            }),
            change_text!("  config   ~ /etc/item                              link root:root new\n"),
            config_diff_json!(
                r#"{"after":{"group":"root","kind":"link","owner":"root","target":"new"},"before":{"group":"root","kind":"link","owner":"root","target":"old"},"kind":"changed","path":"/etc/item"}"#
            ),
        ),
        (
            entry(EntryChange::Changed {
                path: copy_of(b"/etc/item"),
                before: directory(0o750),
                after: directory(0o700),
                sizes: None,
                hunks: None,
            }),
            change_text!("  config   ~ /etc/item                              directory 0700 root:root\n"),
            config_diff_json!(
                r#"{"after":{"group":"root","kind":"directory","mode":"0700","owner":"root"},"before":{"group":"root","kind":"directory","mode":"0750","owner":"root"},"kind":"changed","path":"/etc/item"}"#
            ),
        ),
        (
            file_change(None),
            change_text!("  config   ~ /etc/item                              0644 root:root\n"),
            config_diff_json!(
                r#"{"after":{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","group":"root","kind":"file","mode":"0644","owner":"root"},"before":{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","group":"root","kind":"file","mode":"0644","owner":"root"},"kind":"changed","path":"/etc/item"}"#
            ),
        ),
        (
            file_change(Some(Hunks { binary: true, left_out: 5, lines: Box::new([]) })),
            change_text!("  config   ~ /etc/item                              0644 root:root\n"),
            config_diff_json!(
                r#"{"after":{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","group":"root","kind":"file","mode":"0644","owner":"root"},"before":{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","group":"root","kind":"file","mode":"0644","owner":"root"},"hunks":{"binary":true,"left_out":5,"lines":[]},"kind":"changed","path":"/etc/item"}"#
            ),
        ),
        (
            file_change(Some(Hunks { binary: false, left_out: 5, lines: Box::new([]) })),
            change_text!(
                "  config   ~ /etc/item                              0644 root:root\n               ... 5 lines left out\n"
            ),
            config_diff_json!(
                r#"{"after":{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","group":"root","kind":"file","mode":"0644","owner":"root"},"before":{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","group":"root","kind":"file","mode":"0644","owner":"root"},"hunks":{"binary":false,"left_out":5,"lines":[]},"kind":"changed","path":"/etc/item"}"#
            ),
        ),
    ]);
}
fn file_change(hunks: Option<Hunks>) -> Record {
    entry(EntryChange::Changed {
        path: copy_of(b"/etc/item"),
        before: fixtures::file([1; 32]),
        after: fixtures::file([2; 32]),
        sizes: None,
        hunks,
    })
}

#[test]
fn keyed_accounts_and_directives() {
    let mut added = config_change();
    added.groups = Box::new([GroupKeyed::Added { key: copy_of(b"ops"), value: GroupKind::Sysusers { gid: None } }]);
    added.users = Box::new([UserKeyed::Added { key: copy_of(b"alice"), value: user() }]);
    added.presets = Box::new([PresetKeyed::Added { key: copy_of(b"mail.service"), value: Action::Enable }]);
    added.tmpfiles =
        Box::new([LineKeyed::Added { key: copy_of(b"/tmp/item"), value: line(crate::parts::config::LineKind::FPlus) }]);
    let mut removed = config_change();
    removed.groups = Box::new([GroupKeyed::Changed {
        key: copy_of(b"ops"),
        before: GroupKind::Sysusers { gid: Some(1500) },
        after: GroupKind::Package,
    }]);
    removed.users = Box::new([UserKeyed::Removed { key: copy_of(b"alice"), value: user() }]);
    removed.presets = Box::new([PresetKeyed::Removed { key: copy_of(b"mail.service"), value: Action::Enable }]);
    removed.tmpfiles = Box::new([LineKeyed::Removed {
        key: copy_of(b"/tmp/item"),
        value: line(crate::parts::config::LineKind::FPlus),
    }]);
    check(&[
        (
            change(PartChange::Config(added)),
            change_text!(
                "  config   + groups ops                             -\n",
                "  config   + users alice                            - ops /home/alice /bin/sh\n",
                "  config   + presets mail.service                   enable\n",
                "  config   + tmpfiles /tmp/item                     f+ - - - 7d target\n"
            ),
            change_json!(
                r#"{"kind":"config","groups":[{"key":"ops","kind":"added","value":{"kind":"sysusers"}}],"users":[{"key":"alice","kind":"added","value":{"gecos":"","group":"ops","home":"/home/alice","kind":"sysusers","shell":"/bin/sh"}}],"members":[],"presets":[{"key":"mail.service","kind":"added","value":{"kind":"enable"}}],"tmpfiles":[{"key":"/tmp/item","kind":"added","value":{"age":"7d","argument":"target","type":"f+"}}],"tree":[]}"#
            ),
        ),
        (
            change(PartChange::Config(removed)),
            change_text!(
                "  config   ~ groups ops                             package\n",
                "  config   - users alice                            - ops /home/alice /bin/sh\n",
                "  config   - presets mail.service                   enable\n",
                "  config   - tmpfiles /tmp/item                     f+ - - - 7d target\n"
            ),
            change_json!(
                r#"{"kind":"config","groups":[{"after":{"kind":"package"},"before":{"gid":1500,"kind":"sysusers"},"key":"ops","kind":"changed"}],"users":[{"key":"alice","kind":"removed","value":{"gecos":"","group":"ops","home":"/home/alice","kind":"sysusers","shell":"/bin/sh"}}],"members":[],"presets":[{"key":"mail.service","kind":"removed","value":{"kind":"enable"}}],"tmpfiles":[{"key":"/tmp/item","kind":"removed","value":{"age":"7d","argument":"target","type":"f+"}}],"tree":[]}"#
            ),
        ),
    ]);
}
