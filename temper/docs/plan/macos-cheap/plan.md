# macos-cheap: the plan

From `inputs/request.md`: temper's acceptance story on macOS, cheaply. The user's rule governs every choice: "make sure that anything we do is relatively cheap".

## Parts and tasks

| Part | Task | Needs | Minutes | What it does |
|---|---|---|---|---|
| workflow | `workflow` | | 30 | `.github/workflows/macos.yml`, every target compiled |
| pins | `pin-move` | | 40 | versions, every digest, `digest(binary, platform, arch, form)`, readers |
| pins | `conformance` | `pin-move` | 90 | re-records at the new binaries; the story on Linux at them |
| fetch | `fetch` | `pin-move` | 110 | `temper forge fetch` on macOS: token, one redirect, archive, two digests |
| story | `habits` | `pin-move` | 75 | pipe check, login, `parent::Guard`, `base64`; every target compiled for Darwin |
| story | `story-step` | `workflow`, `fetch`, `habits` | 45 | the workflow fetches, shows the binaries run, runs the story; closes the status entries |

Six tasks. Steps 1 to 4 of the request are the tasks. Step 5, the story green on the runner, is the first run's to show (below).

## Critical path

`pin-move` (40), `fetch` (110), `story-step` (45): about 195 implementer-minutes, then the end check.

## What the cut found

- **ghcr.io answers one redirect:** both blobs answer `307` to `pkg-containers.githubusercontent.com`, then `200`; the archives' digests match their addresses. The first unverified assumption of the request is settled for 17.0.1 and 13.2.1.
- **The bottles' binaries stand alone by their load commands:** `libSystem`, `libresolv`, `CoreFoundation`, `Security`, no Homebrew path. Forgejo's minimum is macOS 15, the runner's 13. That they run is still the first run's to show, and the workflow prints their versions apart from the story for it.
- **The formulae's lines:** `forgejo-runner`'s build gives `version=v13.2.1`, so `names_runner` should hold; Forgejo's `make build` puts `17.0.1` in the binary.
- **A survey of the story's path** (static, nothing ran on macOS): the story reaches the fetch, the pipe check, the login and `base64 -w0`; `parent::Guard` stops the compile in two places, one of them new (`forge_land/rung.rs`). It never reaches `/usr/bin/rg`, and `left` reads empty under Group, so the test needs no change there.

## What the cut decided

- **rg and git stay as they are.** The request names "smith's rg and git paths" under "what the story needs at run time, and no more". The story's scripts call `delegate`, `wait`, `read`, `edit`, `write` and `finish`; rg runs only for `List` and `Search`, and `/usr/bin/git` exists on macOS. So the plan leaves them; the draft's status keeps 4.1 as a lag. If the first run shows otherwise it is a small follow-up.
- **The login is `/usr/bin/id -un` on macOS,** not a `getpwuid` in skein's shell as the draft has it: no change to skein, and Linux stays byte for byte. Recorded as **Design departs** in the draft's status. The pipe check is Linux's alone, as the request allows.
- **Every Darwin compile the builders can reach:** `habits` tries a throwaway stand-in for Apple's C compiler under `cargo check`, which links nothing, so that `ring`'s build script passes and the 121 members that reach rustls are compiled on a builder. It has never been tried; if it fails, the workflow has those members.
- **`pin-move` is split from `conformance`** so that `fetch` and `habits` do not wait for a recording.

## Landing

- **The trunk lands whole, so the workflow cannot land first.** fab's integration plans land when every task has merged and the end check is green; landing at slice boundaries is fab's Later. The request's "the workflow lands early" here means a small plan on a short path. The workflow's first run comes after the whole plan lands.
- **Step 5 stays open at landing.** No builder runs macOS; the first scheduled run shows, and its failures are follow-up tasks. Once the file is on `main`, a follow-up's branch can be run by hand on GitHub before it lands (`workflow_dispatch` on that ref), which saves a landing per round. Reading results into after-landing is not this plan's.

## The machine action

Question `builders-binaries` (in `plan.toml`, for ops): the builders need the new binaries, and `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` must move when the trunk lands, or the process and live tiers refuse the old ones on main. No task waits: the gate needs neither binary, and `conformance`, `habits` and `story-step` fetch their own copies.

## skein and jig

- skein: `skein-forge`'s pin; what `conformance`'s diff forces (expected: nothing); a `cfg` gate on a test if the Darwin compile shows one.
- jig: nothing but a `cfg` gate on a test, if the Darwin compile shows one.
- hearth: nothing. The workflow leaves it out.

## Overlaps with the drafts beside this one

- `temper/crates/temper/src/lib.rs`: `habits` touches `serve` and `login`; `serve-without-box/init-box` changed the same file, elsewhere. Unavoidable, and small.
- `temper/docs/design/protocol/setup.status.md`, `testing.status.md`, `forge.status.md`: long one-line entries that other drafts edit. This plan's entries stand on lines of their own, so that merges meet no conflict.
- `live-login-guard` touches `tests/support/forge_land/rung.rs`; `habits` gives `parent::Guard` on macOS instead of changing that file.
- `debt-9` and `debt-10` touch `temper-service` and `temper-engine-*`; `pin-move` touches their test literals only if they go red.
