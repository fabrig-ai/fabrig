//! The reserved hearth task as seen by the builder's protocol world.
use super::{Form, fixtures, render_files, reserved::CONFIGURATION_EXIT};
use fixtures::reserved::hearth as f;

#[test]
fn the_hearth_task_passes_every_reserved_rule() {
    let run = render_files(&fixtures::thin_with(f::HEARTH), &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(
            &run.stdout,
            &["  request ", "  users ", "  members ", "  presets ", "  download ", "  fetch "]
        ),
        "  request    git mmdebstrap openssh-client uidmap\n  download   /usr/local/bin/hearth file 0755 root:root ca2dff161724\n  fetch      ca2dff161724 https://git.example.org/ai/hearth/releases/download/v0.1.0/hearth-x86_64-linux\n  users      hearth 59999 hearth /nonexistent /usr/sbin/nologin\n  members    hearth systemd-journal\n  presets    hearth-activator.service enable\n  presets    systemd-confext.service disable\n  presets    systemd-sysext.service disable\n"
    );
    assert_eq!(run.code, Some(0));
}

#[test]
fn an_empty_hearth_task_requests_no_packages() {
    let run = render_files(fixtures::TEST_REPOSITORY, &[b"test-min"], Form::Text);
    assert_eq!(fixtures::lines_starting(&run.stdout, &["  request"]), "");
    assert_eq!(run.code, Some(0));
}

#[test]
fn a_container_renders_without_the_hearth_declaration() {
    let run = render_files(&f::no_hearth(f::CONTAINER), &[b"chat-prod"], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["target ", "tasks/", "hosts/"]),
        "target       chat-prod   base chat   environment prod\n"
    );
    assert_eq!(run.code, Some(0));
}

#[test]
fn a_host_names_the_missing_hearth_declaration() {
    let run = render_files(&f::no_hearth(f::CONTAINER), &[b"east-prod"], Form::Text);
    assert_eq!(run.stdout, b"tasks/hearth/task.json: east-prod: no task hearth under tasks/\n");
    assert_eq!(run.code, Some(CONFIGURATION_EXIT));
}

#[test]
fn a_container_names_hearth_at_its_key() {
    let run = render_files(&f::no_hearth(f::CONTAINER_HEARTH), &[b"chat-prod"], Form::Text);
    assert_eq!(run.stdout, b"hosts/chat/chat-prod.json:1:57: chat-prod: hearth: \"hearth\" is reserved: a container has no hearth task (protocol/format/tasks-hearth.md, 1)\n");
    assert_eq!(run.code, Some(CONFIGURATION_EXIT));
}
