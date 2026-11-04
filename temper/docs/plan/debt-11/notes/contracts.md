# debt-11: the contracts

The run's binding calls: what crosses the tasks. `tools/lints/design.md` is the design; this holds the
literal text it leaves out, and the map of this plan's base.

## 1. The definitions

`lint-check` writes these files byte for byte. No other task changes them; a crate that cannot take one
is a scoped `#[expect]` (`notes/common.md`), or a question.

`tools/lints/profiles/step.toml`:

```toml
[lints]
workspace = true
```

`tools/lints/profiles/harness.toml`:

```toml
[lints.rust] # harness: tools/lints/design.md, 2.2
unsafe_code = "forbid"
elided_lifetimes_in_paths = "deny"
unused_must_use = "deny"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
wildcard_enum_match_arm = "allow"
match_wildcard_for_single_variants = "deny"
unwrap_used = "deny"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
disallowed_types = "deny"
disallowed_macros = "deny"
disallowed_methods = "deny"
```

`tools/lints/profiles/binary.toml`: `harness`'s text, with its first line
`[lints.rust] # binary: tools/lints/design.md, 2.3` and `wildcard_enum_match_arm = "deny"`.

`tools/lints/profiles/ring.toml`:

```toml
[lints.rust] # ring: tools/lints/design.md, 2.4
unsafe_code = "deny"
unsafe_op_in_unsafe_fn = "deny"
elided_lifetimes_in_paths = "deny"
unused_must_use = "deny"
let_underscore_drop = "deny"
unused_qualifications = "warn"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "deny"
wildcard_enum_match_arm = "deny"
match_wildcard_for_single_variants = "deny"
indexing_slicing = "deny"
arithmetic_side_effects = "deny"
as_conversions = "deny"
unwrap_used = "deny"
let_underscore_must_use = "deny"
mem_forget = "deny"
todo = "deny"
unimplemented = "deny"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
disallowed_types = "deny"
disallowed_macros = "deny"
disallowed_methods = "deny"
dbg_macro = "deny"
missing_assert_message = "deny"
iter_over_hash_type = "deny"
```

`tools/lints/clippy/step.toml`: the header below, then `jig/clippy.toml` from its line
`# Unit tests run with std and may assert by panicking.` to its end, with one reason changed:
`core::default::Default`'s reads `only the derives of skein's programming-model.md, 10.2; a default can
be a live state`.

```toml
# clippy variant: step (tools/lints/design.md, 4).
# What the step crates may not use: skein/docs/foundation/programming-model.md, 10.
#
# Applies to every crate below this directory that has no clippy.toml of its
# own. A step crate that builds a container on a type that is out says so with a
# scoped #[expect(clippy::disallowed_types, reason = "...")].
#
# panic! is out through the clippy::panic lint rather than here, and
# format_args! is not listed: unreachable!("why") and assertion messages expand
# to both, and those stay in.
#
# Some of the subset cannot be checked by clippy and stays a review rule:
# `loop`, `while` and recursion; closures stored in locals; defining traits and
# generic items; trait impls by hand (Drop, Deref, operators); `if let` on own
# enums and tuple scrutinees; macro_rules!.

```

`tools/lints/clippy/ordinary.toml`:

```toml
# clippy variant: ordinary (tools/lints/design.md, 4).
# What ordinary Rust may not use (skein/docs/foundation/programming-model.md, 10.2):
# std, closures, traits and formatting are in, but a run is one thread and one
# loop, and a seed replays (skein/docs/foundation/testing-strategy.md, 1).

# Test code may unwrap (skein/docs/foundation/testing-strategy.md, 4); a harness
# in a crate's src/ says what it expects instead.
allow-unwrap-in-tests = true

