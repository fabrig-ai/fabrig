//! Claude Code's pinned native builds (protocol/setup/owned.md, 6).
use crate::{
    Origin,
    fetch::{Failure, Named, Pin},
    joined,
};
use alloc::boxed::Box;
use skein_forge::forgejo::pin::Platform;

pub const VERSION: &str = "2.1.311";
/// A platform's release digest and exact file size.
#[derive(Debug)]
pub struct Build {
    pub platform: &'static str,
    pub digest: [u8; 32],
    pub bytes: u64,
}
#[derive(Clone, Copy, Debug)]
pub enum Libc {
    Gnu,
    Musl,
}
const LINUX_X64: Build = Build {
    platform: "linux-x64",
    digest: [
        0x86, 0xdc, 0x1f, 0xcc, 0x69, 0x49, 0x81, 0x59, 0xea, 0xf1, 0x5f, 0x44, 0x66, 0x13, 0xe8, 0x47, 0x01, 0x0f,
        0x90, 0x36, 0xbf, 0xbb, 0xdf, 0xfd, 0x4c, 0x49, 0x35, 0x0b, 0xe8, 0x48, 0x9a, 0xda,
    ],
    bytes: 293_776_752,
};
const LINUX_ARM64: Build = Build {
    platform: "linux-arm64",
    digest: [
        0x2e, 0x24, 0x96, 0xc9, 0xa8, 0x06, 0x5d, 0xf9, 0x3a, 0xcb, 0x0f, 0xca, 0x2d, 0x34, 0x1f, 0x59, 0xa6, 0xb8,
        0x77, 0x76, 0x49, 0xb2, 0xe0, 0x25, 0xab, 0x8c, 0x9b, 0xcd, 0x5e, 0xef, 0x61, 0xf3,
    ],
    bytes: 293_658_808,
};
const LINUX_X64_MUSL: Build = Build {
    platform: "linux-x64-musl",
    digest: [
        0xcd, 0x40, 0xdd, 0x32, 0x71, 0x45, 0xee, 0x47, 0xaa, 0x0d, 0xea, 0xcd, 0x57, 0x18, 0x01, 0xc1, 0xd7, 0x82,
        0x38, 0x8f, 0x70, 0x9d, 0x71, 0x24, 0x2e, 0xee, 0x6b, 0x91, 0xde, 0x8e, 0xd1, 0x1f,
    ],
    bytes: 287_442_392,
};
const LINUX_ARM64_MUSL: Build = Build {
    platform: "linux-arm64-musl",
    digest: [
        0x17, 0x21, 0x50, 0xe3, 0xf9, 0xea, 0x07, 0xe8, 0xe2, 0xe2, 0xe2, 0x98, 0x62, 0xd7, 0xd9, 0xa2, 0x5f, 0x5c,
        0x23, 0x4d, 0x8e, 0x90, 0x9e, 0x40, 0x63, 0xdf, 0x23, 0x76, 0x65, 0xbf, 0x09, 0xb9,
    ],
    bytes: 285_822_152,
};
const DARWIN_ARM64: Build = Build {
    platform: "darwin-arm64",
    digest: [
        0x85, 0xd8, 0xc0, 0x87, 0x07, 0xfc, 0xfd, 0x1e, 0x78, 0xff, 0xa0, 0x20, 0x4e, 0xef, 0xbc, 0x6e, 0xc9, 0xe4,
        0xfe, 0x22, 0xf3, 0x75, 0xfb, 0x27, 0x67, 0xef, 0x85, 0xd9, 0x57, 0xed, 0x31, 0x61,
    ],
    bytes: 276_185_504,
};
const DARWIN_X64: Build = Build {
    platform: "darwin-x64",
    digest: [
        0xd3, 0x4e, 0x1c, 0xb8, 0x20, 0x15, 0xb0, 0x50, 0x47, 0x1a, 0x97, 0x57, 0x4a, 0xa6, 0xea, 0x6f, 0x69, 0x8e,
        0xcf, 0x3a, 0x97, 0xa6, 0x1c, 0x98, 0x09, 0x61, 0xf4, 0x3a, 0xd9, 0xf8, 0x38, 0xf5,
    ],
    bytes: 287_420_928,
};
/// Selects the build for the shell's platform, architecture and libc.
pub fn build(platform: Platform, machine: &str, libc: Libc) -> Result<&'static Build, Failure> {
    let build = match platform {
        Platform::Linux => match libc {
            Libc::Gnu => match machine {
                "x86_64" => &LINUX_X64,
                "aarch64" => &LINUX_ARM64,
                _ => return Err(Failure::Architecture(machine.as_bytes().into())),
            },
            Libc::Musl => match machine {
                "x86_64" => &LINUX_X64_MUSL,
                "aarch64" => &LINUX_ARM64_MUSL,
                _ => return Err(Failure::Architecture(machine.as_bytes().into())),
            },
        },
        Platform::Macos => match machine {
            "x86_64" => &DARWIN_X64,
            "aarch64" => &DARWIN_ARM64,
            _ => return Err(Failure::Architecture(machine.as_bytes().into())),
        },
    };
    Ok(build)
}
/// The pinned build's cache path.
#[must_use]
pub fn file(cache: &[u8]) -> Box<[u8]> {
    let cache = match cache.strip_suffix(b"/") {
        Some(cache) => cache,
        None => cache,
    };
    joined(&[cache, b"/claude-code/", VERSION.as_bytes(), b"/claude"])
}
/// The release's one binary request, at the caller's origin.
#[must_use]
pub fn pin(cache: &[u8], origin: Origin, build: &Build) -> Pin {
    Pin {
        binary: Named::ClaudeCode,
        origin,
        path: joined(&[b"/claude-code-releases/", VERSION.as_bytes(), b"/", build.platform.as_bytes(), b"/claude"]),
        digest: build.digest,
        file: file(cache),
        bottle: None,
    }
}

/// Claude Code is fetched when its program is first needed (protocol/setup/owned.md, 6).
pub const FETCHED: &[u8] = b"fetched";

#[cfg(test)]
mod tests;
