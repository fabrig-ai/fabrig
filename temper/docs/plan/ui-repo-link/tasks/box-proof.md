+++
id = "box-proof"
part = "proof"
kind = "feature"
needs = ["box-sign-in"]
files = [
  "temper/crates/temper/tests/support/forgejo_box_sign_in.rs",
  "temper/crates/temper/tests/support/mod.rs",
  "temper/crates/temper/tests/process.rs",
  "temper/docs/development/ui-repo-link-review.md",
  "temper/docs/design/protocol/sign-in.status.md",
]
effort = "high"
minutes = 150
+++

# The proof on a real Forgejo, the browser's walk, and the attacker's review

## Goal

Repeat the laptop's measurements (`sign-in.md`, 9.9) against temper's own issuer, on a real Forgejo the box set up. Walk it in a browser. Then read part 2 as an attacker would, and fix or report each thing found.

Acceptance:

1. **The process-tier story**, `a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click`, beside `forgejo_sign_in`'s (`temper/crates/temper/tests/process.rs`, about 2042): a fresh `temper box` on `skein_throwaway_forgejo`'s binary, the owned forge set up through the client's act, with a first repository. A scripted client with a cookie jar, as `support/forgejo_sign_in.rs`'s `Browser` is, opens the printed link, then `ROOT/user/oauth2/temper?redirect_to=%2FOWNER%2FNAME` as the client's own address gives it. It asserts:
   - it lands on the repository's page, signed in as the person, never Forgejo's Link account page, in the redirects it counts;
   - Forgejo's API names no second account: its user list before and after is the same, and the person's external login has `sub` `1:ADDRESS`;
   - a second press in the same jar lands the same, through `/oauth/authorize` again;
   - Forgejo's own login with the password, read from the `person_password` file (by the story alone, never logged), still works;
   - `box.log` and the page's meta are as the design says; a second box start adds no second source;
   - with the source's step made to fail (a `forgejo` binary that refuses `auth`, or as the world allows), the box runs, the set-up is done, the row's forge has no sign-in meta and has the off meta, and `box.log` has its one line.
2. **The browser's walk** on this builder, on a fresh box with a real Forgejo on this machine, by a throwaway CDP driver (`notes/common.md`), at 1440 and 420:
   - the switcher's link opens a new tab on the repository's page, signed in, with no password, no Link account page and no second account; a second click the same;
   - the project's name still switches project, and opens no tab;
   - the time "Continuing to Forgejo…" shows, in ms, at each click: it must not hold longer than jig's answer takes;
   - Forgejo's own login with the password still works in a fresh profile;
   - "Open Forgejo ↗" is on Forges.

   Screenshots at 1440 and 420 in `~/src/fabrig/ui-repo-link/` on the builder, outside any worktree, named by step. If a step can't run in a browser here, walk it in the process tier and say what a browser didn't show.