disallowed-types = [
    { path = "std::collections::HashMap", reason = "iteration order depends on a random seed: BTreeMap" },
    { path = "std::collections::HashSet", reason = "iteration order depends on a random seed: BTreeSet" },
    { path = "std::hash::RandomState", reason = "iteration order would depend on a random seed: BTreeMap" },
    { path = "std::time::Instant", reason = "a run's time comes from its world; on the ring, skein_shell::Clock" },
    { path = "std::time::SystemTime", reason = "a run's time comes from its world; on the ring, skein_shell::Clock" },
]

disallowed-methods = [
    { path = "std::thread::spawn", reason = "one thread, one loop, so that a run replays" },
    { path = "std::thread::Builder::spawn", reason = "one thread, one loop, so that a run replays" },
    { path = "std::thread::scope", reason = "one thread, one loop, so that a run replays" },
    { path = "std::env::var", reason = "a run's settings come from its code, not the environment" },
]
```

`tools/lints/clippy/ring.toml`:

```toml
# clippy variant: ring (tools/lints/design.md, 4).
# What a crate on the ring, or one that plays the kernel, may not use: ordinary
# Rust (skein/docs/foundation/programming-model.md, 10.2) with no std handle that
# closes on drop (2.1) and no shared ownership. A descriptor is a plain Fd, closed
# by a Close record; a simulator or a fake machine plays the kernel and holds none.

doc-valid-idents = ["io_uring", ".."]

# Its own tests may assert by panicking.
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true

disallowed-types = [
    { path = "std::collections::HashMap", reason = "iteration order depends on a random seed: BTreeMap" },
    { path = "std::collections::HashSet", reason = "iteration order depends on a random seed: BTreeSet" },
    { path = "std::hash::RandomState", reason = "iteration order would depend on a random seed: BTreeMap" },
    { path = "std::time::Instant", reason = "a run's time comes from its world; on the ring, skein_shell::Clock" },
    { path = "std::time::SystemTime", reason = "a run's time comes from its world; on the ring, skein_shell::Clock" },
    { path = "std::sync::Arc", reason = "one thread, one owner" },
    { path = "std::sync::Mutex", reason = "one thread" },
    { path = "std::sync::RwLock", reason = "one thread" },
    { path = "std::os::fd::OwnedFd", reason = "a descriptor is closed by a Close record, not on drop", allow-invalid = true },
    { path = "std::fs::File", reason = "a descriptor is closed by a Close record, not on drop" },
    { path = "std::net::TcpStream", reason = "a descriptor is closed by a Close record, not on drop" },
    { path = "std::net::TcpListener", reason = "a descriptor is closed by a Close record, not on drop" },
    { path = "std::net::UdpSocket", reason = "a descriptor is closed by a Close record, not on drop" },
    { path = "std::process::Child", reason = "std::process is not used (io.md, 6)" },
    { path = "std::process::Command", reason = "std::process is not used (io.md, 6)" },
]

