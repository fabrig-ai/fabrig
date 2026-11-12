# gallery: status

2026-11-12. Where the tree's code lags or departs from [gallery.md](gallery.md).

- **Not yet:** 2: `work` and `work-drafts` hold no landed draft
  [in flight: change 664].
- **Not yet:** 2: the Model step's Claude Code states,
  `welcome-model-claude-terminal`, `-signing-in`, `-signed-in`,
  `-no-terminal` and `-not-finished`, and `welcome-model-choose` with no
  terminal to lend [in flight: change 1946].


- **Departs:** 3: the gallery's tests exceed their 1 CPU s budget:
  `cargo test -p temper-client-gallery-tool --lib`, serial, measured
  2.644 CPU s. The overrun lies in `each_state_holds_what_its_name_says`
  and `print_writes_every_page_and_the_bundle`. A gallery cost repair owns
  finding and fixing the cause (`skein/docs/foundation/testing-strategy.md`,
  8.1, question 2); only if that fails do measurements go to the user.
  1's path printed for both widths adds to it: the print test alone took
  1.483 s of wall time before the second drawing and 4.069 s after
  (`look-round-24`'s `gallery-narrow`, at `436533537a`); no CPU figure is
  on record. How work runs' six states, `how` to `night-how`, and the
  fifteen of the rail, Work and Deployment, `rail-new-change` to
  `night-limits`, and the thirteen of Home and Needs you across projects,
  `all-home` to `night-needs`, and the two of Away on All projects,
  `all-home-away` and `all-home-away-on`, add to it, and so do the rows
  `ui-p5a-phone` added to `print_writes_every_page_and_the_bundle`, which
  read the `needs-list`, `ask-chat` and `rail-new-change` prints, with no
  CPU figure on record either; so do the five states `ui-reaches` added,
  `home-decided`, `needs-decided`, `needs-why`, `needs-stop-asking` and
  `needs-stop-asking-set`, with none on record, the four `ui-n3a-only`
  added, `needs-only-you`, `needs-only-you-done`, `night-needs-only-you`
  and `accounts-expiring`, with none on record either, and What reaches
  you's columns in `how` and `how-approve` and its two states,
  `how-reaches` and `night-how-reaches`, which `ui-p3b-reaches` printed in
  place of `settings-away` and `settings-away-you-approve`, with none on
  record, and Team's role's card's two, `team-role` and
  `night-team-role`, which goal 713 added, with none on record.

Section 2 holds what reaches a person, by plan `ui-reaches`: those five
states and
`start-goal-draft`'s "When it needs you", each with its row of
`each_state_holds_what_its_name_says`; that row checks
`start-goal-draft`'s "As the project says" is offered, not that it is
chosen.

- **Not yet:** 2: `accounts-try-stopped` and the trial's own `welcome-ready-*`
  samples (starting, answering, answered, stopped-starting, stopped-answering,
  waiting, waited, plan and key) are not in the gallery. Its Ready samples
  cover the host's check alone; the view's tests draw the trial states.
  The model-check plan's close added the catalogue rows in section 2,
  but not their sample fixtures or names. `home-try-stopped` is present.
- **Departs:** 2: the gallery's comparison pages do not hold one copy of
  the greenfield scenario. `home-morning` is Hello with no goals or Away
  switch, `home-with-goals` has the Solo world's goals, and `home-away`
  has four decisions still open to Overturn, where the mock's first is
  already landed. `needs-one` has no goal templates; `goals-reference`
  adds the large goal; chats and settings use the Reference and Solo
  worlds' parties, accounts and templates. Their counts, words, available
  tabs and resulting page lengths differ because the gallery draws those
  domains (`crates/temper-client-gallery-tool/src/samples.rs`), not the mock's stored state.
- **Not yet:** 2: `how-draft`'s hand-made `roles` answer has no
  drafter, and each role's `who` says when it runs alone, where
  `whole-story/how.html` says what it does too.
- **Not yet:** 2: `sheet-from-home` and `sheet-from-chats` are not in
  the gallery (`crates/temper-client-gallery-tool/src/samples.rs`).
- **Not yet:** 2: `goal-sending` has one pending message and "Being sent",
  but no earlier accepted message or next words in the composer.
  `goal-toast` answers Pause without delivering a paused task document, so
  its toast says "Paused" while the header and acts still read running and
  Pause (`crates/temper-client-gallery-tool/src/samples.rs`, `goal_domain`).
- **Not yet:** 1: a `home-*` state's title is its headline, then
  "temper", where the shell's is "Home · temper" (`../view.md`, 1):
  `crates/temper-client-gallery-tool/src/lib.rs`, `heading`, reads the level-1 heading, not
  the view's title.
- **Departs:**  the elsewhere-forge address departure remains `README.status.md`, 2.3. 2: `google-sign-in`,
  `forge-details`, `forge-setting-up-middle` and `welcome-setup-failed` are
  states the gallery holds (`crates/temper-client-gallery-tool/names.txt`,
  `crates/temper-client-gallery-tool/src/samples.rs`) that the table does not list.

`README.md`, sections 2.3 and 2.4 and `look.md` have `/terminal`, its parsed and canonical
`code` query, transport sign-in by cookie, the signed-out state with
Google's way back where it is given, and the approval's code, asked
time and field under the outside rules. `gallery.md`, section 2's twelve
`approve-*` gallery states are present, with domain, view and shell
coverage. `README.md`, section 3 keeps the rendered browser check and approval at a
public name for later; no real-browser result is supplied here.

`look.md`'s *Outside a project* has no known implementation gap: the
top bar and the column on the eight tokens and the overlay's rules under
`temper.css`'s outside marker, the placeholder's star in the signal with
no dot, and the old look's tokens, rules and `files/brand.svg` gone.
Focused tests hold every class `temper.css` selects to a `look.rs` name
and keep `#b8664e` out of the bundle's and the server's pages' sources.
`gallery.md`, section 2's `signed-out`, `out-of-date`, `no-project` and
`approve-signed-out-google` are present. The rendered comparison against
the request's pages is `README.md`, section 3's, after landing.

`look.md` and `public.md`, section 2 have no known implementation gap in `look-round-22`'s
repairs: the column's field-name rule, `.temper-outside form >
label[for]:has(+ input)`, outweighing skein's small label; at night the
root's ground, `:root[data-night]:has(body #app)`, giving way to the
body's; the welcome's top bar, under the welcome's main, packed to its
end, its steps taking the free space and an end straight after the brand
its own; and "Sign in"
on the public pages wherever the view's `Ways` give the origin, Google's
way or not. The view's public table has the no-way and unknown-way rows,
and the browser tier's public story expects "temper" and "Sign in" on
every page (`public.md`, 4), with no passing run supplied here. No focused test
reads the rules, whose effect is rendered; that comparison is `README.md`, section 3
's, after landing.

- **Not yet:** 2: `home-step-pair`'s and `needs-approval-first`'s sizes [in flight: change 535].
