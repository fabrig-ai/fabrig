+++
id = "link-why"
part = "link-why"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/**",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-protocol-engine-clients/src/component.rs",
  "jig/crates/jig-protocol-engine-clients/src/mint.rs",
  "jig/crates/jig-protocol-engine-clients/src/sign_in.rs",
  "jig/crates/jig-protocol-engine-clients/src/routes.rs",
  "jig/crates/jig-client-protocol-link/src/boundary.rs",
  "jig/crates/jig-client-protocol-link/src/sign_in.rs",
  "jig/crates/jig-client-protocol-link/src/tests/sign_in.rs",
  "jig/crates/jig-client/src/client.rs",
  "jig/crates/jig-client/src/tests/sign_in.rs",
  "jig/crates/jig-client-shell-web/src/sign_in.rs",
  "jig/crates/jig-client-shell-web/src/tests.rs",
  "jig/tests/engine-clients/tests/suite/sign_in.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
]
effort = "high"
minutes = 100
+++

# A sign-in link that can't sign you in says why

## Goal

Opening a link that can't sign a person in shows one page, whatever the cause. Have the engine answer why, carry it through the wire and the client's link view, and have the page say it. A made-up link goes the same way.

The words, whole, compared whole in tests. The first and the last are the user's, in the user's words:

| Why | The paragraph |
|---|---|
| `LinkUsed` | `This link was already used. A fresh one is in your terminal.` |
| `LinkExpired` | `This link has expired. A fresh one is in your terminal.` |
| `LinkUnknown` | `This link isn't from this temper. Use the one its terminal printed.` |

Acceptance, each held by a test in the package named:

- `jig-engine-clients-world`: a spent secret is answered `link_used`, and past its expiry `link_expired`, each drawing a fresh link when none is live. A secret this component never drew, of 32 bytes or empty, is answered `link_unknown`. The 9th link back reads as unknown.
- `jig-engine-clients-world`: a sign-in crossing a reserved link with another secret draws no fresh link, as it does now, and is answered by the same rule. A core's refusal of the link's own sign-in still answers `refused`, and that secret is `link_used` afterwards. With the printed link not enabled, the answer stays `refused`.
- `jig-client`: each of the three reaches `not_signed_in` by its own name, is told, is not sent again, and clears on the next `SignIn`.
- `jig-client-shell-web`: at the link's route, a `secret` that doesn't decode to 32 bytes is taken with an empty secret and the home address; no `secret` at all, or another path, takes nothing.
- `temper-client-view`: each word shows, in either shell, with or without a way of signing in, through `assert_link_main`. A page with no link keeps `Signed out. To sign in, open a sign-in link temper printed.`
- The answers say nothing of a live link, and compare by digest in constant time. Nothing logs a secret.

Not in this task: `temper open`'s and `temper ctl`'s words, which stay as they are for all three; a person whose cookie is live, who stays shown this paragraph until they reload, as they are shown today's words now.

## Inputs

- `jig/docs/design/protocol/client.md`, 8, *The printed link*, and within it *Why a link can't sign in*.
- `jig/docs/design/client/domain/client.md`, 12, `Link { secret, held }`.
- `jig/docs/design/protocol/browser.md`, 11, *The printed link, opened in a browser*.
- `temper/docs/design/client/README.md`, 5.1, *signed out, the link spent*.
- `temper/docs/plan/ui-small-1/notes/common.md`.

## Owns and may change

Owns the files in the header. A branch normally stays in its tree: this one crosses into temper because the arm the compiler forces in `temper-client-view/src/frame.rs` is the word, and the plan is held to three tasks.

The wire's change goes in place (`jig/docs/design/protocol/README.md`, 7), its version staying as it is.

**Readers, surveyed by `rg` across jig and temper.** Each may change as said, and only so:

- Add the three to the arm that holds `Refused`, with no other change:
  - `jig/testing/jig-client-kit/src/joint/component.rs`, near line 515
  - `jig/testing/jig-client-protocol-kit/src/native/wire.rs`, near line 180
  - `temper/tests/client/src/component.rs`, near line 622
  - `temper/crates/temper/tests/support/person.rs`, near line 580
  - `temper/crates/temper/tests/support/managed_client.rs`, near line 38
  - `temper/crates/temper-client-shell-terminal/src/terminal/native.rs`, near line 34, so that `Notice::SignInRefused` still says what it says
  - `temper/crates/temper-client-shell-terminal/src/client.rs`, near line 362
- `temper/crates/temper-client-shell-terminal/src/client.rs`, near line 303, decodes the answer itself: each new answer to its `Signing`, never to `Lost`, which is sent again.
- `temper/crates/temper-client-shell-cli/src/host.rs`, near line 328: the three take `None`'s arm, `the sign-in link was refused`.
- `jig/testing/jig-engine-clients-kit/src/peer.rs`, near line 845: a `panic!` for each.
- `temper/crates/temper-client-view/src/sign_in.rs`, near line 40, doesn't change. The paragraph takes the sign-in page's place as `Full`'s does.
- The browser tier doesn't change: `temper/tests/client-browser/src/lib.rs`'s `states().signed_out` is seen only after Sign out, with no link (`browser_sign_in.rs`, `browser_public.rs`), and `history.rs`'s `ctl_face` opens a spent link for what it prints, expecting no words.

**Tests that read `Refused` for a link, whose answer changes:**

- `jig/tests/engine-clients/tests/suite/sign_in.rs`: `a_given_secret_spent_draws_a_fresh_link` and near line 510 and 530. Near line 547, the core's refusal, stays. `a_given_secret_without_the_printed_link_is_no_link` stays. Read the one near line 58 before you choose its answer.
- `temper/crates/temper-service/src/tests/clients.rs`, near line 71, becomes `LinkUsed`.
- `jig/crates/jig-protocol-engine-clients/src/tests/services.rs`, near line 85, a service's, stays.
- `jig/crates/jig-client-shell-web/src/tests.rs`: the rows of `OTHER` with a bad or empty `secret` leave it for the new rows.
- `temper/crates/temper-client-shell-cli/src/tests/fixtures.rs` and `temper/crates/temper-client-shell-terminal/src/client/tests.rs` keep `Refused`, which still exists, unless a test there is about a link's secret.

**This changes what a shared answer means, so its readers' tests run:** the packages in step 8, not only the ones you edited.

Departs from the implementer's text: four status files are open, each to drop the **Not yet** entry this closes:

- `jig/docs/design/protocol/client.status.md`, 8, *Why a link can't sign in*.
- `jig/docs/design/client/domain/client.status.md`, 12, `not_signed_in`.
- `jig/docs/design/protocol/browser.status.md`, 11.
- `temper/docs/design/client/README.status.md`, 5.1, *signed out, the link spent*.

Options, each pre-approved:

- If the codec refuses an empty `secret 64`, the shell sends 32 zero bytes in its place, and the engine row sends those too. Say so in the report.
- If a `Word`'s number is taken, take the next free one. Keep the words beside `SignInNotReady` in each of `words.rs`'s lists.
- If an exhaustive match outside this list needs an arm, it takes `Refused`'s, and goes in the report.
- If a catch-up finds the schema or generated files changed by `model-catalogue-2` or `ui-proposal-2`, keep their schema lines and regenerate. Never merge generated files by hand.

## Steps

1. **The wire.** `jig/crates/jig-client-protocol-wire/schema/client-v19.schema`:

   ```
   enum SignInAnswer { signed_in: SignedIn, refused, not_ready, busy, full: SignInsFull, link_used, link_expired, link_unknown }
   ```

   Regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`.

2. **The engine.** In `jig/crates/jig-protocol-engine-clients/src/sign_in.rs`:

   ```rust
   /// How many past links a component keeps the digests of (protocol/client.md, 8).
   pub(crate) const PAST_LINKS: u32 = 8;
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub(crate) enum Past { Used, Expired }
   ```

   `Component` keeps a bounded list of `(digest, Past)`, its oldest dropped when full. A link goes in as `Used` wherever `spending` drops it without giving it back (`Event::SignInRefused`; `sign_in_reply` but for its `return_link` arms), and as `Expired` where `Route::Link` finds it expired. Closing and aborting record nothing. One function answers a secret not live:

   ```rust
   // `link_used`, `link_expired` or `link_unknown`; `refused` with the printed link not enabled.
   fn link_answer(&self, secret: &[u8]) -> Box<[u8]>
   ```

   Both `routes::refused()` calls in `Route::Link`'s arm take it. Record the expired link before you answer, so that a link's own expired secret reads as `link_expired`. `fresh_link` and when it draws don't change. `routes.rs` gains the three encoders by `sign_in_busy`'s form.

3. **The link.** `jig/crates/jig-client-protocol-link/src/boundary.rs`, `Signing` gains `LinkUsed`, `LinkExpired` and `LinkUnknown`; `sign_in.rs` decodes each, with `reader.remaining() == 0` as its neighbours do.

4. **The client.** `jig/crates/jig-client/src/client.rs`, `NotSignedIn` gains the same three names. `fn signing`'s first match gives each `Some(NotSignedIn::...)`, and its second puts them in the final arm with `Refused`, so nothing is sent again.

5. **The web shell.** `jig/crates/jig-client-shell-web/src/sign_in.rs::take`: past the path check, a `secret=` value that doesn't decode to 32 bytes gives `Taken { secret: empty, address: home }`. `base64::decode`'s bound stays. No `secret=` stays `None`.

6. **The words.** `temper/crates/temper-client-view/src/words.rs`: `Word::SignInLinkUsed`, `Word::SignInLinkExpired` and `Word::SignInLinkUnknown`, beside `SignInNotReady` in each list. `frame.rs`'s `unavailable_main` gives each its word.

7. **The tests,** as rows where a table holds them:
   - `jig/crates/jig-client/src/tests/sign_in.rs`, `each_link_retry_answer_is_visible_and_told_until_the_next_sign_in`: three rows, with backoff `None`.
   - `jig/crates/jig-client-protocol-link/src/tests/sign_in.rs`, near line 161: three rows.
   - `temper/crates/temper-client-view/src/tests/frame.rs`: turn `a_busy_link_says_to_open_it_again_in_either_shell` and `a_not_ready_link_says_starting_or_stopping_in_either_shell` into one table test holding those two and the three new rows.
   - `jig/crates/jig-client-shell-web/src/tests.rs`: `OTHER` keeps the rows that take nothing, and one new table holds those that take an empty secret.
   - `jig/tests/engine-clients/tests/suite/sign_in.rs`: one new table test, `a_link_that_cannot_sign_in_answers_why`, for what the changed stories don't already hold.

8. Narrow tests as you go, by name. Then the closing checks of your text, across both trees, with the readers' tests: `heavy cargo nextest run -p jig-client-protocol-wire -p jig-client-wire-world -p jig-protocol-engine-clients -p jig-protocol-engine-clients-http -p jig-engine-clients-world -p jig-client-protocol-link -p jig-client -p jig-client-shell-web -p jig-test-service`, then `heavy cargo nextest run -p temper-client-view -p temper-client-domain -p temper-client-shell-terminal -p temper-client-shell-cli -p temper-client-shell-web -p temper-service -p temper-client-world -p temper-one-process-world`. The process tier's world compiles and doesn't run: `heavy cargo check -p temper --tests`.

9. `cargo fmt`, commit and push.

## Report

The implementer's report, with which rule you broke by hand for each new row, what the row near line 58 of `sign_in.rs` became, and any option you took.

## Rebase (2026-11-07, step merge, foreman ui-small-1/link-why/foreman-1)

resolve both merge conflicts by preserving small-words changes and link-why additions
Rebase onto the current landing trunk. The conflicting commit is `738ed091e3` (“Clarify stock guidance and remove repeated or blaming messages”), merged into `origin/ui-small-1/integration` by `5b682d72a5`.
- In `temper/crates/temper-client-view/src/words.rs`, retain the shortened `HomeNeedsEmpty` paragraph from small-words and all three `SignInLink*` variants, IDs, list entries and paragraphs from link-why.
- Regenerate the combined phrase declarations; neither side’s counts describe the combined vocabulary. Run `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'`, then verify the same test without write mode.
- In `temper/docs/design/client/README.status.md`, retain both deletions: remove the completed 5.1 spent-link **Not yet** entry and the completed 3.5 Home / 5.10 semantic-conflict **Not yet** entry. Preserve the surrounding **Design departs** entries.
- Run `each_link_reason_has_its_paragraph_in_either_shell` in `temper-client-view` through heavy to verify the retained link paragraphs.

## Rebase (2026-11-07, step merge, foreman ui-small-1/link-why/foreman-2)

rebase words.rs preserving both vocabularies and regenerate combined declarations
Rebase onto the current landing trunk, `origin/ui-small-1/integration`.
- The merge conflicts in `temper/crates/temper-client-view/src/words.rs`. Main’s `183c94f536` (“Show reviewed proposal settings and keep phone controls clear”), incorporated through `c1d54122b3`, adds proposal words beside this branch’s link words.
- Preserve `WorkProposedChange`, `WorkProposedChangeCount` and `WorkProposingChange`, including their IDs, list entries and English phrases.
- Preserve all three `SignInLink*` variants, IDs, list entries and exact paragraphs. Keep the shortened `HomeNeedsEmpty` paragraph already retained.
- Regenerate the combined declarations rather than choosing either side’s counts: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'`.
- Verify that same test without write mode, then run `heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::frame::each_link_reason_has_its_paragraph_in_either_shell)'`.
- `temper/docs/design/client/README.status.md` merged automatically. Preserve incoming proposal status changes and this branch’s removal of the completed 5.1 spent-link **Not yet** entry.
