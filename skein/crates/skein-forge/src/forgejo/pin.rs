//! The pinned releases and digests (skein/docs/design/forge/README.md, 4.3).

/// The pinned Forgejo release that conformance checks and owned mode runs (skein/docs/design/forge/README.md, 4.3).
pub const VERSION: &str = "17.0.1";
/// The pinned forgejo-runner release (skein/docs/design/forge/README.md, 4.3).
pub const RUNNER_VERSION: &str = "13.2.1";

/// SHA-256 of `forgejo-17.0.1-linux-amd64` (skein/docs/design/forge/README.md, 4.3).
pub const FORGEJO_SHA256_LINUX_AMD64: &str = "51a8cdf4a1252e79f4b3855d7a7891af6c6635105420768c460e2f0690385d6f";
/// SHA-256 of `forgejo-17.0.1-linux-arm64` (skein/docs/design/forge/README.md, 4.3).
pub const FORGEJO_SHA256_LINUX_ARM64: &str = "1e99403746406e8199efc903ddd1940d5728a60e50241f7aa6c9f2b907bd8ec9";
/// SHA-256 of `forgejo-runner-13.2.1-linux-amd64` (skein/docs/design/forge/README.md, 4.3).
pub const RUNNER_SHA256_LINUX_AMD64: &str = "bb369fe1ecb50fdc983c0b1d993f19b0920df6e5864511892b3cdbf16d858b34";
/// SHA-256 of `forgejo-runner-13.2.1-linux-arm64` (skein/docs/design/forge/README.md, 4.3).
pub const RUNNER_SHA256_LINUX_ARM64: &str = "eeddb7902f79abe488972f4189fbe3ea02762ee827d1b406b469420bf0d62dd9";

/// SHA-256 of the macOS arm64 bottle of Forgejo 17.0.1 (skein/docs/design/forge/README.md, 4.3).
pub const FORGEJO_BOTTLE_SHA256_MACOS_ARM64: &str = "19db7c548064dfd4097264a7b4c4cf3b228bfd6190bc33a3c6e8a9b26c3d0e06";
/// SHA-256 of the bottle's `forgejo/17.0.1/bin/forgejo` (skein/docs/design/forge/README.md, 4.3).
pub const FORGEJO_SHA256_MACOS_ARM64: &str = "4b6ed0e7e6ef034d298872125f69332e2aad49367435f66c58ee5eb2c5680cf6";
/// SHA-256 of the macOS arm64 bottle of forgejo-runner 13.2.1 (skein/docs/design/forge/README.md, 4.3).
pub const RUNNER_BOTTLE_SHA256_MACOS_ARM64: &str = "0a8cb32f024121e50ebca1840ed2f5650d65d3eb3e183679c0f8f0bc14d6fb48";
/// SHA-256 of the bottle's `forgejo-runner/13.2.1/bin/forgejo-runner` (skein/docs/design/forge/README.md, 4.3).
pub const RUNNER_SHA256_MACOS_ARM64: &str = "2829f0125068f0ad92f908cb08c4d3c2f458cc22eec15fd675c332823cbece4b";

/// A binary pinned to its release and digests (forge/README.md, 4.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Binary {
    Forgejo,
    Runner,
}

/// The operating system a pin is for.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Platform {
    Linux,
    Macos,
}

/// What a pin's digest is of: the binary itself, or the archive that holds it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    Binary,
    Archive,
}

/// The pinned digest, or `None` where there is no such pin: `arch` is the release
/// architecture name, `amd64` or `arm64`; Linux has no `Archive`; macOS has `arm64` alone.
#[must_use]
pub fn digest(binary: Binary, platform: Platform, arch: &str, form: Form) -> Option<&'static str> {
    match (binary, platform, form) {
        (Binary::Forgejo, Platform::Linux, Form::Binary) => {
            if arch == "amd64" {
                Some(FORGEJO_SHA256_LINUX_AMD64)
            } else if arch == "arm64" {
                Some(FORGEJO_SHA256_LINUX_ARM64)
            } else {
                None
            }
        }
        (Binary::Runner, Platform::Linux, Form::Binary) => {
            if arch == "amd64" {
                Some(RUNNER_SHA256_LINUX_AMD64)
            } else if arch == "arm64" {
                Some(RUNNER_SHA256_LINUX_ARM64)
            } else {
                None
            }
        }
        (Binary::Forgejo | Binary::Runner, Platform::Linux, Form::Archive) => None,
        (Binary::Forgejo, Platform::Macos, Form::Binary) => {
            if arch == "arm64" {
                Some(FORGEJO_SHA256_MACOS_ARM64)
            } else {
                None
            }
        }
        (Binary::Forgejo, Platform::Macos, Form::Archive) => {
            if arch == "arm64" {
                Some(FORGEJO_BOTTLE_SHA256_MACOS_ARM64)
            } else {
                None
            }
        }
        (Binary::Runner, Platform::Macos, Form::Binary) => {
            if arch == "arm64" {
                Some(RUNNER_SHA256_MACOS_ARM64)
            } else {
                None
            }
        }
        (Binary::Runner, Platform::Macos, Form::Archive) => {
            if arch == "arm64" {
                Some(RUNNER_BOTTLE_SHA256_MACOS_ARM64)
            } else {
                None
            }
        }
    }
}

/// Whether Forgejo's first `--version` line names `VERSION`, ended by nothing, `+` or a space.
#[must_use]
pub fn names_forgejo(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("forgejo version ") else {
        return false;
    };
    rest.split(['+', ' ']).next() == Some(VERSION)
}
/// Whether forgejo-runner's first `--version` line is `forgejo-runner version v` and `RUNNER_VERSION`, nothing more.
#[must_use]
pub fn names_runner(line: &str) -> bool {
    line.strip_prefix("forgejo-runner version v") == Some(RUNNER_VERSION)
}
