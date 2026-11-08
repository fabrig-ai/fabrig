+++
id = "strip-sign"
part = "release-strip"
kind = "feature"
needs = []
files = [
  ".github/workflows/release-macos.yml",
  "temper/docs/design/release.status.md",
]
effort = "medium"
minutes = 15
+++

# The Apple Silicon release file, stripped and signed again ad hoc

## Goal

The site publishes temper stripped. Apple Silicon's file must be stripped on macOS, by Apple's `strip`, since stripping invalidates the linker's ad hoc signature without which Apple Silicon runs no file, and only Apple's `codesign` remakes it. One new step in `.github/workflows/release-macos.yml`, after the build and before the checks, strips `target/release/temper` in place, signs it again ad hoc and verifies the signature; the existing checks and the artifact then take the stripped file. Nothing in this plan runs the workflow: its next dispatch, after landing, is its check.

Acceptance, each checked by reading the file (`notes/common.md`), item by item, and reported with the file's line numbers:

1. **The new step** sits between the build step (`The build, under the release profile as the workspace has it`) and `An arm64 Mach-O executable`, and runs, in this order: `xcrun --find strip` (the log shows which `strip` ran); `ls -l target/release/temper`; `xcrun strip target/release/temper`, in place, with no option; `ls -l target/release/temper` again (the log shows the sizes before and after); `codesign --force --sign - target/release/temper`; `codesign --verify --verbose target/release/temper`, which on failure prints an `::error title=codesign::...` annotation saying the signature does not verify after the strip and exits 1; then `codesign --display --verbose=2 target/release/temper` (the log shows `Signature=adhoc`). No copy of the binary is made: the file stripped is the file the later steps read.
2. **Everything else is as it was:** the build step's line `TEMPER_BUNDLE=embed cargo build --release --locked -p temper --bin temper`, the two checks (`file -b` and `--help`), the artifact's step and upload (`temper-Darwin-arm64`, `temper` and `commit`, 7 days), `on:`, `permissions:`, `concurrency:`, `env:`, the job's keys and every `uses:` pin, byte for byte. Nothing sets a profile, flag or environment variable: no `CARGO_PROFILE_*`, `RUSTFLAGS`, `--config`, no cargo configuration written. The file's diff against the commit your branch starts from is the header comment's lines and the new step's lines, nothing else.
3. **The header comment** says the file is stripped and signed again ad hoc before it is checked, and still cites `temper/docs/design/release.md, 5.1`.
4. **Status:** in `temper/docs/design/release.status.md`, the **Not yet** entry `5.1: .github/workflows/release-macos.yml neither strips the file nor signs it again ad hoc.` is deleted and the paragraph rewrapped so that it reads `... 5: \`scripts/release.sh\`. 6: the walk on a fresh machine.`; no other line of any design file changes.
5. **The file names nothing of the lab** (`notes/common.md`); `heavy cargo nextest run -p fabrig-boundary --test outward` passes after the commit, its count in the report.

## Inputs

- `temper/docs/plan/temper-release-strip/notes/common.md`.
- `temper/docs/plan/temper-release-strip/inputs/request-strip.md`, whole: items 1 to 3 of its "What" are acceptance 1 and 2.
- `temper/docs/design/release.md`, 5.1 (*Apple Silicon's file*), its bullets **Stripped on the runner**, **Its signature is ad hoc** and **Its checks, before the upload**, written by this cut; read only.
- `.github/workflows/release-macos.yml` as it stands: the style of its steps' names, comments and annotations.

## Owns and may change

Owns `.github/workflows/release-macos.yml`, and in `temper/docs/design/release.status.md` the one entry of acceptance 4.

Nothing else: not `macos.yml`, not `Cargo.toml`, not code, not `temper/docs/design/release.md` (what the work taught goes in the report).

Options, each decided:

- **`xcrun strip`, not a bare `strip`:** the request names Xcode's `strip`, and `xcrun` resolves the selected Xcode's toolchain whatever `PATH` holds. Keep it.
- **Step names and comments** may be reworded in the file's style (what is true, in the design's words, citing no section but the header's); the sketch's commands, their order and the annotation are not yours to change.
- **`actionlint`:** if it is on `PATH`, run it on the file and quote its output in the report; a finding against a line of acceptance 1 or 2 is fixed only where the fix keeps that item, else reported. If it is not on `PATH`, say so; install nothing, fetch nothing (`notes/common.md`).

## Steps

1. In `.github/workflows/release-macos.yml`, replace the header comment's first lines:

   ```yaml
   # Apple Silicon's file for the site (temper/docs/design/release.md, 5.1):
   # temper built for release at the dispatched commit, stripped and signed
   # again ad hoc, checked, and left as the run's artifact. It writes nothing
   # to the repository and uses no secret.
   ```

   and insert, between the build step and `An arm64 Mach-O executable`, with a blank line on each side as the other steps have:

   ```yaml
         # Stripping invalidates the linker's ad hoc signature, without which
         # Apple Silicon runs no file; codesign makes it again, in place, so
         # the checks and the artifact below take the stripped file.
         - name: Stripped by Xcode's strip, and signed again ad hoc
           run: |
             xcrun --find strip
             ls -l target/release/temper
             xcrun strip target/release/temper
             ls -l target/release/temper
             codesign --force --sign - target/release/temper
             if ! codesign --verify --verbose target/release/temper; then
                 echo "::error title=codesign::target/release/temper's signature does not verify after the strip"
                 exit 1
             fi
             codesign --display --verbose=2 target/release/temper
   ```

2. Read the file through against acceptance 1 to 3, one item at a time, and read `git diff` of it against your branch's base for acceptance 2. Run `actionlint` if it is on `PATH` (Options).
3. Delete the status entry (acceptance 4).
4. `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`. No `cargo fmt` or clippy: no Rust changes.
5. Commit (why: the site publishes temper stripped, and Apple Silicon's file can only be stripped where Apple's `codesign` can sign it again, so the workflow strips it, re-signs it ad hoc and verifies it before its checks), then `heavy cargo nextest run -p fabrig-boundary --test outward` (it reads git's tracked files, so it runs after the commit), and push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

fab's implementer report, with: acceptance 1 to 5 each with the file's line numbers that hold it; the file's `git diff --stat` against the branch's base; `actionlint`'s output, or that it was not on `PATH`; the outward test's count; and anything in the sketch you changed, with why. No run of the workflow: its next dispatch is after landing.
