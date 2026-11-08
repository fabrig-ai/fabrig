# temper-release-strip: the Apple Silicon release binary, stripped

## Why

The user, 11-08: the binary that fabrig.ai offers for trying temper in a box is published
stripped and gzipped. The Linux one is stripped on the host that publishes it. The Apple
Silicon one has to be stripped on macOS, by Apple's own `strip`, since stripping invalidates
the ad-hoc signature that Apple Silicon needs to run it, and the signature is remade there.

## What

In `.github/workflows/release-macos.yml`, and nothing else:

1. **After the build and before the checks,** a step that strips `target/release/temper` with
   the runner's `strip` (Xcode's), then signs it again ad hoc, `codesign --force --sign -`,
   and checks it with `codesign --verify`.
2. **The existing checks then run on the stripped file:** the arm64 Mach-O check and
   `temper --help`. So the uploaded artifact holds the stripped, re-signed binary.
3. The build itself stays as it is: the release profile as the workspace has it, with no
   profile, flag or environment change. The artifact's name and its two files stay as they are.

## Not in this plan

- Gzip and the `.sha256` files: they're made on the host that publishes them.
- Linux, the release profile, `macos.yml`, and any code.
- The later "real deployment" set under `/temper/v{MAJOR}/`, which waits on another draft.

## Done when

The workflow's change is on main. Its next dispatch, by ops after landing, is its check.
