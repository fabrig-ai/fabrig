use std::{path::Path, process::Command};
const SETTINGS: [&str; 13] = [
    "core.hooksPath=/dev/null",
    "core.fsmonitor=false",
    "protocol.version=2",
    "protocol.allow=never",
    "protocol.https.allow=always",
    "protocol.http.allow=always",
    "credential.helper=",
    "http.sslVerify=true",
    "gc.auto=0",
    "maintenance.auto=false",
    "rerere.enabled=false",
    "merge.conflictStyle=merge",
    "submodule.recurse=false",
];
pub(crate) fn command(git: &Path, home: &Path, dir: &Path, identity: Option<(&str, &str)>, commit: bool) -> Command {
    let mut cmd = Command::new(git);
    cmd.current_dir(dir).env_clear().env("HOME", home).env("LC_ALL", "C").env("TZ", "UTC");
    for (key, value) in [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("GIT_ASKPASS", ""),
    ] {
        cmd.env(key, value);
    }
    let name = if identity.is_some() || commit { "writer" } else { "temper" };
    cmd.env("GIT_COMMITTER_NAME", name).env("GIT_COMMITTER_EMAIL", format!("{name}@temper.invalid"));
    if commit {
        cmd.env("GIT_AUTHOR_NAME", "writer").env("GIT_AUTHOR_EMAIL", "writer@temper.invalid");
    }
    if let Some((login, token)) = identity {
        cmd.env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "http.extraHeader")
            .env("GIT_CONFIG_VALUE_0", format!("Authorization: Basic {}", base64(&format!("{login}:{token}"))));
    }
    for setting in SETTINGS {
        cmd.args(["-c", setting]);
    }
    cmd
}
pub(crate) fn base64(text: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in text.as_bytes().chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        for n in [a >> 2, ((a & 3) << 4) | (b >> 4), ((b & 15) << 2) | (c >> 6), c & 63] {
            out.push(char::from(TABLE[usize::from(n)]));
        }
        if chunk.len() < 3 {
            out.pop();
            out.push('=');
        }
        if chunk.len() < 2 {
            out.pop();
            out.pop();
            out.push('=');
            out.push('=');
        }
    }
    out
}
