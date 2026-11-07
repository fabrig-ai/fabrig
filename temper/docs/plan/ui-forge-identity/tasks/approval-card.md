+++
id = "approval-card"
part = "approval"
kind = "feature"
needs = ["approval-client"]
files = [
  "temper/crates/temper-client-view/src/pages/approval.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/pages/inbox.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/forge_sign_in.rs",
  "temper/crates/temper-client-view/src/forge/repository.rs",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/repository_link.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-view/src/tests/accounts.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/forge.status.md",
]
effort = "high"
minutes = 130
+++

# The approval's card under Needs you, on Home and a goal's page, and a box's proposal's sentence

## Goal

A change waiting on a review on the forge is a card under Needs you, drawn as every other card is, in the greenfield mock's `needs.html` (`temper/docs/design/client/forge.md`, 4.1): its kind "an approval", "Approve TITLE on Forgejo", the rule, how many wait behind it, why the viewer's review wouldn't count where it wouldn't, "Review on Forgejo ↗" to the pull request's files page where it would, "Open" and "Later". It draws on Home and as a goal's band too, and adds to the rail's and the banner's counts. And a proposal holding a landing rule with an enforcing approval on an owned Forgejo says, on its card, "On this machine temper's agents and CI jobs run as you, so this stops their mistakes, not one trying to get past it." (`temper/docs/design/client/README.md`, 5.13). `temper-client-domain` already merges the rows (`approval-client`); this task draws them.

Acceptance:

1. On Needs you (browser), a `Need::Approval` draws as `forge.md`, 4.1 says: its mono line, `h1.flow-q` question, `.ctx` sentences in their order and words, the primary review link in `.choices` exactly when `yours` is `Counts` and the address is safe, then the pair "Open" (to `Task { task }`) and "Later" (`Act::Needs(Act::Later { object })`). The flow's dots, total and `.next-up` count the merged needs (`Session::needs`); `.next-up` reads "Next: TITLE · SHORT" for one.
2. The address: on a Forgejo whose page facts offer sign-in, `ROOT/user/oauth2/temper?redirect_to=` and `/OWNER/NAME/pulls/N/files` encoded as a repository's link is (`forge.md`, 5.4); otherwise the row's `address`; an unsafe one is none.
3. Home's first card and its other lines, and a goal's band, draw an approval as `forge.md`, 4.1 and `README.md`, 3.5 and 5.8 say; Home's count and "Answer in a row" follow the merged needs.
4. The banner's navigation count is `me`'s `needs_total` plus every row of `approvals`.
5. A proposal's card, in a chat and on Needs you, draws the sentence after its items where `root.forge().owned_approval(number)` holds, and not otherwise.
6. The terminal draws none of it: `temper-client-shell-terminal`'s goldens are unchanged.
7. The gallery holds `needs-approval` and `needs-approval-no-account` (`names.txt` sorted, and `src/tests.rs`' `names_are_the_promised_ones` and `each_state_holds_what_its_name_says` holding them); the CSS rule sits under `/* ==== ui: needs ==== */`.
8. The **Not yet** entries *3.5, *An approval*...* and *5.13, *What a rule on a box holds against*...* are gone from `temper/docs/design/client/README.status.md`, and *4.1: no card under Needs you...* from `temper/docs/design/client/forge.status.md`.
9. The tests of Steps, 8 bite; `heavy cargo nextest run -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-gallery` passes; clippy clean for those.

This task departs from the implementer's text in one way: three status entries are yours to remove (acceptance 8), because the design already says what this task builds.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`: reach for the mock; `temper.css` under its marker; the gallery regenerated, never hand-merged.
- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The approval read*, *The client*: binds `Need`, `Session::needs`, `shown`, `count`, `Side::approvals`, `Side::owned_approval`.
- `temper/docs/design/client/forge.md`, 4.1, whole: the words, acts and address bind; 5.4, *A repository's link*, for the encoding.
- `temper/docs/design/client/README.md`, 3.5: *Needs you* (*The card*, *An approval*, *SHORT*, *Its acts*) and Home's *Needs you* region (*The first*, *The others*), *What the client keeps* (*The rail's count*); 5.8, *Needs you bands*; 5.13, *What a rule on a box holds against*.
- `temper/docs/design/client/greenfield/needs.html`, `app.js` (`askCard`, `needsPage`), `mock.css`.

## Owns and may change

**Owned:** the `files` above. `root.rs` only to pass `&view.forge_sign_in` to the pages that draw the card; `words.rs` only for the new words and its regenerated counts: `model-catalogue-3/models-read` writes both, and the gallery and `temper.css`.

**Readers and neighbours:**

- `inbox::main`, `home::main` and the goal page's `main` (the one `root.rs` calls for a goal's `Task`) gain `sign_in: &crate::forge_sign_in::Forges`, passed from `root.rs` as `settings::repositories` is; nothing else changes in their callers.
- The gallery: `src/needs.rs`' `domain` answers the forge family's `approvals` watch for the two new names, as `samples.rs`' `home_domain` answers `Watch::Trials` (`Down::Open { watch, subject: Watchable::Family { .. } }`, then `Up::FamilyDelivered`), with `wire::approvals`. `samples.rs` gains the two samples beside `needs-proposal`; `names.txt` keeps its order; `src/tests.rs`' needs expectations gain theirs.
- `temper-client-shell-web`'s tests read `temper.css`: run them.

**Options, each with its default:**

