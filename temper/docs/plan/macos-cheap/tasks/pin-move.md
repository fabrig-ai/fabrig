+++
id = "pin-move"
part = "pins"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/pin.rs", "skein/crates/skein-forge/src/forgejo/tests/pin.rs", "temper/crates/temper-forge-setup/src/fetch.rs", "temper/crates/temper-forge-setup/src/limits.rs", "temper/crates/temper-forge-setup/src/tests/fetch.rs", "temper/crates/temper-forge-setup-shell/tests/suite/prepare.rs", "temper/crates/temper/tests/suite/forge.rs", "temper/crates/temper/tests/support/fetch.rs", "skein/testing/skein-throwaway-forgejo/**", "skein/tests/forge/conformance/README.md", "temper/docs/help/server.md"]
effort = "medium"
minutes = 40
+++

# The pin moves to Forgejo 17.0.1 and runner 13.2.1, and gains macOS's digests

## Goal

`skein_forge::forgejo::pin` pins the versions Homebrew bottles, with every digest of `skein/docs/design/forge.md`, 4.3, and a `digest` selected by platform, architecture and form. Every reader follows. This task moves constants and literals; it fetches and records nothing (`conformance` does).

Acceptance:

- `pin.rs` holds the constants and digests of forge.md, 4.3, byte for byte, and the surface of `notes/contracts.md`, "The pin";
- `digest` answers every pinned combination and `None` for every other, by a test;
- `temper forge fetch` on Linux fetches what it fetched, at the new version: same paths, `Platform::Linux` and `Form::Binary`;
- the focused tests of the packages below pass.

## Inputs

- `skein/docs/design/forge.md`, 4.3.
- `temper/docs/plan/macos-cheap/notes/common.md`; `temper/docs/plan/macos-cheap/notes/contracts.md`, "The pin".

## Owns and may change

Owns `pin.rs` and its tests.

Readers, each changed only in the line that names the pin:

- `temper/crates/temper-forge-setup/src/fetch.rs`: the call to `pin::digest`, as `pin::digest(binary, Platform::Linux, arch, Form::Binary)`, its error path as now. Nothing else of the fetch: `fetch` owns it next.
- `temper/crates/temper-forge-setup/src/limits.rs`: the comment over `FETCH_BYTES`, its largest pin Forgejo 17.0.1 amd64, 122,057,320 bytes. The bound does not move.
- The literals that name the pin: `temper/crates/temper-forge-setup/src/tests/fetch.rs` (58, 65, 249, 259), `temper/crates/temper-forge-setup-shell/tests/suite/prepare.rs` (18, 38), `temper/crates/temper/tests/suite/forge.rs` (237, 253, 402, 449), `temper/crates/temper/tests/support/fetch.rs` (91), `skein/tests/forge/conformance/README.md` (78, 91), `skein/testing/skein-throwaway-forgejo/tests/support/attached.rs` (18). Line numbers are the trunk's at this spec.
- `temper/docs/help/server.md` (168), the runner's version: a departure from the implementer's text, for this one line. It is read by tests: `temper-docs`' tests run.

Pre-approved:

- A literal `17.0.0` or `13.2.0` elsewhere that is sample data and stays green (`temper-client-*`, `temper-service`'s `family.rs`, `temper-engine-domain-forge`, `tests/client`, `tools/gallery`, `upgrade.rs`'s backup names) stays. One that goes red because it meets a pin-derived value follows the pin, and the report names it.
- `temper/crates/temper-init/src/person.rs`' comment cites Forgejo's source at a release and stays. So does the fixture `red-changes.log`.
- `skein/tests/forge/conformance/facts.txt`, `transcripts/` and `temper/tools/git-transcripts/facts.txt` are `conformance`'s: do not touch them, even though they name the old versions.

## Steps

1. In `pin.rs`, per `notes/contracts.md`: the versions; the four Linux digests' new values; the four macOS constants; `Platform`, `Form`; `digest`, an exhaustive `match` on `(binary, platform, form)` with `arch` checked in each arm, in the crate's strict subset. Doc comments name what each constant is of and cite `forge.md`, 4.3.
2. In `tests/pin.rs`: the table test over every combination, `None` for `(Linux, Archive)`, for `(Macos, "amd64")`, and for the existing odd names; `pinned_digests_are_64_lowercase_hex_digits` over all eight.
3. The readers above.
4. This changes what a shared item means (the version every reader checks): run the tests of its readers, not only the crate's: `heavy cargo nextest run -p skein-forge -p skein-throwaway-forgejo -p forgejo-conformance -p temper-forge-setup -p temper-forge-setup-shell -p temper-service -p temper-engine-domain-forge -p temper-client-domain -p temper-client-view -p temper-docs -p temper-client-world -p temper`. If a name is not a package, use `cargo metadata` for the right one. Check `git grep -n 'pin::digest'` holds no reader you missed.
5. `cargo fmt`, the old-forms count of each tree, clippy of the changed packages, the Darwin check of `skein-forge` and `temper-forge-setup`. Commit and push.

## Report

As the implementer's text has it, naming each literal that followed the pin outside the list above.
