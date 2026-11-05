+++
id = "signin-model"
part = "signin-model"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/fake_backends/walk/sign_in.rs",
  "temper/crates/temper/tests/fake_backends/walk.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 150
+++

# A fake box's ChatGPT sign-in leaves a usable model

## Goal

On a fresh `temper box --fake-backends`, "Continue with ChatGPT" came back to `/welcome?at=ready`
reading "Model none yet", with no failure, and "Create your project" disabled. Show it in a failing
test, find the fault, and fix it in the lowest tier that shows it. A sign-in that does end with no
usable model must say why on Ready.

Acceptance:

- A test that failed for this reason before the fix and passes after it: a fake box's ChatGPT
  sign-in leaves an account with a usable model, and Ready enables "Create your project".
- The fault is fixed where it lies, with the narrowest test beside it, in the package that holds it.
- Your report's first section answers: is this a regression since `68cddad304`, the commit whose
  walk the user recorded (the commit that broke it, or that it is not one), or the fake's, a sign-in
  that returned no served model? Say what you ran to know.
- Ready draws every `Refused` and `Failed` ending under "Model none yet" in its words, with a fresh
  "Continue with ChatGPT" (`temper/docs/design/client/README.md`, 3.5), with a `temper-client-view`
  test for each ending `welcome_end` left silent.
- `temper/docs/design/client/README.status.md`: the 3.5 **Not yet** entry on Ready's endings is gone.

## Inputs

- `temper/docs/plan/fake-signin-model/notes/common.md`.
- `temper/docs/plan/fake-signin-model/inputs/request.md`, which holds the walk's stop. If your builder
  has its whole report, `/home/free/src/fabrig/fab/runs/look-round-3/record-landing-1/report.md`, read
  its section "Walk and exact stop" only.
