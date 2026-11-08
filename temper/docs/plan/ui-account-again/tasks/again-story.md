+++
id = "again-story"
part = "story"
kind = "feature"
needs = ["redeclare", "ctl-words"]
files = ["temper/tests/client/tests/suite/account_tools.rs", "temper/tests/client/src/front/account_acts.rs"]
effort = "high"
minutes = 50
+++

# A story: forget a program account, declare it again under its name

## Goal

The meeting point of this plan: one thin story through the real engine,
the real accounts child and `temper ctl`, on the client world's fake
worker. A person declares a program login, forgets its account, and
declares the same name again. That declaration succeeds, its host is
checked, and a further declaration of the now-live name is refused with
ctl's *name taken* words.

Acceptance:

- One new test in `temper/tests/client/tests/suite/account_tools.rs`,
  beside `a_tools_declaration_is_visible_with_its_host_in_accounts` and on
  its fixture (`accounts::tools_fixture(fixtures::SEED)`):
  `a_forgotten_declaration_is_declared_again_under_its_name`. In order:
  1. `account declare 17 claude_code claude-work --key declare-login`:
     status 0.
  2. `account list --json`: read ACCOUNT, the account holding the
     `claude-work` source.
  3. `account forget ACCOUNT`: status 0, once the fake worker has
     answered the program's sign-out (below). The text says `Signed out on
     every machine.`
  4. `account declare 17 claude_code claude-work --key declare-again`:
     status 0, and stdout has
     `temper program-login --config CONFIG --data DATA --cache CACHE claude-work`.
  5. The engine asked the worker to check `claude-work`'s login after
     step 4 (the revival's `CheckProgram`, `notes/contracts.md`, 2), seen
     on the fake worker's link as the front exposes it.
  6. `account list --json`: a program source named `claude-work` again,
     with `"program_hosts":[{"name":"worker"`, as the sibling story
     asserts.
  7. `account declare 17 claude_code claude-work --key declare-taken`:
     status 2, and stderr has `account declare: program name claude-work
     is taken` and, on the next line, ctl's NEXT for it, as
     `notes/contracts.md`, 5, gives it.
- The story passes in the focused suite and is cheap: one fixture, no
  sleep, no extra fake time beyond what the forget's answer needs. The
  report gives its wall time from nextest's line.

## Inputs

- `temper/docs/plan/ui-account-again/notes/common.md` and
  `notes/contracts.md`, sections 2, 3 and 5.
- `jig/docs/design/domain/accounts.md`, 3.4 (*A program source's name*,
  *Signing out or removing a program's source*).
- `temper/docs/design/client/tools.md`, 5.5 (the `account declare` and
  `account forget` rows; *A refused account act says why and what next*).
- `temper/docs/design/client/tools.md`, 11.2: `tests/client`, the world
  this story lives in.

Departures from the implementer's text, and why:

- **Your needs are `redeclare` and `ctl-words`:** the revival is the
  first's, the *name taken* words the second's. If either is missing from
  your base, stop: `Question:` naming which.

## Owns and may change

Owns:

- `temper/tests/client/tests/suite/account_tools.rs`: the new test.
- `temper/tests/client/src/front/account_acts.rs`: a helper the story
  needs, added beside `program_checked` (say, `program_signed_out`, which
  sends the fake worker's sign-out answer for a name, as
  `program_checked` sends its `Logins`), never changing what an existing
  helper does.

Reader and neighbour files: none. Another story's fixture is not yours.

Pre-approved options:

- **The forget's answer:** if the fake worker answers a program's
  sign-out by itself, use that. If it does not, drive its answer through
  the engine's link, as `program_checked` does, with the helper above.
  If the answer needs the engine's configured worker named, take the name
  the sibling story reads (`worker`).
- **Step 5's evidence:** if the front records what the engine sends the
  fake worker, assert the check of `claude-work` there. If it records
  nothing the story can read, assert instead that `account list --json`
  after step 4 shows the source's host observation reset (no stale
  `usable` from before the forget) and say so in the report; do not add
  recording to the front for it.
- **Step 3's words:** if the fixture's worker set makes the forget
  unconfirmed (a worker not in contact), assert the hosts it names
  instead of `Signed out on every machine.`, and say so.

## Steps

1. Read the sibling story and `front/account_acts.rs` (`tools_fixture`,
   `program_checked`, `settle`), and how the engine routes a program's
   sign-out to a worker and its answer back
   (`jig/crates/jig-domain-fleet/src/programs.rs`, `sign_out`).
2. Write the story. Run it alone:
   `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/a_forgotten_declaration_is_declared_again_under_its_name/)'`
   (check the count).
3. Break it once by hand: on a scratch copy of the base, before
   `redeclare`'s change (or by restoring `DeclarationRefusal::Taken` for
   the declaring holder in `jig-domain-accounts`' `declare`, then
   reverting), see step 4 fail with exit 2. Restore.
4. Once, at the end: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-world`.
5. Commit (`ui-account-again: a forgotten program account declared again,
   end to end`), and push.

## Report

As the implementer's text says, with the story's wall time, which option
you took for each of the three above, and the rule broken by hand and
what failed.
