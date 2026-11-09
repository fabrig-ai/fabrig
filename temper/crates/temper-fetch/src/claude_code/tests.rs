use crate::{
    Origin,
    claude_code::{self, Libc},
    fetch::{self, Failure, Fetched, Named},
};
use skein_forge::forgejo::pin::Platform;

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust compares release pins to the design's table (programming-model.md, 10.2)"
)]
fn every_build_is_pinned_to_the_design_table() {
    let design = include_str!("../../../../docs/design/protocol/setup.md");
    assert!(design.contains(&format!("**The pin:** `VERSION` `{}`", claude_code::VERSION)));
    for (platform, arch, libc, name) in [
        (Platform::Linux, "x86_64", Libc::Gnu, "linux-x64"),
        (Platform::Linux, "aarch64", Libc::Gnu, "linux-arm64"),
        (Platform::Linux, "x86_64", Libc::Musl, "linux-x64-musl"),
        (Platform::Linux, "aarch64", Libc::Musl, "linux-arm64-musl"),
        (Platform::Macos, "aarch64", Libc::Gnu, "darwin-arm64"),
        (Platform::Macos, "x86_64", Libc::Gnu, "darwin-x64"),
        (Platform::Macos, "aarch64", Libc::Musl, "darwin-arm64"),
        (Platform::Macos, "x86_64", Libc::Musl, "darwin-x64"),
    ] {
        let build = claude_code::build(platform, arch, libc).unwrap();
        assert_eq!(build.platform, name);
        let row = design.lines().find(|line| line.contains(&format!("| `{name}` |"))).unwrap();
        let columns: Vec<_> = row.split('|').map(str::trim).collect();
        assert_eq!(String::from_utf8(fetch::hex(&build.digest).into()).unwrap(), columns[2].trim_matches('`'));
        assert_eq!(build.bytes, columns[3].replace(',', "").parse::<u64>().unwrap());
        let pin =
            claude_code::pin(b"/cache", Origin { host: b"downloads.claude.ai".as_slice().into(), port: 443 }, build);
        assert_eq!(
            fetch::url(&pin.origin, &pin.path).as_ref(),
            format!("https://downloads.claude.ai/claude-code-releases/2.1.311/{name}/claude").as_bytes()
        );
        assert_eq!(pin.file, claude_code::file(b"/cache/"));
        assert_eq!(pin.digest, build.digest);
        assert!(pin.bottle.is_none());
        assert_eq!(pin.binary, Named::ClaudeCode);
    }
    assert_eq!(claude_code::file(b"/cache").as_ref(), b"/cache/claude-code/2.1.311/claude");
    for platform in [Platform::Linux, Platform::Macos] {
        for libc in [Libc::Gnu, Libc::Musl] {
            assert_eq!(
                claude_code::build(platform, "riscv64", libc).unwrap_err(),
                Failure::Architecture(b"riscv64".as_slice().into())
            );
        }
    }
}

#[test]
fn claude_progress_is_not_a_box_line() {
    let fetched = Fetched { binary: Named::ClaudeCode, done: 3, total: Some(3) };
    assert_eq!(fetched.line().as_ref(), b"fetched claude-code 3 of 3 bytes\n");
    assert_eq!(Fetched::read(&fetched.line()), None);
}