3. **The review,** `temper/docs/development/ui-repo-link-review.md`: part 2 read as **an attacker on the same machine** (another local user, with no access to the box's files) and **in another browser tab** (a page the person opened, on any origin, on another port of `127.0.0.1` among them). A table: what they try, what happens, evidence (a test, a command you ran, or code you read, by path), fixed or left and why. Try each by running it where you can. Each finding is fixed in this task when small, and reported in `## Left` with a recommendation when not. At least these, and whatever else you find:
   - a redirect, no PKCE and a wrong verifier at the live routes, with the public client;
   - another local user's process, holding no secret, running the public client's flow: what can it get with a code and its own verifier, and when can it get a code at all (Forgejo's port held or squatted while Forgejo is down);
   - a local process on temper's port while temper is down and Forgejo up: Forgejo fetches its token from whoever answers, and verifies no signature (`sign-in.md`, 9.2), and `auto` then links with no password;
   - cross-site: `127.0.0.1:8367` and `127.0.0.1:FORGE` are one site to `SameSite`, whatever jig's cookie says of another site's navigation; Forgejo's cookies reach temper, and a page on another port is same-site;
   - a tab pressing `ROOT/user/oauth2/temper?redirect_to=...` for the person with no consent, with hostile `redirect_to` values; a hostile OWNER or NAME from the forge in the client's address;
   - `/oauth/authorize` forged by a tab, with `client_id=forgejo`, its own state and challenge;
   - whether anyone but the owner can reach `/oauth/authorize` signed in on a box;
   - the key's and the source's files: modes, places, what a log or a process's argv shows (`ps` while `add-oauth` runs), against `AGENTS.md`, 3.
4. **What isn't proved on a real, non-fake box,** a section at the end of the review, by this task's own account: the walk's box is fresh, on a builder, with loopback addresses; what differs on a person's machine, on a public name, after an upgrade.
5. The reports are honest about the laptop's table: each of its four rows is repeated here or said not to be. `no email claim` can't be reached through temper's own issuer with a box's document, which is 9.9's point: hold it by a step test (`addresses` empty gives no `email`), and say so.

## Inputs

- `temper/docs/design/protocol/sign-in.md`, 9.2, 9.3, 9.9.
- `temper/docs/design/client/forge.md`, 5.4, *A repository's link*.
- `skein/docs/design/oauth-issuer.md`, 2 and 5.
- `AGENTS.md`, 3.
- `temper/docs/plan/ui-repo-link/inputs/request.md`, *The proof*; `notes/common.md`.

## Owns and may change

- Owns: this spec's `files`.
- A fix the review finds, when small: in the crate it belongs in (`temper-service`, `temper-forge-setup`, `temper-box`, `skein-oauth-issuer`, `jig-protocol-engine-clients`), failing test first, and a design line if the rule is new (this departs from the implementer's text; say which). A fix past about thirty lines, or one that changes 9.9's decisions, is reported with a recommendation, not made.
- **The process tier's budget is the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). If the story is over it: can it be made faster; otherwise report measurements. Don't move it.
- Options:
  - Setting up through the client's act isn't reachable from the process tier's helpers: use `support/owned.rs` and `support/first_repository.rs`, which do it. If they can't drive a real Forgejo on a box, the story uses `temper ctl`, as `support/boxed.rs` does.
  - No second browser profile in CDP: a fresh `--user-data-dir`.
  - If Chromium's new tab can't be followed by the driver, follow its URL in a new target and say so.

## Steps

Write the story's assertions first, and run them before fixing anything. Read a failure's log before a rerun. Walk the browser. Write the review, trying each row. Drop what this task closes from `sign-in.status.md`; leave a **Not yet** for anything left. `heavy cargo nextest run -p temper --test process -E 'test(=a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click)'`, and the focused tests of whatever a fix touched; `cargo fmt`; clippy of each package; citations. Commit and push.

## Report

As the implementer's text says, with each row of the review in one line, the walk's numbers, and what isn't proved on a real box.

## Answers

2026-11-06. **The first run, `box-proof-1`, wrote no report: what now?** Run the task again, from the top; nothing in this spec is changed in what it asks. That run ended after 21.7 minutes with no tokens counted and no branch pushed, so nothing of it carries over and its cause is unknown from the lab's records. `box-sign-in` is on the trunk, and its report says the builder's real Forgejo took the public source with no `--secret`, so nothing is known to block this task.

- **Keep your work as you go.** Commit and push `ui-repo-link/box-proof` as each acceptance item is reached, the story first and the review's rows as they are tried, and keep the review file current, so a run that dies leaves what it found. This replaces the single "Commit and push" at the end of `## Steps`.
- **Run the story, by name and alone.** `AGENTS.md`, 5 has a plan's new process-tier story written and compiled, not run. This task is the request's own proof (`inputs/request.md`, *The proof*) and the review is to try each thing where it can, so the story and the browser's walk run here, one story at a time; the whole process and browser tiers don't.
- **The command:** the default profile filters out the `process` binary (`notes/common.md`), so it is `heavy cargo nextest run --profile process-temper -p temper -E 'test(=a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click)'`, not the one in `## Steps`.
