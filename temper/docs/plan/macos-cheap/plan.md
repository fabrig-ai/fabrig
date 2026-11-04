# macos-cheap: the plan

From `inputs/request.md`: temper's acceptance story on macOS, cheaply. The user's rule governs every choice.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| workflow | `workflow` | | `.github/workflows/macos.yml`, every target compiled |
| pins | `pin-move` | | versions, all digests, `digest(binary, platform, arch, form)`, readers |
| pins | `conformance` | `pin-move` | re-records at the new binaries; the story on Linux at them |

Later batches: `fetch`, the run-time habits and the story's support and step, once the survey of the story's path reports.

## Critical path as it stands

`pin-move` (40), then `conformance` (90).

## What the cut found

- **ghcr.io answers one redirect:** both blobs answer `307` to `pkg-containers.githubusercontent.com`, then `200`; the archives' digests match their addresses. The first unverified assumption of the request is settled for 17.0.1 and 13.2.1.
- **The bottles' binaries stand alone by their load commands:** `libSystem`, `libresolv`, `CoreFoundation`, `Security`, no Homebrew path. Forgejo's minimum is macOS 15, the runner's 13. That they run is still the first run's to show.
- **The formulae's versions and lines:** `forgejo-runner`'s build gives `version=v13.2.1`, so `names_runner` holds; Forgejo's `make build` gives `17.0.1` in the binary.

## Landing

- **The trunk lands whole.** fab's integration plans land when every task has merged and the end check is green (landing at slice boundaries is fab's Later), so "the workflow lands early" here means a small plan with a short path: the workflow lands with the plan, not before it. A follow-up branch can be run by hand on GitHub, before it lands, once the file is on `main`.
- **Step 5 stays open at landing.** No builder runs macOS; the first scheduled run shows, and its failures are follow-up tasks. Reading results into after-landing is not this plan's.

## skein and jig

- skein: `skein-forge`'s pin, and what the conformance's diff forces (expected: nothing).

## Overlaps with the drafts beside this one

- none yet: `debt-9` and `debt-10` touch `temper-service` and `temper-engine-*`, not `temper-forge-setup`; `kv-chunk-target` is `skein-kv`.
