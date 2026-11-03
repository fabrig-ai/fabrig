# collector-starved: standing rules

Every task of this plan names this note.

- The code is the root's `tools/tiers/`, Python and C beside no tree: read `AGENTS.md`;
  no tree's `AGENTS.md` applies. No Cargo package changes, so no `cargo fmt`, clippy or
  `old_forms.py` run is needed (each tree's `scripts/old-forms.txt` counts its own Rust
  files).
- The design is `tools/tiers/design.md`, 7.3 and 7.4, written by this cut; read it, never
  edit it. What the code taught goes in the report.
- Tests are `tools/tiers/test_cpu.py`'s own, offline, with the fakes it already has
  (`CollectorTests`' `record`, `xml`, `starved`, `cli`, `assert_cancelled_verdict`): no
  real nextest, heavy queue or service. They run through heavy, one command at a time,
  as `tools/old_forms/README.md` runs its suites:
  `heavy python3 -I -B tools/tiers/test_cpu.py [CollectorTests.NAME ...]`.
- `tools/tiers/test_run.py` also loads the collector (through `run`); run it once at the
  end: `heavy python3 -I -B tools/tiers/test_run.py`.
- fab, in the agents repository, loads `tools/tiers/cpu.py` from the checkout under test
  (its `fab/cpu.py`): it calls `summary(directory, {}, profile)`, `prepare`, `execute` and
  `toml_config`, and replaces `summarize_records` in `summary`'s globals with a function of
  the same six positional parameters `(items, pair, limits, run, timeouts, failures)` that
  calls the original. Keep those names and signatures; add summary keys, never rename or
  remove one.
- Before pushing: `python3 -I -B tools/citations/citations.py --base origin/main` from the
  checkout root, which builds nothing.
