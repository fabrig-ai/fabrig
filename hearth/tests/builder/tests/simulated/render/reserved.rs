//! Reserved declarations and paths as diagnosed by the builder's protocol world.
use super::{Form, fixtures, render_files};
use fixtures::{File, reserved as f};

// hearth/docs/design/client/README.md, 3.3.
pub(super) const CONFIGURATION_EXIT: u8 = 2;

fn assert_refusal(files: &[File], message: &[u8]) {
    let run = render_files(&fixtures::thin_with(files), &[], Form::Text);
    assert_eq!(run.stdout, message, "{}", String::from_utf8_lossy(&run.stdout));
    assert_eq!(run.code, Some(CONFIGURATION_EXIT), "{}", String::from_utf8_lossy(message));
}

#[test]
fn a_reserved_command_line_key_prints_as_render_md_shows() {
    assert_refusal(core::slice::from_ref(&f::WAIT), b"tasks/motd/task.json:1:23: x1-prod: motd: \"rootwait\" is reserved: tiny-initramfs would wait for ever for a missing root (domain/machine/boot.md, 3.2)\n");
}

#[test]
fn a_password_preseed_prints_reserved_and_not_an_unlisted_type() {
    assert_refusal(core::slice::from_ref(&f::PASSWORD), b"tasks/motd/task.json:1:23: x1-prod: motd: \"password\" is reserved: a password preseed leaves its value in /var/cache/debconf/passwords.dat (protocol/format/tasks-reserved.md, 1)\n");
}

#[test]
fn a_line_past_2048_bytes_names_the_parameter_that_crosses_it() {
    let run = render_files(&fixtures::thin_with(&[f::long_line()]), &[], Form::Text);
    assert!(run.stdout.starts_with(b"tasks/motd/task.json:1:1040: x1-prod: motd: "));
    assert!(String::from_utf8_lossy(&run.stdout).contains("the command line is 2141 bytes"));
    assert_eq!(run.code, Some(CONFIGURATION_EXIT));
}

const PATH_REFUSALS: &[(&[File], &[u8])] = &[
    (f::KEY_IN_TREE, b"tasks/motd/task.json:1:12: x1-prod: motd: /etc/ssl/private/mx1.key is a secret's path; declare it under secrets (protocol/format/README.md, 7.1)\n"),
    (core::slice::from_ref(&f::FSTAB), b"tasks/motd/tree/etc/fstab: x1-prod: motd: \"/etc/fstab\" is reserved: the activator writes this path at prepare (protocol/format/tasks-reserved.md, 1)\n"),
    (core::slice::from_ref(&f::DATA_PATH), b"tasks/motd/task.json:1:20: x1-prod: motd: \"/srv/data/spool\" is reserved: the class mount hides this path (protocol/format/tasks.md, 7)\n"),
];

#[test]
fn reserved_paths_exit_2() {
    for (files, message) in PATH_REFUSALS {
        assert_refusal(files, message);
    }
}

const SECRET_LIVE_REFUSALS: &[(&File, &[u8])] = &[
    (&f::SECRET_HEARTH, b"tasks/motd/task.json:1:71: x1-prod: motd: \"mail/tls/mx1.key\" is reserved: hearth must not be able to read this secret or identity file (protocol/format/README.md, 7.1)\n"),
    (&f::LIVE_NEVER, b"tasks/motd/task.json:1:19: x1-prod: motd: \"systemd-networkd.service\" is reserved: this core service must not restart live (protocol/format/tasks-live.md, 1)\n"),
    (&f::LIVE_NONE, b"tasks/motd/task.json:1:18: x1-prod: motd: \"x.service\" is reserved: action none names no unit (protocol/format/tasks-live.md, 1)\n"),
];

#[test]
fn reserved_secrets_and_live_entries_exit_2() {
    for (file, message) in SECRET_LIVE_REFUSALS {
        assert_refusal(core::slice::from_ref(file), message);
    }
}
