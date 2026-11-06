# model-check: the client's tasks and the look that lands beside them

For tasks that read `temper-client-view/src/pages/model_check.rs` as `look-round-12` left it.

**`look-round-12` lands the pane, and this plan takes it by catch-up.** Your base should hold it: `git grep -n "pub(crate)
fn line" temper/crates/temper-client-view/src/pages/model_check.rs` finds it, and `temper-client-domain`'s
`pages/welcome.rs` has `model_counts`.

- **If it's there:** go on.
- **If it isn't,** `look-round-12` hasn't landed yet: run `git fetch origin refs/heads/look-round-12/integration`, then
  `git merge --no-edit FETCH_HEAD` as your branch's first commit, and say so in your report. That merge is
  pre-approved, and only that one. A conflict with this plan's own commits is yours to resolve; a conflict you can't
  explain is a question.

`temper/docs/design/client/README.md`, 3.5, *The try*, *Ready* and *A way's page* are read as they stand once that
is merged. `temper/docs/design/domain/trial.md`, 6, wins where they say less, and `README.md` isn't yours: `trial-close` brings
it in line.

**`look-round-12` draws; this plan feeds.** No rule of `temper.css` and no node order of Model's line moves here. A state
the pages have is drawn as they have it.

**The fake-backend walks' binary:** `temper/crates/temper/tests/fake_backends/**` is compiled into `temper`'s `process` test,
so `heavy cargo nextest run -p temper --test process`. Where a spec says `--test fake_backends`, it means this.
