+++
id = "ctl-words"
part = "ctl-words"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tools/src/jig/render/outcome.rs", "temper/crates/temper-client-tools/src/tests/jig/account_acts.rs", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 50
+++

# ctl says why a refused account act was refused, and what to do

## Goal

`temper ctl` prints "account declare: This act is not offered" and
nothing else when the engine refuses a declaration. For the three
account-act refusals that have one reason a person can act on, print
that reason and the next step, in text and in JSON.

Acceptance:

- One new table test in
  `temper/crates/temper-client-tools/src/tests/jig/account_acts.rs`,
  `account_refusals_say_why_and_what_next`, with one row per line of
  `notes/contracts.md`, section 5, each run in text and in `--json`. Each
  row submits the act through `f::account_root(false)`, delivers what the
  act needs to be sent (`me` with `f::provider_offers()` for a
  declaration, as `declare_validates_the_offer_and_sends_only_the_contract_fields`
  does; `f::accounts()` for `account retry 43`), answers it
  `root.answered(call, c::Answer::Refused { refusal, at: None })`, and
  asserts exit status 2 and the exact words:
  - text: `TOOL: WHY` and NEXT, each a whole line, as section 5 gives
    them, NAME `work-login` and P the declaration's provider (`9`, as
    `account declare 9 helper work-login` sends it);
  - JSON: valid with unique keys (`f::unique_keys`), and `"next":"NEXT"`.
- A fourth row, a refusal that keeps today's words: `account declare`
  refused `c::Refusal::Unknown` prints what it prints on the base (read
  it from a run on the base) and no `next`. It holds the new words to
  these three cases.
- NAME and P come from the request's ask, not from the live call alone,
  so a restored request prints the same.
- Every other test in the crate still passes; clippy and fmt clean.

## Inputs

- `temper/docs/plan/ui-account-again/notes/common.md` and
  `notes/contracts.md`, sections 4 and 5 (binding: the words are exact).
- `temper/docs/design/client/tools.md`, 5.5: the bullet *A refused
  account act says why and what next*.
- `jig/docs/design/domain/accounts.md`, section 5: the table's *declare a
  program's login* and *order, uses, resume* rows, which say what each
  refusal means for that act.
- `temper/docs/design/client/tools.status.md`: the **Not yet** entry
  starting `5.5: *A refused account act says why and what next*`.

Departures from the implementer's text, and why:

- **You delete one status entry,** the **Not yet** bullet above, once your
  acceptance holds. Touch no other design text. The cut wrote it for you
  to close.
- **The engine's side is another task's** (`redeclare`, in jig). Your
  rows script the engine's answers, and `c::Refusal::NameTaken` is
  already in the client contract, so you need none of its code.

## Owns and may change

Owns:

- `temper/crates/temper-client-tools/src/jig/render/outcome.rs`:
  `Operation` and `Operation::of` (what the ask carries for these words),
  `refused_text`, `refused_json`, and small helpers beside them.
- `temper/crates/temper-client-tools/src/tests/jig/account_acts.rs`: the
  new table test.
- The status entry above.

Reader and neighbour files:

- `temper/crates/temper-client-tools/src/tests/fixtures.rs`: a helper
  the rows need, added, never changing what existing helpers do.
- Any literal of `Operation` the compiler names (a new field): the field
  alone.

Pre-approved options:

- **Where the words live:** as `const` byte strings beside
  `refused_text`, or in a small function both `refused_text` and
  `refused_json` call, measured with `Text::measure` as `sign_out` is.
  Your choice; keep one copy of each line.
- **What `Operation` keeps:** the declaration's provider and name, as one
  `Option` field, read from `c::AccountChange::DeclareProgram` in
  `Operation::of`. If `Operation` already holds them under another name,
  use that.
- **If a help page under `temper/docs/help/` quotes "This act is not
  offered" for one of these acts,** change that line to the new words and
  say so. The cut's survey found none.

## Steps

1. Read `outcome.rs` (`Operation`, `Operation::of`, `print`,
   `refused_text`, `refused_json`, `encode`'s `next` keys) and the
   declaration, `retry` and key-refusal tests in `account_acts.rs`.
2. Write the table test. Run it red on the base:
   `heavy cargo nextest run -p temper-client-tools --lib -E 'test(=tests::jig::account_acts::account_refusals_say_why_and_what_next)'`
   (check the count; use the path `rg` shows for the module).
3. Implement, sketched:

   ```rust
   // outcome.rs
   fn account_refusal(operation: &Operation, refusal: &c::Refusal, out: &mut Text) -> bool
   // writes "WHY\nNEXT" after "TOOL: " for the three cases and returns
   // true; false leaves refused_text's generic path.
   fn account_next(operation: &Operation, refusal: &c::Refusal) -> Option<...>
   // the NEXT line, for refused_json's "next"; the same source of words.
   ```

   `refused_text` calls it before `acts::refusal`, after the
   proposal/restore/close/cancel/fund cases. `operation.word` is
   `b"declare"` for `DeclareProgram` and `b"retry"` for `Resume`
   (`Operation::of`).
4. Break each row's rule once by hand (swap the refusal a case matches,
   drop the JSON `next`), see the row fail for that reason, restore.
5. Once, at the end: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-tools`; then the direct
   dependents' focused tests: `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/account_tools/)'`
   and `heavy cargo check -p temper --tests`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit (`ui-account-again: ctl says why an account act was refused`,
   the body naming the three cases and why), and push.

## Report

As the implementer's text says, with the exact text and JSON each row
printed, and each rule broken by hand and what failed.
