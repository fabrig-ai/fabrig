+++
id = "first-run-walks"
part = "first-run-walks"
kind = "feature"
needs = ["journey-landing"]
files = [
  "temper/crates/temper/tests/fake_backends/walk/copy.rs",
  "temper/crates/temper/tests/fake_backends/walk.rs",
]
effort = "high"
minutes = 150
+++

# The proof: the first run, walked

## Goal

The whole first run walked in a browser on fresh boxes, and every page and state rendered beside the mock. This is the proof the
request asks of the whole plan. Request, *How the cut shapes it*.

Acceptance:

- **Three box walks,** each a fresh `temper box --fake-backends`, in headless Chromium through CDP:
  1. Forgejo with an empty repository, then Claude (`temper program-login` run by the walk, as `Act::ProgramLogin`), then Create, to
     Home with "work/hello is in Hello".
  2. The same with ChatGPT.
  3. "Nothing for now", then "Make an empty one" from Home, to the repository in the project.
- **The copies' act in a process walk on a connected fake forge,** committed: `walk/copy.rs`: the requests Ready's act sends, in its
  order, two addresses from the fake forge's catalogue (`temper/docs/design/draft/repository-copies.md`, 6), `into { project, home:
  first }`, queued on the one forge's writing run, both adopted, the first the project's home, and the rows reading as Home needs
  them on the way. An owned Forgejo reaches no `.invalid` host, so a walk can't copy on the box.
- **Renders** of every page and state of the request beside the mock's page at 1440, 1000 and 420, plus its open states: the sheet
  with each choice, "Each step" open, the steps hovered. Saved in `~/src/fabrig/ui-welcome/`, never committed.
- **The report says** which builder, each walk's steps and times, what differed from the mock and why, and **what a real box's copy
  left unproved** (Forgejo's own migration from a public host).

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, *How the cut shapes it*.
- `temper/crates/temper/tests/fake_backends/walk/{program_here,sign_in,open_default,copy}.rs`.
- `temper/docs/design/client/README.md`, 3.5, *The welcome*.

## Owns and may change

Owns the files in `files`.

May also change, for a fault a walk finds: a line of `temper/crates/temper-client-view/src/**`, `temper/crates/temper-client-domain/src/**`
or `temper-client-shell-web/temper.css`, with its test, named in the report. A fault beyond a line: report it under `## Left`, with the walk
that shows it, and go on.

Options, pre-approved:

- **The driver** is a throwaway script, untracked and gone before the push, by `notes/common.md`, Render proofs. The third walk's "Make an
  empty one" runs on the owned Forgejo, as the first two's `work/hello` does.
- If a process walk can't read a state on the way, it reads `forge list --json` and `repository list`, as `walk/copy.rs` does today.
- `claude-always` must have landed for a box's Model to offer Claude from init's own documents. On `--fake-backends` the rewrite adds it
  regardless: the walks hold either way, and the report says which.

## Steps

1. The process walk, in `walk/copy.rs`, then the test that runs it: `rg 'copy::' temper/crates/temper/tests/fake_backends` names it, for
   `heavy cargo nextest run -p temper --test fake_backends -E 'test(=NAME)'`; check the count.
2. The three box walks. A walk that hits a fault: fix a line and walk again, else `## Left`.
3. The renders: a throwaway server of the mock's checkout, or `file://`; set `temper-greenfield-mode` to `first-run` and
   `temper-greenfield-first-run` as `notes/common.md` says.
4. **Check,** `notes/common.md`: delete the driver; commit and push.

## Report

As the acceptance says.