disallowed-methods = [
    { path = "std::thread::spawn", reason = "one thread, one loop, so that a run replays" },
    { path = "std::thread::Builder::spawn", reason = "one thread, one loop, so that a run replays" },
    { path = "std::thread::scope", reason = "one thread, one loop, so that a run replays" },
    { path = "std::env::var", reason = "a run's settings come from its code, not the environment" },
    { path = "std::collections::HashMap::new", reason = "a random seed: BTreeMap, or a fixed hasher" },
    { path = "std::collections::HashMap::with_capacity", reason = "a random seed: BTreeMap, or a fixed hasher" },
    { path = "std::collections::HashSet::new", reason = "a random seed: BTreeSet, or a fixed hasher" },
    { path = "std::collections::HashSet::with_capacity", reason = "a random seed: BTreeSet, or a fixed hasher" },
]
```

## 2. Which package takes which profile
Every package below writes its own block or none at this plan's base. Every other package of the four
trees has `[lints] workspace = true`, is on `step`, and stays there.
- **hearth, `harness`** (13): `hearth/testing/hearth-fake-git`, `hearth/testing/hearth-fake-mmdebstrap`, `hearth/testing/hearth-recordings`, `hearth/testing/hearth-test-mirror`, `hearth/tests/accounts`, `hearth/tests/builder`, `hearth/tests/config`, `hearth/tests/conformance`, `hearth/tests/git`, `hearth/tests/mmdebstrap`, `hearth/tests/render`, `hearth/tests/template`, `hearth/tests/tree`
- **hearth, `ring`** (1): `hearth/crates/hearth`
- **jig, `binary`** (2): `jig/crates/jig-client-shell-native`, `jig/crates/jig-client-shell-web`
- **jig, `harness`** (39): `jig/testing/jig-client-fake-person-browser`, `jig/testing/jig-client-scripted-engine`, `jig/testing/jig-client-test-kit`, `jig/testing/jig-client-test-shell-web`, `jig/testing/jig-conformance`, `jig/testing/jig-fake-parties`, `jig/testing/jig-fake-store`, `jig/testing/jig-fake-workers`, `jig/testing/jig-ops-fake-production`, `jig/testing/jig-test-profile`, `jig/testing/jig-test-store-files`, `jig/testing/jig-test-system`, `jig/testing/jig-test`, `jig/tests/accounts`, `jig/tests/boundary`, `jig/tests/brief`, `jig/tests/channel`, `jig/tests/client-browser`, `jig/tests/client-protocol`, `jig/tests/client-wire`, `jig/tests/client`, `jig/tests/conformance`, `jig/tests/core`, `jig/tests/engine-clients`, `jig/tests/engine-workers`, `jig/tests/fleet`, `jig/tests/host`, `jig/tests/notes`, `jig/tests/one-process`, `jig/tests/ops`, `jig/tests/people`, `jig/tests/protocol-accounts`, `jig/tests/store-codecs`, `jig/tests/store`, `jig/tests/tasks`, `jig/tests/test-connector`, `jig/tests/views`, `jig/tests/worker-link`, `jig/tests/worker-wire`
- **skein, `binary`** (3): `skein/crates/skein-ui-bundle`, `skein/crates/skein-ui-web`, `skein/examples/ui/shell-web`
- **skein, `harness`** (43): `skein/crates/skein-fake-checkout`, `skein/crates/skein-fake-peers`, `skein/testing/skein-fake-channel`, `skein/testing/skein-fake-person-browser`, `skein/testing/skein-fake-person`, `skein/testing/skein-forge-conformance`, `skein/testing/skein-throwaway-forgejo`, `skein/tests/browser`, `skein/tests/channel`, `skein/tests/conformance/real`, `skein/tests/conformance/sim`, `skein/tests/echo`, `skein/tests/fake-checkout`, `skein/tests/fake-forge`, `skein/tests/fake-peers`, `skein/tests/forge/client`, `skein/tests/forge/conformance`, `skein/tests/forge/github`, `skein/tests/forge/machine`, `skein/tests/forge/protocol`, `skein/tests/forge/setup`, `skein/tests/heap`, `skein/tests/http`, `skein/tests/io`, `skein/tests/json`, `skein/tests/kv`, `skein/tests/lib`, `skein/tests/link`, `skein/tests/llm-connection`, `skein/tests/llm`, `skein/tests/oauth-accounts`, `skein/tests/oauth-issuer`, `skein/tests/oauth`, `skein/tests/playground`, `skein/tests/protocol`, `skein/tests/readiness`, `skein/tests/ring`, `skein/tests/sim`, `skein/tests/tls`, `skein/tests/ui-text`, `skein/tests/ui-web`, `skein/tests/ui`, `skein/tests/world`
- **skein, `ring`** (3): `skein/crates/skein-shell`, `skein/examples/echo/shell`, `skein/examples/ui/server`
- **temper, `binary`** (7): `temper/crates/temper-client-bundle`, `temper/crates/temper-client-shell-cli`, `temper/crates/temper-client-shell-terminal`, `temper/crates/temper-client-shell-web`, `temper/crates/temper-forge-setup-shell`, `temper/crates/temper-playground`, `temper/crates/temper`
- **temper, `harness`** (24): `temper/crates/temper-fake-git-hosted`, `temper/crates/temper-fake-git`, `temper/testing/temper-fixtures`, `temper/testing/temper-github-app`, `temper/tests/client-browser`, `temper/tests/client`, `temper/tests/codecs`, `temper/tests/conformance`, `temper/tests/engine/domain`, `temper/tests/engine/forge`, `temper/tests/engine/protocol-forge`, `temper/tests/fake-checkout`, `temper/tests/one-process`, `temper/tests/playground`, `temper/tests/real-loop`, `temper/tests/system`, `temper/tests/worker/checkout`, `temper/tests/worker/domain`, `temper/tests/worker/protocol-git`, `temper/tools/gallery`, `temper/tools/git-transcripts`, `temper/tools/journeys`, `temper/tools/stage-one`, `temper/tools/twin`

## 3. Each level that changes
The manifests' tables at this plan's base against section 1's, for the packages of section 2. "(unset)"
is the lint's default: `allow` for a restriction lint and for `elided_lifetimes_in_paths`, `warn` for a
pedantic lint and a `disallowed_*` one. The gate denies warnings, so a change between `warn` and `deny`
moves nothing in it. Packages are named where few, counted where many.

| Lint | From | To | Packages |
|---|---|---|---|
| `clippy.allow_attributes` | (unset) | deny | 14 packages |
| `clippy.allow_attributes_without_reason` | (unset) | deny | 15 packages |
| `clippy.disallowed_macros` | (unset) | deny | 97 packages |
| `clippy.disallowed_methods` | (unset) | deny | 22 packages |
| `clippy.disallowed_types` | (unset) | deny | 22 packages |
| `clippy.match_wildcard_for_single_variants` | (unset) | deny | 18 packages |
| `clippy.missing_errors_doc` | (unset) | allow | 70 packages |
| `clippy.missing_panics_doc` | (unset) | allow | `jig-test-profile`, `skein-browser-world`, `skein-ui-web-tests` |
| `clippy.module_name_repetitions` | (unset) | allow | 130 packages |
| `clippy.multiple_unsafe_ops_per_block` | (unset) | deny | `skein-echo-shell`, `skein-ui-example-server` |
| `clippy.panic` | deny | (unset) | `skein-fake-peers` |
| `clippy.pedantic` | (unset) | warn | `jig-test-profile`, `skein-browser-world`, `skein-ui-web-tests` |
| `clippy.single_match_else` | allow | (unset) | `skein-fake-peers` |
| `clippy.undocumented_unsafe_blocks` | (unset) | deny | `skein-echo-shell`, `skein-ui-example-server` |
| `clippy.unwrap_used` | (unset) | deny | 9 packages |
| `clippy.wildcard_enum_match_arm` | (unset) | allow | 7 packages |
| `clippy.wildcard_enum_match_arm` | (unset) | deny | `skein-ui-bundle`, `skein-ui-example-shell-web`, `skein-ui-web` |
| `clippy.wildcard_enum_match_arm` | deny | allow | 114 packages |
| `rust.elided_lifetimes_in_paths` | (unset) | deny | 11 packages |
| `rust.missing_docs` | deny | (unset) | `skein-fake-checkout` |
| `rust.unsafe_code` | (unset) | forbid | `jig-test-profile` |
| `rust.unsafe_code` | deny | forbid | `skein-browser-world` |
| `rust.unsafe_code` | forbid | deny | `skein-echo-shell`, `skein-ui-example-server` |
| `rust.unsafe_op_in_unsafe_fn` | (unset) | deny | `skein-echo-shell`, `skein-ui-example-server` |
| `rust.unused_must_use` | (unset) | deny | `jig-test-profile`, `skein-browser-world`, `skein-ui-web-tests` |

A package's `#[expect]` of a lint that its profile now allows is unfulfilled and goes: above all
`clippy::wildcard_enum_match_arm` in `harness` crates, which reads 61 sites in crates with their own
block at this plan's base, some of them on `binary`, where they stay.

