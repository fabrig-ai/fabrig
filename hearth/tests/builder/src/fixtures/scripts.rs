use super::File;
pub const PRESETS: &[File] = &[
    File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"presets":{"ssh.service":{"action":"enable"},"getty@tty1.service":{"action":"disable"}},"tmpfiles":{"/var/log/journal":{"type":"d","mode":"2755","owner":"root","group":"systemd-journal"}},"scripts":{"customize.sh":{"order":300}}}"# },
    CUSTOMIZE_SCRIPT,
];
pub const PRESETS_MOVED: &[File] = &[
    File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"presets":{"ssh.service":{"action":"disable"},"getty@tty1.service":{"action":"disable"}},"tmpfiles":{"/var/log/journal":{"type":"d","mode":"0755","owner":"root","group":"systemd-journal"}},"scripts":{"customize.sh":{"order":300}}}"# },
    File { path: "tasks/motd/customize.sh", mode: 0o644, body: b"update-ca-certificates\nsystemctl daemon-reload\n" },
];
pub const CUSTOMIZE_SCRIPT: File =
    File { path: "tasks/motd/customize.sh", mode: 0o644, body: b"update-ca-certificates\n" };
