# macos-cheap: the run's binding calls

The shapes that cross tasks. A task builds to these; a change to one is a question.

## The pin (`skein/crates/skein-forge/src/forgejo/pin.rs`; `skein/docs/design/forge.md`, 4.3)

```rust
pub const VERSION: &str = "17.0.1";
pub const RUNNER_VERSION: &str = "13.2.1";
// FORGEJO_SHA256_LINUX_AMD64, _LINUX_ARM64, RUNNER_SHA256_LINUX_AMD64, _LINUX_ARM64 stay, at the new digests.
pub const FORGEJO_BOTTLE_SHA256_MACOS_ARM64: &str = "...";
pub const FORGEJO_SHA256_MACOS_ARM64: &str = "...";
pub const RUNNER_BOTTLE_SHA256_MACOS_ARM64: &str = "...";
pub const RUNNER_SHA256_MACOS_ARM64: &str = "...";

/// The operating system a pin is for.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Platform { Linux, Macos }

/// What a pin's digest is of: the binary itself, or the archive that holds it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Form { Binary, Archive }

/// The pinned digest, or `None` where there is no such pin: `arch` is the release
/// architecture name, `amd64` or `arm64`; Linux has no `Archive`; macOS has `arm64` alone.
#[must_use]
pub fn digest(binary: Binary, platform: Platform, arch: &str, form: Form) -> Option<&'static str>;
```

## The workflow's seam (`.github/workflows/macos.yml`)

- The file has one job, `macos`. Every check is a step that runs through `$RUN NAME COMMAND...` (the former workflow's runner of each check), under `if: ${{ !cancelled() && steps.rust.outcome == 'success' }}`.
- The step that installs Rust and nextest has `id: rust`.
- A later task adds its steps between the step named `Compile every target` and the step named `The disk`, and touches nothing else of the file but what its spec says.

## The fetch (`temper/crates/temper-forge-setup/src/fetch.rs`; `temper/docs/design/protocol/setup.md`, 6.6)

`pins` takes the platform as data; `crates/temper/src/forge.rs` gives it from `cfg!(target_os = "macos")`:

```rust
pub fn pins(owned: &Owned, platform: Platform, machine: &str) -> Result<Box<[Pin]>, Failure>;

pub struct Pin {
    pub origin: Origin,
    pub path: Box<[u8]>,
    /// The binary's digest: what a file in place, and the end of a fetch, is held to.
    pub digest: [u8; 32],
    pub file: Box<[u8]>,
    /// macOS's: the bottle the binary comes out of.
    pub bottle: Option<Bottle>,
}
pub struct Bottle {
    pub digest: [u8; 32],
    pub member: Box<[u8]>,
    /// ghcr.io's first request alone; never a mirror's.
    pub token: bool,
}
pub enum Event {
    // ...as today, and:
    Answered { status: u16, length: Option<u64>, location: Option<Box<[u8]>> },
    Extracted(Extraction),
}
pub enum Extraction { Digest([u8; 32]), Bytes, Failed }
pub enum Request {
    // ...as today, and:
    Get { origin: Origin, path: Box<[u8]>, authorization: Option<Box<[u8]>> },
    Extract { archive: Box<[u8]>, member: Box<[u8]>, part: Box<[u8]>, bound: u64 },
}
pub enum Failure {
    // ...as today, and:
    Redirect { url: Box<[u8]> },
    Gone { url: Box<[u8]>, status: u16 },
    Extraction { url: Box<[u8]> },
}
```

A bottle's order of effects: `Hash`, `Get` (with the token where `token`), a `Get` of the `location` (none), `Open` of `<file>.archive`, `Read`/`Write`, `Extract`, `Finish`, `Remove` of `<file>.archive`. On a wrong archive digest or a bound passed in the body: `Remove` of `<file>.archive`. On anything wrong at `Extracted`: `Remove` of `<file>.part`, then of `<file>.archive`.
