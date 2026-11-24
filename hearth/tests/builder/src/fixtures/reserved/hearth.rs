use super::super::File;
pub const HEARTH: &[File] = &[
    File { path: "tasks/hearth/task.json", body: include_bytes!("hearth.json"), mode: 0o644 },
    File {
        path: "tasks/hearth/tree/etc/systemd/system/hearth-activator.service",
        body: b"[Service]\nExecStart=/usr/local/bin/hearth\n",
        mode: 0o644,
    },
    File { path: "tasks/hearth/tree/usr/local/lib/hearth/askpass", body: b"#!/bin/sh\n", mode: 0o755 },
    File {
        path: "tasks/hearth/tree/etc/hearth/alarms.json.j2",
        body: b"{{ settings.alarms | tojson }}\n",
        mode: 0o644,
    },
];
pub const CONTAINER: File = File {
    path: "hosts/chat/chat-prod.json",
    body: br#"{"placement": {"kind": "container", "on": "east-prod"}}"#,
    mode: 0o644,
};
pub const CONTAINER_HEARTH: File = File {
    path: "hosts/chat/chat-prod.json",
    body: br#"{"placement": {"kind": "container", "on": "east-prod"}, "hearth": {}}"#,
    mode: 0o644,
};
#[must_use]
pub fn no_hearth(container: File) -> Vec<File> {
    let mut files = Vec::new();
    for file in super::super::CONTAINERS {
        if file.path != super::EMPTY_HEARTH.path && file.path != container.path {
            files.push(*file);
        }
    }
    files.push(container);
    files
}