`skein-echo-shell` and `skein-ui-example-server` move from `forbid` to the profile's `deny` on
`unsafe_code`: each keeps its level with `#![forbid(unsafe_code)]` at its crate root.

## 4. Which `clippy.toml` is which

Each line is a directory that holds the file at this plan's base, but for `temper/tools/`, which is new.
The tree in a line is written once.

- **`step`:** `skein/`, `jig/`, `temper/`, `hearth/`.
- **`ordinary`, the worlds:** `skein/tests/`, `jig/tests/`, `temper/tests/`, `hearth/tests/`, `temper/tools/`.
- **`ordinary`, skein:** `crates/skein-fail-stop`, `crates/skein-fake-checkout`, `crates/skein-fake-peers`,
  `crates/skein-ui-bundle`, `crates/skein-ui-web`, `examples/ui/shell-web`, `testing/skein-codegen`,
  `testing/skein-fake-channel`, `testing/skein-fake-person`, `testing/skein-fake-person-browser`,
  `testing/skein-forge-conformance`, `testing/skein-throwaway-forgejo`, `testing/skein-world`.
- **`ordinary`, jig:** `crates/jig-client-shell-native`, `crates/jig-client-shell-web`, and every
  `testing/*/clippy.toml` (17).
- **`ordinary`, temper:** `crates/temper`, `crates/temper-client-bundle`, `crates/temper-client-shell-cli`,
  `crates/temper-client-shell-terminal`, `crates/temper-client-shell-web`, `crates/temper-fake-git`,
  `crates/temper-fake-git-hosted`, `crates/temper-forge-setup-shell`, `crates/temper-playground`,
  `testing/temper-fixtures`, `testing/temper-github-app`.
