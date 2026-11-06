+++
id = "three-lines"
part = "words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tests/client-browser/src/lib.rs",
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
]
effort = "medium"
minutes = 30
+++

## Goal

Three lines of words change. A used sign-in link stops naming `temper serve` to someone who ran `temper box`. The configuring chat stops writing a repository's address and port into instructions and reasons. It also stops telling the person "Proposal t1/4 awaits an owner".

That last sentence comes from the model paraphrasing the stock, not from a tool. `temper-engine-smith/src/answers.rs`, `proposed`, answers `proposed as t1/4 to the deployment's owners`, and `configuring.txt` says "Give its number; say it awaits an owner." So the stock changes and the tool's answer stays: the key is `temper ctl`'s (`temper ctl 1 proposal decide t1/1`), and `a_settings_proposal_answer_prints_its_ctl_name` and the engine worlds read it.

Acceptance:

- The browser's signed-out paragraph reads exactly `Signed out. To sign in, open a sign-in link temper printed.`
- `configuring.txt` holds the three edits below, word for word, and every test that reads it passes.
- A test in `temper-service/src/tests/stock.rs` is red without each stock edit.
- The three **Not yet** entries of Steps, 4, are gone from their status files.

## Inputs

- `temper/docs/plan/ui-qa-small/notes/common.md`: read it first; it departs from the implementer's text.
- `temper/docs/design/client/README.md`, 5.1, *signed out*, and `temper/docs/design/client/browser.md`, 2.4: the paragraph's words, already changed.
- `temper/docs/design/domain/agent.md`, 8.1, *Its instructions*: the stock's new sentences, already written.

## Owns and may change

Owned, for these edits alone:

- `temper/crates/temper-client-view/src/words.rs`: `Word::SignedOutMain`'s browser arm only.
- `temper/crates/temper-client-view/src/tests/frame.rs`: `signed_out_the_browser_main_says_how_to_sign_in`.
- `temper/tools/gallery/src/tests.rs`: the `signed-out` entry's name.
- `temper/tests/client-browser/src/lib.rs`: `States::signed_out`'s text. Compiled, not run.
- `temper/crates/temper-service/src/stock/configuring.txt`.
- `temper/crates/temper-service/src/tests/stock.rs`, `temper/crates/temper-service/src/tests/stock_brief.rs`.

Also: the status lines of Steps, 4.

Options, each pre-approved:

- If `words.rs` holds a counted bound that the shorter word trips, regenerate it as its test says.
- If `stock_brief.rs`'s table moves, including `largest` or its `room` if charter 3 becomes the largest, take the new figures from the failure. They follow from the added bytes. If a figure moves by far more than those bytes, stop and read why before changing it.
- The terminal and `temper open` keep their own words. `temper serve printed` in `temper-client-shell-terminal`, `temper/src/open.rs`, `temper/src/commands.rs` and their tests is out of scope: the request names one word. Don't change them.
- Don't change `temper-engine-smith/src/answers.rs`.
- Don't reword any other sentence of `configuring.txt`.

Overlap: `model-catalogue-1/configuring-told` rewrites paragraphs of `configuring.txt`, and already conflicts with main there, with or without this task. If a catch-up meets it, take their paragraphs and make the three edits below in them, whatever their wording has become.

## Steps

1. **The sign-in word.** Change the browser arm in `words.rs` and its three readers above:

   ```
   Signed out. To sign in, open a sign-in link temper printed.
   ```

2. **The stock.** In `configuring.txt`, exactly these three edits:

   1. In "Say nothing has changed…", replace `only when you first give a proposal's number,` with `only when you first tell them of a proposal,`.
   2. In "Propose once…", replace `Give its number; say it awaits an owner.` with:

      ```
      Tell the person it waits in this chat for their Accept, or an owner's if they are not one, without its key (t1/4 in the tool's answer).
      ```

      "Or an owner's" keeps it true for a member, whose chat proposes as an owner's does and whose proposal waits for an owner (`agent.md`, 8.1).
   3. Insert a paragraph of its own directly before `Read project-ways and project-questions,`, followed by a blank line, so that it comes right after the paragraph ending "Put rules typed here in charters' instructions, in the person's words.":

      ```
      In instructions and reasons, name a repository by its name, never by an address or port.
      ```

      "Copy to charter documents' forge/repository unchanged, ports too" stays as it is: those two fields hold the address, and prose names the repository.

3. **Tests** in `temper-service/src/tests/stock.rs`, beside the other `configuring_txt_*`:

   - In `configuring_txt_says_nothing_has_changed_in_one_place`, assert the new phrase in place of the old.
   - New `configuring_txt_says_a_proposal_waits_for_an_accept_without_its_key`: contains `waits in this chat for their Accept` and `without its key`; contains neither `Give its number` nor `awaits an owner`.
   - New `configuring_txt_names_a_repository_by_its_name`: contains `name a repository by its name, never by an address or port`, and still contains `ports too`.
   - `the_configuring_agent_is_charter_3_and_template_3` counts the stock's bytes (`5642`): the new count.

   Break each rule once by hand and see its test fail. Fix what the byte count moves in `stock_brief.rs`.

4. **Status.** Delete these **Not yet** entries:

   - `temper/docs/design/client/README.status.md`: 5.1, *signed out*.
   - `temper/docs/design/client/browser.status.md`: 2.4, the signed-out paragraph.
   - `temper/docs/design/domain/agent.status.md`: 8.1, "`configuring.txt` still has it give a proposal's number…".

5. **Checks**, through `heavy`, one at a time:

   - `heavy cargo nextest run -p temper-client-view --lib -E 'test(signed_out)'`
   - `heavy cargo nextest run -p temper-client-gallery`
   - `heavy cargo check -p temper-client-browser-world --tests`
   - `heavy cargo nextest run -p temper-service --lib -E 'test(stock)'`
   - `heavy cargo nextest run -p temper-engine-domain-world -E 'test(configuring)'`, whose worlds include the stock's bytes.

   Then what the implementer's text asks at the end: `cargo fmt`, `old_forms.py`, clippy on the changed packages, their focused tests. Run `python3 -I -B tools/citations/citations.py --base origin/main` too.

6. Commit and push.

## Report

The usual report, and:

- Say that item 4's sentence was the model's paraphrase of the stock, not a tool's answer, with the two lines that show it.
- Name any baseline you regenerated, with its old and new figure.
- List under `## Left` the `temper serve printed` lines left in the terminal and in `temper open`, as outside the request.
