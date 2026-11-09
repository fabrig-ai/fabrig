use super::File;
pub const MOTD_KIND: File = File { path: "tasks/motd/tree/etc/motd", body: b"Welcome, and be kind.\n", mode: 0o644 };

pub const MOTD_DIFF_TEXT: &str = "west-prod                                           rebuild from config\n  config   ~ /etc/motd                              0644 root:root\n               @@ -1 +1 @@\n               -Welcome.\n               +Welcome, and be kind.\nx1-prod                                             rebuild from config\n  config   ~ /etc/motd                              0644 root:root\n               @@ -1 +1 @@\n               -Welcome.\n               +Welcome, and be kind.\n\n2 of 4 targets change; 2 are unchanged.\n";

pub const EMPTY_HEARTH: File = File { path: "tasks/hearth/task.json", body: b"{}", mode: 0o644 };
pub const FLEET_JSON: File = File { path: "fleet.json", body: super::DEBIAN, mode: 0o644 };
pub const PROD_ENV: File = File { path: "envs/prod.json", body: b"{}", mode: 0o644 };
pub const LAB_ENV: File = File { path: "envs/lab.json", body: b"{}", mode: 0o644 };
pub const METAL_EFI: &[u8] = br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"}}"#;
#[must_use]
#[expect(clippy::disallowed_methods, reason = "hearth's round: reads PATH for test repository git commands")]
pub fn git_path() -> std::ffi::OsString {
    std::env::var_os("PATH").expect("git is required on PATH")
}
