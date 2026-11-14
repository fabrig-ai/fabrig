use crate::parts::config::{
    self, Action, Group, GroupKeyed, GroupKind, Line, LineKeyed, LineKind, Preset, PresetKeyed, Tmpfile, User,
    UserKeyed, UserKind,
};
use skein_lib::bytes::copy_of;

fn empty() -> config::Config {
    config::Config {
        presets: Box::new([]),
        tmpfiles: Box::new([]),
        scripts: Box::new([]),
        groups: Box::new([]),
        users: Box::new([]),
        members: Box::new([]),
        tree: Box::new([]),
    }
}
fn unchanged() -> config::Change {
    config::Change {
        presets: Box::new([]),
        tmpfiles: Box::new([]),
        scripts: None,
        groups: Box::new([]),
        users: Box::new([]),
        members: Box::new([]),
        tree: Box::new([]),
    }
}

#[test]
fn groups_are_removed_added_changed_in_key_order() {
    let before = config::Config {
        groups: Box::new([
            Group { name: copy_of(b"a"), kind: GroupKind::Package },
            Group { name: copy_of(b"c"), kind: GroupKind::Sysusers { gid: None } },
        ]),
        ..empty()
    };
    let after = config::Config {
        groups: Box::new([
            Group { name: copy_of(b"b"), kind: GroupKind::Sysusers { gid: Some(12) } },
            Group { name: copy_of(b"c"), kind: GroupKind::Package },
        ]),
        ..empty()
    };
    assert_eq!(
        config::change(Some(&before), Some(&after)),
        config::Change {
            groups: Box::new([
                GroupKeyed::Removed { key: copy_of(b"a"), value: GroupKind::Package },
                GroupKeyed::Added { key: copy_of(b"b"), value: GroupKind::Sysusers { gid: Some(12) } },
                GroupKeyed::Changed {
                    key: copy_of(b"c"),
                    before: GroupKind::Sysusers { gid: None },
                    after: GroupKind::Package
                },
            ]),
            ..unchanged()
        }
    );
}

fn sysuser() -> UserKind {
    UserKind::Sysusers {
        uid: Some(12),
        group: copy_of(b"staff"),
        gecos: copy_of(b"Staff"),
        home: copy_of(b"/home/staff"),
        shell: copy_of(b"/bin/sh"),
    }
}
#[test]
fn users_are_removed_added_changed_in_key_order() {
    let before = config::Config {
        users: Box::new([
            User { name: copy_of(b"a"), kind: UserKind::Package },
            User { name: copy_of(b"c"), kind: sysuser() },
        ]),
        ..empty()
    };
    let after = config::Config {
        users: Box::new([
            User { name: copy_of(b"b"), kind: sysuser() },
            User { name: copy_of(b"c"), kind: UserKind::Package },
        ]),
        ..empty()
    };
    assert_eq!(
        config::change(Some(&before), Some(&after)),
        config::Change {
            users: Box::new([
                UserKeyed::Removed { key: copy_of(b"a"), value: UserKind::Package },
                UserKeyed::Added { key: copy_of(b"b"), value: sysuser() },
                UserKeyed::Changed { key: copy_of(b"c"), before: sysuser(), after: UserKind::Package },
            ]),
            ..unchanged()
        }
    );
}

#[test]
fn presets_are_removed_added_changed_in_key_order() {
    let before = config::Config {
        presets: Box::new([
            Preset { unit: copy_of(b"a"), action: Action::Enable },
            Preset { unit: copy_of(b"c"), action: Action::Disable },
        ]),
        ..empty()
    };
    let after = config::Config {
        presets: Box::new([
            Preset { unit: copy_of(b"b"), action: Action::Disable },
            Preset { unit: copy_of(b"c"), action: Action::Enable },
        ]),
        ..empty()
    };
    assert_eq!(
        config::change(Some(&before), Some(&after)),
        config::Change {
            presets: Box::new([
                PresetKeyed::Removed { key: copy_of(b"a"), value: Action::Enable },
                PresetKeyed::Added { key: copy_of(b"b"), value: Action::Disable },
                PresetKeyed::Changed { key: copy_of(b"c"), before: Action::Disable, after: Action::Enable },
            ]),
            ..unchanged()
        }
    );
}

fn line(kind: LineKind) -> Line {
    Line { kind, mode: Some(0o755), owner: Some(copy_of(b"root")), group: None, age: None, argument: None }
}
#[test]
fn tmpfiles_are_removed_added_changed_in_key_order() {
    let before = config::Config {
        tmpfiles: Box::new([
            Tmpfile { path: copy_of(b"/a"), line: line(LineKind::D) },
            Tmpfile { path: copy_of(b"/c"), line: line(LineKind::F) },
        ]),
        ..empty()
    };
    let after = config::Config {
        tmpfiles: Box::new([
            Tmpfile { path: copy_of(b"/b"), line: line(LineKind::F) },
            Tmpfile { path: copy_of(b"/c"), line: line(LineKind::D) },
        ]),
        ..empty()
    };
    assert_eq!(
        config::change(Some(&before), Some(&after)),
        config::Change {
            tmpfiles: Box::new([
                LineKeyed::Removed { key: copy_of(b"/a"), value: line(LineKind::D) },
                LineKeyed::Added { key: copy_of(b"/b"), value: line(LineKind::F) },
                LineKeyed::Changed { key: copy_of(b"/c"), before: line(LineKind::F), after: line(LineKind::D) },
            ]),
            ..unchanged()
        }
    );
}