- `temper/docs/design/client/README.md`, 3.5 (*Ready*; this cut added its "Any other refusal or
  failure" sentence), 4.8 (what makes a model usable), 5.5 (A sign-in).
- `temper/docs/design/protocol/setup.md`, 7.5 (the fake issuer and the signed-in bearer).
- `skein/docs/foundation/testing-strategy.md`, 8.1.

## Owns and may change

Owns, as `files` lists them. `temper/crates/temper/tests/fake_backends/walk/sign_in.rs` is new.

May also change:

- `temper/crates/temper/tests/fake_backends/walk.rs`: an `Act::Visit` arm, `Walk`'s count of calls,
  and the `mod sign_in;` line. Nothing else: `repository-copies-1/copy-walk` is changing this file
  on its own trunk.
- `temper/crates/temper/tests/fake_backends/walk/open_default.rs`: `pub(super)` on the helpers the new
  walk shares (`browser_bounds`, `named`, `live`). Nothing else.
- `temper/crates/temper/tests/support/**`: only what `Step::Visit` needs to follow a sign-in's
  address on a box referee.
- **Where the fault lies, one of these, its tests beside it:**
  - the fake: `temper/crates/temper-fake/src/oauth.rs`, `src/oauth/tests.rs`, `src/llm.rs`,
    `src/credential.rs`, `src/backends.rs`, `src/backends/tests.rs`,
    `temper/crates/temper/src/fake.rs`, `temper/crates/temper/src/fake_backends.rs`;
  - skein's sign-in: `skein/crates/skein-oauth/**`, `skein/crates/skein-oauth-accounts/**`,
    `skein/crates/skein-fake-oauth/**`, `skein/crates/skein-fake-peers/src/oauth*`,
    `skein/tests/oauth-accounts/**`;
  - jig's accounts: `jig/crates/jig-protocol-accounts/**`, `jig/crates/jig-domain-accounts/**`,
    `jig/crates/jig-domain-core/src/documents/accounts.rs`,
    `jig/crates/jig-domain-core/src/runs/coverage.rs`;
  - the client's reading: `temper/crates/temper-client-domain/src/pages/welcome.rs`,
    `temper/crates/temper-client-domain/src/forge/model.rs`, and their tests.

Options, pre-approved:

- **The walk's logs,** `~/src/fabrig/look-round-3/project-home/walk/record-landing-1/logs`, are on
  the builder that ran it. If your builder has them, `box.stderr` and the `.json` observations are
  yours to read; if not, reproduce afresh. Either way the failing test comes first.
- **The test can show it lower** than the fake box: add the lower test too, and keep the box walk
  all the same, since no test signs in with ChatGPT on a fake box today.
- **The fault is in skein or jig:** reproduce it in that tree's tier, not only in temper's
  (`AGENTS.md`, 5). A fix that changes what a shared item means, not only its signature, also runs
  the tests of every package that reads it, by `cargo metadata`, and the report names them.
- **The walk needs no Forgejo to fail:** say so in the report, and still walk it with Forgejo, the
  order the person walked and the one that makes Ready's Create reachable.
- **Over budget:** the walk joins `process-temper`, which already holds Forgejo walks. Report its
  CPU time. If it sends that tier over, follow the three questions (`AGENTS.md`, 5).

Stop with a `Question:`, proposed words and your recommendation, if a sign-in really ends `Made` or
`Renewed`, not declined, with no account usable: the design has no words for that. Also stop if the
fix needs a wire or a stored record (`notes/common.md`), or a file beyond this list.

## Steps

The sketches below bind the test's shape. What you read below as facts were read from the code by
the cut; none of them is the fault until your test says so.

Facts read:

- No test signs in with ChatGPT on a fake box. `fake_backends/walk` reaches a usable model by a key
  (`here_key.rs`) and a program's login (`program_here.rs`). The ChatGPT stories,
  `temper/crates/temper/tests/support/sign_in.rs` and `sign_in_welcome.rs`, run their own issuer, not
  `temper fake oauth`, and no Forgejo. Their peers' settings differ from
  `temper/crates/temper-fake/src/oauth.rs`'s; compare them.
- `fake_backends/walk/open_default.rs` already drives Ready on a fake box through a browser-bounded
  `NativeClient`, by a `Referee` over a `Page`. Its `open_and_reload` presses Create and expects "My
  project", so it doesn't fit a walk that adopts `work/hello`: the new walk has its own small referee.
- `Step::Visit` (`temper/crates/temper/tests/support/ctl.rs`) follows the previous call's printed
  address on the box referee's visitor; `account connect PROVIDER --key KEY` prints that address.
- Ready reads "Model none yet" with nothing under it in three ways, and they point in different
  directions: `last_end` is `None` and no own line is in `signing`; `last_end` is `Made` or `Renewed`
  but no line is usable (`temper-client-domain/src/forge/model.rs`, or
  `serves_work` in `jig-domain-core/src/documents/accounts.rs`); or `last_end` is another `Refused` or
  `Failed`, which `welcome_end` (`temper-client-view/src/pages/connect_model.rs`) returns from
  without drawing. Print which one the box is in before you theorise.
- Since `68cddad304` the account paths took `a887cc1390` and `5b8f4eec32` (the OAuth exchange),
  `24a0273b17` and `3f84a7ea5a` (the accounts host), among 327 commits.

1. **The failing test:** `temper/crates/temper/tests/fake_backends/walk/sign_in.rs`, in the
   `process` binary:

   ```rust
   //! The ChatGPT plan on the fake box: the fake sign-in leaves a usable model for Ready.
   use super::{Act, Call, Walk, usable};

   #[test]
   fn a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model_and_ready_can_create() {
       let walk = Walk::owned("walk-sign-in");
       let seen = walk.run_then(
           vec![
               Act::Now(Call::new(&["forge", "setup", "--key", "walk-here", "--json"])),
               Act::Now(Call::new(&["account", "connect", "1", "--key", "walk-plan", "--json"])),
               Act::Visit,
               Act::Now(Call::new(&["account", "list", "--until", "settled", "--json"])),
               Act::Now(Call::new(&["account", "list", "--json"])),
           ],
           ready_can_create,
       );
       assert!(usable(seen.answers.last().expect("accounts")), "the sign-in left a usable model");
   }

   /// Ready, by the browser's bounds: the plan's model line, and Create enabled. It presses nothing.
   fn ready_can_create(config: &Path, client: &Path) { /* a referee as open_default's */ }
   ```

   `Act::Visit` maps to `Step::Visit` and, like `Act::Mark`, counts as no call. `ready_can_create`
   opens `Address::Welcome { step: Step::Ready }` and waits for a paragraph named from
   "Model ChatGPT plan" and an enabled "Create your project"; `sign_in_welcome.rs` reads the same
   line. Run it:
   `heavy cargo nextest run -p temper --test process -E 'test(=fake_backends::walk::sign_in::a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model_and_ready_can_create)'`,
   and read why it fails.
2. **Regression or the fake's:** in your slot, on a throwaway detached checkout of `68cddad304` with
   the test carried over, run it once. If the harness has moved too far for that, read those commits'
   diffs instead, and say so. Return to your branch before going on.
3. **The fix,** where the fault lies, with its narrow test.
4. **Ready's words:** in `welcome_end`, let every `Refused` and `Failed` reach `sign_in_end`; only
   `Made` and `Renewed` not declined stay out, and the match lists every variant. Its tests go beside
   `ready_ending` in `temper-client-view/src/tests/welcome.rs`, one line per ending, with the words
   5.5 gives. `ready_ending` looks for `Role::Alert`, and `accounts::end` draws `Role::Status`: a
   sibling helper reads that role, and each ending keeps the role it has on the Model step.
5. **Status:** remove the 3.5 entry from `README.status.md`. This edit is a stated departure from
   "Not yours" in your text.
6. **Check:** as your text says, for every package you changed and its dependents, plus
   `python3 -I -B tools/citations/citations.py --base origin/main` from the root. Commit and push.

## Report

As your text says, with: regression or the fake's, and how you know; which of the three states the
box was in; the rule each new test pins and the break that bit it; the walk's CPU time in
`process-temper`; the builder's `hostname`.

## Answers

2026-11-04. Question: land the verified Ready-ending fix and fake-box coverage, or hold them until
the original sign-in failure reproduces?

**Land them.** The branch's two commits, `dda8950279` and `d3ab955d88`, are the task's work, with no
sign-in change. This replaces the first two acceptance items, and the rest stand:

- The first item is met by the two fake-box walks, which pass and bite under the scope mutation. No
  test failed on the unmodified base, because the walk's stop did not reproduce on a fresh fake box
  through a native client: that finding is the result, and nobody is to invent a fault to fit it.
- The second item is met by `welcome_end`, the one fault shown, with its test in `temper-client-view`.
- The report's first section stands as written: not reproduced, breaking commit and original ending
  unknown, the fake unchanged since `68cddad304` for this sign-in.

The next run changes no code. It checks that `fake-signin-model/signin-model` still holds both commits,
and reports `Done:`, carrying this report's findings and its `## Left`. If its `hostname` is
`debian-16gb-hel1-6` and the walk's logs are there, it reads `05-ready.json`, `06-ready-settled.json`
and `box.stderr` and adds what they show of `last_end`, `signing` and the sign-in's end. Otherwise it
reaches no other machine. A fault those logs show is reported, not fixed, in this task.

The two process failures under `## Left` are not this task's and hold nothing here.

The walk that stopped drove Chromium with the recording bundle, and these walks do not.
`record-landing` walks that path again, so it is where the fault shows if it remains.
