+++
id = "conformance"
part = "pins"
kind = "feature"
needs = ["pin-move"]
files = ["skein/tests/forge/conformance/facts.txt", "skein/tests/forge/conformance/transcripts/**", "skein/tests/forge/conformance/deliveries/**", "temper/tools/git-transcripts/**"]
effort = "high"
minutes = 90
+++

# Forgejo 17.0.1 and runner 13.2.1 hold what the conformance checked

## Goal

The pin moved on no release notes, so conformance is the check. Record skein's forge conformance and temper's git transcripts at the new binaries, keep what they recorded, fix what differs, and run temper's acceptance story on Linux at the new binaries.

Acceptance:

- `skein/tests/forge/conformance/{facts.txt,transcripts/,deliveries/}` and `temper/tools/git-transcripts/` are recordings of 17.0.1 and 13.2.1, every fact that held at 17.0.0 still `holds`, or the report names it with what was done;
- the tests that read those recordings pass;
- the process story passes on the builder at the new binaries.

## Inputs

- `skein/docs/design/forge.md`, 4.3 and section 9.
- `skein/tests/forge/conformance/README.md`; `temper/tools/git-transcripts/README.md`.
- `temper/docs/plan/macos-cheap/notes/common.md`.

## Owns and may change

Owns the recordings above, and may run the tools' commands.

May change, for a difference the recording shows, the fake that answers it or the decoding that reads it, in `skein/crates/skein-fake-forge*`, `skein/crates/skein-forge/src/forgejo/**`, or a test's literal that names the recorded value.

Pre-approved:

- A departure from the implementer's text: **it fetches.** Only the two named URLs below, by `curl`, into `target/pins/` of your slot, each checked by its pin's digest. Nothing else of the network, and they are never committed.
- The builders' `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` still name 17.0.0, and moving them is ops', not yours. You give yours per command with `heavy env`. In `## Left`, say the machines still need the new binaries (the plan's question `builders-binaries`).
- If a diff is only the version text, ids or times the masks do not cover, keep the recording.
- If a fact that held no longer does, and the fix is a field or a match arm: make it, and name it in the report. If it needs more, or Forgejo's behaviour changed in a way temper relies on: push what you have and stop with `Question:`, naming the fact, the old and new answers, and whether to adapt or to hold the pin.
- `skein/docs/design/forge.md`'s section 9 counts (24 hold, 7 recorded) are the lead's: report any change to them rather than editing the design.

## Steps

1. Binaries, in the path shape the machines use, so that their name is what `--version` prints:

   ```sh
   mkdir -p target/pins/forgejo-17.0.1 target/pins/forgejo-runner-13.2.1
   curl -fsSL -o target/pins/forgejo-17.0.1/forgejo https://codeberg.org/forgejo/forgejo/releases/download/v17.0.1/forgejo-17.0.1-linux-amd64
   curl -fsSL -o target/pins/forgejo-runner-13.2.1/forgejo-runner https://code.forgejo.org/forgejo/runner/releases/download/v13.2.1/forgejo-runner-13.2.1-linux-amd64
   ```

   Check each with `sha256sum` against `FORGEJO_SHA256_LINUX_AMD64` and `RUNNER_SHA256_LINUX_AMD64`; `chmod 0755`. Use absolute paths below.
2. Record skein's, ordinary and `--component`, into `target/conformance-recording`, as the README has it, each under `heavy`, then copy `facts.txt`, `transcripts/` and `deliveries/` in.
3. Record the git transcripts, as their README has it, with `heavy env TEMPER_FORGEJO=<absolute path> cargo run -p git-transcripts -- ...`.
4. Read `git diff --stat`, then the hunks in question: what changed beyond versions? Resolve as above.
5. The tests of the recordings' readers: `heavy cargo nextest run -p skein-forge -p skein-forge-machine-world -p temper-worker-protocol-git-world -p git-transcripts -p forgejo-conformance`, taking the package names from their `Cargo.toml`s, and the packages you changed.
6. The story, at the new binaries, on Linux:

   ```sh
   heavy env TEMPER_BUNDLE=none TEMPER_FORGEJO=<absolute>/forgejo TEMPER_FORGEJO_RUNNER=<absolute>/forgejo-runner cargo nextest run --profile process-temper -p temper --test process -E 'test(=a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart)'
   ```

   Check that it selected one test.
7. `cargo fmt`, the old-forms count, clippy of what you changed. Delete `target/pins` and the recordings' scratch. Commit and push.

## Report

As the implementer's text has it, with: the two binaries' `--version` lines; each fact whose verdict changed; what the transcripts' diff held beyond versions; the story's result and time.
