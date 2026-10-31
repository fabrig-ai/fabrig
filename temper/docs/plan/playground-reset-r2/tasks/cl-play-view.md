+++
id = "cl-play-view"
part = "client"
kind = "feature"
needs = ["cl-play-domain", "cl-connect"]
files = [
  "temper/crates/temper-client-view/src/{frame,root,choice,pages,playground,playground_words}.rs",
  "temper/crates/temper-client-view/src/pages/{accounts,connect_model,chats,chat,task,change,inbox,agent}.rs",
  "temper/crates/temper-client-view/src/pages/accounts/**",
  "temper/crates/temper-client-view/src/accounts/**",
  "temper/crates/temper-client-view/src/accounts.rs",
  "temper/crates/temper-client-view/src/tests/{demo,demo_start,frame,choice,accounts,connect_model,playground}.rs",
  "temper/docs/design/client/journeys/welcome-playground/**",
]
effort = "high"
minutes = 85
+++

## Goal

The playground's pages (draft 6.1, 6.2): while it runs, every page's
banner is one line, "This is the playground: a scripted model and
in-memory forge. Use the wizard to [set up temper]", its link sending
`EndPlayground`; Accounts lists the Scripted model and offers nothing to
add, nor does Connect a model; while it leaves, every page shows
"Leaving the playground…" in place of its main. What goes from the view
(6.3): the setup strip, the demo banner's variants and Connect links,
the halves' readers, the retired playground's readers, the scripted
account's "until a model of yours connects" and "stepped aside" words.

Acceptance (`temper-client-view`, `src/tests/`; new file
`playground.rs` for the new cases):

- `the_playground_banner_is_one_line_on_every_page` (each `Address`
  kind, the welcome included, while `Running`; none without a
  playground).
- `set_up_temper_sends_the_end` (pressing the link gives
  `Action::Forge(forge::Act::EndPlayground)`).
- `leaving_replaces_every_page_with_its_line` (no banner, no chatbar, a
  `Main` with "Leaving the playground…").
- `accounts_offer_nothing_to_add_while_the_playground_runs` and
  `connect_model_offers_nothing_to_add_while_the_playground_runs`.
- `the_scripted_account_says_it_serves_the_playground_alone`.
- `no_setup_strip_is_drawn`.
- `demo.rs`, `demo_start.rs`, `frame.rs`, `choice.rs`, `accounts.rs`,
  `connect_model.rs`: tests of the halves, the three banner variants,
  the setup strip, the retirement and the stepped-aside line are deleted
  (listed in the report); the start's tests (chats' start, its failures
  and retry) pass as today.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/tasks/cl-play-domain.md` (the
  `playground()` read and the shims it kept) and `cl-surface.md`, steps 4,
  8, 11.
- `temper/docs/design/draft/playground-reset.md`, 6.1, 6.2, 6.3.
- `temper/docs/design/client/README.md`, 4.8 and 5.1 (the banners: what
  goes).

## Owns and may change

Owns the files in the header. May add `mod playground; mod
playground_words;` in `temper/crates/temper-client-view/src/lib.rs` (the
lines only). Not yours: `V/forge/**` (`cl-connect-view`),
`V/pages/{welcome,project}.rs`, the domain. Stop reading the domain's
shims (`Domain::demo()`, `forge::demo::{Halves, halves, current}`,
`retired_demo`, `listed::setup`) in your files; do not delete them
(`cl-tidy` does, once `cl-welcome` has stopped reading them too).

## Steps

V = `temper/crates/temper-client-view/src`.

1. **The banner (new V/playground.rs, V/playground_words.rs).**
   `pub(crate) fn banner(b, root, words, bindings)`: while
   `root.playground()` is `Some(Running(_))`, a region (`look::ACCOUNTS`,
   as the demo banner's) holding one paragraph: "This is the playground:
   a scripted model and in-memory forge. Use the wizard to " then a
   link "set up temper" bound to `Meaning::EndPlayground` (the forge
   bindings, as `V/frame.rs` binds forge acts). Words in
   `playground_words.rs` after `V/accounts/words.rs`'s pattern.
2. **Leaving:** `pub(crate) fn leaving(b, root, words)`: while
   `Some(Leaving)`, `V/root.rs` draws, after the frame's header and in
   place of any page's main and chatbar, a `Main` with the paragraph
   "Leaving the playground…", and no banner.
3. **V/root.rs and V/frame.rs:** the banner is drawn on every address
   while the playground runs (the welcome's bypass and `demo_banner`'s
   exclusions go); `demo_banner`, the setup strip (`setup_underway`'s
   use there) and the three variants go; `choice::banner` (the
   no-account banner) is not drawn while the playground runs. The
   frame's navigation stops excluding a retired project.
4. **Accounts and Connect a model (V/pages/accounts.rs,
   accounts/rows.rs, connect_model.rs, V/accounts/words.rs):** while the
   playground runs: the Scripted model's row, "Scripted model · the
   playground's, for its project alone · held by the deployment"; no
   add, connect or key controls; Connect a model draws "Nothing to add
   while the playground runs." and the banner's link again. The
   "stepped aside" line and its `Word` go.
5. **The other readers** (V/choice.rs, V/pages.rs, V/pages/{chats,chat,
   task,change,inbox}.rs): read `root.playground()` (running: its
   project, provider and forge, as `Demo` holds them) for what they read
   from `root.demo()`; drop the retired project's label and the
   model-half usability (the scripted model is the playground's project's
   alone: draft 6.1).
6. **Checks:** the tests above; `cargo fmt`; `heavy cargo clippy -p
   temper-client-view -p temper-client-gallery --all-targets -- -D
   warnings`; one `heavy cargo nextest run -p temper-client-view -p
   temper-client-shell-web -p temper-client-shell-terminal -p
   temper-client-gallery -p temper-client-world -p
   temper-client-journeys`.
7. **Journeys:** `welcome-playground`'s chats page now shows the one-line
   banner: reprint (`heavy cargo run -p temper-client-journeys`) and
   commit; change a README sentence only if it is now wrong.
8. Old forms from `temper/`; commit and push.

## Report

- Each test deleted, by file and name.
- Which shim readers remain outside your files (for `cl-tidy`).
- What the code taught for `client/README.md` 4.8 and 5.1.
