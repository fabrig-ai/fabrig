+++
id = "grader-worst-case"
part = "shell-cwd-grader"
kind = "feature"
needs = []
files = ["smith/benchmarks/tasks/repository/shell-cwd/grader/src/lib.rs", "smith/docs/design/benchmarks.status.md"]
effort = "high"
minutes = 60
+++

# smith: the shell-cwd grader compiles against its frozen tree

## Goal

`smith/benchmarks/tasks/repository/shell-cwd/grader/src/lib.rs`, in `Harness::new`, calls `tools::valid(&limits)`. The grader builds against the frozen tree `shell-cwd-56c0c59`, whose `smith-domain-tools` exports `worst_case` and has no `pub fn valid`, so that grade fails for every candidate, the reference included. Put the line back. Then prove it on a bundle if one can be had inside the time this spec gives, and otherwise say "not validated" plainly and record it.

Acceptance:

- The line reads `assert!(tools::worst_case(&limits).is_some());`, and `git grep -n 'tools::valid' -- smith/benchmarks/tasks/repository/shell-cwd` finds nothing. Nothing else in `grader/` changes.
- `task.toml`, `seed/bundle.toml` and `reference/**` don't change. **`version` stays 3:** the cut's decision. `smith/docs/design/benchmarks.md`, 5.2 changes `version` with the grader so that results of different versions are never compared, and no result exists at 3 (`smith/docs/design/benchmarks.status.md`, 9, 16.4 and 11.4).
- `heavy cargo nextest run -p smith-bench` passes. It reads the task's tree and doesn't build this grader, so it shows only that the catalogue still holds; no test pins this line, and no new test is wanted (the request's proof is the validate below).
- **Either** `smith-bench validate repository/shell-cwd` ran on a bundle whose digest matches `seed/bundle.toml`, with the seed failing and the reference passing, and the status entry below goes. **Or** the report's first line after `Done:` says `not validated`, says what stopped it, and the status entry below is rewritten. Either is `Done:`.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/inputs/request.md`, its smith section
- `smith/docs/design/benchmarks.md`, 5.2
- `smith/benchmarks/README.md`, from "The repository suite uses `shell-cwd`": how a bundle is made
- `smith/benchmarks/src/bundle.rs` and `smith/benchmarks/src/validate.rs`

## What the cut already checked

Don't repeat these; they save you the search.

- The frozen source `56c0c59735f3a163753fdd43e20a7e87df369ce3` is in neither this repository nor the old smith mirror. The skein pin `f5fca44f2c9ee92f2dc52d8dc845212ca692275f` isn't in this repository; it is in the old skein mirror.
- The old mirrors are on hel8: `~/src/fabrig/attic/smith/mirror.git` and `~/src/fabrig/attic/skein/mirror.git`, with `smith.bundle` and `skein.bundle` beside them. `~/src/fabrig/coord/old-repos*`, which the request names, holds briefs and surveys and no repository.
- In the smith mirror, many commits of 2026-10-09 both hold the frozen `crates/smith-domain-tools/src/limits.rs` (blob `2865805`) and pin that skein in `Cargo.toml` or `Cargo.lock`; `bf96e949`, `49057ae9`, `44e51206` and `a2c2e53f` among them. In this repository, `839ac73994` and `530f42dfef` hold the blob but not that pin. The frozen tree is `git archive`-exact or it fails the digest: expect a search among these, not one try.
- Hel8, hetzner6 and hetzner8 hold no bundle.

## Owns and may change

- **Owns:** the one line of `grader/src/lib.rs`, and the `11.4` entry of `smith/docs/design/benchmarks.status.md`, which departs from the implementer's text: you write that entry, in the way below, and nothing else under `docs/design/`.
- **May read,** departing from the rules on where you search: `$XDG_STATE_HOME/smith-bench/bundles/shell-cwd-56c0c59/` on your own builder, whatever it holds, and the four paths above on hel8, read-only, by `scp` or `ssh` in a login shell. Read nothing else there. If your builder can't reach hel8, that is what stopped it: not validated.
- **Scratch** under `~/src/fabrig/`, outside any slot, removed before you finish. A bundle goes in the state directory above only once its digest matches, and it stays there: it is what the task asks a machine to hold; name the machine in your report. Never put vendored sources or a bundle in the slot or the branch.
- **Fetch nothing from the host the frozen `Cargo.lock` names.** Point that source at the skein mirror (`git`'s `url.<base>.insteadOf`, set through the environment for those commands, not in your global config). `cargo vendor` reaches crates.io as the README says.
- **The rebuild has 40 minutes,** from the first `scp`. Vendor once per `Cargo.lock` and swap the source tree under it rather than vendoring each candidate; a commit's digest either matches `sha256` in `seed/bundle.toml` or it doesn't. Don't change the seed's digest, the manifest or the README to make one fit. At 40 minutes with no match: not validated.
- **If validate runs on a matching bundle and something fails:**
  - The oracle doesn't compile on `tools::worst_case`: `Question:`, with the compiler's output.
  - Anything else (the first grade's two lines, its 80 s, the seed passing, the reference failing): `Done:`, with the output under `## Left`, and the entry says what was seen. Deadlines are the user's; change none.

## The status entry

`smith/docs/design/benchmarks.status.md`, under **Not yet**, `11.4`.

- **Validated:** delete the `11.4` entry whole.
- **Not validated:** the entry reads as follows, with its last sentence in your words:

  ```
  - 11.4: `shell-cwd` at version 3 has not been seen on its bundle.
    Only `smith-bench validate repository/shell-cwd` on a machine
    holding it shows that the first grade finds the frozen workspace's
    two lines (section 15), that the grade keeps its 80 s, that the
    oracle compiles against the frozen tree (its `tools::worst_case`
    is read from the blob `reference/solution.patch` names and from
    the rewritten commits holding it) and that the seed fails and the
    reference passes. <Not validated: what the rebuild tried and what
    stopped it, in a sentence, naming no machine.>
  ```

## Steps

1. Change the line; `heavy cargo nextest run -p smith-bench`; commit.
2. The bundle: your builder's state directory, then the rebuild, in its time. If you have one: `heavy cargo run -p smith-bench -- validate repository/shell-cwd`.
3. The status entry; `python3 -I -B tools/citations/citations.py --all` reads 0 after you commit. Commit and push.

## Report

The common note's items; validated or `not validated`, on its own line, first; candidates tried and each one's digest; validate's output for the seed and the reference if it ran.