- **`ordinary`, hearth:** `testing/hearth-recordings`, `testing/hearth-test-mirror`.
- **`ring`, skein:** `crates/skein-shell`, `crates/skein-sim`, `crates/skein-sim-world`,
  `crates/skein-playground`, `crates/skein-fake-machine`, `testing/skein-conformance`,
  `testing/skein-heap`, `testing/skein-scratch`, `examples/echo/shell`, `examples/ui/server`.
- **`ring`, hearth:** `crates/hearth`, `testing/hearth-fake-git`, `testing/hearth-fake-machine`,
  `testing/hearth-fake-mmdebstrap`.
- **Deleted:** `skein/tests/forge/conformance`, `skein/tests/forge/github`, `jig/tests/client-protocol`,
  `temper/tools/gallery`, `temper/tools/git-transcripts`, `temper/tools/journeys`, `temper/tools/stage-one`,
  `temper/tools/twin`. Each inherits its parent's.

`smith/`'s and the root `tests/clippy.toml` stay as they are: the check does not read them.

What the variants move, against this plan's base:

- **Gain** `RandomState`, `thread::Builder::spawn`, `thread::scope` and `env::var`: every crate on `ordinary`
  that was on the replay bans alone, jig's and temper's test crates and shells above all. A sweep at the
  base finds `env::var` 14 times and `thread::scope` 4 times in crates with their own block, most in `temper`.
- **Gain the replay bans:** `temper-fixtures`, `temper-github-app` (empty lists at the base),
  `temper-client-bundle` (maps only), `skein-fail-stop` (no methods).
- **Lose** `Arc`, `Mutex` and `RwLock`: `skein-world`, `skein-codegen`, `skein-ui-bundle`, `skein-ui-web`.
  `skein-world` and `skein-codegen` run files and processes, so `ring` is not theirs.
- **Lose** the `Cell` ban: `jig/tests/client-protocol`. Its scoped exceptions for `Cell` go with it.
- **Gain the handle bans in full:** `skein-sim-world`, `skein-playground`, `skein-heap`, `skein-scratch`,
  `skein-conformance`, and the fakes of `ring`. A sweep at the base finds none of them using a handle.
