//! The pinned releases and digests (skein/docs/design/forge/dialects.md, 1.3).

/// The pinned Forgejo release that conformance checks and owned mode runs (skein/docs/design/forge/dialects.md, 1.3).
pub const VERSION: &str = "17.0.1";
/// The pinned forgejo-runner release (skein/docs/design/forge/dialects.md, 1.3).
pub const RUNNER_VERSION: &str = "13.2.1";

/// SHA-256 of `forgejo-17.0.1-linux-amd64` (skein/docs/design/forge/dialects.md, 1.3).
pub const FORGEJO_SHA256_LINUX_AMD64: &str = "51a8cdf4a1252e79f4b3855d7a7891af6c6635105420768c460e2f0690385d6f";
/// SHA-256 of `forgejo-17.0.1-linux-arm64` (skein/docs/design/forge/dialects.md, 1.3).
pub const FORGEJO_SHA256_LINUX_ARM64: &str = "1e99403746406e8199efc903ddd1940d5728a60e50241f7aa6c9f2b907bd8ec9";
/// SHA-256 of `forgejo-runner-13.2.1-linux-amd64` (skein/docs/design/forge/dialects.md, 1.3).
pub const RUNNER_SHA256_LINUX_AMD64: &str = "bb369fe1ecb50fdc983c0b1d993f19b0920df6e5864511892b3cdbf16d858b34";
/// SHA-256 of `forgejo-runner-13.2.1-linux-arm64` (skein/docs/design/forge/dialects.md, 1.3).
pub const RUNNER_SHA256_LINUX_ARM64: &str = "eeddb7902f79abe488972f4189fbe3ea02762ee827d1b406b469420bf0d62dd9";

/// SHA-256 of the bottle's `forgejo/17.0.1/bin/forgejo` (skein/docs/design/forge/dialects.md, 1.3).
pub const FORGEJO_SHA256_MACOS_ARM64: &str = "4b6ed0e7e6ef034d298872125f69332e2aad49367435f66c58ee5eb2c5680cf6";
/// SHA-256 of the bottle's `forgejo-runner/13.2.1/bin/forgejo-runner` (skein/docs/design/forge/dialects.md, 1.3).
pub const RUNNER_SHA256_MACOS_ARM64: &str = "2829f0125068f0ad92f908cb08c4d3c2f458cc22eec15fd675c332823cbece4b";

/// A binary pinned to its release and digests (forge/dialects.md, 1.3).
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

/// The pinned binary's digest, or `None` where there is no such pin.
#[must_use]
pub fn digest(binary: Binary, platform: Platform, arch: &str) -> Option<&'static str> {
    match (binary, platform, arch) {
        (Binary::Forgejo, Platform::Linux, "amd64") => Some(FORGEJO_SHA256_LINUX_AMD64),
        (Binary::Forgejo, Platform::Linux, "arm64") => Some(FORGEJO_SHA256_LINUX_ARM64),
        (Binary::Runner, Platform::Linux, "amd64") => Some(RUNNER_SHA256_LINUX_AMD64),
        (Binary::Runner, Platform::Linux, "arm64") => Some(RUNNER_SHA256_LINUX_ARM64),
        (Binary::Forgejo, Platform::Macos, "arm64") => Some(FORGEJO_SHA256_MACOS_ARM64),
        (Binary::Runner, Platform::Macos, "arm64") => Some(RUNNER_SHA256_MACOS_ARM64),
        (Binary::Forgejo | Binary::Runner, Platform::Linux | Platform::Macos, _) => None,
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
