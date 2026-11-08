# temper-release-macos: Apple Silicon's release file, from a dispatched GitHub workflow

The request: `inputs/request.md`. The site's install line downloads `https://fabrig.ai/temper/$(uname -s)-$(uname -m)`; Apple Silicon's file, `Darwin-arm64`, needs a Mac, and GitHub's hosted `macos-26` runners are free on this public repository. One new workflow, `.github/workflows/release-macos.yml`, builds it at a dispatched commit and hands it over as an artifact. `notes/common.md` holds the rules.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `release-macos` | `tasks/release-workflow.md` | | the workflow: dispatch only, Rust 1.95 with wasm32, `TEMPER_BUNDLE=embed cargo build --release --locked -p temper --bin temper`, the `file` and `--help` checks, the artifact `temper-Darwin-arm64` (`temper`, `commit`, 7 days); deletes its **Not yet** entry | 25 |

**Critical path:** `release-workflow`, about 25 implementer-minutes. It is the plan's only task, released with the first push.

## What was decided

- **Design written here:** `temper/docs/design/release.md` is the download's document, and the request is a download. It gains 5.1, *Apple Silicon's file*, a subsection of section 5 (*A release*): the document closes with section 7's open questions, and surviving sections are never renumbered (`DESIGN.md`, 4), so a new section 8 after them was out. 5.1 says what the request says, as design: the workflow's trigger, permissions, toolchain, cache, build, checks and artifact; that the file is the site's, not a release's; and that the site's host fetches it and starts the workflow, outside the repository. Section 1 gains a bullet, the intro a clause, and section 7 an open question (Apple Silicon at a release: checksum, Developer ID, notarization). `release.status.md` gains **Not yet** 5.1, which the task deletes. `temper/docs/design/README.md`'s line for `release.md` names the file, and `domain/testing.md`, 2.8 says that macOS's one workflow is the one that tests, the release file's being 5.1's. No draft's status changes: the request is the plan's own input, and no draft says otherwise.
- **One task, not two.** The workflow is one file; its design entry's deletion is a line in the same task, as earlier plans did (a stated departure from "Not yours: any `docs/design/`", in `notes/common.md`).
- **The spec carries the whole file as its sketch.** In a workflow the outer layer (triggers, keys, commands, pins) is nearly all of it, and the request fixes each; the implementer owns step names and comments, and checks the file by reading against eight numbered items.
- **Pins checked by the cut:** `actions/checkout` v6.1.0 and `actions/cache/restore` v5.1.0 at `macos.yml`'s commits; `actions/upload-artifact` v7.0.2 at `cf430e030ddbb5b0abf93d22962f4752f3646cd9`, its newest release tag (2026-10-07), a lightweight tag, whose `action.yml` has `name`, `path`, `if-no-files-found`, `retention-days` and runs on `node24`. One anonymous read of the action's public repository (`git ls-remote` and a shallow fetch of that commit), so that no implementer needs a lookup; the spec forbids any.
- **The registry restored, never saved;** its `path:` block, key and restore-key byte for byte `macos.yml`'s, since actions/cache's version hashes the paths. No target cache, as the request says.
- **The build's environment is the request's line and nothing more:** the workflow's `env:` is `RUST_VERSION` and `CARGO_TERM_COLOR` alone. `macos.yml`'s `CARGO_INCREMENTAL` and `CARGO_PROFILE_*_DEBUG` are left out, so nothing touches the release profile.
- **The checks tolerate libmagic's word order:** `file -b` must contain `Mach-O 64-bit`, `executable` and `arm64`, since newer libmagic prints `Mach-O 64-bit arm64 executable, flags:...` where older prints `Mach-O 64-bit executable arm64`. `--help`'s first line must start `usage: temper `, which `temper/crates/temper/src/commands.rs`'s `help` prints today. Each failed check annotates the run, as `macos.yml`'s do, so a failure reads without the logs.
- **The artifact's files are staged** in `$RUNNER_TEMP/artifact/`, so the zip's root holds `temper` and `commit` and nothing under `target/`. `commit` is `git rev-parse HEAD`, the commit built.
- **One run at a time, whatever the ref:** the concurrency group is the plain `release-macos`, where `macos.yml`'s is per ref.
- **The timeout is the user's:** `[[question]]` `release-macos-timeout`; the task writes the recommended default, `timeout-minutes: 120`, `macos.yml`'s own, and waits for nothing.
- **Runs:** none of the workflow's own; its first dispatch is after landing. The task runs the outward boundary test (it reads every tracked file), the citations check and temper's old-forms count.

No stored format, protocol or public interface changes. The artifact's name and its two files are a contract with the site's host, written in `release.md`, 5.1.

## Overlaps with drafts beside this one

- No draft beside this one writes `.github/`, `temper/docs/design/release.md` or its status, `temper/docs/design/README.md` or `temper/docs/design/domain/testing.md`.
- `look-outside-pages/account` and `look-ask-ways/account` write `temper/docs/design/STATUS.md`; this plan leaves it alone: its paragraph already names the release (`release.md`) among what lags, and its index lists `release.status.md`.