- Where the card's parts live: a new `pages/approval.rs` holding the words (the rule, behind, cut and why sentences, the question), `address`, and the parts drawn on Needs you, Home and a band, called from `needs.rs`, `home.rs` and `goal.rs`. Default that; `ask.rs`' `Asked` is the inbox card's and doesn't fit a row.
- The address helper: `forge_sign_in::Forge::open_path(&self, path: &[u8]) -> Option<Box<[u8]>>`, of which `open_repository` becomes a caller, the path's segments encoded by `forge/repository.rs`' `encoded` and the whole as the query value. Default that; if `open_repository`'s encoding can't be shared as is, a sibling `open_pull_files(owner, name, number)` and say so.
- The link: drawn as `forge/repository.rs`' `name` draws "OWNER/NAME ↗" in a new tab, `safe_url` first, styled as the primary choice (`look::` the recommended choice's style); its name "Review on Forgejo ↗".
- Role words: a small table in `approval.rs` for roles 0 to 3 ("an owner", "a maintainer", "a member", "an observer"; "owners", "maintainers", "members", "observers") and "someone with role R" / "people with role R" past them, each a `Word` with a number hole where it has one.
- The forge's word: "Forgejo" for `forges::Kind::Forgejo`, "GitHub" for `GitHub`, from the row's `kind`.
- The gallery's two samples: `World::Solo` at `Address::Inbox { filter: Filter::ALL }`; `needs-approval` a Forgejo row, `Counts`, `behind` 2, the project's rule of one owner; `needs-approval-no-account` a GitHub row, `NoAccount`, a deployment's rule of two maintainers with one counted. If a Home sample is one line more in `samples.rs`, `home-needs-approval` too.
- The CSS: one rule under `/* ==== ui: needs ==== */` making a link in `.temper-ask-choices` (Needs you, Home) and a band's primary link draw as the button it stands for: no underline, the button's padding, min-height and radius. Reuse `.temper-recommended`'s colours.

## Steps

1. `forge_sign_in.rs`, `forge/repository.rs`: the address helper; `approval.rs`'s `address(row: &approvals::Row, sign_in: &Forges) -> Option<Box<[u8]>>`.
2. `words.rs`: the new `Word`s (the kind "an approval", the question, the rule's forms, behind's two forms, the cut, the three whys, "Review on Forgejo ↗" and "on GitHub", the proposal's sentence), their ids after the largest, their English; then `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'` to regenerate `PHRASES`, `SEGMENTS` and `ENGLISH_BYTES` (the test's exact path is yours to find), never by hand.
3. `needs.rs` (`main`, `flow`, the card) and `inbox.rs` (the browser branch only): `session.needs(inbox, root.forge().approvals())` in place of `open`, a `Need::Card` drawn as today, a `Need::Approval` by `approval.rs`. The terminal branch of `inbox::main` stays as it is.
4. `home.rs` and `goal.rs`: the same for Home's region and a goal's bands.
5. `frame.rs`'s `navigation`: `me.needs_total` plus `root.forge().approvals().rows().len()`, saturating.
6. `cards/proposal.rs`: after the items' lines, where `root.forge().owned_approval(facts.key.number)`, the sentence in the items' look.
7. `temper.css`, the gallery (above), `names.txt` and its tests.
8. Tests, each a table or rows of one:

   ```rust
   // tests/needs.rs, beside proposal_parts_accept_reject_and_task_open:
   #[test]
   fn approval_parts_say_the_rule_and_offer_the_review_where_it_counts() {
       // (row: kind, scope, rule, role, required, counted, behind, whole, yours, address) ->
       //   (question, ctx sentences in order, primary's name and address or none, pair names)
       // Forgejo, project rule 1, role 1, required 1, behind 0, Counts      -> "Review on Forgejo ↗"
       // GitHub, deployment rule 2, role 0, required 2, counted 1, behind 3 -> "... 2 approvals by owners; 1 so far." "3 more changes wait behind this one."
       // read in part -> the cut sentence; NoAccount on Forgejo and on GitHub; TooFew; OtherRole -> no primary
       // Counts with an empty or unsafe address -> no primary
       // Later on it moves it last and keeps its count; Open goes to Task { task }.
   }
   ```

   `tests/repository_link.rs`' `an_address_for_each_kind_of_forge` gains the files page's rows (Forgejo with sign-in facts, the encoded `redirect_to`; without, the row's address; GitHub's), and `a_name_cannot_leave_its_segment_or_the_query` a dot segment's row for the files path. `tests/home.rs`: an approval as Home's first card and as one of its lines, rows of `first_need_keeps_card_order_navigation_and_terminal_words` and `additional_needs_have_five_rows_and_browser_answer_link`. `tests/goal.rs`: a row of `one_band_per_open_need_in_the_inboxs_order` with an approval band, its link and its "Open". `tests/frame.rs`' `navigation_shows_the_inbox_count_only_when_nonzero` and `tests/rail.rs`' `rail_and_tabbar_keep_the_order_addresses_current_states_and_count` gain an approval row. `tests/cards.rs`' `a_settings_proposal_draws_its_title_lines_and_items`: `owned_approval` true draws the sentence, false doesn't. Bites: drop the `Counts` check for the primary, see the NoAccount row fail; drop the encoding of `redirect_to`, see the sign-in row fail; drop `owned_approval`'s test, see the false row fail; restore.
9. The status entries (acceptance 8).
10. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-shell-web -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-gallery`; `heavy cargo check --tests -p temper-client-world`. If a journey recording goes stale with the merged needs, `temper-client-journeys again` (`notes/common.md`). Commit ("temper-client-view: an approval's card under Needs you, on Home and a goal's page, and a box's proposal's sentence"), push.

## Report

As the implementer's text says, with each rule broken for a bite and the failure seen, the options taken, the gallery's new names, and each edit in `root.rs`.
