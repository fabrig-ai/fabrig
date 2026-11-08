# temper-release-strip: the Apple Silicon release binary, stripped

The request: `inputs/request-strip.md`. The site publishes temper stripped and gzipped. Linux's file is stripped on the host that publishes it; Apple Silicon's must be stripped on macOS, since stripping invalidates the ad hoc signature Apple Silicon needs to run it, and only Apple's `codesign` remakes it. One new step in `.github/workflows/release-macos.yml` strips and re-signs the file before the checks. `notes/common.md` holds the rules.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `release-strip` | `tasks/strip-sign.md` | | after the build: `xcrun strip`, `codesign --force --sign -`, `codesign --verify` (annotated), sizes and `Signature=adhoc` in the log; the header comment; the existing checks and artifact take the stripped file unchanged; deletes its **Not yet** entry | 15 |

**Critical path:** `strip-sign`, about 15 implementer-minutes. It is the plan's only task, released with the first push.

## What was decided

- **Design written here:** `temper/docs/design/release.md`, 5.1 gains the bullet *Stripped on the runner* (Xcode's `strip` in place, the ad hoc signature remade by `codesign --force --sign -` and checked by `codesign --verify`, why it happens on the runner, the release profile left as it is), and *Its signature is the linker's* becomes *Its signature is ad hoc*, the one `codesign` makes. The checks are said to run on the stripped file, `--help` showing also that Apple Silicon runs it under the new signature; the artifact's `temper` is the stripped, signed binary. Section 1's bullet and section 7's Apple Silicon default follow. No section is added or renumbered.
- **The checksum sentence corrected:** 5.1's intro said no checksum file goes beside the site's file; the request says the site's host makes the gzip and the `.sha256`. The intro now says the workflow writes no checksum, and *Where it goes is the site's* says the host serves it gzipped and with its checksum, both made there. The path `/temper/Darwin-arm64` is left as it stood: the request names no other, and the set under `/temper/v{MAJOR}/` waits on another draft.
- **`release.status.md`** gains **Not yet** 5.1, which the task deletes. No draft's status changes: the request is the plan's own input, and no draft says otherwise (`draft/macos.md` and `draft/first-release.md` say nothing of stripping or the file's signature). `temper/docs/design/README.md`'s line for `release.md` and `domain/testing.md`, 2.8 stay true as they are.
- **`xcrun strip`,** not a bare `strip`: the request names Xcode's, and `xcrun` resolves the selected Xcode's toolchain whatever `PATH` holds. `strip` takes no option: on a dynamically linked Mach-O executable it keeps only what dynamic linking needs.
- **In place:** the step strips `target/release/temper` itself, so the two checks and the artifact's step read the stripped file with no line of theirs changed (the request's item 2), and the artifact's name and two files stay as they are (item 3).
- **The log carries the evidence** for ops' first dispatch, the request's "Done when": which `strip` ran, the sizes before and after, and `codesign --display`'s `Signature=adhoc`. `codesign --verify` failing annotates the run, as the file's other checks do; a failed `strip` or `codesign --force` fails the step under bash's `-e`, and nothing is uploaded.
- **One task, not two:** the step is one file's; its design entry's deletion is a line in the same task, as `temper-release-macos` did (a stated departure, in `notes/common.md`). Effort medium: the spec carries the whole change.
- **No question:** the timeout, runner, pins, build line and artifact stay as they are; nothing here is the user's to decide.
- **Runs:** none of the workflow's own; its next dispatch is after landing. The task runs the outward boundary test (it reads every tracked file), the citations check and temper's old-forms count.

No stored format, protocol or public interface changes. The artifact's name and its two files, the contract with the site's host in `release.md`, 5.1, stay as they are; its `temper` is now stripped.

## Overlaps with drafts beside this one

- No draft beside this one lists `.github/`, `temper/docs/design/release.md` or `release.status.md`. The queued drafts (`reference-walks`, `v0.1.0-ladder`, `configuring-walks`, `post-landing-probes`) list no files yet; `v0.1.0-ladder` may reach `release.md`'s release sections later, not 5.1.
- `look-ask-ways/account` and `tui-box-front/account` write `temper/docs/design/STATUS.md`; this plan leaves it alone: its paragraph already names `release.md` among what lags, and its index lists `release.status.md`.
